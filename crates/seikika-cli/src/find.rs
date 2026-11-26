//! `seikika tools find`: the tools a repository already uses, and whether each is in the box
//! (docs/design/finding-tools.md, D01M801A9QNVCSBBJATYV721TCS).
//!
//! About the box: it reports, it doesn't decide. The inventory is deterministic: config files,
//! CI and hook commands, package manifests and installer files, matched against a catalog
//! (`tool-catalog.yaml`). It can't tell a tool in use from a stale config, or see a check a
//! repository wrote itself, so it lists the scripts CI runs as well, and the `find-tools` agent
//! skill checks each candidate before anything goes in the box.

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

const CATALOG: &str = include_str!("tool-catalog.yaml");
/// Directories never read: build output, dependencies and caches.
const SKIP: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "vendor",
    "dist",
    "build",
    ".venv",
    "venv",
    "__pycache__",
    ".next",
    ".turbo",
];
/// How deep the walk goes: deep enough for a monorepo's packages, shallow enough to be quick.
const DEPTH: usize = 6;
/// Evidence items shown per tool and kind of evidence.
const SHOWN: usize = 5;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    name: String,
    about: String,
    #[serde(default)]
    configs: Vec<String>,
    pyproject: Option<String>,
    #[serde(default)]
    npm: Vec<String>,
    #[serde(default)]
    pypi: Vec<String>,
    ci: String,
    sarif: Option<String>,
    checked: Option<String>,
    kit: Option<String>,
    fitting: Option<String>,
    connector: Option<String>,
    note: Option<String>,
    #[serde(default)]
    install: Map<String, Value>,
}

#[derive(Debug, Default, Serialize)]
struct Found {
    tool: String,
    about: String,
    /// `in box` (a connector runs it), `pinned` (in `tools:` only) or `candidate`.
    status: String,
    evidence: Vec<String>,
    slot: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sarif: Option<String>,
    /// Where the slot's command was checked; `None`: unchecked, an agent must run it.
    #[serde(skip_serializing_if = "Option::is_none")]
    checked: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

/// The files the walk sees, relative to the root, split into configs and command files.
struct Tree {
    root: PathBuf,
    files: Vec<String>,
    /// Each file is read once, whichever tools look at it.
    texts: RefCell<HashMap<String, Rc<str>>>,
}

fn walk(root: &Path) -> Tree {
    let files = walkdir::WalkDir::new(root)
        .max_depth(DEPTH)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir() && SKIP.contains(&&*e.file_name().to_string_lossy()))
        })
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            let rel = e.path().strip_prefix(root).ok()?;
            Some(rel.to_string_lossy().replace('\\', "/"))
        })
        .collect::<Vec<_>>();
    let mut files = files;
    files.sort();
    Tree {
        root: root.to_owned(),
        files,
        texts: RefCell::default(),
    }
}

/// A config glob as a regex over a relative path: it matches the file's name, or the end of its
/// path when the glob has a `/`.
fn glob(pattern: &str) -> Regex {
    let body = regex::escape(pattern).replace(r"\*", "[^/]*");
    Regex::new(&format!("(?:^|/){body}$")).expect("catalog globs are valid")
}

/// A shell script directly under `scripts/`. Other languages' scripts mention tools in code and
/// strings far more often than they run them; the agent reads those (skill find-tools).
fn is_shell_script(path: &str) -> bool {
    path.strip_prefix("scripts/")
        .is_some_and(|rest| !rest.contains('/') && (rest.ends_with(".sh") || !rest.contains('.')))
}

/// Files whose lines are commands: CI, hooks, task runners and installer configs.
fn is_command_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let yaml = name.ends_with(".yml") || name.ends_with(".yaml");
    (path.starts_with(".github/workflows/") && yaml)
        || path.starts_with(".circleci/")
        || path.starts_with(".husky/")
        || is_shell_script(path)
        || matches!(
            name,
            ".gitlab-ci.yml"
                | ".pre-commit-config.yaml"
                | "lefthook.yml"
                | ".lefthook.yml"
                | "Makefile"
                | "justfile"
                | "Taskfile.yml"
                | "package.json"
                | "mise.toml"
                | ".mise.toml"
                | ".tool-versions"
                | "aqua.yaml"
                | "tox.ini"
                | "noxfile.py"
        )
}

impl Tree {
    fn read(&self, rel: &str) -> Rc<str> {
        let mut texts = self.texts.borrow_mut();
        let text = texts.entry(rel.to_owned()).or_insert_with(|| {
            std::fs::read_to_string(self.root.join(rel))
                .unwrap_or_default()
                .into()
        });
        Rc::clone(text)
    }

    fn named(&self, name: &str) -> impl Iterator<Item = &String> {
        self.files
            .iter()
            .filter(move |f| f.rsplit('/').next() == Some(name))
    }

    fn configs(&self, e: &Entry) -> Vec<String> {
        let globs: Vec<Regex> = e.configs.iter().map(|g| glob(g)).collect();
        let mut out: Vec<String> = self
            .files
            .iter()
            .filter(|f| globs.iter().any(|g| g.is_match(f)))
            .map(|f| format!("config  {f}"))
            .collect();
        if let Some(section) = &e.pyproject {
            let head = format!("[tool.{section}");
            out.extend(
                self.named("pyproject.toml")
                    .filter(|f| {
                        self.read(f)
                            .lines()
                            .any(|l| l.trim_start().starts_with(&head))
                    })
                    .map(|f| format!("config  {f} [tool.{section}]")),
            );
        }
        out
    }

    fn packages(&self, e: &Entry) -> Vec<String> {
        let mut out = Vec::new();
        for f in self.named("package.json") {
            let v: Value = serde_json::from_str(&self.read(f)).unwrap_or(Value::Null);
            for dep in &e.npm {
                let declared = ["dependencies", "devDependencies"]
                    .iter()
                    .any(|k| v[k].get(dep).is_some());
                if declared {
                    out.push(format!("package {f} ({dep})"));
                }
            }
        }
        for dep in &e.pypi {
            let pinned = Regex::new(&format!(
                r#"(?m)(?:^|["'\s]){}(?:[<>=~!\[ ;"']|$)"#,
                regex::escape(dep)
            ))
            .expect("escaped");
            let python = self.files.iter().filter(|f| {
                f.ends_with("pyproject.toml")
                    || f.rsplit('/')
                        .next()
                        .is_some_and(|n| n.starts_with("requirements") && n.ends_with(".txt"))
            });
            out.extend(
                python
                    .filter(|f| pinned.is_match(&self.read(f)))
                    .map(|f| format!("package {f} ({dep})")),
            );
        }
        out
    }

    /// Per catalog entry, the first line in each command file that runs it. One pass over the
    /// files for every entry at once: CI can be tens of megabytes (gh-aw's lock files).
    fn commands(&self, catalog: &[Entry]) -> Vec<Vec<String>> {
        let set =
            regex::RegexSet::new(catalog.iter().map(|e| &e.ci)).expect("catalog regexes are valid");
        let mut out = vec![Vec::new(); catalog.len()];
        for f in self.files.iter().filter(|f| is_command_file(f)) {
            let mut seen = vec![false; catalog.len()];
            let text = self.read(f);
            let lines = text
                .lines()
                .enumerate()
                .filter(|(_, l)| !l.trim_start().starts_with('#'));
            for (n, line) in lines {
                for i in set.matches(line).iter() {
                    if !std::mem::replace(&mut seen[i], true) {
                        out[i].push(format!("command {f}:{}  {}", n + 1, line.trim()));
                    }
                }
            }
        }
        out
    }

    /// Repository scripts that CI and hooks run: checks the repository wrote itself, which no
    /// catalog knows. An agent decides whether each is a tool for the box.
    fn scripts(&self) -> Vec<String> {
        let re = Regex::new(r"(?:^|[\s(])(?:\./)?((?:scripts|tools|bin|ci|hack)/[\w./-]+\.(?:sh|py|js|mjs|ts|rb))\b")
            .expect("valid");
        let mut out: Vec<String> = Vec::new();
        let callers = self
            .files
            .iter()
            .filter(|f| is_command_file(f) && !f.starts_with("scripts/"));
        for f in callers {
            for c in re.captures_iter(&self.read(f)) {
                let script = c[1].to_owned();
                if self.files.contains(&script) && !out.contains(&script) {
                    out.push(script);
                }
            }
        }
        out.sort();
        out
    }
}

/// What the portal already carries: pinned tool names, connector packages and command words.
struct Box_ {
    pinned: Vec<String>,
    packages: Vec<String>,
    words: String,
}

fn in_box(portal: Option<&Path>) -> Result<Option<Box_>, String> {
    let Some(p) = portal else { return Ok(None) };
    let (_, m) = crate::install::manifest(p)?;
    Ok(Some(Box_ {
        pinned: m.tools.keys().cloned().collect(),
        packages: m.connectors.iter().map(|c| c.package.clone()).collect(),
        words: m
            .connectors
            .iter()
            .filter_map(|c| c.config.as_ref())
            .map(Value::to_string)
            .collect(),
    }))
}

fn status(e: &Entry, b: Option<&Box_>) -> &'static str {
    let Some(b) = b else { return "candidate" };
    let runs = b.words.contains(&format!("\"{}\"", e.name))
        || e.connector.as_ref().is_some_and(|c| b.packages.contains(c));
    if runs {
        "in box"
    } else if b.pinned.contains(&e.name) {
        "pinned"
    } else {
        "candidate"
    }
}

/// How a candidate goes in the box.
fn next(e: &Entry) -> Option<String> {
    let package = |k: &str| format!("github:seikika/seikika/packages/{k}@<tag>");
    match (e.kit.as_deref(), &e.sarif) {
        (Some("sarif"), Some(cmd)) => Some(format!(
            "include {} with tool: {}, version: <pin>, install: {}, command: {:?}",
            package("sarif"),
            e.name,
            Value::Object(e.install.clone()),
            cmd.split_whitespace().collect::<Vec<_>>()
        )),
        (Some(kit), _) if kit != "sarif" => Some(format!("include {}", package(kit))),
        _ if e.connector.is_some() => Some(format!(
            "add a {} connector",
            e.connector.as_deref().unwrap_or_default()
        )),
        _ => None,
    }
}

fn slot(e: &Entry) -> String {
    match (&e.sarif, &e.fitting, &e.connector) {
        (Some(_), _, _) => "sarif".into(),
        (_, Some(f), _) => format!("fitting: {f}"),
        (_, _, Some(c)) => format!("connector: {c}"),
        _ => "none known".into(),
    }
}

fn found(e: &Entry, tree: &Tree, commands: Vec<String>, b: Option<&Box_>) -> Option<Found> {
    let mut evidence = Vec::new();
    for mut group in [tree.configs(e), tree.packages(e), commands] {
        let more = group.len().saturating_sub(SHOWN);
        group.truncate(SHOWN);
        evidence.extend(group);
        if more > 0 {
            evidence.push(format!("… {more} more"));
        }
    }
    let status = status(e, b);
    if evidence.is_empty() && status == "candidate" {
        return None;
    }
    Some(Found {
        tool: e.name.clone(),
        about: e.about.clone(),
        status: status.into(),
        evidence,
        slot: slot(e),
        sarif: e.sarif.clone(),
        checked: e.checked.clone(),
        next: (status != "in box").then(|| next(e)).flatten(),
        note: e.note.clone(),
    })
}

/// The portal to compare with: the one given, else `<root>/seikika.yaml`, else
/// `<root>/portal/seikika.yaml`.
fn portal_for(root: &Path, given: Option<&Path>) -> Option<PathBuf> {
    given.map(Path::to_owned).or_else(|| {
        [root.to_owned(), root.join("portal")]
            .into_iter()
            .find(|p| p.join("seikika.yaml").is_file())
    })
}

fn print_text(root: &Path, portal: Option<&Path>, found: &[Found], scripts: &[String]) {
    println!("repository: {}", root.display());
    match portal {
        Some(p) => println!("portal: {}", p.display()),
        None => println!("portal: none (every tool found is a candidate)"),
    }
    for f in found {
        println!("\n{:<9} {} ({})", f.status, f.tool, f.about);
        for ev in &f.evidence {
            println!("  {ev}");
        }
        match (&f.sarif, &f.checked) {
            (Some(cmd), Some(at)) => println!("  slot    sarif: {cmd}  [checked: {at}]"),
            (Some(cmd), None) => {
                println!("  slot    sarif: {cmd}  [unchecked: run it and read the SARIF]")
            }
            _ => println!("  slot    {}", f.slot),
        }
        if let Some(n) = &f.next {
            println!("  next    {n}");
        }
        if let Some(n) = &f.note {
            println!("  note    {n}");
        }
    }
    if !scripts.is_empty() {
        println!(
            "\nscripts CI and hooks run (not in the catalog; an agent decides if each is a tool):"
        );
        for s in scripts {
            println!("  {s}");
        }
    }
    println!(
        "\nThis is evidence, not a verdict: check each candidate before it goes in the box (skill find-tools)."
    );
}

/// `seikika tools find [repo] [--portal p] [--format text|json]`.
pub fn find(root: &Path, portal: Option<&Path>, json_out: bool) -> ExitCode {
    let catalog: Vec<Entry> = match serde_norway::from_str(CATALOG) {
        Ok(c) => c,
        Err(e) => return crate::install::fail(format!("tool catalog: {e}")),
    };
    let portal = portal_for(root, portal);
    let b = match in_box(portal.as_deref()) {
        Ok(b) => b,
        Err(e) => return crate::install::fail(e),
    };
    let tree = walk(root);
    let found: Vec<Found> = catalog
        .iter()
        .zip(tree.commands(&catalog))
        .filter_map(|(e, commands)| found(e, &tree, commands, b.as_ref()))
        .collect();
    let scripts = tree.scripts();
    if json_out {
        let out = json!({"repository": root, "portal": portal, "tools": found, "scripts": scripts});
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else {
        print_text(root, portal.as_deref(), &found, &scripts);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_parses_and_its_patterns_compile() {
        let c: Vec<Entry> = serde_norway::from_str(CATALOG).unwrap();
        assert!(c.len() >= 20);
        for e in &c {
            Regex::new(&e.ci).unwrap();
            e.configs.iter().for_each(|g| drop(glob(g)));
            assert!(
                e.sarif.is_none() || e.kit.is_some(),
                "{}: a SARIF tool needs a kit",
                e.name
            );
        }
    }

    #[test]
    fn globs_match_names_and_path_ends() {
        assert!(glob("eslint.config.*").is_match("web/eslint.config.mjs"));
        assert!(!glob("eslint.config.*").is_match("web/eslint.config/x.js"));
        assert!(glob(".github/actionlint.yaml").is_match(".github/actionlint.yaml"));
        assert!(!glob(".github/actionlint.yaml").is_match("actionlint.yaml"));
    }

    #[test]
    fn evidence_status_and_scripts() {
        let dir = std::env::temp_dir().join(format!("seikika-find-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        std::fs::create_dir_all(dir.join("scripts")).unwrap();
        std::fs::create_dir_all(dir.join("node_modules/x")).unwrap();
        std::fs::write(
            dir.join("pyproject.toml"),
            "[tool.ruff]\nline-length = 100\n",
        )
        .unwrap();
        std::fs::write(dir.join("node_modules/x/.eslintrc.json"), "{}").unwrap();
        std::fs::write(dir.join("scripts/check-names.py"), "").unwrap();
        std::fs::write(
            dir.join(".github/workflows/ci.yml"),
            "# semgrep runs elsewhere\njobs:\n  l:\n    steps:\n      - run: ruff check .\n      - run: python3 scripts/check-names.py\n",
        )
        .unwrap();
        let tree = walk(&dir);
        let c: Vec<Entry> = serde_norway::from_str(CATALOG).unwrap();
        let commands = tree.commands(&c);
        let at = |name: &str| c.iter().position(|e| e.name == name).unwrap();
        let ruff = &c[at("ruff")];
        let f = found(ruff, &tree, commands[at("ruff")].clone(), None).unwrap();
        assert_eq!(f.status, "candidate");
        assert_eq!(f.evidence[0], "config  pyproject.toml [tool.ruff]");
        assert_eq!(
            f.evidence[1],
            "command .github/workflows/ci.yml:5  - run: ruff check ."
        );
        assert!(f.next.unwrap().contains("packages/sarif"));
        let eslint = c.iter().find(|e| e.name == "eslint").unwrap();
        assert!(
            found(eslint, &tree, commands[at("eslint")].clone(), None).is_none(),
            "node_modules is skipped"
        );
        let semgrep = c.iter().find(|e| e.name == "semgrep").unwrap();
        assert!(
            found(semgrep, &tree, commands[at("semgrep")].clone(), None).is_none(),
            "a comment isn't a command"
        );
        assert_eq!(tree.scripts(), ["scripts/check-names.py"]);
        let b = Box_ {
            pinned: vec!["ruff".into()],
            packages: vec![],
            words: String::new(),
        };
        assert_eq!(status(ruff, Some(&b)), "pinned");
        let b = Box_ {
            words: r#"{"command":["ruff","check"]}"#.into(),
            ..b
        };
        assert_eq!(status(ruff, Some(&b)), "in box");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
