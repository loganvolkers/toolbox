//! One `js-workspace` record for the checkout root: pnpm/npm workspace file, `packageManager`
//! pin, root lockfile, linker policy, mise pins, and whether CI installs from a frozen lockfile
//! (packages/pnpm-expo, D01M7Y6X38AERPBT0YH1AG5TBRD).

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

/// The `js-workspace` record for the repository root.
pub fn record(root: &Path, paths: &[String], has_expo_apps: bool) -> Value {
    let pkg = read_json(root, "package.json");
    let package_manager = pkg["packageManager"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_owned();
    let engines_node = pkg["engines"]["node"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_owned();
    let (wildcard_ranges, git_ranges) = root_ranges(&pkg);
    let (workspace_file, has_workspace_file) = workspace_file(root, &pkg);
    let (linker, linker_declared, linker_hoisted) = linker_policy(root);
    let (mise_node, mise_pnpm, mise_concrete) = mise_pins(root);
    let (ci_node_versions, ci_node_version_sources) = ci_node_versions(root, paths);
    let ci_frozen = ci_frozen_install(root, paths);
    let package_manager_pinned = package_manager_pinned(&package_manager);
    let ci_node_ok = ci_node_satisfies(&ci_node_versions, &engines_node);
    let mise_matches = mise_matches_package_manager(&mise_pnpm, &package_manager);
    let toolchain_aligned = package_manager_pinned
        && ci_node_ok
        && mise_concrete
        && mise_matches
        && !ci_node_versions.is_empty();
    json!({
        "path": ".",
        "hasWorkspaceFile": has_workspace_file,
        "workspaceFile": workspace_file,
        "packageManager": package_manager,
        "packageManagerPinned": package_manager_pinned,
        "rootLockfile": root_lockfile(root),
        "ciFrozenInstall": ci_frozen,
        "linker": linker,
        "linkerDeclared": linker_declared,
        "linkerHoisted": linker_hoisted,
        "miseNode": mise_node,
        "misePnpm": mise_pnpm,
        "miseConcrete": mise_concrete,
        "miseMatchesPackageManager": mise_matches,
        "ciNodeVersions": ci_node_versions,
        "ciNodeVersionSources": ci_node_version_sources,
        "enginesNode": engines_node,
        "ciNodeSatisfiesEngines": ci_node_ok,
        "toolchainAligned": toolchain_aligned,
        "wildcardRanges": wildcard_ranges,
        "gitRanges": git_ranges,
        "hasExpoApps": has_expo_apps,
    })
}

fn read_json(root: &Path, rel: &str) -> Value {
    std::fs::read_to_string(root.join(rel))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null)
}

fn workspace_file(root: &Path, pkg: &Value) -> (String, bool) {
    if root.join("pnpm-workspace.yaml").is_file() {
        return ("pnpm-workspace.yaml".into(), true);
    }
    if pkg["workspaces"].is_array() || pkg["workspaces"].is_object() {
        return ("package.json#workspaces".into(), true);
    }
    (String::new(), false)
}

fn root_lockfile(root: &Path) -> String {
    for name in [
        "pnpm-lock.yaml",
        "package-lock.json",
        "npm-shrinkwrap.json",
        "yarn.lock",
    ] {
        if root.join(name).is_file() {
            return name.into();
        }
    }
    String::new()
}

/// Linker from `.npmrc` (`node-linker=…`) or `pnpm-workspace.yaml` (`nodeLinker:`).
fn linker_policy(root: &Path) -> (String, bool, bool) {
    if let Ok(text) = std::fs::read_to_string(root.join(".npmrc")) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or(line).trim();
            if let Some(value) = line.strip_prefix("node-linker=") {
                let value = value.trim().to_owned();
                let hoisted = value == "hoisted";
                return (value, true, hoisted);
            }
        }
    }
    if let Ok(text) = std::fs::read_to_string(root.join("pnpm-workspace.yaml")) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or(line).trim();
            if let Some(rest) = line.strip_prefix("nodeLinker:") {
                let value = rest
                    .trim()
                    .trim_matches(|c: char| matches!(c, '"' | '\''))
                    .to_owned();
                if !value.is_empty() {
                    let hoisted = value == "hoisted";
                    return (value, true, hoisted);
                }
            }
        }
    }
    (String::new(), false, false)
}

fn mise_pins(root: &Path) -> (String, String, bool) {
    let text = std::fs::read_to_string(root.join("mise.toml")).unwrap_or_default();
    if text.trim().is_empty() {
        // No mise file: treat as concrete (nothing to drift); alignment then depends on CI alone.
        return (String::new(), String::new(), true);
    }
    let node = tool_pin(&text, "node");
    let pnpm = tool_pin(&text, "pnpm");
    let concrete = is_concrete(&node) && is_concrete(&pnpm);
    (node, pnpm, concrete)
}

fn tool_pin(text: &str, name: &str) -> String {
    let re = Regex::new(&format!(r#"(?m)^\s*{name}\s*=\s*["']([^"']+)["']"#)).unwrap();
    re.captures(text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_owned()))
        .unwrap_or_default()
}

fn is_concrete(pin: &str) -> bool {
    let p = pin.trim();
    if p.is_empty() {
        return false;
    }
    !matches!(
        p.to_ascii_lowercase().as_str(),
        "latest" | "lts" | "*" | "stable" | "current"
    )
}

/// `pnpm@x.y.z`, optionally with corepack's integrity hash (`pnpm@x.y.z+sha512.<hex>`), which pins
/// more tightly than the version alone.
fn package_manager_pinned(pm: &str) -> bool {
    Regex::new(r"^pnpm@\d+\.\d+\.\d+(\+sha\d+\.[0-9A-Za-z+/=]+)?$")
        .unwrap()
        .is_match(pm.trim())
}

fn mise_matches_package_manager(mise_pnpm: &str, package_manager: &str) -> bool {
    let Some(want) = package_manager.strip_prefix("pnpm@") else {
        return false;
    };
    let want = want.split('+').next().unwrap_or(want); // drop corepack's `+sha512.…`
    if mise_pnpm.trim().is_empty() {
        // No pnpm pin in mise: nothing to drift from (a mise file without one isn't concrete,
        // which fails alignment on its own).
        return true;
    }
    let want_parts: Vec<&str> = want.split('.').collect();
    if want_parts.len() < 2 {
        return false;
    }
    let got = mise_pnpm.trim().trim_start_matches('v');
    if got.is_empty() {
        return false;
    }
    // Exact match, or major.minor match (e.g. mise `10.28.0` or `10.28`).
    if got == want {
        return true;
    }
    let got_parts: Vec<&str> = got.split('.').collect();
    got_parts.len() >= 2 && got_parts[0] == want_parts[0] && got_parts[1] == want_parts[1]
}

/// Node versions CI sets up: `node-version: X`, or `node-version-file: F` (read from the checkout,
/// e.g. `.nvmrc`). Skips GitHub Actions expressions (`${{…}}`) and other non-semver tokens so they
/// do not poison `ciNodeSatisfiesEngines`. Returns `(versions, sources)` where each source names the
/// workflow path that contributed the version (for scorecard evidence).
fn ci_node_versions(root: &Path, paths: &[String]) -> (Vec<String>, Vec<String>) {
    let re = Regex::new(r##"node-version:\s*['"]?([^\s'"#]+)"##).unwrap();
    let file = Regex::new(r##"node-version-file:\s*['"]?([^\s'"#]+)"##).unwrap();
    let mut found = Vec::new();
    let mut sources = Vec::new();
    for rel in workflows(paths) {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
        for c in re.captures_iter(&text) {
            if let Some(m) = c.get(1) {
                let v = m.as_str();
                if is_concrete_ci_node_version(v) {
                    found.push(v.to_owned());
                    sources.push(format!("{rel} node-version={v}"));
                }
            }
        }
        for c in file.captures_iter(&text) {
            let file_rel = &c[1];
            let pinned = std::fs::read_to_string(root.join(file_rel)).unwrap_or_default();
            let v = pinned.lines().next().unwrap_or("").trim();
            if is_concrete_ci_node_version(v) {
                found.push(v.to_owned());
                sources.push(format!("{rel} node-version-file={file_rel} ({v})"));
            }
        }
    }
    found.sort();
    found.dedup();
    sources.sort();
    sources.dedup();
    (found, sources)
}

/// True when a captured `node-version` token is a concrete semver major (optionally `v`-prefixed),
/// e.g. `22`, `22.1.0`, `v24.4.1`. Rejects Actions expressions (`${{…}}`), `latest` / `lts/*`, and
/// anything whose major component is not an integer.
fn is_concrete_ci_node_version(token: &str) -> bool {
    let t = token.trim();
    if t.is_empty() {
        return false;
    }
    if t.contains("{{") || t.starts_with('$') {
        return false;
    }
    let t = t.trim_start_matches('v');
    t.split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .is_some()
}

/// True when CI uses a lockfile-faithful pnpm/npm install and never `--no-frozen-lockfile`. In CI,
/// `pnpm install` fails when the lockfile needs an update (pnpm's `install` docs), so a plain
/// `pnpm install` in a workflow is frozen; `npm install` is not, and only `npm ci` counts.
fn ci_frozen_install(root: &Path, paths: &[String]) -> bool {
    let mut saw_good = false;
    let mut saw_bad = false;
    for rel in workflows(paths) {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
        for line in text.lines() {
            if compile(r"--no-frozen-lockfile\b").is_match(line) {
                saw_bad = true;
            }
            if compile(r"\bpnpm\s+ci\b").is_match(line)
                || compile(r"\bpnpm\s+i(nstall)?\b").is_match(line)
                || compile(r"\bnpm ci\b").is_match(line)
            {
                saw_good = true;
            }
        }
    }
    saw_good && !saw_bad
}

fn workflows(paths: &[String]) -> Vec<&String> {
    paths
        .iter()
        .filter(|p| {
            p.starts_with(".github/workflows/") && (p.ends_with(".yml") || p.ends_with(".yaml"))
        })
        .collect()
}

/// Coarse check: every CI `node-version` major is ≥ the engines floor major when engines is `>=N`.
fn ci_node_satisfies(versions: &[String], engines: &str) -> bool {
    if versions.is_empty() {
        return false;
    }
    let floor = engines_floor(engines);
    versions.iter().all(|v| {
        let major = v
            .trim()
            .trim_start_matches('v')
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok());
        match (major, floor) {
            (Some(m), Some(f)) => m >= f,
            (Some(_), None) => true,
            _ => false,
        }
    })
}

fn engines_floor(engines: &str) -> Option<u32> {
    let re = Regex::new(r">=\s*(\d+)").unwrap();
    re.captures(engines)
        .and_then(|c| c.get(1)?.as_str().parse().ok())
        .or_else(|| {
            engines
                .trim()
                .trim_start_matches('v')
                .split('.')
                .next()
                .and_then(|s| s.parse().ok())
        })
}

fn root_ranges(pkg: &Value) -> (Vec<String>, Vec<String>) {
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
                git.push(shown);
            }
        }
    }
    wild.sort();
    git.sort();
    (wild, git)
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

fn compile(pat: &str) -> Regex {
    Regex::new(pat).unwrap_or_else(|_| panic!("static regex {pat}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root = std::env::temp_dir().join(format!("seikika-jsws-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_pinned_workspace_with_frozen_ci_and_concrete_mise_aligns() {
        let tmp = scratch(
            "good",
            &[
                (
                    "package.json",
                    r#"{"name":"app","private":true,"packageManager":"pnpm@10.28.0","engines":{"node":">=22"},"devDependencies":{"eslint":"9.0.0"}}"#,
                ),
                ("pnpm-workspace.yaml", "packages:\n  - apps/*\n"),
                ("pnpm-lock.yaml", "lockfileVersion: '9.0'\n"),
                (".npmrc", "node-linker=hoisted\n"),
                ("mise.toml", "[tools]\nnode = \"22\"\npnpm = \"10.28.0\"\n"),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  check:\n    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: 22\n      - run: pnpm install --frozen-lockfile\n",
                ),
            ],
        );
        let paths = vec![
            "package.json".into(),
            "pnpm-workspace.yaml".into(),
            "pnpm-lock.yaml".into(),
            ".npmrc".into(),
            "mise.toml".into(),
            ".github/workflows/ci.yml".into(),
        ];
        let rec = record(&tmp.0, &paths, true);
        assert_eq!(rec["hasWorkspaceFile"], true);
        assert_eq!(rec["packageManagerPinned"], true);
        assert_eq!(rec["rootLockfile"], "pnpm-lock.yaml");
        assert_eq!(rec["ciFrozenInstall"], true);
        assert_eq!(rec["linkerDeclared"], true);
        assert_eq!(rec["linkerHoisted"], true);
        assert_eq!(rec["miseConcrete"], true);
        assert_eq!(rec["toolchainAligned"], true);
        assert_eq!(rec["wildcardRanges"], json!([]));
    }

    /// Out-of-box findings on real repositories (docs/design/repo-value.md R2): a corepack hash pin,
    /// a plain `pnpm install` in CI, Node from `.nvmrc`, and no mise file are all fine.
    #[test]
    fn hash_pins_plain_ci_installs_and_nvmrc_align_without_mise() {
        let tmp = scratch(
            "nvmrc",
            &[
                (
                    "package.json",
                    r#"{"packageManager":"pnpm@10.33.0+sha512.1056ab","engines":{"node":">=24 <25"}}"#,
                ),
                ("pnpm-workspace.yaml", "packages: [apps/*]\n"),
                ("pnpm-lock.yaml", "lockfileVersion: '9.0'\n"),
                (".nvmrc", "24.4.1\n"),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  x:\n    steps:\n      - with:\n          node-version-file: .nvmrc\n      - run: |\n          pnpm install\n",
                ),
            ],
        );
        let paths = vec![
            "package.json".into(),
            "pnpm-workspace.yaml".into(),
            "pnpm-lock.yaml".into(),
            ".github/workflows/ci.yml".into(),
        ];
        let rec = record(&tmp.0, &paths, false);
        assert_eq!(rec["packageManagerPinned"], true);
        assert_eq!(rec["ciFrozenInstall"], true);
        assert_eq!(rec["ciNodeVersions"], json!(["24.4.1"]));
        assert_eq!(
            rec["ciNodeVersionSources"],
            json!([".github/workflows/ci.yml node-version-file=.nvmrc (24.4.1)"])
        );
        assert_eq!(rec["toolchainAligned"], true);
        // `--no-frozen-lockfile` still fails, and a bare range is still not a pin.
        assert!(!package_manager_pinned("pnpm@10"));
        assert!(!package_manager_pinned("pnpm@^10.33.0"));
    }

    /// Matrix / Actions expressions must not enter `ciNodeVersions` (they used to capture as
    /// `${{` and fail `ciNodeSatisfiesEngines`). A sibling literal `22` still counts.
    #[test]
    fn matrix_node_expressions_are_skipped_but_literal_majors_count() {
        let tmp = scratch(
            "matrix-node",
            &[
                (
                    "package.json",
                    r#"{"packageManager":"pnpm@10.28.0","engines":{"node":">=22"}}"#,
                ),
                ("pnpm-workspace.yaml", "packages: [apps/*]\n"),
                ("pnpm-lock.yaml", "lockfileVersion: '9.0'\n"),
                ("mise.toml", "[tools]\nnode = \"22\"\npnpm = \"10.28.0\"\n"),
                (
                    ".github/workflows/nightly.yml",
                    "jobs:\n  matrix:\n    strategy:\n      matrix:\n        node: [22, 24]\n    steps:\n      - with:\n          node-version: ${{ matrix.node }}\n      - run: pnpm install --frozen-lockfile\n  fixed:\n    steps:\n      - with:\n          node-version: 22\n      - run: pnpm install --frozen-lockfile\n",
                ),
            ],
        );
        let paths = vec![
            "package.json".into(),
            "pnpm-workspace.yaml".into(),
            "pnpm-lock.yaml".into(),
            "mise.toml".into(),
            ".github/workflows/nightly.yml".into(),
        ];
        let rec = record(&tmp.0, &paths, false);
        assert_eq!(rec["ciNodeVersions"], json!(["22"]));
        assert_eq!(rec["ciNodeSatisfiesEngines"], true);
        assert_eq!(
            rec["ciNodeVersionSources"],
            json!([".github/workflows/nightly.yml node-version=22"])
        );
        assert_eq!(rec["toolchainAligned"], true);
        assert!(!is_concrete_ci_node_version("${{"));
        assert!(!is_concrete_ci_node_version("${{ matrix.node }}"));
        assert!(!is_concrete_ci_node_version("latest"));
        assert!(!is_concrete_ci_node_version("lts/*"));
        assert!(is_concrete_ci_node_version("22"));
        assert!(is_concrete_ci_node_version("v24.4.1"));
    }

    #[test]
    fn latest_mise_pins_fail_alignment() {
        let tmp = scratch(
            "mise-latest",
            &[
                (
                    "package.json",
                    r#"{"packageManager":"pnpm@10.28.0","engines":{"node":">=22"}}"#,
                ),
                ("pnpm-workspace.yaml", "packages: [apps/*]\n"),
                ("pnpm-lock.yaml", "lockfileVersion: '9.0'\n"),
                (
                    "mise.toml",
                    "[tools]\nnode = \"latest\"\npnpm = \"latest\"\n",
                ),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  x:\n    steps:\n      - with:\n          node-version: 22\n      - run: pnpm install --frozen-lockfile\n",
                ),
            ],
        );
        let paths = vec![
            "package.json".into(),
            "pnpm-workspace.yaml".into(),
            "pnpm-lock.yaml".into(),
            "mise.toml".into(),
            ".github/workflows/ci.yml".into(),
        ];
        let rec = record(&tmp.0, &paths, false);
        assert_eq!(rec["miseConcrete"], false);
        assert_eq!(rec["toolchainAligned"], false);
        assert_eq!(rec["miseNode"], "latest");
    }
}
