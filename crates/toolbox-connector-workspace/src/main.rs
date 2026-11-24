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
//!   per file for file tests), so existing Betterer ratchets roll up into scorecards;
//! - `skill` (`skills` globs): one per Agent Skills directory (`SKILL.md`), with spec frontmatter
//!   checks and the counts `maintainer-skill` thresholds (docs/design/skill-standards.md);
//! - `connector` (`connectors` globs, usually `crates/toolbox-connector-*/Cargo.toml`): one per
//!   first-party connector crate, with the protocol facts `maintainer-connector` thresholds
//!   (docs/design/connector-standards.md);
//! - `dependency` (`"dependencies": true`): one per direct external Cargo dependency, with the
//!   requirement, the `Cargo.lock` versions and whether `[workspace.dependencies]` declares it
//!   (docs/design/dependency-standards.md);
//! - `agent-doc` (`agentDocs` globs): one per `AGENTS.md` and the aliases harnesses still read
//!   (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, Copilot instructions, Cursor and Claude rule files),
//!   with link, command and size facts (docs/design/agents-md-standards.md);
//! - `plugin` (`plugins` globs): one per Agent Plugins `plugin.json`, with the 1.0.0 manifest and
//!   `mcp.json` facts `maintainer-plugin` thresholds (docs/design/plugin-standards.md);
//! - `package-json` (`packages` globs): one per npm manifest, with its ranges, lockfile and the
//!   script CI can run (docs/design/package-standards.md);
//! - `js-workspace` (`"jsWorkspace": true`): one record for the checkout root — pnpm/npm workspace
//!   file, `packageManager` pin, root lockfile, linker, mise pins and frozen CI
//!   (packages/pnpm-expo);
//! - `expo-app` (`expoApps` globs): one per Expo `app.json` / `app.config.*`, with SDK deps, EAS,
//!   Metro and doctor-in-CI facts (packages/pnpm-expo);
//! - `storybook` (`storybooks` globs): one per Storybook `main.*` config, with deps, a11y addon,
//!   telemetry, story files and CI build/test signals (packages/storybook);
//! - `json-schema` (`schemas` globs): one per JSON Schema document, with its dialect, id and
//!   whether the binary compiles it (docs/design/schema-standards.md);
//! - `site-page` (`pages` globs): one per Starlight content page, with its title, sidebar reach,
//!   links and secret shapes (docs/design/site-standards.md);
//! - `example` (`examples` globs): one per example or experiment write-up, with its links and the
//!   command that checks it (docs/design/example-standards.md);
//! - `portal` (`portals` globs, usually `**/.toolbox/toolbox.yaml`): one per child portal a parent
//!   federates, with the actions it declares and how to reach it (docs/design/federated-actions.md);
//! - `assessment` (`assessments` globs): one per assessment file (YAML or JSON, see
//!   `schema/assessment.schema.json`): agents' judgements of entities against the portal's rubrics.
//!
//! `docs` and `file` records also carry `captures` (each configured regex's first group, unique,
//! sorted, with runs of whitespace replaced by `-`) and `counts` (matches per configured regex),
//! so a portal can relate files to the
//! commands and tools they mention without the connector knowing about them. `file` records also
//! carry `matchLines` (for each `counts` regex, the line of every match).
//!
//! `exclude` globs remove paths from every walk; `"lastChange": false` leaves `lastChange` null
//! and skips the history walk. The full reference is docs/reference/workspace-connector.md.
//!
//! `TOOLBOX_CONNECTOR_CONFIG`: `{"root"?: ".", "exclude"?: [glob], "lastChange"?: true,
//! "docs"?: [glob], "files"?: [glob],
//! "captures"?: {name: regex}, "counts"?: {name: regex}, "crates"?: true, "repo"?: false,
//! "betterer"?: [glob], "skills"?: [glob], "connectors"?: [glob], "dependencies"?: false,
//! "agentDocs"?: [glob], "plugins"?: [glob], "packages"?: [glob], "schemas"?: [glob],
//! "examples"?: [glob], "pages"?: [glob], "assessments"?: [glob], "jsWorkspace"?: false,
//! "expoApps"?: [glob], "storybooks"?: [glob], "portals"?: [glob]}`.

mod agentsmd;
mod betterer;
mod connectors;
mod data;
mod deps;
mod examples;
mod expo;
mod hygiene;
mod js_workspace;
mod npm;
mod plugins;
mod portals;
mod repo;
mod rust;
mod site;
mod skills;
mod specs;
mod storybook;
mod workflows;

use regex::Regex;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toolbox_connect::files::{self, glob};
use toolbox_connect::sdk::{Out, Source, run_stdio};

struct Workspace {
    root: PathBuf,
    /// Paths never read, whatever globs match them (generated code, vendored trees, tests).
    exclude: Vec<Regex>,
    docs: Vec<Regex>,
    files: Vec<Regex>,
    betterer: Vec<Regex>,
    /// Assessment files (`assessment` records).
    assessments: Vec<Regex>,
    /// GitHub Actions workflow and action metadata files (`workflow` and `action-ref` records).
    workflows: Vec<Regex>,
    /// Agent skill files (`skill` records), usually `**/SKILL.md`.
    skills: Vec<Regex>,
    /// Connector crate manifests (`connector` records), usually `crates/toolbox-connector-*/Cargo.toml`.
    connectors: Vec<Regex>,
    /// Emit one `dependency` record per direct external Cargo dependency.
    dependencies: bool,
    /// Agent instruction files (`agent-doc` records): `AGENTS.md` and harness aliases.
    agent_docs: Vec<Regex>,
    /// Agent plugin manifests (`plugin` records), usually `plugins/*/plugin.json`.
    plugins: Vec<Regex>,
    /// npm manifests (`package-json` records), usually `**/package.json`.
    packages: Vec<Regex>,
    /// JSON Schema documents (`json-schema` records), usually `schema/*.json`.
    schemas: Vec<Regex>,
    /// Example and experiment write-ups (`example` records).
    examples: Vec<Regex>,
    /// Starlight content pages (`site-page` records).
    pages: Vec<Regex>,
    /// Expo app configs (`expo-app` records), e.g. `apps/*/app.json`.
    expo_apps: Vec<Regex>,
    /// Storybook main configs (`storybook` records), e.g. `**/.storybook/main.*`.
    storybooks: Vec<Regex>,
    /// Child portal manifests (`portal` records), e.g. `**/.toolbox/toolbox.yaml`.
    portals: Vec<Regex>,
    captures: Vec<(String, Regex)>,
    counts: Vec<(String, Regex)>,
    crates: bool,
    /// Emit one `repo` record for the checkout (opt-in: `"repo": true`).
    repo: bool,
    /// Emit one `js-workspace` record for the checkout root (opt-in: `"jsWorkspace": true`).
    js_workspace: bool,
    /// Declared records (`data`): YAML or JSON files read into a configured record kind.
    data: Vec<data::Source>,
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
/// `"lastChange": false` turns the field off (it is `null`): the history walk is the slowest part
/// of a large checkout, and only standards about staleness read it.
static LAST_CHANGE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Per checkout root: each path's last commit date, from one `git log` over the history (instead
/// of one process per file, which took 36 s on PostHog's 22,000 files).
type Dates = std::sync::Arc<BTreeMap<String, String>>;
static LAST_CHANGES: std::sync::OnceLock<std::sync::Mutex<BTreeMap<PathBuf, Dates>>> =
    std::sync::OnceLock::new();

fn last_changes(root: &Path) -> Dates {
    let cache = LAST_CHANGES.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap_or_else(|e| e.into_inner());
    cache
        .entry(root.to_path_buf())
        .or_insert_with(|| {
            let mut map = BTreeMap::new();
            let out = std::process::Command::new("git")
                .args([
                    "log",
                    "--format=%x01%cs",
                    "--name-only",
                    "--relative",
                    "--no-renames",
                ])
                .current_dir(root)
                .output();
            let text = out.map(|o| o.stdout).unwrap_or_default();
            let mut date = String::new();
            for line in String::from_utf8_lossy(&text).lines() {
                if let Some(d) = line.strip_prefix('\u{1}') {
                    date = d.to_owned();
                } else if !line.is_empty() {
                    // Newest first: the first date seen for a path is its last change.
                    map.entry(line.to_owned()).or_insert_with(|| date.clone());
                }
            }
            std::sync::Arc::new(map)
        })
        .clone()
}

fn last_change(root: &Path, path: &str) -> Option<String> {
    if !LAST_CHANGE.load(std::sync::atomic::Ordering::Relaxed) {
        return None;
    }
    last_changes(root).get(path).cloned()
}

fn workflow_text(root: &Path, paths: &[String], globs: &[Regex]) -> String {
    paths
        .iter()
        .filter(|p| globs.iter().any(|r| r.is_match(p)))
        .filter_map(|p| std::fs::read_to_string(root.join(p)).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

/// `D84` stays `D84`. A ULID citation is canonicalised to uppercase.
fn canon_decision(id: &str) -> String {
    let rest = id.get(1..).unwrap_or("");
    if rest.chars().all(|c| c.is_ascii_digit()) {
        format!("D{rest}")
    } else {
        format!("D{}", rest.to_ascii_uppercase())
    }
}

/// `id` and `legacy` from a decision file's frontmatter.
fn decision_file_ids(text: &str) -> Vec<String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Vec::new();
    };
    let Some(end) = rest.find("\n---") else {
        return Vec::new();
    };
    rest[..end]
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            let key = key.trim();
            if key != "id" && key != "legacy" {
                return None;
            }
            let value = value.trim().trim_matches('"');
            (!value.is_empty()).then(|| canon_decision(value))
        })
        .collect()
}

/// Decision ids declared by `docs/decisions/*.md` (`id`, and `legacy` for a backfilled row).
fn known_decisions(root: &Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let Ok(dir) = std::fs::read_dir(root.join("docs/decisions")) else {
        return ids;
    };
    for entry in dir.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            ids.extend(decision_file_ids(&text));
        }
    }
    ids
}

/// Citations that are not a decision file. The log directory is the index, so it is not checked.
/// A citation is a legacy `D` plus one to four digits, or `D` plus a 26-character ULID.
fn missing_decisions(text: &str, known: &BTreeSet<String>, rel: &str) -> Vec<String> {
    if rel == "docs/decisions.md" || rel.starts_with("docs/decisions/") {
        return Vec::new();
    }
    let re = Regex::new(r"(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b").unwrap();
    let mut missing: Vec<String> = re
        .captures_iter(text)
        .filter_map(|c| c.get(1).map(|m| canon_decision(m.as_str())))
        .filter(|id| !known.contains(id))
        .collect();
    missing.sort();
    missing.dedup();
    missing
}

/// Relative Markdown links in `text` (the document at `rel`): how many, and the targets that don't
/// resolve to a file. A target resolves as a reader would follow it: as written, or as a page
/// (`x.md`, `x.mdx`, `x/index.md`, `x/README.md`). A link starting with `/` is relative to the
/// repository root on GitHub, and to a site's content root on a docs site, which is some folder
/// above the document, possibly behind a base path (`/gh-aw/reference/auth/` for
/// `docs/src/content/docs/reference/auth.mdx`).
fn links_in(root: &Path, rel: &str, text: &str) -> (usize, BTreeSet<String>) {
    let link = Regex::new(r"\]\(([^)\s#]+)(?:#[^)]*)?\)").unwrap();
    let dir = Path::new(rel).parent().unwrap_or(Path::new(""));
    let (mut links, mut broken) = (0, BTreeSet::new());
    // Code is an example, not a link: drop fenced blocks and inline code first.
    let fenced = Regex::new(r"(?ms)^\s*(```|~~~).*?^\s*(```|~~~)").unwrap();
    let inline = Regex::new(r"`[^`\n]*`").unwrap();
    let prose = inline
        .replace_all(&fenced.replace_all(text, ""), "")
        .into_owned();
    for c in link.captures_iter(&prose) {
        let target = &c[1];
        if target.contains("://") || target.starts_with("mailto:") {
            continue;
        }
        links += 1;
        if !resolves(root, dir, target) {
            broken.insert(target.to_owned());
        }
    }
    (links, broken)
}

/// Whether `target`, linked from a document in `dir`, names a file (see [`links_in`]).
fn resolves(root: &Path, dir: &Path, target: &str) -> bool {
    let page = |p: PathBuf| {
        let s = p.to_string_lossy().trim_end_matches('/').to_owned();
        [
            s.clone(),
            format!("{s}.md"),
            format!("{s}.mdx"),
            format!("{s}/index.md"),
            format!("{s}/index.mdx"),
            format!("{s}/README.md"),
        ]
        .iter()
        .any(|c| Path::new(c).exists())
    };
    let Some(route) = target.strip_prefix('/') else {
        return page(root.join(dir).join(target));
    };
    let route = route.trim_end_matches('/');
    let behind_base = route.split_once('/').map_or("", |(_, rest)| rest);
    dir.ancestors().any(|above| {
        [route, behind_base]
            .iter()
            .any(|r| !r.is_empty() && page(root.join(above).join(r)))
    })
}

impl Workspace {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("TOOLBOX_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("TOOLBOX_CONNECTOR_CONFIG: {e}"))?;
        LAST_CHANGE.store(
            cfg["lastChange"].as_bool().unwrap_or(true),
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(Workspace {
            root: PathBuf::from(cfg["root"].as_str().unwrap_or(".")),
            exclude: globs(&cfg, "exclude")?,
            docs: globs(&cfg, "docs")?,
            files: globs(&cfg, "files")?,
            betterer: globs(&cfg, "betterer")?,
            assessments: globs(&cfg, "assessments")?,
            workflows: globs(&cfg, "workflows")?,
            skills: globs(&cfg, "skills")?,
            connectors: globs(&cfg, "connectors")?,
            dependencies: cfg["dependencies"].as_bool().unwrap_or(false),
            agent_docs: globs(&cfg, "agentDocs")?,
            plugins: globs(&cfg, "plugins")?,
            packages: globs(&cfg, "packages")?,
            schemas: globs(&cfg, "schemas")?,
            examples: globs(&cfg, "examples")?,
            pages: globs(&cfg, "pages")?,
            expo_apps: globs(&cfg, "expoApps")?,
            storybooks: globs(&cfg, "storybooks")?,
            portals: globs(&cfg, "portals")?,
            captures: named_regexes(&cfg, "captures")?,
            counts: named_regexes(&cfg, "counts")?,
            crates: cfg["crates"].as_bool().unwrap_or(true),
            repo: cfg["repo"].as_bool().unwrap_or(false),
            js_workspace: cfg["jsWorkspace"].as_bool().unwrap_or(false),
            data: data::sources(&cfg)?,
        })
    }

    /// Relative paths of every file under the root (skipping build and VCS directories and
    /// `exclude` globs).
    fn walk(&self) -> Vec<String> {
        let mut paths = files::walk(&self.root);
        paths.retain(|p| !Self::matches(&self.exclude, p));
        paths
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
        let (links, broken) = links_in(&self.root, rel, &text);
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
            "missingDecisions": missing_decisions(&text, &known_decisions(&self.root), rel),
        }))
    }

    /// For each `counts` regex, the 1-based line of every match (so evidence can say where).
    fn match_lines(&self, text: &str) -> Value {
        let starts: Vec<usize> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        let line = |at: usize| starts.partition_point(|&s| s <= at);
        let out: Map<String, Value> = self
            .counts
            .iter()
            .map(|(name, re)| {
                let lines: Vec<usize> = re.find_iter(text).map(|m| line(m.start())).collect();
                (name.clone(), json!(lines))
            })
            .collect();
        Value::Object(out)
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
            "matchLines": self.match_lines(&text),
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
        let mut kinds: Vec<&str> = cmd["kinds"]
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
                    "skill",
                    "connector",
                    "dependency",
                    "agent-doc",
                    "plugin",
                    "package-json",
                    "json-schema",
                    "example",
                    "site-page",
                    "assessment",
                    "js-workspace",
                    "expo-app",
                    "storybook",
                ]
            });
        if cmd["kinds"].as_array().is_none() {
            kinds.extend(self.data.iter().map(|d| d.kind.as_str()));
        }
        if self.crates && kinds.contains(&"crate") {
            self.emit_crates(None, out)?;
            out.snapshot_complete("crate");
        }
        let paths = self.walk();
        self.emit_repo(&paths, &kinds, out);
        self.emit_docs_and_files(&paths, &kinds, out);
        self.emit_workflows(&paths, &kinds, out);
        self.emit_betterer(&paths, &kinds, out);
        self.emit_data(&paths, &kinds, out);
        self.emit_assessments(&paths, &kinds, out);
        self.emit_skills(&paths, &kinds, out);
        self.emit_connectors(&paths, &kinds, out);
        self.emit_dependencies(&kinds, out);
        self.emit_agent_docs(&paths, &kinds, out);
        self.emit_plugins(&paths, &kinds, out);
        self.emit_packages(&paths, &kinds, out);
        self.emit_schemas(&paths, &kinds, out);
        self.emit_examples(&paths, &kinds, out);
        self.emit_portals(&paths, &kinds, out);
        self.emit_pages(&paths, &kinds, out);
        self.emit_expo_apps(&paths, &kinds, out);
        self.emit_storybooks(&paths, &kinds, out);
        self.emit_js_workspace(&paths, &kinds, out);
        Ok(())
    }

    fn emit_repo(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.repo && kinds.contains(&"repo") {
            let (name, data) = repo::record(&self.root, paths);
            out.record("repo", &format!("repo:{name}"), data);
            out.snapshot_complete("repo");
        }
    }

    fn emit_docs_and_files(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
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
    }

    /// `assessment` records; files that aren't assessments are logged and skipped.
    fn emit_assessments(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if !kinds.contains(&"assessment") || self.assessments.is_empty() {
            return;
        }
        let mut records = Vec::new();
        for rel in paths.iter().filter(|p| Self::matches(&self.assessments, p)) {
            match self.assessment(rel) {
                Ok(r) => records.push(r),
                Err(e) => out.log("warn", &format!("{rel}: {e}")),
            }
        }
        toolbox_connect::assessment::link_reviews(&mut records);
        for (id, data) in records {
            out.record("assessment", &id, data);
        }
        out.snapshot_complete("assessment");
    }

    /// One assessment file as `(external id, record)`.
    fn assessment(&self, rel: &str) -> Result<(String, Value), String> {
        let text = std::fs::read_to_string(self.root.join(rel)).map_err(|e| e.to_string())?;
        let doc: Value = serde_norway::from_str(&text).map_err(|e| e.to_string())?;
        if !doc["entity"].is_string() || !doc["rubric"].is_string() {
            return Err("not an assessment (needs entity and rubric)".into());
        }
        let id = toolbox_connect::assessment::external_id(&doc);
        Ok((
            id,
            toolbox_connect::assessment::record_in(&doc, Some(rel), Some(&self.root)),
        ))
    }

    fn emit_betterer(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if !kinds.contains(&"betterer") || self.betterer.is_empty() {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.betterer, p)) {
            let text = std::fs::read_to_string(self.root.join(rel)).unwrap_or_default();
            for (test, mut data) in betterer::parse(&text) {
                data["path"] = json!(rel);
                out.record("betterer", &format!("betterer:{rel}#{test}"), data);
            }
        }
        out.snapshot_complete("betterer");
    }

    fn skill_record(&self, rel: &str) -> Value {
        let mut rec = skills::analyse(&self.root, rel);
        if let Some(obj) = rec.as_object_mut() {
            obj.insert("lastChange".into(), json!(last_change(&self.root, rel)));
        }
        rec
    }

    fn connector_record(&self, rel: &str) -> Value {
        let mut rec = connectors::analyse(&self.root, rel);
        if let Some(obj) = rec.as_object_mut() {
            obj.insert("lastChange".into(), json!(last_change(&self.root, rel)));
        }
        rec
    }

    fn emit_connectors(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.connectors.is_empty() || !kinds.contains(&"connector") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.connectors, p)) {
            let rec = self.connector_record(rel);
            let dir = rec["dir"].as_str().unwrap_or(rel).to_owned();
            out.record("connector", &format!("connector:{dir}"), rec);
        }
        out.snapshot_complete("connector");
    }

    fn emit_dependencies(&self, kinds: &[&str], out: &mut Out) {
        if !self.dependencies || !kinds.contains(&"dependency") {
            return;
        }
        if let Ok(meta) = self.cargo_metadata() {
            for rec in deps::records(&self.root, &meta) {
                let name = rec["name"].as_str().unwrap_or_default();
                let eco = rec["ecosystem"].as_str().unwrap_or("cargo");
                out.record("dependency", &format!("dependency:{eco}:{name}"), rec);
            }
        }
        out.snapshot_complete("dependency");
    }

    /// Declared records for each configured `data` kind that was requested.
    fn emit_data(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        for src in self
            .data
            .iter()
            .filter(|d| kinds.contains(&d.kind.as_str()))
        {
            for rel in paths.iter().filter(|p| Self::matches(&src.globs, p)) {
                match data::records(&self.root, rel, src) {
                    Ok(recs) => {
                        for (id, rec) in recs {
                            out.record(&src.kind, &id, rec);
                        }
                    }
                    Err(e) => out.log("warn", &format!("{rel}: {e}")),
                }
            }
            out.snapshot_complete(&src.kind);
        }
    }

    fn emit_examples(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.examples.is_empty() || !kinds.contains(&"example") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.examples, p)) {
            out.record(
                "example",
                &format!("example:{rel}"),
                examples::analyse(&self.root, rel),
            );
        }
        out.snapshot_complete("example");
    }

    fn emit_portals(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.portals.is_empty() || !kinds.contains(&"portal") {
            return;
        }
        let found: Vec<(&String, Value)> = paths
            .iter()
            .filter(|p| Self::matches(&self.portals, p))
            .map(|rel| (rel, portals::analyse(&self.root, rel)))
            .collect();
        for (rel, rec) in portals::refuse_duplicates(found) {
            out.record("portal", &portals::id(rel), rec);
        }
        out.snapshot_complete("portal");
    }

    fn emit_pages(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.pages.is_empty() || !kinds.contains(&"site-page") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.pages, p)) {
            out.record(
                "site-page",
                &format!("site-page:{rel}"),
                site::analyse(&self.root, rel),
            );
        }
        out.snapshot_complete("site-page");
    }

    fn emit_schemas(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.schemas.is_empty() || !kinds.contains(&"json-schema") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.schemas, p)) {
            out.record(
                "json-schema",
                &format!("json-schema:{rel}"),
                specs::analyse(&self.root, rel),
            );
        }
        out.snapshot_complete("json-schema");
    }

    fn emit_packages(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.packages.is_empty() || !kinds.contains(&"package-json") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.packages, p)) {
            out.record(
                "package-json",
                &format!("package-json:{rel}"),
                npm::analyse(&self.root, rel),
            );
        }
        out.snapshot_complete("package-json");
    }

    fn expo_app_paths<'a>(&self, paths: &'a [String]) -> Vec<&'a String> {
        paths
            .iter()
            .filter(|p| Self::matches(&self.expo_apps, p))
            .collect()
    }

    fn emit_expo_apps(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.expo_apps.is_empty() || !kinds.contains(&"expo-app") {
            return;
        }
        for rel in self.expo_app_paths(paths) {
            let rec = expo::analyse(&self.root, rel, paths);
            let dir = rec["path"].as_str().unwrap_or(rel).to_owned();
            out.record("expo-app", &format!("expo-app:{dir}"), rec);
        }
        out.snapshot_complete("expo-app");
    }

    fn storybook_paths<'a>(&self, paths: &'a [String]) -> Vec<&'a String> {
        paths
            .iter()
            .filter(|p| Self::matches(&self.storybooks, p))
            .collect()
    }

    fn emit_storybooks(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.storybooks.is_empty() || !kinds.contains(&"storybook") {
            return;
        }
        for rel in self.storybook_paths(paths) {
            let rec = storybook::analyse(&self.root, rel, paths);
            let dir = rec["path"].as_str().unwrap_or(rel).to_owned();
            out.record("storybook", &format!("storybook:{dir}"), rec);
        }
        out.snapshot_complete("storybook");
    }

    fn emit_js_workspace(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if !self.js_workspace || !kinds.contains(&"js-workspace") {
            return;
        }
        let has_expo = !self.expo_apps.is_empty() && !self.expo_app_paths(paths).is_empty();
        let rec = js_workspace::record(&self.root, paths, has_expo);
        out.record("js-workspace", "js-workspace:.", rec);
        out.snapshot_complete("js-workspace");
    }

    fn emit_agent_docs(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.agent_docs.is_empty() || !kinds.contains(&"agent-doc") {
            return;
        }
        let workflows = workflow_text(&self.root, paths, &self.workflows);
        for rel in paths.iter().filter(|p| Self::matches(&self.agent_docs, p)) {
            out.record(
                "agent-doc",
                &format!("agent-doc:{rel}"),
                agentsmd::analyse(&self.root, rel, &workflows),
            );
        }
        out.snapshot_complete("agent-doc");
    }

    fn emit_plugins(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.plugins.is_empty() || !kinds.contains(&"plugin") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.plugins, p)) {
            let rec = plugins::analyse(&self.root, rel);
            let dir = rec["dir"].as_str().unwrap_or(rel).to_owned();
            out.record("plugin", &format!("plugin:{dir}"), rec);
        }
        out.snapshot_complete("plugin");
    }

    fn emit_skills(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.skills.is_empty() || !kinds.contains(&"skill") {
            return;
        }
        for rel in paths.iter().filter(|p| Self::matches(&self.skills, p)) {
            let rec = self.skill_record(rel);
            let dir = rec["dir"].as_str().unwrap_or(rel).to_owned();
            out.record("skill", &format!("skill:{dir}"), rec);
        }
        out.snapshot_complete("skill");
    }

    /// `workflow` records for files matching `workflows`, and `action-ref` records derived from them.
    fn emit_workflows(&self, paths: &[String], kinds: &[&str], out: &mut Out) {
        if self.workflows.is_empty() {
            return;
        }
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
            self.refresh_id(id, &mut crates, out);
        }
        if !crates.is_empty() {
            self.emit_crates(Some(&crates), out)?;
        }
        Ok(())
    }

    fn refresh_id(&self, id: &str, crates: &mut BTreeSet<String>, out: &mut Out) {
        let kind = id.split(':').next().unwrap_or("record");
        if self.refresh_catalog(kind, id, out) {
            return;
        }
        if let Some(src) = self.data.iter().find(|d| d.kind == kind) {
            self.refresh_data(src, id, out);
            return;
        }
        match kind {
            "crate" => {
                if let Some(name) = id.strip_prefix("crate:") {
                    crates.insert(name.to_owned());
                }
            }
            "repo" => self.refresh_repo(id, out),
            "doc" | "file" => self.refresh_text(id, kind, out),
            "workflow" => self.refresh_workflow(id, out),
            "action-ref" => self.refresh_named_action(id, out),
            "dependency" => self.refresh_dependency(id, out),
            _ => out.not_found(kind, id),
        }
    }

    /// File-shaped records. Returns whether `kind` was one of them.
    fn refresh_catalog(&self, kind: &str, id: &str, out: &mut Out) -> bool {
        match kind {
            "skill" => self.refresh_skill(id, out),
            "connector" => self.refresh_connector(id, out),
            "agent-doc" => self.refresh_agent_doc(id, out),
            "plugin" => self.refresh_plugin(id, out),
            "package-json" => self.refresh_package(id, out),
            "json-schema" => self.refresh_schema(id, out),
            "example" => self.refresh_example(id, out),
            "portal" => self.refresh_portal(id, out),
            "site-page" => self.refresh_page(id, out),
            "expo-app" => self.refresh_expo_app(id, out),
            "storybook" => self.refresh_storybook(id, out),
            "js-workspace" => self.refresh_js_workspace(id, out),
            _ => return false,
        }
        true
    }

    /// One declared record: re-read the source's files and emit the entry with this id.
    fn refresh_data(&self, src: &data::Source, id: &str, out: &mut Out) {
        let found = self
            .walk()
            .iter()
            .filter(|p| Self::matches(&src.globs, p))
            .filter_map(|rel| data::records(&self.root, rel, src).ok())
            .flatten()
            .find(|(eid, _)| eid == id);
        match found {
            Some((_, rec)) => out.record(&src.kind, id, rec),
            None => out.not_found(&src.kind, id),
        }
    }

    fn refresh_repo(&self, id: &str, out: &mut Out) {
        let (name, data) = repo::record(&self.root, &self.walk());
        if id == format!("repo:{name}") {
            out.record("repo", id, data);
        } else {
            out.not_found("repo", id);
        }
    }

    fn refresh_text(&self, id: &str, kind: &str, out: &mut Out) {
        let rel = id.split_once(':').map(|(_, r)| r).unwrap_or("");
        let rec = if kind == "doc" {
            self.doc_record(rel)
        } else {
            self.file_record(rel)
        };
        match rec {
            Some(r) => out.record(kind, id, r),
            None => out.not_found(kind, id),
        }
    }

    fn refresh_workflow(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("workflow:").unwrap_or("");
        match std::fs::read_to_string(self.root.join(rel)) {
            Ok(text) => out.record("workflow", id, workflows::record(rel, &text)),
            Err(_) => out.not_found("workflow", id),
        }
    }

    fn refresh_named_action(&self, id: &str, out: &mut Out) {
        let name = id.strip_prefix("action-ref:").unwrap_or("");
        self.refresh_action_ref(name, id, out);
    }

    fn refresh_page(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("site-page:").unwrap_or("");
        if Self::matches(&self.pages, rel) && self.root.join(rel).is_file() {
            out.record("site-page", id, site::analyse(&self.root, rel));
        } else {
            out.not_found("site-page", id);
        }
    }

    fn refresh_example(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("example:").unwrap_or("");
        if Self::matches(&self.examples, rel) && self.root.join(rel).is_file() {
            out.record("example", id, examples::analyse(&self.root, rel));
        } else {
            out.not_found("example", id);
        }
    }

    fn refresh_portal(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("portal:").unwrap_or("");
        if Self::matches(&self.portals, rel) && self.root.join(rel).is_file() {
            out.record("portal", id, portals::analyse(&self.root, rel));
        } else {
            out.not_found("portal", id);
        }
    }

    fn refresh_schema(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("json-schema:").unwrap_or("");
        if Self::matches(&self.schemas, rel) && self.root.join(rel).is_file() {
            out.record("json-schema", id, specs::analyse(&self.root, rel));
        } else {
            out.not_found("json-schema", id);
        }
    }

    fn refresh_package(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("package-json:").unwrap_or("");
        if Self::matches(&self.packages, rel) && self.root.join(rel).is_file() {
            out.record("package-json", id, npm::analyse(&self.root, rel));
        } else {
            out.not_found("package-json", id);
        }
    }

    fn refresh_expo_app(&self, id: &str, out: &mut Out) {
        let dir = id.strip_prefix("expo-app:").unwrap_or("");
        let paths = self.walk();
        let found = self.expo_app_paths(&paths).into_iter().find(|rel| {
            let rec = expo::analyse(&self.root, rel, &paths);
            rec["path"].as_str() == Some(dir)
        });
        match found {
            Some(rel) => out.record("expo-app", id, expo::analyse(&self.root, rel, &paths)),
            None => out.not_found("expo-app", id),
        }
    }

    fn refresh_storybook(&self, id: &str, out: &mut Out) {
        let dir = id.strip_prefix("storybook:").unwrap_or("");
        let paths = self.walk();
        let found = self.storybook_paths(&paths).into_iter().find(|rel| {
            let rec = storybook::analyse(&self.root, rel, &paths);
            rec["path"].as_str() == Some(dir)
        });
        match found {
            Some(rel) => out.record("storybook", id, storybook::analyse(&self.root, rel, &paths)),
            None => out.not_found("storybook", id),
        }
    }

    fn refresh_js_workspace(&self, id: &str, out: &mut Out) {
        if id != "js-workspace:." || !self.js_workspace {
            out.not_found("js-workspace", id);
            return;
        }
        let paths = self.walk();
        let has_expo = !self.expo_apps.is_empty() && !self.expo_app_paths(&paths).is_empty();
        out.record(
            "js-workspace",
            id,
            js_workspace::record(&self.root, &paths, has_expo),
        );
    }

    fn refresh_agent_doc(&self, id: &str, out: &mut Out) {
        let rel = id.strip_prefix("agent-doc:").unwrap_or("");
        if Self::matches(&self.agent_docs, rel) && self.root.join(rel).is_file() {
            let paths = self.walk();
            let workflows = workflow_text(&self.root, &paths, &self.workflows);
            out.record(
                "agent-doc",
                id,
                agentsmd::analyse(&self.root, rel, &workflows),
            );
        } else {
            out.not_found("agent-doc", id);
        }
    }

    fn refresh_plugin(&self, id: &str, out: &mut Out) {
        let dir = id.strip_prefix("plugin:").unwrap_or("");
        let rel = format!("{dir}/plugin.json");
        if Self::matches(&self.plugins, &rel) && self.root.join(&rel).is_file() {
            out.record("plugin", id, plugins::analyse(&self.root, &rel));
        } else {
            out.not_found("plugin", id);
        }
    }

    fn refresh_dependency(&self, id: &str, out: &mut Out) {
        let name = id.rsplit(':').next().unwrap_or("");
        let found = self.cargo_metadata().ok().and_then(|meta| {
            deps::records(&self.root, &meta)
                .into_iter()
                .find(|r| r["name"].as_str() == Some(name))
        });
        match found {
            Some(rec) => out.record("dependency", id, rec),
            None => out.not_found("dependency", id),
        }
    }

    fn refresh_connector(&self, id: &str, out: &mut Out) {
        let dir = id.strip_prefix("connector:").unwrap_or("");
        let rel = format!("{dir}/Cargo.toml");
        if Self::matches(&self.connectors, &rel) && self.root.join(&rel).is_file() {
            out.record("connector", id, self.connector_record(&rel));
        } else {
            out.not_found("connector", id);
        }
    }

    fn refresh_skill(&self, id: &str, out: &mut Out) {
        let dir = id.strip_prefix("skill:").unwrap_or("");
        let rel = format!("{dir}/SKILL.md");
        if Self::matches(&self.skills, &rel) && self.root.join(&rel).is_file() {
            out.record("skill", id, self.skill_record(&rel));
        } else {
            out.not_found("skill", id);
        }
    }
}

impl Source for Workspace {
    fn name(&self) -> &str {
        "workspace"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
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
            ("skill", !self.skills.is_empty()),
            ("connector", !self.connectors.is_empty()),
            ("dependency", self.dependencies),
            ("agent-doc", !self.agent_docs.is_empty()),
            ("plugin", !self.plugins.is_empty()),
            ("package-json", !self.packages.is_empty()),
            ("json-schema", !self.schemas.is_empty()),
            ("example", !self.examples.is_empty()),
            ("site-page", !self.pages.is_empty()),
            ("assessment", !self.assessments.is_empty()),
            ("js-workspace", self.js_workspace),
            ("expo-app", !self.expo_apps.is_empty()),
            ("storybook", !self.storybooks.is_empty()),
            ("portal", !self.portals.is_empty()),
        ];
        let mut kinds: Vec<&str> = on.iter().filter(|(_, b)| *b).map(|(k, _)| *k).collect();
        let declared: Vec<&str> = self.data.iter().map(|d| d.kind.as_str()).collect();
        kinds.extend(&declared);
        let refresh: Map<String, Value> = kinds
            .iter()
            .filter(|k| !matches!(**k, "betterer" | "assessment"))
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

#[cfg(test)]
mod decision_ids {
    use super::{decision_file_ids, known_decisions, missing_decisions};

    #[test]
    fn a_file_declares_its_id_and_its_legacy_number() {
        let text = "---\nid: D01ARYZ6S410000000000000000\nlegacy: D84\ndecided: 2026-11-20T14:12:04Z\n---\n\n# D84\n";
        assert_eq!(
            decision_file_ids(text),
            vec!["D01ARYZ6S410000000000000000".to_string(), "D84".to_string()]
        );
    }

    #[test]
    fn root_relative_links_resolve_from_the_repository_root() {
        let root = std::env::temp_dir().join(format!("toolbox-links-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("docs/internal")).unwrap();
        std::fs::create_dir_all(root.join("products")).unwrap();
        std::fs::write(root.join("products/README.md"), "# Products\n").unwrap();
        let text = "[a](/products/README.md) [b](../../products/README.md) [c](/missing.md) [d](https://x.dev)";
        let (links, broken) = crate::links_in(&root, "docs/internal/layout.md", text);
        assert_eq!(links, 3);
        assert_eq!(broken.into_iter().collect::<Vec<_>>(), ["/missing.md"]);
        // A docs site: pages without extensions, and routes behind a base path from the content root.
        let site = root.join("site/content/docs");
        std::fs::create_dir_all(site.join("reference")).unwrap();
        std::fs::write(site.join("reference/auth.mdx"), "# Auth\n").unwrap();
        std::fs::write(site.join("reference/audit.md"), "# Audit\n").unwrap();
        let text = "[r](/app/reference/auth/) [s](./audit) [t](../reference/auth) [u](/app/reference/nope/) [v](./nope)\n`[x](run_url)`\n```\n[y](code.md)\n```\n";
        let (_, broken) = crate::links_in(&root, "site/content/docs/reference/glossary.md", text);
        assert_eq!(
            broken.into_iter().collect::<Vec<_>>(),
            ["./nope", "/app/reference/nope/"]
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_legacy_citation_and_a_ulid_citation_both_resolve() {
        let root = std::env::temp_dir().join(format!("toolbox-decisions-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let dir = root.join("docs/decisions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("D01ARYZ6S410000000000000000.md"),
            "---\nid: D01ARYZ6S410000000000000000\nlegacy: D84\n---\n",
        )
        .unwrap();
        std::fs::write(dir.join("README.md"), "# Decisions\n").unwrap();
        let known = known_decisions(&root);
        assert!(known.contains("D84"));
        assert!(known.contains("D01ARYZ6S410000000000000000"));
        assert!(
            missing_decisions(
                "see D84 and d01aryz6s410000000000000000",
                &known,
                "docs/plan.md"
            )
            .is_empty()
        );
        assert_eq!(
            missing_decisions("cites D999", &known, "docs/plan.md"),
            vec!["D999".to_string()]
        );
        assert!(missing_decisions("cites D999", &known, "docs/decisions/README.md").is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
