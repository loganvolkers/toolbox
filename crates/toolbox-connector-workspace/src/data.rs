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
//! Actions workflow or action file executes) and `notNaming` (documents in the `naming` fields that
//! never mention the entry's `name`, and `namedIn`, those that exist and do), so a declaration
//! can't claim what isn't there.

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
    /// Fields holding documents that must mention the entry's `name`.
    pub naming: Vec<String>,
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
            naming: list("naming"),
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
const PROGRAMS: [&str; 21] = [
    "toolbox",
    "cargo",
    "npm",
    "pnpm",
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

/// Paths that no workflow under `.github/workflows` or action under `actions/` executes (a
/// workflow file itself counts as run by CI).
fn not_in_ci(root: &Path, entry: &Value, fields: &[String]) -> Vec<String> {
    let mut ci = String::new();
    for dir in [".github/workflows", "actions"] {
        for e in walkdir::WalkDir::new(root.join(dir)).into_iter().flatten() {
            let yml = e
                .path()
                .extension()
                .is_some_and(|x| x == "yml" || x == "yaml");
            if e.file_type().is_file() && yml {
                let text = std::fs::read_to_string(e.path()).unwrap_or_default();
                if let Ok(doc) = serde_norway::from_str::<Value>(&text) {
                    executed(&doc, false, &mut ci);
                }
            }
        }
    }
    values(entry, fields)
        .into_iter()
        .filter(|p| !run_by_ci(&ci, p))
        .map(str::to_owned)
        .collect()
}

/// The strings a workflow or action executes: `run:` and `uses:` values and everything under
/// `with:`, one per line. Step names, comments and other keys don't run anything, so a path named
/// only there is not run by CI.
fn executed(v: &Value, inside_with: bool, out: &mut String) {
    match v {
        Value::String(s) if inside_with => {
            out.push(' ');
            out.push_str(s);
            out.push('\n');
        }
        Value::Object(m) => {
            for (k, x) in m {
                match (k.as_str(), x) {
                    ("run" | "uses", Value::String(s)) => {
                        out.push(' ');
                        out.push_str(s);
                        out.push('\n');
                    }
                    _ => executed(x, inside_with || k == "with", out),
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| executed(x, inside_with, out)),
        _ => {}
    }
}

/// Whether CI text (see [`executed`]) runs `path`: it is a workflow, names it, names a directory holding it (an
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

/// Documents in `fields` that exist but never mention the entry's `name` (case-insensitive): a
/// shared design doc listed for a stage it doesn't describe.
fn not_naming(root: &Path, entry: &Value, fields: &[String]) -> Vec<String> {
    let name = entry["name"].as_str().unwrap_or_default().to_lowercase();
    values(entry, fields)
        .into_iter()
        .filter(|v| {
            let path = v.split('#').next().unwrap_or(v);
            std::fs::read_to_string(root.join(path))
                .is_ok_and(|text| name.is_empty() || !text.to_lowercase().contains(&name))
        })
        .map(str::to_owned)
        .collect()
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
        if !src.naming.is_empty() {
            let silent = not_naming(root, &e, &src.naming);
            let missing = missing(root, &e, &src.naming);
            // Documents that exist and name the entry: what describes it.
            let named: Vec<&str> = values(&e, &src.naming)
                .into_iter()
                .filter(|v| !silent.iter().any(|s| s == v) && !missing.iter().any(|m| m == v))
                .collect();
            data["namedIn"] = json!(named);
            data["notNaming"] = json!(silent);
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
            naming: vec![],
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
        std::fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        std::fs::write(
            dir.join(".github/workflows/ci.yml"),
            "jobs:\n  t:\n    steps:\n      - name: uses rust-toolchain.toml\n        run: scripts/ok.sh --fast\n      - uses: ./actions/report\n        with: {portal: portal}\n",
        )
        .unwrap();
        let ci_src = Source {
            ci: vec!["checks".into()],
            naming: vec!["docs".into()],
            ..src
        };
        std::fs::write(
            dir.join("ci.yaml"),
            "- {id: y, checks: [scripts/ok.sh, rust-toolchain.toml, actions/report/action.yml, portal/tests/a.yaml]}\n",
        )
        .unwrap();
        let r = records(&dir, "ci.yaml", &ci_src).unwrap();
        assert_eq!(r[0].1["notInCi"], json!(["rust-toolchain.toml"]));
        std::fs::write(
            dir.join("named.yaml"),
            "- {id: z, name: Pack, docs: [docs/a.md, docs/pack.md, docs/gone.md]}\n",
        )
        .unwrap();
        std::fs::write(dir.join("docs/pack.md"), "# How we pack\n").unwrap();
        let r = records(&dir, "named.yaml", &ci_src).unwrap();
        assert_eq!(r[0].1["notNaming"], json!(["docs/a.md"]));
        assert_eq!(r[0].1["namedIn"], json!(["docs/pack.md"]));
        std::fs::write(dir.join("bad.yaml"), "- {name: no-id}\n").unwrap();
        assert!(records(&dir, "bad.yaml", &ci_src).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
