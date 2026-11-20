//! Validation of fixture files: `fixtures/<connector>.ndjson` lines are connector messages
//! (schema/connector-messages.schema.json), and record kinds must be declared by that connector.

use crate::diagnostics::Diagnostic;
use crate::manifest::Manifest;
use crate::schema;
use serde_json::Value;
use std::path::Path;

/// Validate every `fixtures/*.ndjson` file under `root`. Returns (messages checked, diagnostics).
pub fn validate_fixtures(root: &Path, manifest: Option<&Manifest>) -> (usize, Vec<Diagnostic>) {
    let dir = root.join("fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "ndjson"))
        .collect();
    files.sort();
    let (mut count, mut out) = (0, Vec::new());
    for path in files {
        let rel = format!(
            "fixtures/{}",
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        let connector = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let declared = manifest.and_then(|m| m.connectors.iter().find(|c| c.name == connector));
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                out.push(diag("fixture.io", &rel, "", e.to_string()));
                continue;
            }
        };
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            count += 1;
            let at = format!(":{}", i + 1);
            let msg: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(e) => {
                    out.push(diag("fixture.json", &rel, &at, e.to_string()));
                    continue;
                }
            };
            for (ptr, m) in schema::errors(schema::connector_messages(), &msg) {
                out.push(diag("fixture.schema", &rel, &format!("{at}{ptr}"), m));
            }
            if let (Some(c), Some("record"), Some(kind)) = (
                declared,
                msg.get("type").and_then(Value::as_str),
                msg.get("kind").and_then(Value::as_str),
            ) && !c.records.iter().any(|r| r == kind)
            {
                out.push(diag(
                    "fixture.record-kind",
                    &rel,
                    &at,
                    format!("record kind {kind} is not declared by connector {connector}"),
                ));
            }
        }
    }
    (count, out)
}

fn diag(code: &'static str, file: &str, path: &str, message: String) -> Diagnostic {
    Diagnostic {
        code,
        file: file.to_string(),
        path: path.to_string(),
        message,
    }
}
