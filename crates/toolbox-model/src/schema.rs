//! The JSON Schemas compiled into the binary. A Toolbox version validates against the schemas it
//! was built with (schema/ at the repo root is the source).

use jsonschema::Validator;
use serde_json::Value;
use std::sync::OnceLock;

pub const MANIFEST_SCHEMA: &str = include_str!("../../../schema/manifest.schema.json");
pub const CONNECTOR_MESSAGES_SCHEMA: &str =
    include_str!("../../../schema/connector-messages.schema.json");
pub const EXECUTOR_MESSAGES_SCHEMA: &str =
    include_str!("../../../schema/executor-messages.schema.json");

fn compile(source: &str) -> Validator {
    let schema: Value = serde_json::from_str(source).expect("embedded schema is valid JSON");
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(true)
        .build(&schema)
        .expect("embedded schema compiles")
}

pub fn manifest() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(MANIFEST_SCHEMA))
}

pub fn connector_messages() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(CONNECTOR_MESSAGES_SCHEMA))
}

pub fn executor_messages() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| compile(EXECUTOR_MESSAGES_SCHEMA))
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

fn is_action_path(path: &str) -> bool {
    path.strip_prefix("/actions/")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_schemas_compile() {
        super::manifest();
        super::connector_messages();
        super::executor_messages();
    }
}
