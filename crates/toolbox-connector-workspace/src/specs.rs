//! JSON Schema documents under `schema/`, read statically (docs/design/schema-standards.md).
//!
//! One record per document. Facts only: the dialect, the canonical id, whether the root is
//! described and titled, whether every `$ref` stays inside the file, and whether the validator
//! or the binary actually compiles it. Thresholds live in the manifest.

use regex::Regex;
use serde_json::Value;
use std::path::Path;

const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

/// Files that compile or apply a schema. A schema none of them names is not part of the contract.
const WIRED_IN: &[&str] = &[
    "crates/toolbox-model/src/schema.rs",
    "examples/reference-portal/tools/validate.py",
];

pub(crate) fn analyse(root: &Path, rel: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let filename = Path::new(rel)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    serde_json::json!({
        "path": rel,
        "name": filename,
        "dialect": doc["$schema"].as_str().unwrap_or("").trim(),
        "schemaId": doc["$id"].as_str().unwrap_or("").trim(),
        "titleChars": doc["title"].as_str().unwrap_or("").trim().chars().count() as i64,
        "descriptionChars": doc["description"].as_str().unwrap_or("").trim().chars().count() as i64,
        "externalRefs": external_refs(&doc),
        "wired": wired(root, &filename),
        "draft202012": doc["$schema"].as_str().unwrap_or("").trim() == DRAFT_2020_12,
        "secretHits": secrets(rel, &text),
    })
}

fn external_refs(node: &Value) -> Vec<String> {
    let mut out = Vec::new();
    collect_refs(node, &mut out);
    out.sort();
    out.dedup();
    out
}

fn collect_refs(node: &Value, out: &mut Vec<String>) {
    match node {
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(Value::as_str)
                && !reference.starts_with('#')
            {
                out.push(reference.to_owned());
            }
            for value in map.values() {
                collect_refs(value, out);
            }
        }
        Value::Array(items) => {
            for value in items {
                collect_refs(value, out);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

fn wired(root: &Path, filename: &str) -> bool {
    !filename.is_empty()
        && WIRED_IN.iter().any(|rel| {
            std::fs::read_to_string(root.join(rel))
                .unwrap_or_default()
                .contains(filename)
        })
}

fn secrets(rel: &str, text: &str) -> Vec<String> {
    let patterns = [
        ("aws-access-key", r"AKIA[0-9A-Z]{16}"),
        ("github-token", r"ghp_[A-Za-z0-9]{36}"),
    ];
    let mut hits = Vec::new();
    for (n, line) in text.lines().enumerate() {
        for (kind, pat) in patterns {
            if compile(pat).is_match(line) {
                hits.push(format!("{}:{} {kind}", rel, n + 1));
            }
        }
    }
    hits
}

fn compile(pat: &str) -> Regex {
    Regex::new(pat).unwrap_or_else(|_| panic!("static regex {pat}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root =
            std::env::temp_dir().join(format!("toolbox-schema-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_draft_2020_12_schema_with_local_refs_is_a_contract() {
        let schema = r##"{
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://toolbox.dev/schemas/demo.json",
            "title": "Demo",
            "description": "A contract.",
            "$ref": "#/$defs/name"
        }"##;
        let tmp = scratch(
            "good",
            &[
                ("schema/demo.schema.json", schema),
                (
                    "crates/toolbox-model/src/schema.rs",
                    "include_str!(\"demo.schema.json\");\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "schema/demo.schema.json");
        assert_eq!(rec["draft202012"], true);
        assert_eq!(rec["externalRefs"], json!([]));
        assert_eq!(rec["wired"], true);
        assert!(rec["descriptionChars"].as_i64().unwrap() > 0);
        assert!(rec["titleChars"].as_i64().unwrap() > 0);
    }

    #[test]
    fn an_external_ref_and_an_old_draft_are_named() {
        let schema = "{\n  \"$schema\": \"https://json-schema.org/draft-07/schema#\",\n  \"$ref\": \"https://example.invalid/other.json\",\n  \"note\": \"AKIAIOSFODNN7EXAMPLE\"\n}\n";
        let tmp = scratch("bad", &[("schema/unsafe.schema.json", schema)]);
        let rec = analyse(&tmp.0, "schema/unsafe.schema.json");
        assert_eq!(rec["draft202012"], false);
        assert_eq!(rec["schemaId"], "");
        assert_eq!(
            rec["externalRefs"],
            json!(["https://example.invalid/other.json"])
        );
        assert_eq!(rec["wired"], false);
        assert_eq!(
            rec["secretHits"],
            json!(["schema/unsafe.schema.json:4 aws-access-key"])
        );
    }

    #[test]
    fn the_schemas_in_this_repo_are_draft_2020_12_and_wired_in() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for rel in [
            "schema/manifest.schema.json",
            "schema/connector-messages.schema.json",
            "schema/executor-messages.schema.json",
        ] {
            let rec = analyse(&root, rel);
            assert_eq!(rec["draft202012"], true, "{rel}");
            assert!(
                rec["schemaId"].as_str().unwrap().starts_with("https://"),
                "{rel}"
            );
            assert!(rec["descriptionChars"].as_i64().unwrap() > 0, "{rel}");
            assert!(rec["titleChars"].as_i64().unwrap() > 0, "{rel}");
            assert_eq!(rec["externalRefs"], json!([]), "{rel}");
            assert_eq!(rec["wired"], true, "{rel}");
            assert_eq!(rec["secretHits"], json!([]), "{rel}");
        }
    }
}
