//! Declared records (`data` config): YAML or JSON files whose entries become records of a configured
//! kind, for facts that live in a repository as data rather than in code, such as SDLC stages, a
//! service's runbook links or a team roster. Each file is a list of objects, or `{records: [...]}`;
//! each object needs a string `id`.
//!
//! Every record also carries `file` (where it was declared), `hash` (the first 12 hex digits of the
//! SHA-256 of the entry as written: what an assessment of it is versioned by) and `missingPaths`
//! (values of the configured `paths` fields that name no file or directory under the root).
//! Optionally, `unresolvedCommands` (values of the `commands` fields whose program is neither a
//! known tool nor a file under the root) and `notInCi` (values of the `ci` fields that no GitHub
//! Actions workflow or action file names), so a declaration can't claim what isn't there.

use regex::Regex;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

/// One `data` source: the record kind, the files, and the fields holding repo-relative paths.
pub struct Source {
    pub kind: String,
    pub globs: Vec<Regex>,
    pub paths: Vec<String>,
    /// Fields holding shell commands, resolved to a known program or a file under the root.
    pub commands: Vec<String>,
    /// Fields holding paths that a CI workflow must run (or be).
    pub ci: Vec<String>,
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
        let list = |key: &str| -> Vec<String> {
            s[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|p| p.as_str().map(str::to_owned))
                .collect()
        };
        out.push(Source {
            kind: kind.to_owned(),
            globs,
            paths: list("paths"),
            commands: list("commands"),
            ci: list("ci"),
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

/// Values of `fields` (a string or a list of strings).
fn values<'a>(entry: &'a Value, fields: &[String]) -> Vec<&'a str> {
    fields
        .iter()
        .flat_map(|f| match &entry[f] {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        })
        .collect()
}

/// Programs a stage's command may start with, besides a path under the root.
const PROGRAMS: [&str; 20] = [
    "toolbox",
    "cargo",
    "npm",
    "npx",
    "node",
    "python",
    "python3",
    "git",
    "dist",
    "yamark",
    "bash",
    "make",
    "ruff",
    "zizmor",
    "actionlint",
    "fga",
    "scorecard",
    "docker",
    "gh",
    "skills-ref",
];

/// Commands whose program (the first word after any `VAR=value` assignments) is neither a known
/// tool nor a file under the root: `echo`, a typo, or a script that was never written.
fn unresolved(root: &Path, entry: &Value, fields: &[String]) -> Vec<String> {
    values(entry, fields)
        .into_iter()
        .filter(|cmd| {
            let program = cmd
                .split_whitespace()
                .find(|w| !w.contains('=') || w.starts_with('-'))
                .unwrap_or_default();
            let path = program.trim_start_matches("./");
            !(PROGRAMS.contains(&program) || (path.contains('/') && root.join(path).exists()))
        })
        .map(str::to_owned)
        .collect()
}

/// Paths that no workflow under `.github/workflows` or action under `actions/` names (a workflow
/// file itself counts as run by CI).
fn not_in_ci(root: &Path, entry: &Value, fields: &[String]) -> Vec<String> {
    let mut ci = String::new();
    for dir in [".github/workflows", "actions"] {
        for e in walkdir::WalkDir::new(root.join(dir)).into_iter().flatten() {
            let yml = e
                .path()
                .extension()
                .is_some_and(|x| x == "yml" || x == "yaml");
            if e.file_type().is_file() && yml {
                ci.push_str(&std::fs::read_to_string(e.path()).unwrap_or_default());
            }
        }
    }
    values(entry, fields)
        .into_iter()
        .filter(|p| !run_by_ci(&ci, p))
        .map(str::to_owned)
        .collect()
}

/// Whether CI text runs `path`: it is a workflow, names it, names a directory holding it (an
/// action used as `./actions/report`, a portal run as `toolbox test portal`), or it is Rust under
/// `crates/` and CI runs `cargo test`.
fn run_by_ci(ci: &str, path: &str) -> bool {
    if path.starts_with(".github/workflows/") || ci.contains(path) {
        return true;
    }
    if path.starts_with("crates/") && path.ends_with(".rs") && ci.contains("cargo test") {
        return true;
    }
    Path::new(path).ancestors().skip(1).any(|d| {
        let d = d.to_string_lossy();
        !d.is_empty()
            && (ci.contains(&format!("./{d}"))
                || ci.contains(&format!(" {d}\n"))
                || ci.contains(&format!(" {d} "))
                || ci.contains(&format!(" {d}/*")))
    })
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
        if !src.commands.is_empty() {
            data["unresolvedCommands"] = json!(unresolved(root, &e, &src.commands));
        }
        if !src.ci.is_empty() {
            data["notInCi"] = json!(not_in_ci(root, &e, &src.ci));
        }
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
            commands: vec!["commands".into()],
            ci: vec![],
        };
        let r = records(&dir, "stages.yaml", &src).unwrap();
        assert_eq!(r[0].0, "stage:m/build");
        assert_eq!(r[0].1["missingPaths"], json!(["docs/gone.md", "ci.yml"]));
        assert_eq!(r[0].1["file"], "stages.yaml");
        assert_eq!(r[0].1["hash"].as_str().unwrap().len(), 12);
        std::fs::create_dir_all(dir.join("scripts")).unwrap();
        std::fs::write(dir.join("scripts/ok.sh"), "").unwrap();
        std::fs::write(
            dir.join("cmds.yaml"),
            "- {id: x, commands: [echo hi, 'A=1 ./scripts/ok.sh', cargo test, scripts/gone.sh]}\n",
        )
        .unwrap();
        let r = records(&dir, "cmds.yaml", &src).unwrap();
        assert_eq!(
            r[0].1["unresolvedCommands"],
            json!(["echo hi", "scripts/gone.sh"])
        );
        std::fs::write(dir.join("bad.yaml"), "- {name: no-id}\n").unwrap();
        assert!(records(&dir, "bad.yaml", &src).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
