//! Declared records (`data` config): YAML or JSON files whose entries become records of a configured
//! kind, for facts that live in a repository as data rather than in code, such as SDLC stages, a
//! service's runbook links or a team roster. Each file is a list of objects, or `{records: [...]}`;
//! each object needs a string `id`.
//!
//! Every record also carries `file` (where it was declared), `hash` (the first 12 hex digits of the
//! SHA-256 of the entry as written: what an assessment of it is versioned by) and `missingPaths`
//! (values of the configured `paths` fields that name no file or directory under the root).

use regex::Regex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

/// One `data` source: the record kind, the files, and the fields holding repo-relative paths.
pub struct Source {
    pub kind: String,
    pub globs: Vec<Regex>,
    pub paths: Vec<String>,
}

/// `data: [{kind, globs: [glob], paths?: [field]}]`.
pub fn sources(cfg: &Value) -> Result<Vec<Source>, String> {
    let mut out = Vec::new();
    for (i, s) in cfg["data"].as_array().into_iter().flatten().enumerate() {
        let kind = s["kind"]
            .as_str()
            .ok_or(format!("data[{i}].kind must be a string"))?;
        let globs = s["globs"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|g| toolbox_connect::files::glob(g.as_str().unwrap_or_default()))
            .collect::<Result<Vec<_>, _>>()?;
        let paths = s["paths"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| p.as_str().map(str::to_owned))
            .collect();
        out.push(Source {
            kind: kind.to_owned(),
            globs,
            paths,
        });
    }
    Ok(out)
}

fn hash(v: &Value) -> String {
    let text = serde_json::to_string(v).unwrap_or_default();
    Sha256::digest(text.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Path values of `fields` in `entry` (a string or a list of strings; `#anchors` ignored) that
/// don't exist under `root`.
fn missing(root: &Path, entry: &Value, fields: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for f in fields {
        let values: Vec<&str> = match &entry[f] {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        for v in values {
            let path = v.split('#').next().unwrap_or(v);
            if !path.is_empty() && !root.join(path).exists() {
                out.push(v.to_owned());
            }
        }
    }
    out
}

/// The records declared in one file: `(external id, data)`, or why the file was skipped.
pub fn records(root: &Path, rel: &str, src: &Source) -> Result<Vec<(String, Value)>, String> {
    let text = std::fs::read_to_string(root.join(rel)).map_err(|e| e.to_string())?;
    let doc: Value = serde_norway::from_str(&text).map_err(|e| e.to_string())?;
    let entries = match &doc {
        Value::Array(a) => a.clone(),
        _ => doc["records"].as_array().cloned().unwrap_or_default(),
    };
    let mut out = Vec::new();
    for (i, e) in entries.into_iter().enumerate() {
        let Some(id) = e["id"].as_str().map(str::to_owned) else {
            return Err(format!("entry {i} has no string `id`"));
        };
        let mut data = e.clone();
        data["file"] = json!(rel);
        data["hash"] = json!(hash(&e));
        data["missingPaths"] = json!(missing(root, &e, &src.paths));
        out.push((format!("{}:{id}", src.kind), data));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_records_with_hash_and_missing_paths() {
        let dir = std::env::temp_dir().join(format!("toolbox-data-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        std::fs::write(dir.join("docs/a.md"), "# A").unwrap();
        std::fs::write(
            dir.join("stages.yaml"),
            "records:\n  - {id: m/build, docs: [docs/a.md#x, docs/gone.md], check: ci.yml}\n",
        )
        .unwrap();
        let src = Source {
            kind: "stage".into(),
            globs: vec![],
            paths: vec!["docs".into(), "check".into()],
        };
        let r = records(&dir, "stages.yaml", &src).unwrap();
        assert_eq!(r[0].0, "stage:m/build");
        assert_eq!(r[0].1["missingPaths"], json!(["docs/gone.md", "ci.yml"]));
        assert_eq!(r[0].1["file"], "stages.yaml");
        assert_eq!(r[0].1["hash"].as_str().unwrap().len(), 12);
        std::fs::write(dir.join("bad.yaml"), "- {name: no-id}\n").unwrap();
        assert!(records(&dir, "bad.yaml", &src).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
