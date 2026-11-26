//! Check a connector's registration, resync, refresh, and cancellation over stdio.
//!
//! `seikika connector test` uses this kit. [`run`] starts a connector and checks these behaviors:
//!
//! 1. The connector registers first, and its resync capabilities name only manifest record kinds.
//! 2. Registration and command responses validate against the [connector message schema].
//! 3. A `resync` of each advertised kind emits records of that kind, `snapshot_complete`, and a
//!    successful `complete`. Every record carries the command ID.
//! 4. A scoped `refresh` returns a record or `not_found` for each requested external ID.
//! 5. A lookup of an unknown external ID returns `not_found`.
//! 6. Repeating the same refresh yields the same records.
//! 7. The connector keeps serving after a `cancel` for an unknown command.
//!
//! Use [`drive_resync`] to inspect raw messages from a single resync.
//!
//! [connector message schema]: https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/connector-messages.schema.json

use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

/// The outcome of one connector conformance check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    /// Human-readable check name.
    pub name: String,
    /// Whether the observed messages satisfied the check.
    pub passed: bool,
    /// Observed result or failure detail. May be empty.
    pub detail: String,
}

struct Conn {
    _child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
}

impl Conn {
    async fn read(&mut self, timeout: Duration) -> Option<Value> {
        match tokio::time::timeout(timeout, self.lines.next_line()).await {
            Ok(Ok(Some(l))) => serde_json::from_str(&l).ok(),
            _ => None,
        }
    }

    async fn send(&mut self, v: &Value) {
        let _ = self.stdin.write_all(format!("{v}\n").as_bytes()).await;
        let _ = self.stdin.flush().await;
    }

    /// Send a command and collect messages until its `complete`.
    async fn exchange(&mut self, cmd: &Value, timeout: Duration) -> Vec<Value> {
        self.send(cmd).await;
        let mut out = Vec::new();
        while let Some(m) = self.read(timeout).await {
            let done = m["type"] == "complete" && m["command_id"] == cmd["command_id"];
            out.push(m);
            if done {
                break;
            }
        }
        out
    }
}

fn command(id: &str, body: Value) -> Value {
    let mut c = json!({"type": "command", "command_id": id, "reason": "conformance", "priority": "background",
                       "deadline": "2099-01-01T00:00:00Z"});
    c.as_object_mut()
        .unwrap()
        .extend(body.as_object().unwrap().clone());
    c
}

/// Starts a connector and returns the outcomes of its conformance checks in execution order.
///
/// Pass the executable and its arguments separately. Use `envs` to override inherited environment
/// variables and pass the manifest's record kind names in `records`. Inspect each
/// [`CheckResult::passed`] to determine whether the connector passed. If the process fails to
/// start, the only result is a failing `starts` check.
///
/// # Panics
///
/// Panics without a Tokio runtime with process I/O and time enabled.
pub async fn run(
    program: &str,
    args: &[String],
    envs: &[(String, String)],
    records: &[String],
) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let mut check = |name: &str, passed: bool, detail: String| {
        results.push(CheckResult {
            name: name.into(),
            passed,
            detail,
        })
    };
    let child = Command::new(program)
        .args(args)
        .envs(envs.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(e) => {
            check("starts", false, format!("cannot start {program}: {e}"));
            return results;
        }
    };
    let stdin = child.stdin.take().unwrap();
    let lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut conn = Conn {
        _child: child,
        stdin,
        lines,
    };
    let t = Duration::from_secs(30);
    let schema = seikika_model::schema::connector_messages();
    let invalid = |msgs: &[Value]| -> Vec<String> {
        msgs.iter()
            .flat_map(|m| {
                seikika_model::schema::errors(schema, m)
                    .into_iter()
                    .map(|(p, e)| format!("{}{p}: {e}", m["type"]))
            })
            .collect()
    };

    // 1. register
    let reg = conn.read(t).await.unwrap_or(Value::Null);
    let ok = reg["type"] == "register";
    let resync_kinds: BTreeSet<String> = reg["capabilities"]["resync"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect();
    let declared: BTreeSet<String> = records.iter().cloned().collect();
    check(
        "registers first",
        ok,
        format!("first message: {}", reg["type"]),
    );
    let errs = invalid(std::slice::from_ref(&reg));
    check(
        "register message is valid",
        ok && errs.is_empty(),
        errs.join("; "),
    );
    let extra: Vec<&String> = resync_kinds.difference(&declared).collect();
    check(
        "capabilities only name declared record kinds",
        extra.is_empty(),
        format!("undeclared: {extra:?}"),
    );

    // 3. resync each kind
    let mut all_msgs = Vec::new();
    let mut sample: Option<(String, String)> = None;
    for (i, kind) in resync_kinds.iter().enumerate() {
        let id = format!("cmd_resync{i}");
        let msgs = conn
            .exchange(
                &command(&id, json!({"command": "resync", "kinds": [kind]})),
                t,
            )
            .await;
        let records: Vec<&Value> = msgs.iter().filter(|m| m["type"] == "record").collect();
        let wrong_kind = records
            .iter()
            .filter(|m| m["kind"] != kind.as_str())
            .count();
        let untagged = records
            .iter()
            .filter(|m| m["command_id"] != id.as_str())
            .count();
        let snap = msgs
            .iter()
            .any(|m| m["type"] == "snapshot_complete" && m["kind"] == kind.as_str());
        let done = msgs.iter().find(|m| m["type"] == "complete");
        let complete = done.map(|m| m["status"].clone()).unwrap_or(Value::Null);
        let reason: String = done
            .and_then(|m| m["errors"].as_array())
            .into_iter()
            .flatten()
            .filter_map(|e| e["message"].as_str())
            .map(|e| format!(": {e}"))
            .collect();
        let acked = msgs.first().is_some_and(|m| m["type"] == "ack");
        check(
            &format!("resync {kind}"),
            acked
                && !records.is_empty()
                && wrong_kind == 0
                && untagged == 0
                && snap
                && complete == "succeeded",
            format!(
                "{} records, ack={acked}, snapshot_complete={snap}, complete={complete}{reason}, wrong kind={wrong_kind}, untagged={untagged}",
                records.len()
            ),
        );
        if sample.is_none()
            && reg["capabilities"]["refresh"][kind]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x == "external_ids"))
            && let Some(r) = records.first()
        {
            sample = Some((
                kind.clone(),
                r["externalId"].as_str().unwrap_or_default().to_owned(),
            ));
        }
        all_msgs.extend(msgs);
    }
    let errs = invalid(&all_msgs);
    check(
        "every resync message is valid",
        errs.is_empty(),
        errs.into_iter().take(5).collect::<Vec<_>>().join("; "),
    );

    // 4–6. scoped refresh
    match sample {
        Some((kind, ext)) => {
            let body = json!({"command": "refresh", "kind": kind, "external_ids": [ext, "missing:does-not-exist"]});
            let a = conn
                .exchange(&command("cmd_refresh1", body.clone()), t)
                .await;
            let b = conn.exchange(&command("cmd_refresh2", body), t).await;
            let ids = |msgs: &[Value], ty: &str| -> BTreeSet<String> {
                msgs.iter()
                    .filter(|m| m["type"] == ty)
                    .filter_map(|m| m["externalId"].as_str().map(str::to_owned))
                    .collect()
            };
            let got = ids(&a, "record");
            check(
                "scoped refresh returns exactly the requested ids",
                got == BTreeSet::from([ext.clone()]),
                format!("records: {got:?}"),
            );
            check(
                "an unknown id is reported as not_found",
                ids(&a, "not_found").contains("missing:does-not-exist"),
                format!("not_found: {:?}", ids(&a, "not_found")),
            );
            let data = |msgs: &[Value]| {
                msgs.iter()
                    .filter(|m| m["type"] == "record")
                    .map(|m| m["data"].clone())
                    .collect::<Vec<_>>()
            };
            check("refresh is idempotent", data(&a) == data(&b), String::new());
            let errs = invalid(&a);
            check(
                "every refresh message is valid",
                errs.is_empty(),
                errs.join("; "),
            );
        }
        None => check(
            "scoped refresh",
            true,
            "connector declares no external_ids refresh scope (skipped)".into(),
        ),
    }

    // 7. cancel for an unknown command, then still serving
    conn.send(&json!({"type": "command", "command_id": "cmd_cancel", "command": "cancel", "target": "cmd_unknown"})).await;
    let still = match resync_kinds.iter().next() {
        Some(kind) => conn
            .exchange(
                &command("cmd_after", json!({"command": "resync", "kinds": [kind]})),
                t,
            )
            .await
            .iter()
            .any(|m| m["type"] == "complete"),
        None => true,
    };
    check("keeps serving after cancel", still, String::new());
    results
}

/// Starts a connector, issues one `resync`, and returns the response messages.
///
/// Pass the executable and its arguments separately. Use `envs` to override inherited
/// environment variables. The returned batch starts after registration and includes the matching
/// `complete` message if received. It can be partial if the process stops responding or emits
/// invalid JSON. Check the completion status if you require a successful resync.
///
/// # Errors
///
/// Returns an error if the process fails to start, the first message is missing or isn't a
/// registration, or no response messages arrive.
///
/// # Panics
///
/// Panics without a Tokio runtime with process I/O and time enabled.
pub async fn drive_resync(
    program: &str,
    args: &[String],
    envs: &[(String, String)],
    kinds: &[String],
) -> Result<Vec<Value>, String> {
    let mut child = Command::new(program)
        .args(args)
        .envs(envs.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("cannot start {program}: {e}"))?;
    let stdin = child.stdin.take().unwrap();
    let lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let mut conn = Conn {
        _child: child,
        stdin,
        lines,
    };
    let t = Duration::from_secs(60);
    let reg = conn
        .read(t)
        .await
        .ok_or_else(|| "connector did not register".to_owned())?;
    if reg["type"] != "register" {
        return Err(format!(
            "first message was {}, expected register",
            reg["type"]
        ));
    }
    let id = "drive_resync";
    let msgs = conn
        .exchange(
            &command(id, json!({"command": "resync", "kinds": kinds})),
            t,
        )
        .await;
    if msgs.is_empty() {
        return Err("no messages for resync".into());
    }
    Ok(msgs)
}
