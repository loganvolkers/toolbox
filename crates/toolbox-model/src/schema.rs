//! The JSON Schemas compiled into the binary. A Toolbox version validates against the schemas it
//! was built with (schema/ at the repo root is the source).

use jsonschema::Validator;
use serde_json::Value;
use std::sync::OnceLock;

/// `schema/manifest.schema.json`: the portal manifest.
pub const MANIFEST_SCHEMA: &str = include_str!("../../../schema/manifest.schema.json");
/// `schema/connector-messages.schema.json`: what connectors send.
pub const CONNECTOR_MESSAGES_SCHEMA: &str =
    include_str!("../../../schema/connector-messages.schema.json");
/// `schema/executor-messages.schema.json`: what action executors send.
pub const EXECUTOR_MESSAGES_SCHEMA: &str =
    include_str!("../../../schema/executor-messages.schema.json");
/// `schema/assessment.schema.json`: an agent's assessment of one entity against a rubric.
pub const ASSESSMENT_SCHEMA: &str = include_str!("../../../schema/assessment.schema.json");

fn compile(source: &str) -> Validator {
    let schema: Value = serde_json::from_str(source).expect("embedded schema is valid JSON");
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .expect("embedded schema compiles")
}

/// The compiled manifest schema.
pub fn manifest() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(MANIFEST_SCHEMA))
}

/// The compiled connector message schema.
pub fn connector_messages() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(CONNECTOR_MESSAGES_SCHEMA))
}

/// The compiled executor message schema.
pub fn executor_messages() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(EXECUTOR_MESSAGES_SCHEMA))
}

/// The compiled assessment schema.
pub fn assessment() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(ASSESSMENT_SCHEMA))
}

/// Validate `doc` and return `(json pointer, message)` pairs. Messages that would embed a large
/// value (e.g. a whole action failing `anyOf`) refer to it as "this value" instead, and known
/// `anyOf` rules get a plain-language hint.
pub fn errors(validator: &Validator, doc: &Value) -> Vec<(String, String)> {
    validator
        .iter_errors(doc)
        .map(|e| {
            let path = e.instance_path().to_string();
            let instance = e.instance().to_string();
            let mut message = e.to_string();
            if instance.len() > 80 && message.starts_with(&instance) {
                message = format!("this value{}", &message[instance.len()..]);
            }
            if message.contains("'anyOf'") && is_action_path(&path) {
                message = "an action must declare `expect` (a postcondition) or `unverified: true` (action-runs §5)".into();
            }
            (path, message)
        })
        .collect()
}

/// Validate an action's inputs against its `inputs` schema (draft 2020-12, so `pattern`, `format`
/// and `items` hold), returning `(json pointer, message)` pairs. `Err` when the schema itself
/// doesn't compile. Toolbox's own keyword `enumFrom` is ignored here; the caller checks it.
pub fn input_errors(schema: &Value, inputs: &Value) -> Result<Vec<(String, String)>, String> {
    let v = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(schema)
        .map_err(|e| format!("the input schema does not compile: {e}"))?;
    Ok(errors(&v, inputs))
}

fn is_action_path(path: &str) -> bool {
    path.strip_prefix("/actions/")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_schemas_compile() {
        // Each compiles (a bad schema panics) and rejects a value that is no document at all.
        let not_a_document = serde_json::json!(42);
        assert!(!super::manifest().is_valid(&not_a_document));
        assert!(!super::connector_messages().is_valid(&not_a_document));
        assert!(!super::executor_messages().is_valid(&not_a_document));
    }
}
