//! MCP server (v1-spec §7.3): JSON-RPC 2.0 over Streamable HTTP at `POST /mcp` (plain JSON
//! responses), with a small task-level toolset. Each tool calls the REST handlers in-process with
//! the caller's own headers, so MCP gets exactly the same authorization and facet shaping, then
//! renders compact Markdown (`content`) next to the JSON (`structuredContent`).

use crate::routes::router;
use crate::state::AppState;
use axum::body::{Body, to_bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use std::fmt::Write;
use std::sync::Arc;
use tower::ServiceExt;

const PROTOCOL: &str = "2025-06-18";

fn tools() -> Value {
    let s = |props: Value, req: &[&str]| json!({"type": "object", "properties": props, "required": req});
    let id = json!({"type": "string", "description": "Entity id, e.g. repository:payments-api"});
    json!([
        {"name": "search", "description": "Find entities by name or title.", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"query": {"type": "string"}, "kind": {"type": "string"}, "limit": {"type": "integer"}}), &["query"])},
        {"name": "get", "description": "One entity with its fields and where each came from.", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id, "fields": {"type": "string", "description": "Comma-separated"}}), &["id"])},
        {"name": "query", "description": "Entities of a kind matching a CEL filter over `entity`, e.g. entity.ownership.status != 'verified'.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"kind": {"type": "string"}, "filter": {"type": "string"}, "fields": {"type": "string"}}), &["kind"])},
        {"name": "graph", "description": "Neighbours of an entity (relations, references, owner).", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id, "relation": {"type": "string"}, "direction": {"enum": ["in", "out", "both"]}, "depth": {"type": "integer"}}), &["id"])},
        {"name": "owner_of", "description": "Who owns an entity, whether the claim is verified (CODEOWNERS), and how to fix it.",
         "annotations": {"readOnlyHint": true}, "inputSchema": s(json!({"id": id}), &["id"])},
        {"name": "scorecard_status", "description": "Scorecard levels and, if you may see them, rule outcomes and fixes.",
         "annotations": {"readOnlyHint": true}, "inputSchema": s(json!({"id": id, "scorecard": {"type": "string"}}), &["id"])},
        {"name": "explain", "description": "Why a field has its value (sources, precedence, evidence) or why a rule passes or fails.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id, "field": {"type": "string"}, "rule": {"type": "string", "description": "<scorecard>/<rule>"}}), &["id"])},
        {"name": "refresh_entity", "description": "Ask the connectors behind an entity to re-read it; returns once the fresh data is visible.",
         "annotations": {"readOnlyHint": false, "idempotentHint": true, "openWorldHint": true}, "inputSchema": s(json!({"id": id}), &["id"])},
        {"name": "list_actions", "description": "Self-service actions available on an entity, and whether you may run them.",
         "annotations": {"readOnlyHint": true}, "inputSchema": s(json!({"id": id}), &["id"])},
        {"name": "action_options", "description": "Allowed values for an action input on an entity.", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"action": {"type": "string"}, "input": {"type": "string"}, "id": id}), &["action", "input", "id"])},
        {"name": "preview_action", "description": "Preview an action: validation, blockers and the approval path. Always preview before running.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"action": {"type": "string"}, "id": id, "inputs": {"type": "object"}}), &["action", "id"])}
    ])
}

fn enc(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

async fn call_rest(
    state: &Arc<AppState>,
    headers: &HeaderMap,
    method: Method,
    uri: String,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    for (k, v) in headers {
        if k != "content-length" && k != "content-type" {
            req = req.header(k, v);
        }
    }
    let req = match body {
        Some(b) => req
            .header("content-type", "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .unwrap();
    let resp = router(state.clone()).oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), 16 << 20)
        .await
        .unwrap_or_default();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn arg<'a>(a: &'a Value, k: &str) -> &'a str {
    a.get(k).and_then(Value::as_str).unwrap_or_default()
}

fn show(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Object(m) if m.get("redacted") == Some(&Value::Bool(true)) => {
            format!(
                "🔒 hidden (needs `{}`; visible to {})",
                m["requires"].as_str().unwrap_or("?"),
                m["visible_to"].as_str().unwrap_or("?")
            )
        }
        other => other.to_string(),
    }
}

/// Run one tool; returns (markdown, structured JSON, is_error).
async fn run_tool(
    state: &Arc<AppState>,
    h: &HeaderMap,
    name: &str,
    a: &Value,
) -> (String, Value, bool) {
    let id = enc(arg(a, "id"));
    let (status, v) = match name {
        "search" => {
            let mut uri = format!(
                "/v1/entities?q={}&limit={}",
                enc(arg(a, "query")),
                a.get("limit").and_then(Value::as_u64).unwrap_or(20)
            );
            if !arg(a, "kind").is_empty() {
                uri += &format!("&kind={}", enc(arg(a, "kind")));
            }
            call_rest(state, h, Method::GET, uri, None).await
        }
        "get" => call_rest(state, h, Method::GET, format!("/v1/entities/{id}"), None).await,
        "query" => {
            let mut uri = format!("/v1/entities?kind={}&limit=200", enc(arg(a, "kind")));
            for k in ["filter", "fields"] {
                if !arg(a, k).is_empty() {
                    uri += &format!("&{k}={}", enc(arg(a, k)));
                }
            }
            call_rest(state, h, Method::GET, uri, None).await
        }
        "graph" => {
            let mut uri = format!(
                "/v1/entities/{id}/graph?depth={}",
                a.get("depth").and_then(Value::as_u64).unwrap_or(1)
            );
            for k in ["relation", "direction"] {
                if !arg(a, k).is_empty() {
                    uri += &format!("&{k}={}", enc(arg(a, k)));
                }
            }
            call_rest(state, h, Method::GET, uri, None).await
        }
        "owner_of" => {
            call_rest(
                state,
                h,
                Method::GET,
                format!("/v1/entities/{id}/explain?field=ownership"),
                None,
            )
            .await
        }
        "scorecard_status" => {
            call_rest(
                state,
                h,
                Method::GET,
                format!("/v1/entities/{id}/scorecards"),
                None,
            )
            .await
        }
        "explain" => {
            let q = if !arg(a, "rule").is_empty() {
                format!("rule={}", enc(arg(a, "rule")))
            } else {
                format!("field={}", enc(arg(a, "field")))
            };
            call_rest(
                state,
                h,
                Method::GET,
                format!("/v1/entities/{id}/explain?{q}"),
                None,
            )
            .await
        }
        "refresh_entity" => {
            let (st, r) = call_rest(
                state,
                h,
                Method::POST,
                format!("/v1/entities/{id}/refresh"),
                None,
            )
            .await;
            if !st.is_success() {
                (st, r)
            } else {
                let mut done = Vec::new();
                for c in r["commands"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    done.push(
                        call_rest(
                            state,
                            h,
                            Method::GET,
                            format!("/v1/commands/{c}?wait=30"),
                            None,
                        )
                        .await
                        .1,
                    );
                }
                (st, json!({"entity": arg(a, "id"), "commands": done}))
            }
        }
        "list_actions" => {
            call_rest(
                state,
                h,
                Method::GET,
                format!("/v1/actions?entity={id}"),
                None,
            )
            .await
        }
        "action_options" => {
            call_rest(
                state,
                h,
                Method::GET,
                format!(
                    "/v1/actions/{}/options/{}?entity={id}",
                    enc(arg(a, "action")),
                    enc(arg(a, "input"))
                ),
                None,
            )
            .await
        }
        "preview_action" => {
            let body = json!({"entity": arg(a, "id"), "inputs": a.get("inputs").cloned().unwrap_or(json!({}))});
            call_rest(
                state,
                h,
                Method::POST,
                format!("/v1/actions/{}/previews", enc(arg(a, "action"))),
                Some(body),
            )
            .await
        }
        other => return (format!("Unknown tool `{other}`."), Value::Null, true),
    };
    if !status.is_success() {
        let msg = format!(
            "**{}** ({}): {}",
            v["title"].as_str().unwrap_or("error"),
            status.as_u16(),
            v["detail"].as_str().unwrap_or("")
        );
        return (msg, v, true);
    }
    (markdown(name, a, &v), v, false)
}

fn markdown(name: &str, a: &Value, v: &Value) -> String {
    let mut s = String::new();
    let snap = v["snapshot"]
        .as_str()
        .map(|x| format!("\n\n_as of snapshot {x}_"))
        .unwrap_or_default();
    match name {
        "search" | "query" => {
            let items = v["items"].as_array().cloned().unwrap_or_default();
            let _ = writeln!(s, "{} result(s):", items.len());
            for e in &items {
                let title = e["fields"]
                    .get("title")
                    .or(e["fields"].get("displayName"))
                    .map(show)
                    .unwrap_or_default();
                let extra = match e["fields"].get("ownership") {
                    Some(o) => format!(
                        " — owner {} ({})",
                        o.get("owner").map(show).unwrap_or("none".into()),
                        o["status"].as_str().unwrap_or("?")
                    ),
                    None => String::new(),
                };
                let _ = writeln!(
                    s,
                    "- `{}` {title}{extra}",
                    e["id"].as_str().unwrap_or_default()
                );
            }
        }
        "get" => {
            let _ = writeln!(
                s,
                "**{}** ({})",
                v["id"].as_str().unwrap_or_default(),
                v["kind"].as_str().unwrap_or_default()
            );
            for (k, val) in v["fields"].as_object().into_iter().flatten() {
                let src = v["provenance"][k]
                    .as_str()
                    .map(|p| format!(" _(from {p})_"))
                    .unwrap_or_default();
                let _ = writeln!(s, "- {k}: {}{src}", show(val));
            }
        }
        "graph" => {
            for e in v["edges"].as_array().into_iter().flatten() {
                let _ = writeln!(
                    s,
                    "- `{}` —{}→ `{}`",
                    e["from"].as_str().unwrap_or(""),
                    e["relation"].as_str().unwrap_or(""),
                    e["to"].as_str().unwrap_or("")
                );
            }
            if s.is_empty() {
                s.push_str("No visible neighbours.\n");
            }
        }
        "owner_of" => {
            let au = &v["authority"];
            let owner = v["value"]
                .get("owner")
                .map(show)
                .unwrap_or_else(|| "no applied owner".into());
            let _ = writeln!(
                s,
                "**{}**: {owner} — ownership **{}**.",
                a["id"].as_str().unwrap_or_default(),
                au["status"].as_str().unwrap_or("?")
            );
            let _ = writeln!(s, "\nWhy: {}.", au["explanation"].as_str().unwrap_or(""));
            if let Some(fix) = au["fix"].as_str() {
                let _ = writeln!(s, "\nFix: {fix}.");
            }
        }
        "scorecard_status" => {
            for (card, c) in v["scorecards"].as_object().into_iter().flatten() {
                if a.get("scorecard")
                    .and_then(Value::as_str)
                    .is_some_and(|x| x != card)
                {
                    continue;
                }
                let _ = writeln!(s, "**{card}**: {}", show(&c["level"]));
                match c.get("rules") {
                    Some(r) if r.get("redacted").is_some() => {
                        let _ = writeln!(s, "- rule details: {}", show(r));
                    }
                    Some(r) => {
                        for (rule, o) in r.as_object().into_iter().flatten() {
                            let fix = o["fix"]
                                .as_str()
                                .map(|f| format!(" — fix: {f}"))
                                .unwrap_or_default();
                            let _ = writeln!(
                                s,
                                "- {rule}: {}{fix}",
                                o["outcome"].as_str().unwrap_or("?")
                            );
                        }
                    }
                    None => {}
                }
            }
        }
        "explain" => {
            if v.get("rule").is_some() {
                let _ = writeln!(
                    s,
                    "**{}/{}** on `{}`: {}",
                    v["scorecard"].as_str().unwrap_or(""),
                    v["rule"].as_str().unwrap_or(""),
                    v["entity"].as_str().unwrap_or(""),
                    show(&v["outcome"])
                );
                let _ = writeln!(s, "- check: `{}`", v["check"].as_str().unwrap_or(""));
                for (k, val) in v["observed"].as_object().into_iter().flatten() {
                    let _ = writeln!(s, "- observed {k}: {}", show(val));
                }
                if let Some(f) = v["fix"].as_str() {
                    let _ = writeln!(s, "- fix: {f}");
                }
            } else {
                let _ = writeln!(
                    s,
                    "**{}** on `{}`: {}",
                    v["field"].as_str().unwrap_or(""),
                    v["entity"].as_str().unwrap_or(""),
                    show(&v["value"])
                );
                let _ = writeln!(
                    s,
                    "- source: {} (precedence {})",
                    v["source"].as_str().unwrap_or("none"),
                    v["precedence"]
                );
                if let Some(e) = v["authority"]["explanation"].as_str() {
                    let _ = writeln!(s, "- {e}");
                }
                if let Some(f) = v["authority"]["fix"].as_str() {
                    let _ = writeln!(s, "- fix: {f}");
                }
            }
        }
        "refresh_entity" => {
            for c in v["commands"].as_array().into_iter().flatten() {
                let _ = writeln!(
                    s,
                    "- {} {} via {}: {} → snapshot {}",
                    c["command"].as_str().unwrap_or(""),
                    c["kind"].as_str().unwrap_or(""),
                    c["connector"].as_str().unwrap_or(""),
                    c["status"].as_str().unwrap_or(""),
                    c["snapshot"].as_str().unwrap_or("pending")
                );
            }
        }
        "list_actions" => {
            for x in v["actions"].as_array().into_iter().flatten() {
                let ok = if x["allowed"] == true {
                    "you may run it".to_owned()
                } else {
                    format!("needs `{}`", x["permission"].as_str().unwrap_or(""))
                };
                let _ = writeln!(
                    s,
                    "- `{}`: {} ({ok})",
                    x["name"].as_str().unwrap_or(""),
                    x["description"].as_str().unwrap_or("")
                );
            }
        }
        "action_options" => {
            for o in v["options"].as_array().into_iter().flatten() {
                let _ = writeln!(s, "- {}", o["value"].as_str().unwrap_or(""));
            }
            if let Some(r) = v["disabled_reason"].as_str() {
                let _ = writeln!(s, "\n(disabled: {r})");
            }
        }
        "preview_action" => {
            let _ = writeln!(
                s,
                "Preview of `{}` on `{}`:",
                v["action"].as_str().unwrap_or(""),
                v["entity"].as_str().unwrap_or("")
            );
            let b = v["blockers"].as_array().cloned().unwrap_or_default();
            let _ = writeln!(
                s,
                "- blockers: {}",
                if b.is_empty() {
                    "none".into()
                } else {
                    b.iter().map(show).collect::<Vec<_>>().join("; ")
                }
            );
            let _ = writeln!(
                s,
                "- approval: {}",
                if v["approval"]["required"] == true {
                    format!("required from {}", v["approval"]["approvers"])
                } else {
                    "none".into()
                }
            );
        }
        _ => s = v.to_string(),
    }
    s + &snap
}

/// `POST /mcp`
pub async fn handle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let Ok(msg) = serde_json::from_slice::<Value>(&body) else {
        return (StatusCode::BAD_REQUEST, axum::Json(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}}))).into_response();
    };
    if msg.get("id").is_none() {
        return StatusCode::ACCEPTED.into_response(); // notification
    }
    // Every request needs an identity (same as REST).
    if let Err(r) = state.caller(&headers).await {
        return r;
    }
    let id = msg["id"].clone();
    let result = match msg["method"].as_str().unwrap_or_default() {
        "initialize" => Ok(json!({
            "protocolVersion": msg["params"]["protocolVersion"].as_str().unwrap_or(PROTOCOL),
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": {"name": "toolbox", "title": state.manifest.portal.title, "version": env!("CARGO_PKG_VERSION")},
            "instructions": state.manifest.mcp.as_ref().and_then(|m| m.get("instructions")).cloned().unwrap_or(Value::Null),
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({"tools": tools()})),
        "tools/call" => {
            let name = msg["params"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            let args = msg["params"]["arguments"].clone();
            let (text, structured, is_error) = run_tool(&state, &headers, &name, &args).await;
            Ok(
                json!({"content": [{"type": "text", "text": text}], "structuredContent": structured, "isError": is_error}),
            )
        }
        m => Err(json!({"code": -32601, "message": format!("method not found: {m}")})),
    };
    let body = match result {
        Ok(r) => json!({"jsonrpc": "2.0", "id": id, "result": r}),
        Err(e) => json!({"jsonrpc": "2.0", "id": id, "error": e}),
    };
    axum::Json(body).into_response()
}
