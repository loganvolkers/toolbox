//! Connector runtime (docs/design/connector-control.md, v1-spec §7.4): one ingest path for every
//! transport, in-image connectors supervised as child processes over stdio, the long-poll
//! endpoint for remote connectors, refresh requests, command status, initial and freshness-driven
//! resyncs, and resync reconciliation with the deletion safety valve.

use crate::routes::problem;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use toolbox_connect::Request;

/// Share of a kind's records a resync may delete before the deletes are held (ocean.md lesson 6).
const DELETION_THRESHOLD: f64 = 0.9;

/// Validate and apply one batch of connector messages (records, deletes and control messages).
pub async fn ingest_messages(
    s: &Arc<AppState>,
    connector: &str,
    messages: Vec<Value>,
) -> Result<u64, Response> {
    let Some(declared) = s.manifest.connectors.iter().find(|c| c.name == connector) else {
        return Err(problem(
            StatusCode::FORBIDDEN,
            "unknown-connector",
            format!("connector {connector} is not declared in the manifest"),
        ));
    };
    let mut errors = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        for (ptr, msg) in
            toolbox_model::schema::errors(toolbox_model::schema::connector_messages(), m)
        {
            errors.push(format!("message {i}{ptr}: {msg}"));
        }
        if matches!(
            m.get("type").and_then(Value::as_str),
            Some("record" | "delete")
        ) {
            let kind = m.get("kind").and_then(Value::as_str).unwrap_or_default();
            if !declared.records.iter().any(|r| r == kind) {
                errors.push(format!(
                    "message {i}: record kind {kind} is not declared by connector {connector}"
                ));
            }
        }
    }
    if !errors.is_empty() {
        return Err(problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid-messages",
            errors.join("; "),
        ));
    }
    let records = messages
        .iter()
        .filter(|m| m.get("type").and_then(Value::as_str) == Some("record"))
        .count();
    let changes_inputs = messages.iter().any(|m| {
        matches!(
            m.get("type").and_then(Value::as_str),
            Some("record" | "delete")
        )
    });
    // Control-only batches (register, ack, complete, …) don't change inputs, so they don't create
    // an input version: the snapshot id changes only when data does.
    let version = if changes_inputs {
        s.store
            .ingest(connector, &messages)
            .await
            .map_err(|e| problem(StatusCode::INTERNAL_SERVER_ERROR, "store", e.to_string()))?
    } else {
        s.store
            .input_version()
            .await
            .map_err(|e| problem(StatusCode::INTERNAL_SERVER_ERROR, "store", e.to_string()))?
    };
    toolbox_telemetry::catalog::record_ingest(records as u64);
    let registered = messages.iter().any(|m| m["type"] == "register");
    for (kind, seen) in s.hub.on_messages(connector, &messages, version) {
        reconcile(s, connector, &kind, &seen).await;
    }
    if registered {
        initial_resync(s, connector);
    }
    s.changed.notify_one();
    Ok(version)
}

/// After a full read of `kind`, delete the records it no longer reports, unless that would
/// remove more than the threshold (then hold the deletes and log).
async fn reconcile(s: &Arc<AppState>, connector: &str, kind: &str, seen: &BTreeSet<String>) {
    let Ok(snap) = s.store.snapshot().await else {
        return;
    };
    let existing: Vec<&String> = snap
        .records
        .iter()
        .filter(|r| r.connector == connector && r.kind == kind)
        .map(|r| &r.external_id)
        .collect();
    let missing: Vec<&String> = existing
        .iter()
        .copied()
        .filter(|id| !seen.contains(*id))
        .collect();
    if missing.is_empty() {
        return;
    }
    if existing.len() > 1 && missing.len() as f64 / existing.len() as f64 >= DELETION_THRESHOLD {
        eprintln!(
            "toolbox: held {} deletes for {connector}/{kind}: a resync may not remove ≥{:.0}% of records",
            missing.len(),
            DELETION_THRESHOLD * 100.0
        );
        return;
    }
    let deletes: Vec<Value> = missing
        .iter()
        .map(|id| json!({"type": "delete", "kind": kind, "externalId": id}))
        .collect();
    let _ = s.store.ingest(connector, &deletes).await;
}

/// A connector that registers and has never been read gets a resync of its declared kinds.
fn initial_resync(s: &Arc<AppState>, connector: &str) {
    let Some(c) = s.manifest.connectors.iter().find(|c| c.name == connector) else {
        return;
    };
    let kinds: Vec<String> = c
        .records
        .iter()
        .filter(|k| s.hub.can_resync(connector, k) && s.hub.last_read(connector, k).is_none())
        .cloned()
        .collect();
    for kind in kinds {
        s.hub.enqueue(Request {
            connector: connector.into(),
            command: "resync",
            kind: None,
            kinds: vec![kind],
            external_ids: vec![],
            reason: "initial sync".into(),
            priority: "background",
            timeout: Duration::from_secs(600),
            requested_by: "toolbox".into(),
        });
    }
}

fn parse_duration(s: &str) -> Option<Duration> {
    let (n, unit) = s.split_at(s.len().checked_sub(1)?);
    let n: u64 = n.parse().ok()?;
    Some(Duration::from_secs(match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        _ => return None,
    }))
}

/// Freshness policies (manifest `connectors[].freshness`): resync a kind when its last full read
/// is older than its policy. Runs every 5 s.
pub async fn freshness_loop(s: Arc<AppState>) {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
        for c in &s.manifest.connectors {
            for (kind, policy) in &c.freshness {
                let (Some(max), Some(last)) =
                    (parse_duration(policy), s.hub.last_read(&c.name, kind))
                else {
                    continue;
                };
                if s.hub.can_resync(&c.name, kind)
                    && toolbox_connect::hub::now().saturating_sub(last) > max.as_secs()
                {
                    s.hub.enqueue(Request {
                        connector: c.name.clone(),
                        command: "resync",
                        kind: None,
                        kinds: vec![kind.clone()],
                        external_ids: vec![],
                        reason: format!("freshness policy {policy}"),
                        priority: "freshness",
                        timeout: Duration::from_secs(600),
                        requested_by: "toolbox".into(),
                    });
                }
            }
        }
    }
}

/// Resolve `${env:X}` and `${secret:Y}` in connector config. Unresolvable keys are dropped (with a
/// warning) so connectors can tell "not configured" from "configured". `TOOLBOX_CONNECTOR_CONFIG_OVERRIDES`
/// (JSON: connector → object) is merged on top, for local runs and tests.
///
/// `${secret:…}` walks the [`toolbox_secrets::SecretResolver`] chain (env → file mount → configured
/// remotes). See `docs/design/secret-vault.md`. Placeholders stay unresolved until this call (D55).
pub fn resolve_config(connector: &str, cfg: &Value) -> Value {
    toolbox_secrets::resolve_config(connector, cfg, toolbox_secrets::global())
}

/// The binary for a connector package: `builtin:<name>` → `toolbox-connector-<name>`, found in
/// `TOOLBOX_CONNECTORS_DIR`, next to the running binary, or on PATH; `github:owner/repo[/app]@version`
/// → the release installed in the cache by `toolbox install`. Other packages are paths.
pub fn binary_for(package: &str) -> String {
    if let Some(spec) = crate::artifacts::parse(package) {
        // A released executable, unpacked into the cache by `toolbox install` (D90).
        return spec.installed().display().to_string();
    }
    let Some(name) = package.strip_prefix("builtin:") else {
        return package.to_owned();
    };
    let file = format!("toolbox-connector-{name}{}", std::env::consts::EXE_SUFFIX);
    // TOOLBOX_CONNECTORS_DIR, then next to toolbox, then PATH: release installers put each
    // connector app in the same bin directory, but a separately installed one may live elsewhere.
    let path = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect::<Vec<_>>())
        .unwrap_or_default();
    let dirs = std::env::var("TOOLBOX_CONNECTORS_DIR")
        .ok()
        .map(std::path::PathBuf::from)
        .into_iter()
        .chain(
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf())),
        )
        .chain(path);
    for d in dirs {
        if d.join(&file).is_file() {
            return d.join(&file).display().to_string();
        }
    }
    file
}

/// Start a supervisor for every in-image connector (unless `TOOLBOX_CONNECTORS=off`).
pub fn spawn_in_image(s: &Arc<AppState>) {
    if std::env::var("TOOLBOX_CONNECTORS").is_ok_and(|v| v == "off") {
        tracing::info!("in-image connectors disabled (TOOLBOX_CONNECTORS=off)");
        return;
    }
    for c in s
        .manifest
        .connectors
        .iter()
        .filter(|c| c.runtime == "in-image")
    {
        let (state, name) = (s.clone(), c.name.clone());
        let program = binary_for(&c.package);
        let config = resolve_config(&c.name, c.config.as_ref().unwrap_or(&Value::Null));
        tokio::spawn(async move { supervise(state, name, program, config).await });
    }
}

async fn supervise(s: Arc<AppState>, name: String, program: String, config: Value) {
    let mut backoff = Duration::from_secs(1);
    loop {
        let child = tokio::process::Command::new(&program)
            .env("TOOLBOX_CONNECTOR_CONFIG", config.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn();
        let mut child = match child {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(connector = %name, error = %e, program = %program, "cannot start connector");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
                continue;
            }
        };
        let mut stdin = child.stdin.take().unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let (writer_state, writer_name) = (s.clone(), name.clone());
        let writer = tokio::spawn(async move {
            loop {
                for cmd in writer_state
                    .hub
                    .take(&writer_name, Duration::from_secs(30))
                    .await
                {
                    if stdin
                        .write_all(format!("{cmd}\n").as_bytes())
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                let _ = stdin.flush().await;
            }
        });
        // Read messages, batching until a control boundary, 500 messages or 100 ms of quiet.
        let mut batch: Vec<Value> = Vec::new();
        loop {
            match tokio::time::timeout(Duration::from_millis(100), lines.next_line()).await {
                Ok(Ok(Some(line))) => {
                    let Ok(m) = serde_json::from_str::<Value>(&line) else {
                        continue;
                    };
                    let boundary = matches!(
                        m["type"].as_str(),
                        Some("register" | "complete" | "snapshot_complete")
                    );
                    if m["type"] == "log" {
                        tracing::info!(connector = %name, msg = m["message"].as_str().unwrap_or(""));
                        continue;
                    }
                    batch.push(m);
                    if boundary || batch.len() >= 500 {
                        flush(&s, &name, &mut batch).await;
                    }
                }
                Ok(_) => break, // stdout closed: the process exited
                Err(_) => flush(&s, &name, &mut batch).await,
            }
        }
        flush(&s, &name, &mut batch).await;
        writer.abort();
        let status = child
            .wait()
            .await
            .map(|st| st.to_string())
            .unwrap_or_default();
        tracing::warn!(connector = %name, %status, ?backoff, "connector exited; restarting");
        tokio::time::sleep(backoff).await;
        backoff = (backoff * 2).min(Duration::from_secs(30));
    }
}

async fn flush(s: &Arc<AppState>, name: &str, batch: &mut Vec<Value>) {
    if batch.is_empty() {
        return;
    }
    let msgs = std::mem::take(batch);
    if let Err(r) = ingest_messages(s, name, msgs).await {
        tracing::error!(connector = %name, status = %r.status(), "connector batch rejected");
    }
}

#[derive(Deserialize)]
pub struct WaitQuery {
    wait: Option<u64>,
}

/// `GET /v1/connectors/{name}/commands?wait=` — the long-poll stream for remote connectors.
pub async fn poll_commands(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(q): Query<WaitQuery>,
) -> Response {
    match crate::routes::connector_of(&s, &headers) {
        Some(c) if c == name => {}
        _ => {
            return problem(
                StatusCode::UNAUTHORIZED,
                "unauthenticated",
                format!("needs connector {name}'s token"),
            );
        }
    }
    let cmds = s
        .hub
        .take(&name, Duration::from_secs(q.wait.unwrap_or(25).min(60)))
        .await;
    Json(json!({"commands": cmds})).into_response()
}

/// `GET /v1/connectors` — registrations and pending commands (operators).
pub async fn list_connectors(State(s): State<Arc<AppState>>, headers: HeaderMap) -> Response {
    match s.caller(&headers).await {
        Ok(c) if c.admin => {}
        Ok(c) => {
            return problem(
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("{} is not a portal admin", c.user),
            );
        }
        Err(r) => return r,
    }
    let regs = s.hub.registrations();
    let connectors: Vec<Value> = s
        .manifest
        .connectors
        .iter()
        .map(|c| {
            let last: serde_json::Map<String, Value> =
                c.records.iter().map(|k| (k.clone(), json!(s.hub.last_read(&c.name, k).map(toolbox_connect::hub::rfc3339)))).collect();
            json!({"name": c.name, "runtime": c.runtime, "registration": regs.get(&c.name), "last_read": last})
        })
        .collect();
    Json(json!({"connectors": connectors, "pending": s.hub.pending()})).into_response()
}

/// `POST /v1/entities/{id}/refresh` — ask the connectors behind an entity to re-read it.
pub async fn refresh_entity(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let Some(e) = p.derived.entities.get(&id) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    };
    let allowed = match s.allowed(&caller, "can_refresh", &id).await {
        Ok(true) => true,
        Ok(false) => {
            // Kinds without a can_refresh relation: anyone who can see the entity may refresh it.
            let ty = id.split(':').next().unwrap_or_default();
            let defined = s.authz.as_ref().is_some_and(|a| {
                toolbox_authz::fga_dsl::has_relation(a.model(), ty, "can_refresh")
            });
            !defined
                && s.visibility(&caller, &[id.as_str()])
                    .await
                    .map(|v| v[0].view)
                    .unwrap_or(false)
        }
        Err(r) => return r,
    };
    if !allowed {
        return problem(
            StatusCode::FORBIDDEN,
            "forbidden",
            format!("{} lacks `can_refresh` on {id}", caller.user),
        );
    }
    let mut groups: std::collections::BTreeMap<(String, String), Vec<String>> = Default::default();
    for (connector, kind, ext) in &e.records {
        if s.hub.can_refresh(connector, kind, "external_ids") {
            groups
                .entry((connector.clone(), kind.clone()))
                .or_default()
                .push(ext.clone());
        }
    }
    if groups.is_empty() {
        return problem(
            StatusCode::CONFLICT,
            "not-refreshable",
            format!("no registered connector can refresh {id}"),
        );
    }
    let commands: Vec<String> = groups
        .into_iter()
        .map(|((connector, kind), ids)| {
            s.hub.enqueue(Request {
                connector,
                command: "refresh",
                kind: Some(kind),
                kinds: vec![],
                external_ids: ids,
                reason: format!("refresh {id}"),
                priority: "interactive",
                timeout: Duration::from_secs(60),
                requested_by: caller.user.clone(),
            })
        })
        .collect();
    (
        StatusCode::ACCEPTED,
        Json(json!({"entity": id, "commands": commands})),
    )
        .into_response()
}

/// `GET /v1/commands/{id}?wait=` — status, and once complete the snapshot that holds the results.
pub async fn command_status(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<WaitQuery>,
) -> Response {
    if let Err(r) = s.caller(&headers).await {
        return r;
    }
    let wait = Duration::from_secs(q.wait.unwrap_or(0).min(60));
    let Some(c) = s.hub.wait(&id, wait).await else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no command {id}"),
        );
    };
    let snapshot = match c.completed_version {
        Some(v) => s
            .wait_for(
                v,
                Duration::from_secs(10).min(wait.max(Duration::from_millis(1))),
            )
            .await
            .map(|p| p.snapshot_id.clone()),
        None => None,
    };
    let mut body = serde_json::to_value(&c).unwrap();
    body["snapshot"] = json!(snapshot);
    Json(body).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use toolbox_secrets::backends::EnvBackend;
    use toolbox_secrets::{resolve_config as resolve_with, SecretResolver};

    #[test]
    fn resolve_config_uses_secret_chain() {
        let resolver = SecretResolver::new(vec![Arc::new(EnvBackend::from_map(BTreeMap::from([(
            "github-app-private-key".into(),
            "pem".into(),
        )])))]);
        let cfg = json!({
            "org": "acme",
            "privateKey": "${secret:github-app-private-key}",
            "missing": "${secret:no-such}",
            "nested": { "token": "${secret:github-app-private-key}" }
        });
        let out = resolve_with("github", &cfg, &resolver);
        assert_eq!(out["org"], "acme");
        assert_eq!(out["privateKey"], "pem");
        assert!(out.get("missing").is_none());
        assert_eq!(out["nested"]["token"], "pem");
    }
}
