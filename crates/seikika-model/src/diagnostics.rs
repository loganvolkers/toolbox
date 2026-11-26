//! Machine-readable diagnostics (stable codes, file, JSON pointer) for people and agents.

use serde::Serialize;
use std::fmt;

/// One problem found by validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Stable code, e.g. `schema`, `xref.map-from`, `cel.syntax`. Agents may match on it.
    pub code: &'static str,
    /// File the problem is in, relative to the portal root.
    pub file: String,
    /// JSON pointer into that file's document (or `:<line>` for NDJSON fixtures).
    pub path: String,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = if self.path.is_empty() {
            String::new()
        } else {
            format!(" {}", self.path)
        };
        write!(f, "{}{}: {} [{}]", self.file, path, self.message, self.code)
    }
}

/// The outcome of validating a portal.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    /// The manifest file that was validated (the root source).
    pub manifest: String,
    /// Source files merged into the manifest via `include`.
    pub included: Vec<String>,
    /// Number of fixture messages validated.
    pub messages: usize,
    pub errors: Vec<Diagnostic>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}
