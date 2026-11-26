//! Implement the record reads for a connector and serve them over stdio or HTTP.
//!
//! Implement [`Source`] and use [`Out`] to report the results of each command. The transport
//! sends registration, acknowledgement, and completion messages for you. Keep stdout reserved
//! for protocol messages in stdio mode. Use stderr or [`Out::log`] for diagnostics.
//!
//! # Examples
//!
//! This connector serves one record kind. Add `seikika-connect`, `serde_json`, and `tokio` with
//! `macros` and `rt` enabled to your binary crate. The example compiles as a doctest and waits
//! for portal commands when run as a binary. Use `seikika connector test` with a manifest that
//! declares `example-item` to exercise the protocol.
//!
//! ```no_run
//! use seikika_connect::sdk::{Out, Source, run_stdio};
//! use serde_json::{Value, json};
//!
//! struct Example;
//!
//! impl Source for Example {
//!     fn name(&self) -> &str { "example" }
//!     fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }
//!
//!     fn capabilities(&self) -> Value {
//!         json!({"refresh": {"example-item": ["external_ids"]}, "resync": ["example-item"]})
//!     }
//!
//!     async fn handle(&mut self, command: &Value, out: &mut Out) -> Result<(), String> {
//!         const KIND: &str = "example-item";
//!         const ID: &str = "example:alpha";
//!         let item = || json!({"title": "Alpha"});
//!         match command["command"].as_str() {
//!             Some("refresh") if command["kind"] == KIND => {
//!                 let ids = command["external_ids"].as_array().ok_or("external_ids required")?;
//!                 for id in ids {
//!                     let id = id.as_str().ok_or("expected a string id")?;
//!                     if id == ID {
//!                         out.record(KIND, id, item());
//!                     } else {
//!                         out.not_found(KIND, id);
//!                     }
//!                 }
//!             }
//!             Some("resync") if command["kinds"].as_array().is_some_and(|k| k.contains(&json!(KIND))) => {
//!                 out.record(KIND, ID, item());
//!                 out.snapshot_complete(KIND);
//!             }
//!             _ => return Err("unsupported command or kind".into()),
//!         }
//!         Ok(())
//!     }
//! }
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() {
//!     run_stdio(Example).await;
//! }
//! ```

use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Supplies a connector's registration and command handlers.
///
/// The SDK calls the handler one command at a time. Make repeated reads safe so a second
/// refresh of the same scope can return the same records when the upstream data is unchanged.
/// The transport ignores `cancel` commands and lets each handler finish before reading the next
/// command.
#[allow(async_fn_in_trait)]
pub trait Source {
    /// Returns the connector name declared in the portal manifest.
    fn name(&self) -> &str;
    /// Returns the connector package version reported during registration.
    ///
    /// Use `env!("CARGO_PKG_VERSION")` in your connector crate to report its version.
    fn version(&self) -> &str;
    /// Returns the control capabilities reported during registration.
    ///
    /// Set `refresh` to an object mapping record kinds to lists of supported scopes, such as
    /// `external_ids`. Set `resync` to the list of kinds this connector can read in full.
    fn capabilities(&self) -> Value;
    /// Handles one command and writes result messages to `out`.
    ///
    /// Read the requested `kind` and scope for `refresh`, or the requested `kinds` for `resync`.
    /// Finish a full read with [`Out::snapshot_complete`]. For an external ID refresh, report
    /// each requested ID even when the source cannot find it.
    ///
    /// # Errors
    ///
    /// Return an error message if the request is unsupported or the read fails. The SDK sends a
    /// `failed` completion with error code `source`. Messages already written to `out` are still
    /// sent, so emit a snapshot completion marker only after a successful full read of its kind.
    async fn handle(&mut self, command: &Value, out: &mut Out) -> Result<(), String>;
}

/// Buffers messages and completion counts for the current command.
///
/// The transport makes this value available to [`Source::handle`] and sends the buffered
/// messages after the handler returns.
pub struct Out {
    lines: Vec<Value>,
    command_id: Option<String>,
    records: u64,
    not_found: u64,
}

impl Out {
    fn new(command_id: Option<String>) -> Self {
        Out {
            lines: Vec::new(),
            command_id,
            records: 0,
            not_found: 0,
        }
    }

    /// Reports the current data for a record and increments the completion's record count.
    ///
    /// Use a connector-declared record kind and a stable external ID such as `example:alpha`.
    /// The ID needs a lowercase alphanumeric or hyphenated prefix, a colon, and a nonempty
    /// suffix. Supply the data object expected by the portal's mapping for that kind.
    pub fn record(&mut self, kind: &str, external_id: &str, data: Value) {
        self.records += 1;
        let mut m =
            json!({"type": "record", "kind": kind, "externalId": external_id, "data": data});
        if let Some(c) = &self.command_id {
            m["command_id"] = json!(c);
        }
        self.lines.push(m);
    }

    /// Reports an explicit deletion of a known record in the source.
    ///
    /// Use [`Self::not_found`] when reporting a lookup miss during a scoped refresh.
    pub fn delete(&mut self, kind: &str, external_id: &str) {
        let mut m = json!({"type": "delete", "kind": kind, "externalId": external_id});
        if let Some(c) = &self.command_id {
            m["command_id"] = json!(c);
        }
        self.lines.push(m);
    }

    /// Reports that a requested external ID could not be found during a scoped refresh.
    ///
    /// This accounts for the ID in the refresh result. Use [`Self::delete`] to request removal.
    pub fn not_found(&mut self, kind: &str, external_id: &str) {
        self.not_found += 1;
        if let Some(c) = &self.command_id {
            self.lines.push(json!({"type": "not_found", "kind": kind, "externalId": external_id, "command_id": c}));
        }
    }

    /// Marks the end of a full read of `kind` during a resync.
    ///
    /// Emit this marker after all records for the kind, including after a successful empty
    /// read. The portal can use it to reconcile records absent from the snapshot.
    pub fn snapshot_complete(&mut self, kind: &str) {
        if let Some(c) = &self.command_id {
            self.lines
                .push(json!({"type": "snapshot_complete", "kind": kind, "command_id": c}));
        }
    }

    /// Adds a diagnostic log message to the command's output.
    ///
    /// Use a protocol level such as `info`, `warn`, or `error`. Keep credentials out of `message`.
    pub fn log(&mut self, level: &str, message: &str) {
        self.lines
            .push(json!({"type": "log", "level": level, "message": message}));
    }
}

fn register(src: &impl Source, instance: &str) -> Value {
    json!({"type": "register", "connector": src.name(), "version": src.version(), "instance": instance,
           "capabilities": src.capabilities()})
}

async fn run_command(src: &mut impl Source, cmd: &Value) -> Vec<Value> {
    let id = cmd["command_id"].as_str().map(str::to_owned);
    let mut out = Out::new(id.clone());
    if cmd["command"] == "cancel" {
        // Commands run to completion one at a time, so there's nothing in flight to cancel.
        return vec![];
    }
    let mut lines = vec![json!({"type": "ack", "command_id": id})];
    let result = src.handle(cmd, &mut out).await;
    let counts = json!({"records": out.records, "not_found": out.not_found});
    lines.append(&mut out.lines);
    lines.push(match result {
        Ok(()) => {
            json!({"type": "complete", "command_id": id, "status": "succeeded", "counts": counts})
        }
        Err(e) => {
            json!({"type": "complete", "command_id": id, "status": "failed", "counts": counts,
                         "errors": [{"code": "source", "message": e}]})
        }
    });
    lines
}

/// Runs an in-image connector over newline-delimited JSON on stdin and stdout.
///
/// Run this on a Tokio runtime. It registers on stdout, then serves commands serially until stdin
/// closes or a read fails. It skips malformed JSON and non-command messages. Output write failures
/// are discarded. Send application diagnostics to stderr so stdout stays machine-readable.
///
/// # Panics
///
/// Panics if polled outside a Tokio runtime.
pub async fn run_stdio(mut src: impl Source) {
    let mut stdout = tokio::io::stdout();
    let instance = format!("{}-{}", src.name(), std::process::id());
    let write = |v: &Value| format!("{v}\n");
    let _ = stdout
        .write_all(write(&register(&src, &instance)).as_bytes())
        .await;
    let _ = stdout.flush().await;
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let Ok(cmd) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if cmd["type"] != "command" {
            continue;
        }
        for m in run_command(&mut src, &cmd).await {
            let _ = stdout.write_all(write(&m).as_bytes()).await;
        }
        let _ = stdout.flush().await;
    }
}

/// Registers a remote connector and long-polls the portal for commands.
///
/// Run this on a Tokio runtime. Pass the portal base URL without a trailing slash. Supply the
/// connector's bearer token, or pass `None` to identify it with `x-seikika-connector` in dev mode.
/// After registration this loops until cancelled. It retries failed polls after a delay and
/// submits each command's response batch once. Submission failures are discarded.
///
/// # Errors
///
/// Returns an error if the registration request fails or responds with a client or server error.
///
/// # Panics
///
/// Panics if the HTTP client cannot initialize, for example if TLS setup fails.
pub async fn run_remote(
    mut src: impl Source,
    url: &str,
    token: Option<&str>,
) -> Result<(), String> {
    let http = reqwest::Client::new();
    let name = src.name().to_owned();
    let post = |msgs: Vec<Value>| {
        let mut r = http.post(format!("{url}/v1/ingest")).json(&msgs);
        r = match token {
            Some(t) => r.bearer_auth(t),
            None => r.header("x-seikika-connector", &name),
        };
        r.send()
    };
    let instance = format!("{name}-{}", std::process::id());
    post(vec![register(&src, &instance)])
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    loop {
        let mut req = http.get(format!("{url}/v1/connectors/{name}/commands?wait=25"));
        req = match token {
            Some(t) => req.bearer_auth(t),
            None => req.header("x-seikika-connector", &name),
        };
        let cmds: Value = match req.send().await {
            Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
            _ => {
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };
        for cmd in cmds["commands"].as_array().into_iter().flatten() {
            let msgs = run_command(&mut src, cmd).await;
            if !msgs.is_empty() {
                let _ = post(msgs).await;
            }
        }
    }
}

/// Sends one unsolicited batch to the portal's ingest endpoint.
///
/// Call this on a Tokio runtime from a webhook or other event receiver. Pass the portal base URL
/// without a trailing slash and a list of connector protocol messages. Supply a connector bearer
/// token, or pass `None` to identify `connector` with `x-seikika-connector` in dev mode.
/// A successful HTTP status returns the decoded response body, or JSON `null` if decoding fails.
///
/// # Errors
///
/// Returns an error if the request fails or the portal returns an unsuccessful HTTP status.
///
/// # Panics
///
/// Panics if the HTTP client cannot initialize, for example if TLS setup fails.
pub async fn push(
    url: &str,
    connector: &str,
    token: Option<&str>,
    msgs: Vec<Value>,
) -> Result<Value, String> {
    let mut r = reqwest::Client::new()
        .post(format!("{url}/v1/ingest"))
        .json(&msgs);
    r = match token {
        Some(t) => r.bearer_auth(t),
        None => r.header("x-seikika-connector", connector),
    };
    let resp = r.send().await.map_err(|e| e.to_string())?;
    let status = resp.status();
    let body: Value = resp.json().await.unwrap_or_default();
    if status.is_success() {
        Ok(body)
    } else {
        Err(format!("{status}: {body}"))
    }
}
