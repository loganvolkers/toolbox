//! The `workspace` connector (in-image, stdio): reads a local checkout statically and reports
//! three record kinds (docs/design/toolbox-for-toolbox.md §3):
//!
//! - `crate`: one per Cargo workspace member, from `cargo metadata` and a `syn` parse of its
//!   sources (doc coverage of public items, function length and complexity, tests, unwraps);
//! - `doc`: one per markdown file matching `docs` globs (status banner, sources section, broken
//!   relative links, inline code spans);
//! - `file`: one per file matching `files` globs (scripts, integration tests);
//! - `repo` (opt-in, `"repo": true`): the checkout itself — README, LICENSE, `.github` files and
//!   CODEOWNERS parsed into rules — for repository-level standards;
//! - `workflow` and `action-ref` (`workflows` globs): GitHub Actions workflows and action metadata
//!   (triggers, permissions, timeouts, `uses:` and how each is pinned), and one record per external
//!   action with every ref used and the files using it;
//! - `betterer`: one per test in each Betterer results file matching `betterer` globs (issue counts
//!   per file for file tests), so existing Betterer ratchets roll up into scorecards.
//!
//! `docs` and `file` records also carry `captures` (each configured regex's first group, unique,
//! sorted, with runs of whitespace replaced by `-`) and `counts` (matches per configured regex),
//! so a portal can relate files to the
//! commands and tools they mention without the connector knowing about them.
//!
//! `TOOLBOX_CONNECTOR_CONFIG`: `{"root"?: ".", "docs"?: [glob], "files"?: [glob],
//! "captures"?: {name: regex}, "counts"?: {name: regex}, "crates"?: true, "repo"?: false, "betterer"?: [glob]}`.

mod betterer;
mod repo;
mod rust;
mod workflows;

use regex::Regex;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toolbox_connect::files::{self, glob};
use toolbox_connect::sdk::{Out, Source, run_stdio};

struct Workspace {
    root: PathBuf,
    docs: Vec<Regex>,
    files: Vec<Regex>,
    betterer: Vec<Regex>,
    /// GitHub Actions workflow and action metadata files (`workflow` and `action-ref` records).
    workflows: Vec<Regex>,
    captures: Vec<(String, Regex)>,
    counts: Vec<(String, Regex)>,
    crates: bool,
    /// Emit one `repo` record for the checkout (opt-in: `"repo": true`).
    repo: bool,
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
            workflows: globs(&cfg, "workflows")?,
            captures: named_regexes(&cfg, "captures")?,
            counts: named_regexes(&cfg, "counts")?,
            crates: cfg["crates"].as_bool().unwrap_or(true),
            repo: cfg["repo"].as_bool().unwrap_or(false),
        })
    }

    /// Relative paths of every file under the root (skipping build and VCS directories).
    fn walk(&self) -> Vec<String> {
        files::walk(&self.root)
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

impl Workspace {
    /// Every record of the requested kinds (all by default), each kind closed by `snapshot_complete`.
    fn resync(&self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        let kinds: Vec<&str> = cmd["kinds"]
            .as_array()
            .map(|k| k.iter().filter_map(Value::as_str).collect())
            .unwrap_or_else(|| {
                vec![
                    "repo",
                    "crate",
                    "doc",
                    "file",
                    "betterer",
                    "workflow",
                    "action-ref",
                ]
            });
        if self.crates && kinds.contains(&"crate") {
            self.emit_crates(None, out)?;
            out.snapshot_complete("crate");
        }
        let paths = self.walk();
        if self.repo && kinds.contains(&"repo") {
            let (name, data) = repo::record(&self.root, &paths);
            out.record("repo", &format!("repo:{name}"), data);
            out.snapshot_complete("repo");
        }
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
        if !self.workflows.is_empty() {
            self.emit_workflows(&paths, &kinds, out);
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

    /// Re-read the records named by `external_ids`; `not_found` for those that are gone.
    /// `workflow` records for files matching `workflows`, and `action-ref` records derived from them.
    fn emit_workflows(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        let records: Vec<Value> = paths
            .iter()
            .filter(|p| Self::matches(&self.workflows, p))
            .filter_map(|rel| {
                let text = std::fs::read_to_string(self.root.join(rel)).ok()?;
                Some(workflows::record(rel, &text))
            })
            .collect();
        if kinds.contains(&"workflow") {
            for r in &records {
                let path = r["path"].as_str().unwrap_or_default();
                out.record("workflow", &format!("workflow:{path}"), r.clone());
            }
            out.snapshot_complete("workflow");
        }
        if kinds.contains(&"action-ref") {
            for (name, data) in workflows::action_refs(&records) {
                out.record("action-ref", &format!("action-ref:{name}"), data);
            }
            out.snapshot_complete("action-ref");
        }
    }

    /// One action ref: its usage spans files, so it is re-derived from every workflow.
    fn refresh_action_ref(&self, name: &str, id: &str, out: &mut Out) {
        let records: Vec<Value> = self
            .walk()
            .iter()
            .filter(|p| Self::matches(&self.workflows, p))
            .filter_map(|rel| {
                Some(workflows::record(
                    rel,
                    &std::fs::read_to_string(self.root.join(rel)).ok()?,
                ))
            })
            .collect();
        match workflows::action_refs(&records)
            .into_iter()
            .find(|(n, _)| n == name)
        {
            Some((_, data)) => out.record("action-ref", id, data),
            None => out.not_found("action-ref", id),
        }
    }

    fn refresh(&self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        let mut crates = BTreeSet::new();
        for id in cmd["external_ids"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if id.starts_with("repo:") {
                let (name, data) = repo::record(&self.root, &self.walk());
                if id == format!("repo:{name}") {
                    out.record("repo", id, data);
                } else {
                    out.not_found("repo", id);
                }
            } else if let Some(name) = id.strip_prefix("crate:") {
                crates.insert(name.to_owned());
            } else if let Some(rel) = id.strip_prefix("doc:") {
                match self.doc_record(rel) {
                    Some(r) => out.record("doc", id, r),
                    None => out.not_found("doc", id),
                }
            } else if let Some(rel) = id.strip_prefix("workflow:") {
                match std::fs::read_to_string(self.root.join(rel)) {
                    Ok(text) => out.record("workflow", id, workflows::record(rel, &text)),
                    Err(_) => out.not_found("workflow", id),
                }
            } else if let Some(name) = id.strip_prefix("action-ref:") {
                self.refresh_action_ref(name, id, out);
            } else if let Some(rel) = id.strip_prefix("file:") {
                match self.file_record(rel) {
                    Some(r) => out.record("file", id, r),
                    None => out.not_found("file", id),
                }
            } else {
                out.not_found(id.split(':').next().unwrap_or("record"), id);
            }
        }
        if !crates.is_empty() {
            self.emit_crates(Some(&crates), out)?;
        }
        Ok(())
    }
}

impl Source for Workspace {
    fn name(&self) -> &str {
        "workspace"
    }

    fn capabilities(&self) -> Value {
        // Only the kinds this configuration produces.
        let on = [
            ("repo", self.repo),
            ("crate", self.crates),
            ("doc", !self.docs.is_empty()),
            ("file", !self.files.is_empty()),
            ("betterer", !self.betterer.is_empty()),
            ("workflow", !self.workflows.is_empty()),
            ("action-ref", !self.workflows.is_empty()),
        ];
        let kinds: Vec<&str> = on.iter().filter(|(_, b)| *b).map(|(k, _)| *k).collect();
        let refresh: Map<String, Value> = kinds
            .iter()
            .filter(|k| **k != "betterer")
            .map(|k| (k.to_string(), json!(["external_ids"])))
            .collect();
        json!({"refresh": refresh, "resync": kinds, "max_concurrency": 1})
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => self.resync(cmd, out),
            Some("refresh") => self.refresh(cmd, out),
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
