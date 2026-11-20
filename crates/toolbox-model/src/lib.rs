//! Manifest types, loading, JSON Schema validation and cross-reference checks
//! (docs/v1-spec.md §4, §5).
//!
//! The entry point for `toolbox validate` is [`validate_portal`]: load the source (one YAML/JSON file
//! plus any `include`d files, D54), validate it against the embedded manifest schema, run the
//! cross-reference, form-subset and CEL checks, and validate fixture messages.

pub mod diagnostics;
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
