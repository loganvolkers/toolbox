//! JSON Schema documents under `schema/`, read statically (docs/design/schema-standards.md).
//!
//! One record per document. Facts only: the dialect, the canonical id, whether the root is
//! described and titled, whether every `$ref` stays inside the file, and whether the validator
//! or the binary actually compiles it. Thresholds live in the manifest.

use serde_json::Value;
use std::path::Path;
use syn::visit::Visit;

const DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";

const REFERENCE_VALIDATOR: &str = "examples/reference-portal/tools/validate.py";
const SCHEMA_DIR: &str = "crates/seikika-model/schema";

pub(crate) fn analyse(root: &Path, rel: &str, sources: &[String]) -> Value {
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
        "wired": wired(root, rel, sources),
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

fn wired(root: &Path, rel: &str, sources: &[String]) -> bool {
    let Ok(target) = root.join(rel).canonicalize() else {
        return false;
    };
    sources.iter().any(|rel_source| {
        let source = root.join(rel_source);
        let Ok(text) = std::fs::read_to_string(&source) else {
            return false;
        };
        if source.extension().is_some_and(|ext| ext == "rs") {
            let Ok(file) = syn::parse_file(&text) else {
                return false;
            };
            let Some(dir) = source.parent() else {
                return false;
            };
            let mut includes = SchemaIncludes(Vec::new());
            includes.visit_file(&file);
            return includes.0.iter().any(|include| {
                dir.join(include)
                    .canonicalize()
                    .is_ok_and(|path| path == target)
            });
        }
        if rel_source == REFERENCE_VALIDATOR {
            return reference_validator_uses(rel, &text);
        }
        Path::new(rel)
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| text.contains(name))
    })
}

fn reference_validator_uses(rel: &str, text: &str) -> bool {
    let Ok(name) = Path::new(rel).strip_prefix(SCHEMA_DIR) else {
        return false;
    };
    let name = name.to_string_lossy();
    let double_quoted_call = format!("schema(\"{name}\")");
    let single_quoted_call = format!("schema('{name}')");
    text.lines().any(|line| {
        !line.trim_start().starts_with('#')
            && (line.contains(&double_quoted_call) || line.contains(&single_quoted_call))
    })
}

struct SchemaIncludes(Vec<String>);

impl<'ast> Visit<'ast> for SchemaIncludes {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("include_str")
            && let Ok(path) = mac.parse_body::<syn::LitStr>()
        {
            self.0.push(path.value());
        }
    }
}

fn secrets(rel: &str, text: &str) -> Vec<String> {
    crate::hygiene::secrets(rel, text)
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
            std::env::temp_dir().join(format!("seikika-schema-{name}-{}", std::process::id()));
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
            "$id": "https://seikika.com/schemas/demo.json",
            "title": "Demo",
            "description": "A contract.",
            "$ref": "#/$defs/name"
        }"##;
        let tmp = scratch(
            "good",
            &[
                ("crates/seikika-model/schema/demo.schema.json", schema),
                ("schema/demo.schema.json", schema),
                ("schema/commented.schema.json", schema),
                (
                    "crates/seikika-model/src/schema.rs",
                    "const DEMO: &str = include_str!(\"../schema/demo.schema.json\");\n// include_str!(\"../../../schema/commented.schema.json\");\n",
                ),
            ],
        );
        let sources = ["crates/seikika-model/src/schema.rs".to_owned()];
        let rec = analyse(
            &tmp.0,
            "crates/seikika-model/schema/demo.schema.json",
            &sources,
        );
        assert_eq!(rec["draft202012"], true);
        assert_eq!(
            analyse(&tmp.0, "crates/seikika-model/schema/demo.schema.json", &[])["wired"],
            false
        );
        assert_eq!(rec["externalRefs"], json!([]));
        assert_eq!(rec["wired"], true);
        assert_eq!(
            analyse(&tmp.0, "schema/demo.schema.json", &sources)["wired"],
            false
        );
        assert_eq!(
            analyse(&tmp.0, "schema/commented.schema.json", &sources)["wired"],
            false
        );
        assert!(rec["descriptionChars"].as_i64().unwrap() > 0);
        assert!(rec["titleChars"].as_i64().unwrap() > 0);
    }

    #[test]
    fn an_external_ref_and_an_old_draft_are_named() {
        let schema = "{\n  \"$schema\": \"https://json-schema.org/draft-07/schema#\",\n  \"$ref\": \"https://example.invalid/other.json\",\n  \"note\": \"AKIAIOSFODNN7EXAMPLE\"\n}\n";
        let tmp = scratch("bad", &[("schema/unsafe.schema.json", schema)]);
        let rec = analyse(&tmp.0, "schema/unsafe.schema.json", &[]);
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
    fn reference_validator_checks_schema_files_in_its_schema_directory() {
        let tmp = scratch(
            "reference",
            &[
                ("crates/seikika-model/schema/demo.schema.json", "{}"),
                ("crates/seikika-model/schema/commented.schema.json", "{}"),
                ("schema/demo.schema.json", "{}"),
                (
                    REFERENCE_VALIDATOR,
                    "check(schema('demo.schema.json'), doc)\n# schema('commented.schema.json')\n",
                ),
            ],
        );
        let sources = [REFERENCE_VALIDATOR.to_owned()];
        assert!(wired(
            &tmp.0,
            "crates/seikika-model/schema/demo.schema.json",
            &sources
        ));
        assert!(!wired(
            &tmp.0,
            "crates/seikika-model/schema/commented.schema.json",
            &sources
        ));
        assert!(!wired(&tmp.0, "schema/demo.schema.json", &sources));
    }

    #[test]
    fn the_schemas_in_this_repo_are_draft_2020_12_and_wired_in() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for rel in [
            "crates/seikika-model/schema/manifest.schema.json",
            "crates/seikika-model/schema/connector-messages.schema.json",
            "crates/seikika-model/schema/executor-messages.schema.json",
            "crates/seikika-model/schema/assessment.schema.json",
        ] {
            let rec = analyse(
                &root,
                rel,
                &["crates/seikika-model/src/schema.rs".to_owned()],
            );
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
