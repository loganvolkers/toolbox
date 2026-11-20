//! The `workspace` connector (in-image, stdio): reads a local checkout statically and reports
//! three record kinds (docs/design/toolbox-for-toolbox.md §3):
//!
//! - `crate`: one per Cargo workspace member, from `cargo metadata` and a `syn` parse of its
//!   sources (doc coverage of public items, function length and complexity, tests, unwraps);
//! - `doc`: one per markdown file matching `docs` globs (status banner, sources section, broken
//!   relative links, inline code spans);
//! - `file`: one per file matching `files` globs (scripts, integration tests);
//! - `betterer`: one per test in each Betterer results file matching `betterer` globs (issue counts
//!   per file for file tests), so existing Betterer ratchets roll up into scorecards.
//!
//! `docs` and `file` records also carry `captures` (each configured regex's first group, unique,
//! sorted, with runs of whitespace replaced by `-`) and `counts` (matches per configured regex),
//! so a portal can relate files to the
//! commands and tools they mention without the connector knowing about them.
//!
//! `TOOLBOX_CONNECTOR_CONFIG`: `{"root"?: ".", "docs"?: [glob], "files"?: [glob],
//! "captures"?: {name: regex}, "counts"?: {name: regex}, "crates"?: true, "betterer"?: [glob]}`.

mod betterer;
mod rust;

use regex::Regex;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toolbox_connect::sdk::{Out, Source, run_stdio};

/// Directories never walked.
const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", ".venv"];

struct Workspace {
    root: PathBuf,
    docs: Vec<Regex>,
    files: Vec<Regex>,
    betterer: Vec<Regex>,
    captures: Vec<(String, Regex)>,
    counts: Vec<(String, Regex)>,
    crates: bool,
}

/// A glob over `/`-separated relative paths: `**` spans directories, `*` and `?` don't.
fn glob(pattern: &str) -> Result<Regex, String> {
    let mut re = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    chars.next();
                    re.push_str("(?:.*/)?");
                } else {
                    re.push_str(".*");
                }
            }
            '*' => re.push_str("[^/]*"),
            '?' => re.push_str("[^/]"),
            c => re.push_str(&regex::escape(&c.to_string())),
        }
    }
    re.push('$');
    Regex::new(&re).map_err(|e| format!("glob {pattern}: {e}"))
}

fn named_regexes(cfg: &Value, key: &str) -> Result<Vec<(String, Regex)>, String> {
    let mut out = Vec::new();
    for (name, v) in cfg[key].as_object().into_iter().flatten() {
        let src = v.as_str().ok_or(format!("{key}.{name} must be a string"))?;
        out.push((
            name.clone(),
            Regex::new(src).map_err(|e| format!("{key}.{name}: {e}"))?,
        ));
    }
    Ok(out)
}

fn globs(cfg: &Value, key: &str) -> Result<Vec<Regex>, String> {
    cfg[key]
        .as_array()
        .into_iter()
        .flatten()
        .map(|g| glob(g.as_str().ok_or(format!("{key} entries must be strings"))?))
        .collect()
}

/// The date (YYYY-MM-DD) of the last commit touching `path`, if it is tracked.
fn last_change(root: &Path, path: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["log", "-1", "--format=%cs", "--", path])
        .current_dir(root)
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (!s.is_empty()).then_some(s)
}

impl Workspace {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("TOOLBOX_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("TOOLBOX_CONNECTOR_CONFIG: {e}"))?;
        Ok(Workspace {
            root: PathBuf::from(cfg["root"].as_str().unwrap_or(".")),
            docs: globs(&cfg, "docs")?,
            files: globs(&cfg, "files")?,
            betterer: globs(&cfg, "betterer")?,
            captures: named_regexes(&cfg, "captures")?,
            counts: named_regexes(&cfg, "counts")?,
            crates: cfg["crates"].as_bool().unwrap_or(true),
        })
    }

    /// Relative paths of every file under the root (skipping build and VCS directories).
    fn walk(&self) -> Vec<String> {
        let mut out: Vec<String> = walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| {
                !(e.file_type().is_dir()
                    && SKIP_DIRS.contains(&e.file_name().to_string_lossy().as_ref()))
            })
            .flatten()
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                e.path()
                    .strip_prefix(&self.root)
                    .ok()
                    .map(|p| p.to_string_lossy().replace('\\', "/"))
            })
            .collect();
        out.sort();
        out
    }

    fn matches(set: &[Regex], path: &str) -> bool {
        set.iter().any(|r| r.is_match(path))
    }

    /// `captures` and `counts` for a file's text.
    fn scan(&self, text: &str) -> (Value, Value) {
        let mut caps = Map::new();
        for (name, re) in &self.captures {
            let found: BTreeSet<String> = re
                .captures_iter(text)
                .filter_map(|c| c.get(1))
                .map(|m| m.as_str().split_whitespace().collect::<Vec<_>>().join("-"))
                .collect();
            caps.insert(name.clone(), json!(found));
        }
        let mut counts = Map::new();
        for (name, re) in &self.counts {
            counts.insert(name.clone(), json!(re.find_iter(text).count()));
        }
        (Value::Object(caps), Value::Object(counts))
    }

    fn doc_record(&self, rel: &str) -> Option<Value> {
        let text = std::fs::read_to_string(self.root.join(rel)).ok()?;
        let (captures, counts) = self.scan(&text);
        let title = text
            .lines()
            .find_map(|l| l.strip_prefix("# "))
            .unwrap_or(rel)
            .trim()
            .to_owned();
        let head: Vec<&str> = text.lines().take(15).collect();
        let has_status = head.iter().any(|l| {
            let l = l.trim_start_matches(['>', ' ', '*', '_']);
            l.starts_with("Status") || l.starts_with("Implemented") || l.starts_with("Layer ")
        });
        // `## Sources` or a numbered `## 5. Sources`
        let has_sources = text.lines().any(|l| {
            l.starts_with('#')
                && l.trim_start_matches('#')
                    .trim()
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.')
                    .trim()
                    .starts_with("Sources")
        });
        let link = Regex::new(r"\]\(([^)\s#]+)(?:#[^)]*)?\)").unwrap();
        let dir = Path::new(rel).parent().unwrap_or(Path::new(""));
        let (mut links, mut broken) = (0, BTreeSet::new());
        for c in link.captures_iter(&text) {
            let target = &c[1];
            if target.contains("://") || target.starts_with("mailto:") {
                continue;
            }
            links += 1;
            if !self.root.join(dir).join(target).exists() {
                broken.insert(target.to_owned());
            }
        }
        let span = Regex::new(r"`([^`\n]{1,80})`").unwrap();
        let spans: BTreeSet<&str> = span
            .captures_iter(&text)
            .map(|c| c.get(1).unwrap().as_str().trim())
            .collect();
        Some(json!({
            "path": rel,
            "title": title,
            "words": text.split_whitespace().count(),
            "hasStatus": has_status,
            "hasSources": has_sources,
            "links": links,
            "brokenLinks": broken,
            "codeSpans": spans,
            "captures": captures,
            "counts": counts,
            "lastChange": last_change(&self.root, rel),
        }))
    }

    fn file_record(&self, rel: &str) -> Option<Value> {
        let text = std::fs::read_to_string(self.root.join(rel)).ok()?;
        let (captures, counts) = self.scan(&text);
        Some(json!({
            "path": rel,
            "extension": Path::new(rel).extension().map(|e| e.to_string_lossy().into_owned()),
            "lines": text.lines().count(),
            "captures": captures,
            "counts": counts,
            "lastChange": last_change(&self.root, rel),
        }))
    }

    fn cargo_metadata(&self) -> Result<Value, String> {
        let out = std::process::Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version", "1"])
            .current_dir(&self.root)
            .output()
            .map_err(|e| format!("cargo metadata: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "cargo metadata: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata: {e}"))
    }

    /// `crate` records by crate name.
    fn crate_records(&self) -> Result<BTreeMap<String, Value>, String> {
        let meta = self.cargo_metadata()?;
        let ws_root = PathBuf::from(meta["workspace_root"].as_str().unwrap_or("."));
        let members: BTreeSet<&str> = meta["packages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| p["name"].as_str())
            .collect();
        let mut out = BTreeMap::new();
        for p in meta["packages"].as_array().into_iter().flatten() {
            let name = p["name"].as_str().unwrap_or_default().to_owned();
            let manifest = PathBuf::from(p["manifest_path"].as_str().unwrap_or_default());
            let dir = manifest.parent().unwrap_or(Path::new(".")).to_path_buf();
            let rel = dir
                .strip_prefix(&ws_root)
                .unwrap_or(&dir)
                .to_string_lossy()
                .into_owned();
            let kinds: BTreeSet<&str> = p["targets"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|t| t["kind"].as_array().into_iter().flatten())
                .filter_map(Value::as_str)
                .filter(|k| matches!(*k, "lib" | "bin"))
                .collect();
            let entry = p["targets"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|t| {
                    t["kind"]
                        .as_array()
                        .is_some_and(|k| k.contains(&json!("lib")))
                })
                .or_else(|| {
                    p["targets"].as_array().into_iter().flatten().find(|t| {
                        t["kind"]
                            .as_array()
                            .is_some_and(|k| k.contains(&json!("bin")))
                    })
                })
                .and_then(|t| t["src_path"].as_str())
                .map(PathBuf::from);
            let deps = p["dependencies"].as_array().cloned().unwrap_or_default();
            let normal = |d: &&Value| d["kind"].is_null();
            let internal: BTreeSet<&str> = deps
                .iter()
                .filter(normal)
                .filter_map(|d| d["name"].as_str())
                .filter(|n| members.contains(n))
                .collect();
            let external = deps
                .iter()
                .filter(normal)
                .filter(|d| !d["name"].as_str().is_some_and(|n| members.contains(n)))
                .count();
            let metrics = rust::analyse(&dir, entry.as_deref());
            let mut rec = json!({
                "name": name,
                "path": rel,
                "description": p["description"],
                "targets": kinds,
                "dependsOn": internal,
                "externalDeps": external,
                "readme": dir.join("README.md").is_file(),
                "lastChange": last_change(&ws_root, &rel),
            });
            if let (Value::Object(r), Value::Object(m)) = (&mut rec, metrics) {
                r.extend(m);
            }
            out.insert(name, rec);
        }
        Ok(out)
    }

    fn emit_crates(&self, only: Option<&BTreeSet<String>>, out: &mut Out) -> Result<(), String> {
        let all = self.crate_records()?;
        for (name, rec) in &all {
            if only.is_none_or(|o| o.contains(name)) {
                out.record("crate", &format!("crate:{name}"), rec.clone());
            }
        }
        for missing in only.into_iter().flatten().filter(|n| !all.contains_key(*n)) {
            out.not_found("crate", &format!("crate:{missing}"));
        }
        Ok(())
    }
}

impl Source for Workspace {
    fn name(&self) -> &str {
        "workspace"
    }

    fn capabilities(&self) -> Value {
        json!({"refresh": {"crate": ["external_ids"], "doc": ["external_ids"], "file": ["external_ids"]},
               "resync": ["crate", "doc", "file", "betterer"], "max_concurrency": 1})
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                let kinds: Vec<&str> = cmd["kinds"]
                    .as_array()
                    .map(|k| k.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_else(|| vec!["crate", "doc", "file", "betterer"]);
                if self.crates && kinds.contains(&"crate") {
                    self.emit_crates(None, out)?;
                    out.snapshot_complete("crate");
                }
                let paths = self.walk();
                for (kind, set) in [("doc", &self.docs), ("file", &self.files)] {
                    if !kinds.contains(&kind) {
                        continue;
                    }
                    for rel in paths.iter().filter(|p| Self::matches(set, p)) {
                        let rec = if kind == "doc" {
                            self.doc_record(rel)
                        } else {
                            self.file_record(rel)
                        };
                        if let Some(r) = rec {
                            out.record(kind, &format!("{kind}:{rel}"), r);
                        }
                    }
                    out.snapshot_complete(kind);
                }
                if kinds.contains(&"betterer") && !self.betterer.is_empty() {
                    for rel in paths.iter().filter(|p| Self::matches(&self.betterer, p)) {
                        let text = std::fs::read_to_string(self.root.join(rel)).unwrap_or_default();
                        for (test, mut data) in betterer::parse(&text) {
                            data["path"] = json!(rel);
                            out.record("betterer", &format!("betterer:{rel}#{test}"), data);
                        }
                    }
                    out.snapshot_complete("betterer");
                }
                Ok(())
            }
            Some("refresh") => {
                let mut crates = BTreeSet::new();
                for id in cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    if let Some(name) = id.strip_prefix("crate:") {
                        crates.insert(name.to_owned());
                    } else if let Some(rel) = id.strip_prefix("doc:") {
                        match self.doc_record(rel) {
                            Some(r) => out.record("doc", id, r),
                            None => out.not_found("doc", id),
                        }
                    } else if let Some(rel) = id.strip_prefix("file:") {
                        match self.file_record(rel) {
                            Some(r) => out.record("file", id, r),
                            None => out.not_found("file", id),
                        }
                    }
                }
                if !crates.is_empty() {
                    self.emit_crates(Some(&crates), out)?;
                }
                Ok(())
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
}

#[tokio::main]
async fn main() {
    match Workspace::from_env() {
        Ok(src) => run_stdio(src).await,
        Err(e) => {
            eprintln!("toolbox-connector-workspace: {e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::glob;

    #[test]
    fn globs_match_segments() {
        let g = glob("docs/*.md").unwrap();
        assert!(g.is_match("docs/plan.md"));
        assert!(!g.is_match("docs/design/sdlc.md"));
        let g = glob("docs/**/*.md").unwrap();
        assert!(g.is_match("docs/plan.md"));
        assert!(g.is_match("docs/design/sdlc.md"));
        let g = glob("crates/*/tests/*.rs").unwrap();
        assert!(g.is_match("crates/toolbox-api/tests/api.rs"));
        assert!(!g.is_match("crates/toolbox-api/src/lib.rs"));
    }
}
