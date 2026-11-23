//! npm manifests (`package.json`), read statically (docs/design/package-standards.md).
//!
//! One record per manifest. Facts only: the name, whether npm will refuse to publish it, the
//! dependency ranges, a lockfile beside the manifest or at a workspace root, the scripts a check
//! can run, and whether CI runs an install that may rewrite the lockfile (`npm install`, or
//! `pnpm install` without `--frozen-lockfile`). Thresholds live in the manifest.

use regex::Regex;
use serde_json::{Map, Value, json};
use std::path::Path;

pub(crate) fn analyse(root: &Path, rel: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let pkg: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let (requirements, wildcard_ranges, git_ranges) = ranges(&pkg);
    let dir = package_dir(rel);
    json!({
        "path": rel,
        "name": pkg["name"].as_str().unwrap_or("").trim(),
        "version": pkg["version"].as_str().unwrap_or("").trim(),
        "private": pkg["private"].as_bool().unwrap_or(false),
        "descriptionChars": pkg["description"].as_str().unwrap_or("").trim().chars().count() as i64,
        "engines": pkg["engines"]["node"].as_str().map(|s| !s.trim().is_empty()).unwrap_or(false),
        "requirements": requirements,
        "wildcardRanges": wildcard_ranges,
        "gitRanges": git_ranges,
        "lockfile": lockfile(root, Path::new(&dir)),
        "checkScripts": check_scripts(pkg["scripts"].as_object()),
        "ciUsesInstall": ci_uses_install(root, &dir),
        "secretHits": secrets(rel, &text),
    })
}

fn ranges(pkg: &Value) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut all = Vec::new();
    let mut wild = Vec::new();
    let mut git = Vec::new();
    for section in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        let Some(obj) = pkg[section].as_object() else {
            continue;
        };
        for (name, spec) in obj {
            let spec = spec.as_str().unwrap_or("");
            let shown = format!("{name}@{spec}");
            if is_wildcard(spec) {
                wild.push(shown.clone());
            }
            if is_remote(spec) {
                git.push(shown.clone());
            }
            all.push(shown);
        }
    }
    all.sort();
    wild.sort();
    git.sort();
    (all, wild, git)
}

fn is_local(spec: &str) -> bool {
    let s = spec.trim();
    ["workspace:", "file:", "link:", "portal:"]
        .iter()
        .any(|p| s.starts_with(p))
}

fn is_wildcard(spec: &str) -> bool {
    if is_local(spec) {
        return false;
    }
    let s = spec.trim();
    if s.is_empty() || s == "*" || s == "x" || s == "X" || s.eq_ignore_ascii_case("latest") {
        return true;
    }
    if s.starts_with('>') {
        return true;
    }
    s.split([' ', '|']).any(|part| {
        let p = part.trim().trim_start_matches(['^', '~', '=']);
        p == "*"
            || p == "x"
            || p == "X"
            || p.ends_with(".x")
            || p.ends_with(".X")
            || p.ends_with(".*")
    })
}

fn is_remote(spec: &str) -> bool {
    if is_local(spec) {
        return false;
    }
    let s = spec.trim();
    [
        "git+",
        "git:",
        "github:",
        "gitlab:",
        "bitbucket:",
        "http://",
        "https://",
    ]
    .iter()
    .any(|p| s.starts_with(p))
        || s.contains("git+")
}

/// Lockfile beside the manifest, or at an ancestor up to the checkout root (pnpm/yarn workspaces).
fn lockfile(root: &Path, dir: &Path) -> String {
    let names = [
        "package-lock.json",
        "npm-shrinkwrap.json",
        "pnpm-lock.yaml",
        "yarn.lock",
    ];
    let abs = root.join(dir);
    let mut cur = abs.as_path();
    loop {
        for name in names {
            if cur.join(name).is_file() {
                return name.to_owned();
            }
        }
        if cur == root {
            break;
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => break,
        }
    }
    String::new()
}

fn check_scripts(scripts: Option<&Map<String, Value>>) -> i64 {
    let Some(scripts) = scripts else {
        return 0;
    };
    ["test", "build", "check", "typecheck"]
        .iter()
        .filter(|k| scripts.contains_key(**k))
        .count() as i64
}

fn package_dir(rel: &str) -> String {
    Path::new(rel)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ".".into())
}

fn ci_uses_install(root: &Path, dir: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(root.join(".github/workflows")) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        is_yaml(&path) && blocks_install(&std::fs::read_to_string(&path).unwrap_or_default(), dir)
    })
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yml" | "yaml")
    )
}

/// Lines that belong to a `working-directory:` block for `dir`, until the next directory or job.
fn blocks_install(text: &str, dir: &str) -> bool {
    let mut capture = false;
    let mut install = false;
    for line in text.lines() {
        if let Some(value) = working_directory(line) {
            capture = value == dir;
            continue;
        }
        if capture && job_boundary(line) {
            capture = false;
            continue;
        }
        if capture && unlocked_install(line) {
            install = true;
        }
    }
    install
}

/// An install that may rewrite the lockfile (`npm install`, or pnpm without `--frozen-lockfile`).
fn unlocked_install(line: &str) -> bool {
    if compile(r"\bnpm install\b").is_match(line) {
        return true;
    }
    if compile(r"\bpnpm\s+i(nstall)?\b").is_match(line) {
        return !compile(r"(^|[\s])--frozen-lockfile\b").is_match(line);
    }
    false
}

fn working_directory(line: &str) -> Option<String> {
    let rest = line.split_once("working-directory:")?.1;
    let token = rest
        .trim()
        .trim_matches(|c: char| matches!(c, '"' | '\'' | '{' | '}'))
        .split(|c: char| c == '}' || c.is_whitespace())
        .next()
        .unwrap_or("");
    let token = token.trim_matches(|c: char| matches!(c, '"' | '\''));
    (!token.is_empty()).then(|| token.to_owned())
}

fn job_boundary(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("  ") else {
        return false;
    };
    !rest.starts_with(' ') && rest.contains(':')
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
        let root = std::env::temp_dir().join(format!("toolbox-npm-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_private_package_with_a_lockfile_and_npm_ci_is_bounded() {
        let manifest = r#"{
            "name": "focused",
            "private": true,
            "description": "The UI",
            "engines": {"node": "22"},
            "scripts": {"build": "vite build", "test": "vitest"},
            "dependencies": {"react": "19.3.1", "local": "file:../local"}
        }"#;
        let tmp = scratch(
            "good",
            &[
                ("ui/package.json", manifest),
                ("ui/package-lock.json", "{}\n"),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  ui:\n    defaults: {run: {working-directory: ui}}\n    steps:\n      - run: npm ci\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "ui/package.json");
        assert_eq!(rec["name"], "focused");
        assert_eq!(rec["private"], true);
        assert_eq!(rec["lockfile"], "package-lock.json");
        assert_eq!(rec["wildcardRanges"], json!([]));
        assert_eq!(rec["gitRanges"], json!([]));
        assert_eq!(rec["checkScripts"], 2);
        assert_eq!(rec["engines"], true);
        assert_eq!(rec["ciUsesInstall"], false);
        assert_eq!(rec["secretHits"], json!([]));
    }

    #[test]
    fn a_star_and_a_git_url_and_npm_install_are_named() {
        let manifest = "{\n  \"name\": \"\",\n  \"dependencies\": {\"leftpad\": \"*\", \"other\": \"git+https://example.invalid/leftpad\"},\n  \"note\": \"AKIAIOSFODNN7EXAMPLE\"\n}\n";
        let tmp = scratch(
            "bad",
            &[
                ("app/package.json", manifest),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  app:\n    defaults: {run: {working-directory: app}}\n    steps:\n      - run: npm install --no-audit\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "app/package.json");
        assert_eq!(rec["name"], "");
        assert_eq!(rec["private"], false);
        assert_eq!(rec["wildcardRanges"], json!(["leftpad@*"]));
        assert_eq!(
            rec["gitRanges"],
            json!(["other@git+https://example.invalid/leftpad"])
        );
        assert_eq!(rec["lockfile"], "");
        assert_eq!(rec["ciUsesInstall"], true);
        assert_eq!(
            rec["secretHits"],
            json!(["app/package.json:4 aws-access-key"])
        );
    }

    #[test]
    fn caret_ranges_are_bounded_and_workspace_stars_are_local() {
        assert!(!is_wildcard("^19.0.0"));
        assert!(!is_wildcard("~1.2.3"));
        assert!(!is_wildcard("workspace:*"));
        assert!(is_wildcard("*"));
        assert!(is_wildcard(">=1"));
        assert!(is_wildcard("1.x"));
        assert!(is_wildcard("latest"));
        assert!(!is_remote("file:../ui"));
        assert!(is_remote("github:left/pad"));
        assert!(is_remote("https://example.invalid/pkg.tgz"));
    }

    #[test]
    fn a_nested_package_uses_the_workspace_root_pnpm_lockfile() {
        let tmp = scratch(
            "pnpm-ws",
            &[
                (
                    "package.json",
                    r#"{"name":"root","private":true,"packageManager":"pnpm@10.0.0"}"#,
                ),
                ("pnpm-lock.yaml", "lockfileVersion: '9.0'\n"),
                (
                    "apps/web/package.json",
                    r#"{"name":"web","private":true,"scripts":{"build":"vite build"}}"#,
                ),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  web:\n    defaults: {run: {working-directory: apps/web}}\n    steps:\n      - run: pnpm install --frozen-lockfile\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "apps/web/package.json");
        assert_eq!(rec["lockfile"], "pnpm-lock.yaml");
        assert_eq!(rec["ciUsesInstall"], false);
    }

    #[test]
    fn pnpm_install_without_frozen_lockfile_is_unlocked() {
        let tmp = scratch(
            "pnpm-unlocked",
            &[
                ("apps/web/package.json", r#"{"name":"web","private":true}"#),
                ("pnpm-lock.yaml", "lockfileVersion: '9.0'\n"),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  web:\n    defaults: {run: {working-directory: apps/web}}\n    steps:\n      - run: pnpm i\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "apps/web/package.json");
        assert_eq!(rec["lockfile"], "pnpm-lock.yaml");
        assert_eq!(rec["ciUsesInstall"], true);
        assert!(unlocked_install("      - run: pnpm install"));
        assert!(unlocked_install("      - run: pnpm i"));
        assert!(!unlocked_install(
            "      - run: pnpm install --frozen-lockfile"
        ));
        assert!(!unlocked_install("      - run: pnpm i --frozen-lockfile"));
        assert!(!unlocked_install("      - run: npm ci"));
    }

    #[test]
    fn the_packages_in_this_repo_are_private_locked_and_installed_with_npm_ci() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let ui = analyse(&root, "ui/package.json");
        assert_eq!(ui["name"], "toolbox-minimal-ui");
        assert_eq!(ui["private"], true);
        assert_eq!(ui["lockfile"], "package-lock.json");
        assert!(ui["checkScripts"].as_i64().unwrap() >= 1);
        assert_eq!(ui["descriptionChars"], 0);
        assert_eq!(ui["engines"], false);
        assert_eq!(ui["wildcardRanges"], json!([]));
        assert_eq!(ui["gitRanges"], json!([]));
        // npm ci installs the lockfile as written (OpenSSF Pinned-Dependencies + maintainer-package).
        assert_eq!(ui["ciUsesInstall"], false);
        let builder = analyse(&root, "experiments/typescript-builder/package.json");
        assert_eq!(builder["name"], "toolbox-manifest-builder-prototype");
        assert_eq!(builder["private"], true);
        assert_eq!(builder["lockfile"], "package-lock.json");
        assert!(builder["checkScripts"].as_i64().unwrap() >= 1);
        assert_eq!(builder["ciUsesInstall"], false);
        assert_eq!(builder["descriptionChars"], 0);
    }
}
