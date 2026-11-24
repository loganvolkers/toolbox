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

/// Validate inputs against the action's `inputs` schema (JSON Schema draft 2020-12: `pattern`,
/// `format`, lengths, ranges, `enum`, `items`), plus what the schema can't say: no inputs it
/// doesn't declare, and `enumFrom` options computed for this entity.
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
    for (k, v) in inputs {
        let Some(spec) = props.get(k) else {
            errors.push(toolbox_model::split_value_hint(k, v).unwrap_or_else(|| {
                format!(
                    "{k} is not an input of {name} (inputs: {})",
                    props.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            }));
            continue;
        };
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
    }
    match toolbox_model::schema::input_errors(&action.inputs, &Value::Object(inputs.clone())) {
        Ok(found) => {
            errors.extend(
                found
                    .into_iter()
                    .map(|(path, msg)| match path.trim_start_matches('/') {
                        "" => msg,
                        p => format!("{p}: {msg}"),
                    }),
            )
        }
        Err(e) => errors.push(format!("{name}: {e}")),
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
        Some(Ok(Value::Array(a))) => a.into_iter().map(any_of).collect(),
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

/// An approval entry that is a list of principals is satisfied by any one of them; it is kept as
/// one string, `user:ann|user:bob`, so a run records and matches it like any other approver.
fn any_of(entry: Value) -> Value {
    match entry {
        Value::Array(xs) => Value::String(
            xs.iter()
                .map(|x| x.as_str().map_or_else(|| x.to_string(), str::to_owned))
                .collect::<Vec<_>>()
                .join("|"),
        ),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn action() -> Action {
        serde_json::from_value(json!({
            "name": "set-owners", "kind": "Repository", "description": "d", "scope": "day2",
            "permission": "can_run_actions", "executor": {"type": "command", "run": "x"},
            "unverified": true,
            "inputs": {"type": "object", "required": ["team"], "properties": {
                "team": {"type": "string", "pattern": "^@[a-z0-9-]+/[a-z0-9-]+$"},
                "count": {"type": "integer", "minimum": 1}}}
        }))
        .unwrap()
    }

    fn errs(inputs: Value) -> Vec<String> {
        let Value::Object(m) = inputs else { panic!() };
        validate_inputs(&action(), &Derived::default(), "repository:a", &m)
    }

    #[test]
    fn inputs_meet_the_whole_schema() {
        assert!(errs(json!({"team": "@acme/web"})).is_empty());
        let e = errs(json!({"team": "@acme/web\n* @evil/takeover"}));
        assert!(e.len() == 1 && e[0].starts_with("team: "), "{e:?}");
        let e = errs(json!({"count": 0, "extra": 1}));
        assert!(
            e.iter().any(|x| x.contains("extra is not an input")),
            "{e:?}"
        );
        assert!(
            e.iter()
                .any(|x| x.contains("\"team\" is a required property")),
            "{e:?}"
        );
        assert!(e.iter().any(|x| x.starts_with("count: ")), "{e:?}");
    }
}
