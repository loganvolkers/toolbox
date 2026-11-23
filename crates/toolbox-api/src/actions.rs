//! Actions, M3 slice (docs/design/action-runs.md §3): the action list with permissions, and the
//! Toolbox layer of the preview: authorization (A10), input validation against the schema and
//! the per-entity options, preconditions, the approval path and context lookups. Executor
//! previews, confirmation tokens and runs arrive with M5.

use crate::routes::problem;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use toolbox_expr::{Expr, ExprValue, with_vars};

#[derive(Deserialize)]
pub struct ListQuery {
    entity: Option<String>,
}

/// `GET /v1/actions?entity=` — actions on the entity's kind, with whether the caller may run them.
pub async fn list(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
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
    let kind = match &q.entity {
        Some(id) => match p.derived.entities.get(id) {
            Some(e) => Some(e.kind.clone()),
            None => {
                return problem(
                    StatusCode::NOT_FOUND,
                    "not-found",
                    format!("no entity {id}"),
                );
            }
        },
        None => None,
    };
    let mut out = Vec::new();
    for a in &s.manifest.actions {
        if kind.as_ref().is_some_and(|k| *k != a.kind) {
            continue;
        }
        let allowed = match &q.entity {
            Some(id) => match s.allowed(&caller, &a.permission, id).await {
                Ok(b) => Some(b),
                Err(r) => return r,
            },
            None => None,
        };
        out.push(
            json!({"name": a.name, "kind": a.kind, "scope": a.scope, "description": a.description,
                        "permission": a.permission, "allowed": allowed, "inputs": a.inputs,
                        "annotations": a.annotations, "verified": a.expect.is_some()}),
        );
    }
    Json(json!({"snapshot": p.snapshot_id, "actions": out})).into_response()
}

#[derive(Deserialize)]
pub struct PreviewBody {
    entity: String,
    #[serde(default)]
    inputs: serde_json::Map<String, Value>,
}

/// `POST /v1/actions/{name}/previews` — the Toolbox layer of a preview.
pub async fn preview(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(body): Json<PreviewBody>,
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
    let Some(action) = s.manifest.actions.iter().find(|a| a.name == name) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no action {name}"),
        );
    };
    let Some(entity) = p.derived.entities.get(&body.entity) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {}", body.entity),
        );
    };
    if entity.kind != action.kind {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "wrong-kind",
            format!("{name} applies to {}, not {}", action.kind, entity.kind),
        );
    }
    match s.allowed(&caller, &action.permission, &body.entity).await {
        Ok(true) => {}
        Ok(false) => {
            let mut r = problem(
                StatusCode::FORBIDDEN,
                "forbidden",
                format!(
                    "{} lacks `{}` on {}",
                    caller.user, action.permission, body.entity
                ),
            );
            if let Ok(mut v) = serde_json::to_value(
                json!({"relation": action.permission, "object": body.entity, "user": caller.user}),
            ) {
                v["status"] = 403.into();
            }
            r.headers_mut().insert(
                "x-toolbox-missing-relation",
                action.permission.parse().unwrap(),
            );
            return r;
        }
        Err(r) => return r,
    }

    // Inputs: required, types, minLength, and per-entity options for enumFrom inputs.
    let mut errors = Vec::new();
    let props = action
        .inputs
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for req in action
        .inputs
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !body.inputs.contains_key(req) {
            errors.push(format!("{req} is required"));
        }
    }
    for (k, v) in &body.inputs {
        let Some(spec) = props.get(k) else {
            errors.push(format!("{k} is not an input of {name}"));
            continue;
        };
        let ty = spec.get("type").and_then(Value::as_str).unwrap_or("string");
        let type_ok = match ty {
            "string" => v.is_string(),
            "integer" => v.is_i64() || v.is_u64(),
            "number" => v.is_number(),
            "boolean" => v.is_boolean(),
            "array" => v.is_array(),
            "object" => v.is_object(),
            _ => true,
        };
        if !type_ok {
            errors.push(format!("{k} must be a {ty}"));
            continue;
        }
        if let (Some(min), Some(sv)) = (spec.get("minLength").and_then(Value::as_u64), v.as_str())
            && (sv.chars().count() as u64) < min
        {
            errors.push(format!("{k} must be at least {min} characters"));
        }
        if spec.get("enumFrom").is_some() {
            let opts = p
                .derived
                .options
                .get(&name)
                .and_then(|i| i.get(k))
                .and_then(|e| e.get(&body.entity));
            if !opts.is_some_and(|o| v.as_str().is_some_and(|sv| o.iter().any(|x| x == sv))) {
                errors.push(format!(
                    "{k}={v} is not an option for {} (options: {:?})",
                    body.entity,
                    opts.cloned().unwrap_or_default()
                ));
            }
        }
        if let Some(allowed) = spec.get("enum").and_then(Value::as_array)
            && !allowed.contains(v)
        {
            errors.push(format!("{k}={v} is not one of {allowed:?}"));
        }
    }
    if !errors.is_empty() {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid-inputs",
            errors.join("; "),
        );
    }

    // Preconditions, approval path and context lookups, all over the same snapshot.
    let entity_cel = ExprValue::from_json(&p.derived.entity_json(&body.entity).unwrap());
    let inputs_cel = ExprValue::from_json(&Value::Object(body.inputs.clone()));
    let policy = ExprValue::from_json(&Value::Object(s.manifest.policies.clone()));
    let eval = |src: &str| -> Result<Value, String> {
        let e = Expr::compile(src)?;
        with_vars(
            &[
                ("entity", &entity_cel),
                ("inputs", &inputs_cel),
                ("policy", &policy),
            ],
            |ctx| e.json_in(ctx),
        )
    };
    let mut blockers = Vec::new();
    for pre in &action.preconditions {
        match eval(pre) {
            Ok(Value::Bool(true)) => {}
            Ok(_) => blockers.push(format!("precondition not met: {pre}")),
            Err(e) => blockers.push(format!("precondition failed to evaluate: {pre}: {e}")),
        }
    }
    let approvers = match action.approval.as_deref().map(eval) {
        Some(Ok(Value::Array(a))) => a,
        Some(Ok(other)) => vec![other],
        Some(Err(e)) => {
            blockers.push(format!("approval rule failed to evaluate: {e}"));
            vec![]
        }
        None => vec![],
    };
    let mut related = serde_json::Map::new();
    for (lname, lk) in &action.context {
        let Ok(where_) = Expr::compile(&lk.where_) else {
            continue;
        };
        let matches: Vec<Value> = p
            .derived
            .entities
            .iter()
            .filter(|(_, e)| e.kind == lk.kind)
            .filter(|(cid, _)| {
                let cand = ExprValue::from_json(&p.derived.entity_json(cid).unwrap());
                with_vars(
                    &[
                        ("candidate", &cand),
                        ("entity", &entity_cel),
                        ("inputs", &inputs_cel),
                        ("policy", &policy),
                    ],
                    |ctx| matches!(where_.eval_in(ctx), Ok(v) if v.as_bool() == Some(true)),
                )
            })
            .map(|(cid, _)| p.derived.entity_json(cid).unwrap())
            .collect();
        related.insert(lname.clone(), Value::Array(matches));
    }
    Json(json!({
        "action": name, "entity": body.entity, "snapshot": p.snapshot_id, "requested_by": caller.user,
        "inputs": body.inputs, "blockers": blockers, "warnings": [],
        "approval": {"required": !approvers.is_empty(), "approvers": approvers},
        "related": related,
        "effects": [], "executor_preview": "not available until M5", "confirm_token": null,
    }))
    .into_response()
}
