//! REST API (docs/v1-spec.md §7.2), M2 subset: entities, scorecards, native writes, ingest,
//! snapshot info, health, metrics and the operator export used for differential checks.

use crate::state::{AppState, Published, parse_min_snapshot};
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use toolbox_expr::{Expr, ExprValue, with_vars};
use toolbox_store::NativeValue;

type S = State<Arc<AppState>>;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
        .route("/v1/snapshot", get(snapshot_info))
        .route("/v1/entities", get(list_entities))
        .route("/v1/entities/{id}", get(get_entity))
        .route("/v1/entities/{id}/scorecards", get(entity_scorecards))
        .route("/v1/entities/{id}/native/{field}", put(put_native))
        .route("/v1/ingest", post(ingest))
        .route("/v1/ops/derived", get(ops_derived))
        .route("/v1/me", get(me))
        .route("/v1/connectors", get(crate::connectors::list_connectors))
        .route(
            "/v1/connectors/{name}/commands",
            get(crate::connectors::poll_commands),
        )
        .route(
            "/v1/entities/{id}/refresh",
            post(crate::connectors::refresh_entity),
        )
        .route("/v1/commands/{id}", get(crate::connectors::command_status))
        .route("/v1/entities/{id}/graph", get(crate::reads::graph))
        .route("/v1/entities/{id}/explain", get(crate::reads::explain))
        .route(
            "/v1/actions/{name}/options/{input}",
            get(crate::reads::options),
        )
        .route("/v1/views/home", get(crate::reads::view_home))
        .route("/v1/views/entity/{id}", get(crate::reads::view_entity))
        .route(
            "/v1/views/scorecard/{name}",
            get(crate::reads::view_scorecard),
        )
        .route(
            "/v1/views/campaign/{name}",
            get(crate::reads::view_campaign),
        )
        .route("/v1/views/ops", get(crate::reads::view_ops))
        .route("/v1/scorecards", get(crate::reads::scorecards))
        .route(
            "/v1/scorecards/{name}",
            get(crate::reads::scorecard_results),
        )
        .route("/v1/campaigns", get(crate::reads::campaigns))
        .route("/v1/campaigns/{name}", get(crate::reads::campaign_results))
        .route("/mcp", post(crate::mcp::handle))
        .route("/e/{id}", get(entity_page))
        .fallback(crate::ui::asset)
        .route("/v1/actions", get(crate::actions::list))
        .route("/v1/actions/{name}/previews", post(crate::actions::preview))
        .layer(axum::middleware::from_fn(crate::http_trace::layer))
        .with_state(state)
}

/// RFC 9457 problem details.
pub fn problem(status: StatusCode, code: &str, detail: impl Into<String>) -> Response {
    let body = json!({"type": format!("https://toolbox.dev/problems/{code}"), "title": code, "status": status.as_u16(),
                      "code": code, "detail": detail.into()});
    let mut r = (status, Json(body)).into_response();
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    r
}

fn with_snapshot(p: &Published, body: Value) -> Response {
    let mut r = Json(body).into_response();
    if let Ok(v) = HeaderValue::from_str(&p.snapshot_id) {
        r.headers_mut().insert("x-toolbox-snapshot", v);
    }
    r
}

/// The snapshot to answer from, honouring `min_snapshot` (read-your-writes, P8).
async fn snapshot_for(state: &AppState, min: Option<&str>) -> Result<Arc<Published>, Response> {
    let wanted = match min {
        Some(s) => Some(parse_min_snapshot(s).ok_or_else(|| {
            problem(
                StatusCode::BAD_REQUEST,
                "bad-min-snapshot",
                "min_snapshot must be <version> or <version>.<hash>",
            )
        })?),
        None => None,
    };
    match wanted {
        Some(v) => state
            .wait_for(v, Duration::from_secs(10))
            .await
            .ok_or_else(|| {
                let cur = state
                    .snapshot()
                    .map(|p| p.snapshot_id.clone())
                    .unwrap_or_default();
                problem(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "snapshot-not-ready",
                    format!("snapshot {v} not published within 10 s (current {cur})"),
                )
            }),
        None => state.snapshot().ok_or_else(|| {
            problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "starting",
                "no snapshot yet",
            )
        }),
    }
}

async fn readyz(State(s): S) -> Response {
    match s.snapshot() {
        Some(p) => Json(json!({"ready": true, "snapshot": p.snapshot_id})).into_response(),
        None => problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "waiting for the first snapshot",
        ),
    }
}

async fn metrics(State(s): S) -> impl IntoResponse {
    let lag = match s.snapshot() {
        Some(p) => p
            .published_at
            .elapsed()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0),
        None => 0.0,
    };
    toolbox_telemetry::catalog::set_snapshot_lag(lag);
    let text = toolbox_telemetry::metrics_text();
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], text)
}

#[derive(Deserialize)]
struct SnapQuery {
    min_snapshot: Option<String>,
}

async fn snapshot_info(State(s): S, Query(q): Query<SnapQuery>) -> Response {
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let body = json!({"snapshot": p.snapshot_id, "input_version": p.input_version,
        "manifest_hash": s.manifest_hash, "recompute_ms": p.recompute.as_millis() as u64,
        "entities": p.derived.entities.len(), "tuples": p.derived.tuples.len(),
        "orphans": p.derived.orphans, "diagnostics": p.derived.diagnostics});
    with_snapshot(&p, body)
}

#[derive(Deserialize)]
struct ListQuery {
    kind: Option<String>,
    filter: Option<String>,
    fields: Option<String>,
    cursor: Option<String>,
    limit: Option<usize>,
    q: Option<String>,
    min_snapshot: Option<String>,
}

async fn list_entities(State(s): S, headers: HeaderMap, Query(q): Query<ListQuery>) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let filter = match q.filter.as_deref().map(Expr::compile).transpose() {
        Ok(f) => f,
        Err(e) => {
            return problem(
                StatusCode::BAD_REQUEST,
                "bad-filter",
                format!("filter is not valid CEL: {e}"),
            );
        }
    };
    let fields: Option<Vec<&str>> = q
        .fields
        .as_deref()
        .map(|f| f.split(',').map(str::trim).collect());
    let limit = q.limit.unwrap_or(100).clamp(1, 1000);
    let policy = ExprValue::from_json(&Value::Object(s.manifest.policies.clone()));
    let needle = q.q.as_deref().map(str::to_lowercase);
    // Candidates in id order from the cursor; visibility is checked in batches; filters see only
    // what the caller may see, so a filter can't probe hidden facets.
    let lower = q
        .cursor
        .as_ref()
        .map_or(std::ops::Bound::Unbounded, std::ops::Bound::Excluded);
    let candidates: Vec<&String> = p
        .derived
        .entities
        .range::<String, _>((lower, std::ops::Bound::Unbounded))
        .filter(|(_, e)| {
            q.kind
                .as_deref()
                .is_none_or(|k| k.eq_ignore_ascii_case(&e.kind))
        })
        .filter(|(id, e)| {
            needle.as_ref().is_none_or(|n| {
                let title = e
                    .fields
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_lowercase();
                id.to_lowercase().contains(n) || title.contains(n)
            })
        })
        .map(|(id, _)| id)
        .collect();
    let truthy = ExprValue::from_json(&Value::Bool(true));
    let wants_scores = q.filter.as_deref().is_some_and(|f| f.contains("scores"));
    let mut items: Vec<Value> = Vec::new();
    let mut next = None;
    'outer: for chunk in candidates.chunks(200) {
        let ids: Vec<&str> = chunk.iter().map(|s| s.as_str()).collect();
        let vis = match s.visibility(&caller, &ids).await {
            Ok(v) => v,
            Err(r) => return r,
        };
        for (id, v) in ids.iter().zip(vis) {
            if !v.view {
                continue;
            }
            if let Some(f) = &filter {
                let shaped = s.shape_entity(&p, &caller, id, v, None).unwrap();
                let mut visible = shaped["fields"].clone();
                visible["id"] = json!(id);
                let entity = ExprValue::from_json(&visible);
                // `scores['<scorecard>']` = {level, rules: {rule: outcome}}, as the caller may see it.
                let scores = if wants_scores {
                    ExprValue::from_json(&score_outcomes(s.shape_scores(&p, &caller, id, v)))
                } else {
                    ExprValue::from_json(&json!({}))
                };
                match with_vars(
                    &[
                        ("entity", &entity),
                        ("policy", &policy),
                        ("scores", &scores),
                    ],
                    |ctx| f.eval_in(ctx),
                ) {
                    Ok(b) if b == truthy => {}
                    _ => continue,
                }
            }
            if items.len() == limit {
                next = items
                    .last()
                    .and_then(|v| v["id"].as_str().map(str::to_owned));
                break 'outer;
            }
            items.push(
                s.shape_entity(&p, &caller, id, v, fields.as_deref())
                    .unwrap(),
            );
        }
    }
    with_snapshot(
        &p,
        json!({"snapshot": p.snapshot_id, "items": items, "next_cursor": next}),
    )
}

#[derive(Deserialize)]
struct GetQuery {
    min_snapshot: Option<String>,
    /// Comma-separated fields to return (default: every visible field).
    fields: Option<String>,
}

async fn get_entity(
    State(s): S,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<GetQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let vis = match s.visibility(&caller, &[id.as_str()]).await {
        Ok(v) => v[0],
        Err(r) => return r,
    };
    let fields: Option<Vec<&str>> = q
        .fields
        .as_deref()
        .map(|f| f.split(',').map(str::trim).collect());
    match s
        .shape_entity(&p, &caller, &id, vis, fields.as_deref())
        .filter(|_| vis.view)
    {
        Some(mut body) => {
            body["inbound"] = json!(p.derived.entities[&id].inbound);
            let mut outbound: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (rel, target) in crate::reads::outgoing(&s, &p, &id) {
                outbound.entry(rel).or_default().push(target);
            }
            body["outbound"] = json!(outbound);
            body["snapshot"] = json!(p.snapshot_id);
            with_snapshot(&p, body)
        }
        None => problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        ),
    }
}

#[derive(Deserialize)]
struct PageQuery {
    #[serde(rename = "as")]
    as_user: Option<String>,
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `GET /e/{id}` — the entity page as HTML, so the document itself names the entity and its
/// owner. The SPA still hydrates `#root`. A shell with neither is what an empty index is.
async fn entity_page(
    State(s): S,
    mut headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<PageQuery>,
) -> Response {
    if headers.get("x-toolbox-user").is_none()
        && let Some(user) = q.as_user.as_deref().filter(|u| !u.is_empty())
        && let Ok(value) = HeaderValue::from_str(user)
    {
        headers.insert("x-toolbox-user", value);
    }
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
    let vis = match s.visibility(&caller, &[id.as_str()]).await {
        Ok(v) => v[0],
        Err(r) => return r,
    };
    let Some(entity) = p.derived.entities.get(&id).filter(|_| vis.view) else {
        return problem(StatusCode::NOT_FOUND, "not-found", "no entity");
    };
    let title = entity
        .fields
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(id.as_str());
    let owner = entity
        .fields
        .get("owner")
        .and_then(Value::as_str)
        .unwrap_or("");
    let body = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>{title}</title>\
         <script type=\"module\" src=\"/assets/app.js\"></script>\
         <link rel=\"stylesheet\" href=\"/assets/app.css\"></head><body>\
         <header><p>{title}</p><p>Owner: {owner}</p></header><div id=\"root\"></div></body></html>",
        title = html_escape(title),
        owner = html_escape(owner),
    );
    (
        [
            (header::CONTENT_TYPE, "text/html; charset=utf-8"),
            (header::CONTENT_SECURITY_POLICY, crate::ui::CSP),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        body,
    )
        .into_response()
}

async fn entity_scorecards(
    State(s): S,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<SnapQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let vis = match s.visibility(&caller, &[id.as_str()]).await {
        Ok(v) => v[0],
        Err(r) => return r,
    };
    if !p.derived.entities.contains_key(&id) || !vis.view {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    }
    let cards = s.shape_scores(&p, &caller, &id, vis);
    with_snapshot(
        &p,
        json!({"entity": id, "snapshot": p.snapshot_id, "scorecards": cards}),
    )
}

#[derive(Deserialize)]
struct NativeBody {
    value: Value,
    reason: Option<String>,
}

async fn put_native(
    State(s): S,
    Path((id, field)): Path<(String, String)>,
    headers: HeaderMap,
    Json(body): Json<NativeBody>,
) -> Response {
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
    let kind = s
        .manifest
        .kinds
        .iter()
        .find(|k| k.name == e.kind)
        .expect("entity kinds come from the manifest");
    let sources = kind
        .sources
        .fields
        .get(&field)
        .unwrap_or(&kind.sources.default);
    if !sources.iter().any(|x| x == "native") {
        return problem(
            StatusCode::CONFLICT,
            "not-native",
            format!(
                "{}.{field} is not a native field; its sources are {sources:?}",
                e.kind
            ),
        );
    }
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let may = caller.admin || matches!(s.allowed(&caller, "can_edit", &id).await, Ok(true));
    if !may {
        return problem(
            StatusCode::FORBIDDEN,
            "forbidden",
            format!(
                "{} may not edit native fields of {id} (needs `can_edit` or portal admin)",
                caller.user
            ),
        );
    }
    let revision = headers
        .get(header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim_matches('"').parse::<u64>().ok());
    let author = Some(caller.user.clone());
    let write = NativeValue {
        entity: id,
        field,
        value: body.value,
        revision: revision.map_or(0, |r| r + 1),
        author,
        at: None,
        reason: body.reason,
    };
    match s.store.put_native(vec![write]).await {
        Ok(version) => {
            s.changed.notify_one();
            (
                StatusCode::ACCEPTED,
                Json(json!({"input_version": version})),
            )
                .into_response()
        }
        Err(toolbox_store::StoreError::Conflict(m)) => {
            problem(StatusCode::PRECONDITION_FAILED, "revision-conflict", m)
        }
        Err(e) => problem(StatusCode::INTERNAL_SERVER_ERROR, "store", e.to_string()),
    }
}

/// The connector a request comes from: a configured bearer token, or (dev mode) a header.
pub fn connector_of(s: &AppState, headers: &HeaderMap) -> Option<String> {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    if let Some(tok) = bearer
        && let Some((name, _)) = s.config.connector_tokens.iter().find(|(_, t)| t == tok)
    {
        return Some(name.clone());
    }
    if s.config.auth_mode == "dev" {
        return headers
            .get("x-toolbox-connector")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
    }
    None
}

async fn ingest(State(s): S, headers: HeaderMap, body: Bytes) -> Response {
    let Some(connector) = connector_of(&s, &headers) else {
        return problem(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "ingest needs a connector token",
        );
    };
    let text = String::from_utf8_lossy(&body);
    let messages: Vec<Value> = if text.trim_start().starts_with('[') {
        match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => return problem(StatusCode::BAD_REQUEST, "bad-json", e.to_string()),
        }
    } else {
        let mut out = Vec::new();
        for (i, line) in text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
        {
            match serde_json::from_str(line) {
                Ok(v) => out.push(v),
                Err(e) => {
                    return problem(
                        StatusCode::BAD_REQUEST,
                        "bad-json",
                        format!("line {}: {e}", i + 1),
                    );
                }
            }
        }
        out
    };
    let n = messages.len();
    match crate::connectors::ingest_messages(&s, &connector, messages).await {
        Ok(version) => (
            StatusCode::ACCEPTED,
            Json(json!({"input_version": version, "accepted": n})),
        )
            .into_response(),
        Err(r) => r,
    }
}

/// The reference-format outputs of the current snapshot (operator export for differential checks).
async fn ops_derived(State(s): S, headers: HeaderMap, Query(q): Query<SnapQuery>) -> Response {
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
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    let files: Map<String, Value> = p
        .derived
        .reference_files()
        .into_iter()
        .map(|(k, v)| (k.to_owned(), Value::String(v)))
        .collect();
    with_snapshot(&p, json!({"snapshot": p.snapshot_id, "files": files}))
}

async fn me(State(s): S, headers: HeaderMap) -> Response {
    match s.caller(&headers).await {
        Ok(c) => Json(
            json!({"user": c.user, "subject": c.subject, "method": c.method, "admin": c.admin}),
        )
        .into_response(),
        Err(r) => r,
    }
}

/// Shaped scorecards reduced to `{card: {level, rules: {rule: outcome}}}` for filters (a redacted
/// part stays a redaction marker, so filters can't probe it).
pub(crate) fn score_outcomes(shaped: serde_json::Map<String, Value>) -> Value {
    let mut out = serde_json::Map::new();
    for (card, c) in shaped {
        let rules = match c.get("rules") {
            Some(r) if r.get("redacted").is_some() => r.clone(),
            Some(r) => Value::Object(
                r.as_object()
                    .into_iter()
                    .flatten()
                    .map(|(k, o)| (k.clone(), o["outcome"].clone()))
                    .collect(),
            ),
            None => json!({}),
        };
        out.insert(card, json!({"level": c["level"], "rules": rules}));
    }
    Value::Object(out)
}
