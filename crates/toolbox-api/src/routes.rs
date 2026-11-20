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
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use toolbox_expr::{Expr, to_cel, with_vars};
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

fn entity_body(p: &Published, id: &str, fields: Option<&[&str]>) -> Option<Value> {
    let e = p.derived.entities.get(id)?;
    let pick = |m: &Map<String, Value>| -> Map<String, Value> {
        match fields {
            Some(fs) => m
                .iter()
                .filter(|(k, _)| fs.contains(&k.as_str()))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            None => m.clone(),
        }
    };
    Some(json!({"id": id, "kind": e.kind, "fields": pick(&e.fields), "provenance": e.provenance}))
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
    let m = &s.metrics;
    let (lag, last) = match s.snapshot() {
        Some(p) => (
            p.published_at
                .elapsed()
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            p.recompute.as_secs_f64(),
        ),
        None => (0.0, 0.0),
    };
    let text = format!(
        "# TYPE toolbox_ingest_records_total counter\ntoolbox_ingest_records_total {}\n\
         # TYPE toolbox_ingest_batches_total counter\ntoolbox_ingest_batches_total {}\n\
         # TYPE toolbox_recomputes_total counter\ntoolbox_recomputes_total {}\n\
         # TYPE toolbox_recompute_seconds gauge\ntoolbox_recompute_seconds {last}\n\
         # TYPE toolbox_snapshot_age_seconds gauge\ntoolbox_snapshot_age_seconds {lag}\n",
        m.ingest_records.load(Ordering::Relaxed),
        m.ingest_batches.load(Ordering::Relaxed),
        m.recomputes.load(Ordering::Relaxed),
    );
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

async fn list_entities(State(s): S, Query(q): Query<ListQuery>) -> Response {
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
    let policy = to_cel(&Value::Object(s.manifest.policies.clone()));
    let needle = q.q.as_deref().map(str::to_lowercase);
    let mut items = Vec::new();
    let mut next = None;
    for (id, e) in p.derived.entities.range::<String, _>((
        q.cursor
            .as_ref()
            .map_or(std::ops::Bound::Unbounded, std::ops::Bound::Excluded),
        std::ops::Bound::Unbounded,
    )) {
        if q.kind
            .as_deref()
            .is_some_and(|k| !k.eq_ignore_ascii_case(&e.kind))
        {
            continue;
        }
        if let Some(n) = &needle {
            let title = e
                .fields
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_lowercase();
            if !id.to_lowercase().contains(n) && !title.contains(n) {
                continue;
            }
        }
        if let Some(f) = &filter {
            let entity = to_cel(&p.derived.entity_json(id).unwrap());
            match with_vars(&[("entity", &entity), ("policy", &policy)], |ctx| {
                f.eval_in(ctx)
            }) {
                Ok(cel_bool) if cel_bool == toolbox_expr::to_cel(&Value::Bool(true)) => {}
                Ok(_) => continue,
                Err(_) => continue, // a filter that errors on an entity doesn't match it
            }
        }
        if items.len() == limit {
            next = items
                .last()
                .and_then(|v: &Value| v["id"].as_str().map(str::to_owned));
            break;
        }
        items.push(entity_body(&p, id, fields.as_deref()).unwrap());
    }
    with_snapshot(
        &p,
        json!({"snapshot": p.snapshot_id, "items": items, "next_cursor": next}),
    )
}

async fn get_entity(State(s): S, Path(id): Path<String>, Query(q): Query<SnapQuery>) -> Response {
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    match entity_body(&p, &id, None) {
        Some(mut body) => {
            body["inbound"] = json!(p.derived.entities[&id].inbound);
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

async fn entity_scorecards(
    State(s): S,
    Path(id): Path<String>,
    Query(q): Query<SnapQuery>,
) -> Response {
    let p = match snapshot_for(&s, q.min_snapshot.as_deref()).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    if !p.derived.entities.contains_key(&id) {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    }
    let cards: HashMap<&String, &toolbox_derive::Score> = p
        .derived
        .scores
        .iter()
        .filter_map(|(card, by)| by.get(&id).map(|sc| (card, sc)))
        .collect();
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
    let revision = headers
        .get(header::IF_MATCH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim_matches('"').parse::<u64>().ok());
    let author = headers
        .get("x-toolbox-user")
        .and_then(|v| v.to_str().ok())
        .map(|u| format!("user:{u}"));
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
fn connector_of(s: &AppState, headers: &HeaderMap) -> Option<String> {
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
    let Some(declared) = s.manifest.connectors.iter().find(|c| c.name == connector) else {
        return problem(
            StatusCode::FORBIDDEN,
            "unknown-connector",
            format!("connector {connector} is not declared in the manifest"),
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
    let mut errors = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        for (ptr, msg) in
            toolbox_model::schema::errors(toolbox_model::schema::connector_messages(), m)
        {
            errors.push(format!("message {i}{ptr}: {msg}"));
        }
        if m.get("type").and_then(Value::as_str) == Some("record") {
            let kind = m.get("kind").and_then(Value::as_str).unwrap_or_default();
            if !declared.records.iter().any(|r| r == kind) {
                errors.push(format!(
                    "message {i}: record kind {kind} is not declared by connector {connector}"
                ));
            }
        }
    }
    if !errors.is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid-messages",
            errors.join("; "),
        );
    }
    let records = messages
        .iter()
        .filter(|m| m.get("type").and_then(Value::as_str) == Some("record"))
        .count();
    match s.store.ingest(&connector, &messages).await {
        Ok(version) => {
            s.metrics
                .ingest_records
                .fetch_add(records as u64, Ordering::Relaxed);
            s.metrics.ingest_batches.fetch_add(1, Ordering::Relaxed);
            s.changed.notify_one();
            (
                StatusCode::ACCEPTED,
                Json(json!({"input_version": version, "accepted": messages.len()})),
            )
                .into_response()
        }
        Err(e) => problem(StatusCode::INTERNAL_SERVER_ERROR, "store", e.to_string()),
    }
}

/// The reference-format outputs of the current snapshot (operator export for differential checks).
async fn ops_derived(State(s): S, Query(q): Query<SnapQuery>) -> Response {
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
