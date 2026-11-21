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

/// The built-in MCP tools (v1-spec §7.3), as `tools/list` reports them before promoted actions.
pub fn builtin_tools() -> Value {
    let s = |props: Value, req: &[&str]| json!({"type": "object", "properties": props, "required": req});
    let id = json!({"type": "string", "description": "Entity id, e.g. repository:payments-api"});
    json!([
        {"name": "search", "description": "Find entities by name or title.", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({
             "query": {"type": "string", "description": "Text to find in entity ids and titles (case-insensitive substring)"},
             "kind": {"type": "string", "description": "Only entities of this kind, e.g. Repository"},
             "limit": {"type": "integer", "description": "Maximum number of results (default 20)"}}), &["query"])},
        {"name": "get", "description": "One entity with its fields and where each came from.", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id, "fields": {"type": "string", "description": "Comma-separated"}}), &["id"])},
        {"name": "query", "description": "Entities of a kind matching a CEL filter over `entity`, e.g. entity.ownership.status != 'verified'.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({
             "kind": {"type": "string", "description": "Entity kind to list, e.g. Repository"},
             "filter": {"type": "string", "description": "CEL expression over `entity` (its visible fields and `id`); keeps entities where it is true"},
             "fields": {"type": "string", "description": "Comma-separated fields to return (default: every visible field)"}}), &["kind"])},
        {"name": "graph", "description": "Neighbours of an entity (relations, references, owner).", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id,
             "relation": {"type": "string", "description": "Only edges with this relation name (default: all relations)"},
             "direction": {"enum": ["in", "out", "both"], "description": "Follow outgoing edges, incoming edges or both (default both)"},
             "depth": {"type": "integer", "description": "How many hops to follow, 1 to 3 (default 1)"}}), &["id"])},
        {"name": "owner_of", "description": "Who owns an entity, whether the claim is verified (CODEOWNERS), and how to fix it.",
         "annotations": {"readOnlyHint": true}, "inputSchema": s(json!({"id": id}), &["id"])},
        {"name": "scorecard_status", "description": "Scorecard levels and, if you may see them, rule outcomes and fixes.",
         "annotations": {"readOnlyHint": true}, "inputSchema": s(json!({"id": id, "scorecard": {"type": "string", "description": "Only this scorecard (default: every scorecard)"}}), &["id"])},
        {"name": "scorecard_report", "description": "Results of one scorecard across every entity you may see: level counts and, per entity, the failing rules with their evidence. Filter to one rule's failures to get a work list. Without `scorecard`, lists the scorecards.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({
             "scorecard": {"type": "string", "description": "Scorecard name (omit to list every scorecard with its level counts)"},
             "rule": {"type": "string", "description": "Only entities where this rule has `outcome`"},
             "outcome": {"enum": ["fail", "pass", "unknown"], "description": "Outcome to match for `rule` (default fail)"},
             "level": {"type": "string", "description": "Only entities at this level, e.g. none, bronze, skipped"}}), &[])},
        {"name": "campaign_report", "description": "Progress of a time-bound campaign over existing scorecards: percent meeting the goal, milestones, and entities that still have gaps. Without `campaign`, lists every campaign.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({
             "campaign": {"type": "string", "description": "Campaign name (omit to list every campaign with its progress)"}}), &[])},
        {"name": "assessment_brief", "description": "What to judge an entity on, as a coding agent: the portal's rubric for its kind (criteria with questions and guidance), the entity's facts, its subject hash and a template to fill in. Write the result as an assessment file (or `toolbox assess submit`); scorecards read assessments as data.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id,
             "rubric": {"type": "string", "description": "Rubric name (default: the only rubric for the entity's kind)"}}), &["id"])},
        {"name": "explain", "description": "Why a field has its value (sources, precedence, evidence) or why a rule passes or fails.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({"id": id,
             "field": {"type": "string", "description": "Field to explain, e.g. ownership (used when `rule` is not set)"},
             "rule": {"type": "string", "description": "<scorecard>/<rule>"}}), &["id"])},
        {"name": "refresh_entity", "description": "Ask the connectors behind an entity to re-read it; returns once the fresh data is visible.",
         "annotations": {"readOnlyHint": false, "idempotentHint": true, "openWorldHint": true}, "inputSchema": s(json!({"id": id}), &["id"])},
        {"name": "list_actions", "description": "Self-service actions available on an entity, and whether you may run them.",
         "annotations": {"readOnlyHint": true}, "inputSchema": s(json!({"id": id}), &["id"])},
        {"name": "action_options", "description": "Allowed values for an action input on an entity.", "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({
             "action": {"type": "string", "description": "Action name, as list_actions reports it"},
             "input": {"type": "string", "description": "Name of one of the action's input properties"},
             "id": id}), &["action", "input", "id"])},
        {"name": "preview_action", "description": "Preview an action: validation, blockers and the approval path. Always preview before running.",
         "annotations": {"readOnlyHint": true},
         "inputSchema": s(json!({
             "action": {"type": "string", "description": "Action name, as list_actions reports it"},
             "id": id,
             "inputs": {"type": "object", "description": "Proposed input values, keyed by input property name"}}), &["action", "id"])}
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

/// What a refresh changed: the entity's changed fields, and every entity the caller may see whose
/// fields changed between the snapshot before the refresh and now (capped at 50 ids). Other writes
/// in that range (resyncs, other refreshes) are included, so the range is reported with them.
async fn changes_since(
    state: &Arc<AppState>,
    h: &HeaderMap,
    before: Option<&crate::state::Published>,
    id: &str,
) -> Value {
    let (Some(before), Some(after)) = (before, state.snapshot()) else {
        return Value::Null;
    };
    let b = &before.derived.entities;
    let a = &after.derived.entities;
    let mut changed: Vec<&str> = a
        .iter()
        .filter(|(k, e)| b.get(*k).is_none_or(|old| old.fields != e.fields))
        .map(|(k, _)| k.as_str())
        .chain(b.keys().filter(|k| !a.contains_key(*k)).map(String::as_str))
        .collect();
    changed.sort_unstable();
    let fields: Vec<&String> = match (b.get(id), a.get(id)) {
        (Some(o), Some(n)) => n
            .fields
            .keys()
            .chain(o.fields.keys())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|f| o.fields.get(*f) != n.fields.get(*f))
            .collect(),
        _ => vec![],
    };
    let visible: Vec<&str> = match state.caller(h).await {
        Ok(caller) => match state.visibility(&caller, &changed).await {
            Ok(vis) => changed
                .iter()
                .zip(vis)
                .filter(|(_, v)| v.view)
                .map(|(e, _)| *e)
                .collect(),
            Err(_) => vec![],
        },
        Err(_) => vec![],
    };
    json!({"from": before.snapshot_id, "to": after.snapshot_id, "fields": fields,
           "count": visible.len(), "entities": visible.iter().take(50).collect::<Vec<_>>()})
}

/// With `TOOLBOX_MCP_TRACE=<file>`, append one line per `tools/call` (`{"tool", "error"}`), so a
/// test run can report which tools were *observed* being called, not just mentioned (F9).
fn trace_call(tool: &str, error: bool) {
    trace_line(&json!({"tool": tool, "error": error}));
}

/// At server start (with `TOOLBOX_MCP_TRACE`): declare every built-in tool, so a tool that no test
/// calls shows up with zero calls instead of not at all.
pub fn trace_declare() {
    for t in builtin_tools().as_array().into_iter().flatten() {
        trace_line(&json!({"tool": t["name"], "declared": true}));
    }
}

fn trace_line(v: &Value) {
    let Some(path) = std::env::var_os("TOOLBOX_MCP_TRACE") else {
        return;
    };
    use std::io::Write as _;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{v}");
    }
}

async fn scorecard_report_call(
    state: &Arc<AppState>,
    h: &HeaderMap,
    a: &Value,
) -> (StatusCode, Value) {
    let card = arg(a, "scorecard");
    if card.is_empty() {
        return call_rest(state, h, Method::GET, "/v1/scorecards".into(), None).await;
    }
    let q: Vec<String> = ["rule", "outcome", "level"]
        .iter()
        .filter(|k| !arg(a, k).is_empty())
        .map(|k| format!("{k}={}", enc(arg(a, k))))
        .collect();
    call_rest(
        state,
        h,
        Method::GET,
        format!("/v1/scorecards/{}?{}", enc(card), q.join("&")),
        None,
    )
    .await
}

async fn campaign_report_call(
    state: &Arc<AppState>,
    h: &HeaderMap,
    a: &Value,
) -> (StatusCode, Value) {
    let name = arg(a, "campaign");
    if name.is_empty() {
        return call_rest(state, h, Method::GET, "/v1/campaigns".into(), None).await;
    }
    call_rest(
        state,
        h,
        Method::GET,
        format!("/v1/campaigns/{}", enc(name)),
        None,
    )
    .await
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
        "get" => {
            let mut uri = format!("/v1/entities/{id}");
            if !arg(a, "fields").is_empty() {
                uri += &format!("?fields={}", enc(arg(a, "fields")));
            }
            call_rest(state, h, Method::GET, uri, None).await
        }
        "scorecard_report" => scorecard_report_call(state, h, a).await,
        "campaign_report" => campaign_report_call(state, h, a).await,
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
            let (status, mut v) = call_rest(
                state,
                h,
                Method::GET,
                format!("/v1/entities/{id}/scorecards"),
                None,
            )
            .await;
            // `scorecard` narrows the structured result too, not just the Markdown.
            let only = arg(a, "scorecard");
            if !only.is_empty()
                && let Some(m) = v.get_mut("scorecards").and_then(Value::as_object_mut)
            {
                m.retain(|k, _| k == only);
            }
            (status, v)
        }
        "assessment_brief" => {
            let (st, e) =
                call_rest(state, h, Method::GET, format!("/v1/entities/{id}"), None).await;
            if !st.is_success() {
                (st, e)
            } else {
                match crate::assess::brief(&state.manifest, &e, a["rubric"].as_str()) {
                    Ok(b) => (st, b),
                    Err(msg) => return (msg, Value::Null, true),
                }
            }
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
            let before = state.snapshot();
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
                let changes = changes_since(state, h, before.as_deref(), arg(a, "id")).await;
                (
                    st,
                    json!({"entity": arg(a, "id"), "commands": done, "changes": changes}),
                )
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

fn write_campaign_report(s: &mut String, v: &Value) {
    use std::fmt::Write as _;
    if let Some(list) = v.get("campaigns").and_then(Value::as_array) {
        for c in list {
            let _ = writeln!(
                s,
                "- **{}**: {}% of {}% target ({}/{} meeting; {})",
                c["name"].as_str().unwrap_or(""),
                c["percent"],
                c["targetPercent"],
                c["meeting"],
                c["inScope"],
                c["status"].as_str().unwrap_or("")
            );
        }
        return;
    }
    let c = v.get("campaign").unwrap_or(v);
    let _ = writeln!(
        s,
        "**{}**: {}% of {}% target ({}/{} meeting; {})",
        c["name"].as_str().unwrap_or(""),
        c["percent"],
        c["targetPercent"],
        c["meeting"],
        c["inScope"],
        c["status"].as_str().unwrap_or("")
    );
    for e in c["entities"].as_array().into_iter().flatten() {
        if e["meeting"] == true {
            continue;
        }
        let gaps = e["gaps"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join("; ");
        let _ = writeln!(s, "- `{}`: {gaps}", e["id"].as_str().unwrap_or(""));
    }
}

/// A failing rule's evidence under its bullet: list items one per line (at most 20), other values inline.
fn write_evidence(s: &mut String, evidence: Option<&Value>) {
    use std::fmt::Write as _;
    match evidence {
        None | Some(Value::Null) => {}
        Some(Value::Array(items)) => {
            for item in items.iter().take(20) {
                let _ = writeln!(
                    s,
                    "  - {}",
                    item.as_str()
                        .map_or_else(|| item.to_string(), str::to_owned)
                );
            }
            if items.len() > 20 {
                let _ = writeln!(s, "  - … {} more", items.len() - 20);
            }
        }
        Some(v) => {
            let _ = writeln!(s, "  - evidence: {v}");
        }
    }
}

fn markdown(name: &str, a: &Value, v: &Value) -> String {
    let mut s = String::new();
    let snap = v["snapshot"]
        .as_str()
        .map(|x| format!("\n\n_as of snapshot {x}_"))
        .unwrap_or_default();
    match name {
        "assessment_brief" => return crate::assess::markdown(v),
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
                            write_evidence(&mut s, o.get("evidence"));
                        }
                    }
                    None => {}
                }
            }
        }
        "campaign_report" => write_campaign_report(&mut s, v),
        "scorecard_report" if v.get("scorecards").is_some() => {
            for c in v["scorecards"].as_array().into_iter().flatten() {
                let counts: Vec<String> = c["by_level"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .map(|(l, n)| format!("{l}: {n}"))
                    .collect();
                let _ = writeln!(
                    s,
                    "- **{}** ({}; levels {}): {}",
                    c["name"].as_str().unwrap_or(""),
                    c["kind"].as_str().unwrap_or(""),
                    c["levels"]
                        .as_array()
                        .map(|l| l
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(" < "))
                        .unwrap_or_default(),
                    counts.join(", ")
                );
            }
        }
        "scorecard_report" => {
            let counts: Vec<String> = v["by_level"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(l, n)| format!("{l}: {n}"))
                .collect();
            let _ = writeln!(
                s,
                "**{}**: {} entities ({})",
                v["scorecard"].as_str().unwrap_or(""),
                v["count"],
                counts.join(", ")
            );
            for e in v["entities"].as_array().into_iter().flatten() {
                let _ = writeln!(
                    s,
                    "- `{}`: {}",
                    e["id"].as_str().unwrap_or(""),
                    show(&e["level"])
                );
                if e["failing"].get("redacted").is_some() {
                    let _ = writeln!(s, "  - rule details: {}", show(&e["failing"]));
                    continue;
                }
                for (rule, o) in e["failing"].as_object().into_iter().flatten() {
                    let _ = writeln!(s, "  - {rule}: {}", o["outcome"].as_str().unwrap_or("?"));
                    if let Some(items) = o.get("evidence") {
                        let mut sub = String::new();
                        write_evidence(&mut sub, Some(items));
                        for line in sub.lines() {
                            let _ = writeln!(s, "  {line}");
                        }
                    }
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
                if v.get("evidence").is_some_and(|e| !e.is_null()) {
                    let _ = writeln!(s, "- evidence:");
                    write_evidence(&mut s, v.get("evidence"));
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
            let ch = &v["changes"];
            let fields: Vec<&str> = ch["fields"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            let _ = writeln!(
                s,
                "`{}`: {} (snapshot {} → {})",
                v["entity"].as_str().unwrap_or(""),
                if fields.is_empty() {
                    "no field changed".to_owned()
                } else {
                    format!("changed {}", fields.join(", "))
                },
                ch["from"].as_str().unwrap_or("?"),
                ch["to"].as_str().unwrap_or("?")
            );
            let others: Vec<&str> = ch["entities"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter(|e| Some(*e) != v["entity"].as_str())
                .collect();
            if !others.is_empty() {
                let _ = writeln!(
                    s,
                    "- {} other entit{} changed in that range (refreshes, resyncs or other writes): {}{}",
                    ch["count"]
                        .as_u64()
                        .unwrap_or(0)
                        .saturating_sub(u64::from(!fields.is_empty())),
                    if others.len() == 1 { "y" } else { "ies" },
                    others
                        .iter()
                        .take(10)
                        .map(|e| format!("`{e}`"))
                        .collect::<Vec<_>>()
                        .join(", "),
                    if others.len() > 10 { ", …" } else { "" }
                );
            }
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
        "tools/list" => Ok(json!({"tools": builtin_tools()})),
        "tools/call" => {
            let name = msg["params"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_owned();
            let args = msg["params"]["arguments"].clone();
            let (text, structured, is_error) = run_tool(&state, &headers, &name, &args).await;
            trace_call(&name, is_error);
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
