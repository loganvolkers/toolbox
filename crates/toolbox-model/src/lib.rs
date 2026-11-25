//! Manifest types, loading, JSON Schema validation and cross-reference checks
//! (docs/v1-spec.md §4, §5).
//!
//! The entry point for `toolbox validate` is [`validate_portal`]: load the source (one YAML/JSON file
//! plus any `include`d files, D54), validate it against the embedded manifest schema, run the
//! cross-reference, form-subset and CEL checks, and validate fixture messages.

pub mod diagnostics;
pub mod executors;
pub mod federation;
pub mod forge;
pub mod load;
pub mod manifest;
pub mod messages;
pub mod package;
pub mod schema;
pub mod validate;

pub use diagnostics::{Diagnostic, Report};
pub use load::{LoadError, Source};
pub use manifest::Manifest;
pub use validate::{validate_manifest, validate_portal, validate_source};

/// A hint for a key that is the tail of a value: in `{reason: Replaced by x, which is better.}`
/// the comma ends the value, and YAML reads `which is better.` as a key with no value.
pub fn split_value_hint(key: &str, value: &serde_json::Value) -> Option<String> {
    (value.is_null() && key.contains(char::is_whitespace)).then(|| {
        format!(
            "{key:?} looks like the rest of a value cut at a comma: quote values that contain \
             commas inside {{…}}"
        )
    })
}
