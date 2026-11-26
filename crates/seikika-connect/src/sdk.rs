//! Writing a connector (connector-control §3–§4). Implement [`Source`], then call [`run_stdio`]
//! (in-image: commands on stdin, messages on stdout) or [`run_remote`] (long-poll the server for
//! commands, push messages to `POST /v1/ingest`). The SDK sends `register`, `ack` and `complete`.

use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// A connector's data source.
#[allow(async_fn_in_trait)]
pub trait Source {
    fn name(&self) -> &str;
    /// Connector package version reported on `register`.
    ///
    /// Override this. The previous default used `env!("CARGO_PKG_VERSION")` inside
    /// `seikika-connect`, so every connector that left the default reported the SDK crate's
    /// version instead of its own.
    fn version(&self) -> &str;
    /// `capabilities` for the register message.
    fn capabilities(&self) -> Value;
    /// Handle a `refresh` or `resync` command, writing records to `out`. Return `Ok(())` for
    /// success, or `Err` with a message (the command completes as `failed`).
    async fn handle(&mut self, command: &Value, out: &mut Out) -> Result<(), String>;
}

/// Where messages go; collects counts for `complete`.
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

    pub fn record(&mut self, kind: &str, external_id: &str, data: Value) {
        self.records += 1;
        let mut m =
            json!({"type": "record", "kind": kind, "externalId": external_id, "data": data});
        if let Some(c) = &self.command_id {
            m["command_id"] = json!(c);
        }
        self.lines.push(m);
    }

    pub fn delete(&mut self, kind: &str, external_id: &str) {
        let mut m = json!({"type": "delete", "kind": kind, "externalId": external_id});
        if let Some(c) = &self.command_id {
            m["command_id"] = json!(c);
        }
        self.lines.push(m);
    }

    pub fn not_found(&mut self, kind: &str, external_id: &str) {
        self.not_found += 1;
        if let Some(c) = &self.command_id {
            self.lines.push(json!({"type": "not_found", "kind": kind, "externalId": external_id, "command_id": c}));
        }
    }

    pub fn snapshot_complete(&mut self, kind: &str) {
        if let Some(c) = &self.command_id {
            self.lines
                .push(json!({"type": "snapshot_complete", "kind": kind, "command_id": c}));
        }
    }

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

/// In-image mode: register on stdout, then serve commands from stdin until it closes.
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

/// Remote mode: register through the ingest endpoint, then long-poll for commands.
/// `token` is the connector's bearer token (or, against a dev server, `None` to use the
/// `x-seikika-connector` header).
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

/// Push records outside any command (webhooks, SCIM): one batch through the ingest endpoint.
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
