//! The Toolbox layer of a preview (docs/design/action-runs.md §3): input validation against the
//! schema and the per-entity options, preconditions, the approval path and context lookups, all
//! over one snapshot. Authorization is the caller's: a server checks OpenFGA, a repo portal has
//! none (docs/design/repo-portals.md §2).

use serde_json::{Map, Value};
use toolbox_derive::Derived;
use toolbox_expr::{Expr, ExprValue, with_vars};
use toolbox_model::manifest::Action;

/// What the Toolbox layer found.
#[derive(Debug, Clone, Default)]
pub struct Checked {
    /// Input errors: the run cannot be previewed until they are fixed.
    pub errors: Vec<String>,
    /// Unmet or failing preconditions and approval rules: previewable, not runnable.
    pub blockers: Vec<String>,
    /// Who must approve (`group:security`, `user:alice`); empty when none.
    pub approvers: Vec<Value>,
    /// The action's `context` lookups: name → matching entities.
    pub related: Map<String, Value>,
    /// The target entity as CEL sees it.
    pub entity: Value,
}

/// Fill schema defaults for inputs the caller left out.
pub fn with_defaults(action: &Action, inputs: &Map<String, Value>) -> Map<String, Value> {
    let mut out = inputs.clone();
    if let Some(props) = action.inputs.get("properties").and_then(Value::as_object) {
        for (k, spec) in props {
            if let Some(d) = spec.get("default")
                && !out.contains_key(k)
            {
                out.insert(k.clone(), d.clone());
            }
        }
    }
    out
}

fn type_ok(ty: &str, v: &Value) -> bool {
    match ty {
        "string" => v.is_string(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "boolean" => v.is_boolean(),
        "array" => v.is_array(),
        "object" => v.is_object(),
        _ => true,
    }
}

/// Validate inputs: required, types, `minLength`/`maxLength`, `minimum`/`maximum`, `enum`, and
/// `enumFrom` options computed for this entity.
pub fn validate_inputs(
    action: &Action,
    derived: &Derived,
    entity: &str,
    inputs: &Map<String, Value>,
) -> Vec<String> {
    let name = &action.name;
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
        if !inputs.contains_key(req) {
            errors.push(format!("{req} is required"));
        }
    }
    for (k, v) in inputs {
        let Some(spec) = props.get(k) else {
            errors.push(format!(
                "{k} is not an input of {name} (inputs: {})",
                props.keys().cloned().collect::<Vec<_>>().join(", ")
            ));
            continue;
        };
        let ty = spec.get("type").and_then(Value::as_str).unwrap_or("string");
        if !type_ok(ty, v) {
            errors.push(format!("{k} must be a {ty}"));
            continue;
        }
        if let (Some(min), Some(sv)) = (spec.get("minLength").and_then(Value::as_u64), v.as_str())
            && (sv.chars().count() as u64) < min
        {
            errors.push(format!("{k} must be at least {min} characters"));
        }
        if let (Some(max), Some(sv)) = (spec.get("maxLength").and_then(Value::as_u64), v.as_str())
            && (sv.chars().count() as u64) > max
        {
            errors.push(format!("{k} must be at most {max} characters"));
        }
        if let (Some(min), Some(n)) = (spec.get("minimum").and_then(Value::as_f64), v.as_f64())
            && n < min
        {
            errors.push(format!("{k} must be at least {min}"));
        }
        if let (Some(max), Some(n)) = (spec.get("maximum").and_then(Value::as_f64), v.as_f64())
            && n > max
        {
            errors.push(format!("{k} must be at most {max}"));
        }
        if spec.get("enumFrom").is_some() {
            let opts = derived
                .options
                .get(name)
                .and_then(|i| i.get(k))
                .and_then(|e| e.get(entity));
            if !opts.is_some_and(|o| v.as_str().is_some_and(|sv| o.iter().any(|x| x == sv))) {
                errors.push(format!(
                    "{k}={v} is not an option for {entity} (options: {:?})",
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
    errors
}

/// Evaluate a CEL expression over `entity`, `inputs`, `policy` and, after a run, `outputs`.
pub fn eval(
    src: &str,
    entity: &Value,
    inputs: &Map<String, Value>,
    policies: &Map<String, Value>,
    outputs: Option<&Value>,
) -> Result<Value, String> {
    let e = Expr::compile(src)?;
    let entity = ExprValue::from_json(entity);
    let inputs = ExprValue::from_json(&Value::Object(inputs.clone()));
    let policy = ExprValue::from_json(&Value::Object(policies.clone()));
    let outputs = ExprValue::from_json(outputs.unwrap_or(&Value::Object(Map::new())));
    with_vars(
        &[
            ("entity", &entity),
            ("inputs", &inputs),
            ("policy", &policy),
            ("outputs", &outputs),
        ],
        |ctx| e.json_in(ctx),
    )
}

/// The whole Toolbox layer for one action, entity and inputs. `Err` when the entity is missing or
/// of the wrong kind.
pub fn check(
    action: &Action,
    derived: &Derived,
    policies: &Map<String, Value>,
    entity_id: &str,
    inputs: &Map<String, Value>,
) -> Result<Checked, String> {
    let Some(e) = derived.entities.get(entity_id) else {
        let of_kind: Vec<&String> = derived
            .entities
            .iter()
            .filter(|(_, e)| e.kind == action.kind)
            .map(|(id, _)| id)
            .take(10)
            .collect();
        return Err(format!(
            "no entity {entity_id} (entities of kind {}: {of_kind:?})",
            action.kind
        ));
    };
    if e.kind != action.kind {
        return Err(format!(
            "{} applies to {}, not {}",
            action.name, action.kind, e.kind
        ));
    }
    let entity = derived.entity_json(entity_id).unwrap_or(Value::Null);
    let mut out = Checked {
        errors: validate_inputs(action, derived, entity_id, inputs),
        entity: entity.clone(),
        ..Checked::default()
    };
    if !out.errors.is_empty() {
        return Ok(out);
    }
    for pre in &action.preconditions {
        match eval(pre, &entity, inputs, policies, None) {
            Ok(Value::Bool(true)) => {}
            Ok(_) => out.blockers.push(format!("precondition not met: {pre}")),
            Err(err) => out
                .blockers
                .push(format!("precondition failed to evaluate: {pre}: {err}")),
        }
    }
    out.approvers = match action
        .approval
        .as_deref()
        .map(|a| eval(a, &entity, inputs, policies, None))
    {
        Some(Ok(Value::Array(a))) => a,
        Some(Ok(Value::Null)) | None => vec![],
        Some(Ok(other)) => vec![other],
        Some(Err(err)) => {
            out.blockers
                .push(format!("approval rule failed to evaluate: {err}"));
            vec![]
        }
    };
    let entity_cel = ExprValue::from_json(&entity);
    let inputs_cel = ExprValue::from_json(&Value::Object(inputs.clone()));
    let policy = ExprValue::from_json(&Value::Object(policies.clone()));
    for (lname, lk) in &action.context {
        let Ok(where_) = Expr::compile(&lk.where_) else {
            continue;
        };
        let matches: Vec<Value> = derived
            .entities
            .iter()
            .filter(|(_, c)| c.kind == lk.kind)
            .filter_map(|(cid, _)| derived.entity_json(cid))
            .filter(|cand| {
                let cand = ExprValue::from_json(cand);
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
            .collect();
        out.related.insert(lname.clone(), Value::Array(matches));
    }
    Ok(out)
}
