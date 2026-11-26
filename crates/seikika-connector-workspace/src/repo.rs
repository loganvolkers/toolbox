//! The `repo` record: one per checkout, with the files that repository standards look at (README,
//! LICENSE, CODEOWNERS and the `.github` directory), CODEOWNERS parsed into rules, and every owner
//! it names (sorted, unique). It is the entity
//! that repo-level scorecards (e.g. examples/repo-portal-starter) score.

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

/// GitHub's CODEOWNERS locations, in the order GitHub looks.
const CODEOWNERS: &[&str] = &[".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS"];

/// `owner/name` from the `origin` remote, or the directory name.
fn name(root: &Path) -> String {
    let remote = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(root)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default();
    let tail = remote
        .trim_end_matches(".git")
        .rsplit(['/', ':'])
        .take(2)
        .collect::<Vec<_>>();
    if tail.len() == 2 && !tail[0].is_empty() && !tail[1].is_empty() {
        return format!("{}/{}", tail[1], tail[0]);
    }
    std::fs::canonicalize(root)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "repo".into())
}

/// CODEOWNERS rules: `{pattern, owners, line}`; comments and blank lines skipped.
pub fn parse_codeowners(text: &str) -> Vec<Value> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.split(" #").next().unwrap_or(line).trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let mut parts = line.split_whitespace();
            let pattern = parts.next()?;
            let owners: Vec<&str> = parts.collect();
            Some(json!({"pattern": pattern, "owners": owners, "line": i + 1}))
        })
        .collect()
}

/// The `repo` record for the checkout at `root`; `paths` are its files (relative, sorted).
pub fn record(root: &Path, paths: &[String]) -> (String, Value) {
    let top: Vec<&String> = paths
        .iter()
        .filter(|p| !p.contains('/') || p.starts_with(".github/"))
        .collect();
    let first = |pred: &dyn Fn(&str) -> bool| {
        top.iter()
            .find(|p| pred(&p.to_lowercase()))
            .map(|p| p.to_string())
    };
    let readme = first(&|p| p.starts_with("readme"));
    let license = first(&|p| p.starts_with("license") || p.starts_with("copying"));
    let codeowners = CODEOWNERS
        .iter()
        .find(|c| paths.iter().any(|p| p == *c))
        .map(|c| {
            let text = std::fs::read_to_string(root.join(c)).unwrap_or_default();
            json!({"path": c, "rules": parse_codeowners(&text)})
        });
    let readme_words = readme
        .as_ref()
        .and_then(|r| std::fs::read_to_string(root.join(r)).ok())
        .map_or(0, |t| t.split_whitespace().count());
    let mut owners: Vec<String> = codeowners
        .iter()
        .flat_map(|c| c["rules"].as_array().cloned().unwrap_or_default())
        .flat_map(|r| r["owners"].as_array().cloned().unwrap_or_default())
        .filter_map(|o| o.as_str().map(str::to_owned))
        .collect();
    owners.sort();
    owners.dedup();
    let owners_ok = owners_ok(&owners);
    let name = name(root);
    let supply = supply_chain(root, paths, codeowners.as_ref());
    (
        name.clone(),
        json!({
            "owners": owners,
            "ownersOk": owners_ok,
            "name": name,
            "files": top,
            "readme": readme,
            "readmeWords": readme_words,
            "license": license,
            "codeowners": codeowners,
            "dependabotGaps": supply.dependabot_gaps,
            "codeScanning": supply.code_scanning,
            "codeScanningPinned": supply.code_scanning_pinned,
            "securityPolicy": supply.security_policy,
            "workflowOwners": supply.workflow_owners,
        }),
    )
}

/// True when every owner is `@org/team`, or every owner is a personal `@user` (D01M7Y57FVX0XYNGMX0A0XJ4GK9).
fn owners_ok(owners: &[String]) -> bool {
    if owners.is_empty() || !owners.iter().all(|o| o.starts_with('@')) {
        return false;
    }
    let all_teams = owners.iter().all(|o| o.contains('/'));
    let all_personal = owners.iter().all(|o| !o.contains('/'));
    all_teams || all_personal
}

struct Supply {
    dependabot_gaps: Vec<String>,
    code_scanning: bool,
    code_scanning_pinned: bool,
    security_policy: bool,
    workflow_owners: bool,
}

/// GitHub Advanced Security configuration that lives in the tree. Alert counts need the API and
/// are not reported here.
fn supply_chain(root: &Path, paths: &[String], codeowners: Option<&Value>) -> Supply {
    let ecosystems = dependabot_ecosystems(root, paths);
    let wanted = ["cargo", "npm", "github-actions"];
    let dependabot_gaps = wanted
        .iter()
        .filter(|e| !ecosystems.iter().any(|g| g == *e))
        .map(|e| (*e).to_owned())
        .collect();
    let workflows: Vec<String> = paths
        .iter()
        .filter(|p| {
            p.starts_with(".github/workflows/") && (p.ends_with(".yml") || p.ends_with(".yaml"))
        })
        .filter_map(|p| std::fs::read_to_string(root.join(p)).ok())
        .collect();
    let code_scanning = workflows.iter().any(|t| t.contains("codeql-action"));
    let pinned = Regex::new(r"codeql-action/[\w.-]+@[0-9a-f]{40}").unwrap();
    let code_scanning_pinned = workflows.iter().any(|t| pinned.is_match(t));
    let security_policy = ["SECURITY.md", ".github/SECURITY.md", "docs/SECURITY.md"]
        .iter()
        .any(|p| paths.iter().any(|f| f == p));
    let workflow_owners = codeowners
        .and_then(|c| c["rules"].as_array())
        .is_some_and(|rules| workflow_owned(rules));
    Supply {
        dependabot_gaps,
        code_scanning,
        code_scanning_pinned,
        security_policy,
        workflow_owners,
    }
}

fn dependabot_ecosystems(root: &Path, paths: &[String]) -> Vec<String> {
    if !paths
        .iter()
        .any(|p| p == ".github/dependabot.yml" || p == ".github/dependabot.yaml")
    {
        return Vec::new();
    }
    let rel = if paths.iter().any(|p| p == ".github/dependabot.yml") {
        ".github/dependabot.yml"
    } else {
        ".github/dependabot.yaml"
    };
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let re = Regex::new(r#"package-ecosystem:\s*["']?([A-Za-z0-9_-]+)"#).unwrap();
    let mut found: Vec<String> = re
        .captures_iter(&text)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_owned()))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// The last CODEOWNERS rule that matches `.github/workflows/ci.yml` names an owner.
fn workflow_owned(rules: &[Value]) -> bool {
    let path = ".github/workflows/ci.yml";
    let mut owned = false;
    for rule in rules {
        let Some(pattern) = rule["pattern"].as_str() else {
            continue;
        };
        if codeowners_matches(pattern, path) {
            owned = rule["owners"].as_array().is_some_and(|o| !o.is_empty());
        }
    }
    owned
}

/// A subset of GitHub CODEOWNERS patterns: `*`, a directory prefix, or `*` within one segment.
fn codeowners_matches(pattern: &str, path: &str) -> bool {
    let pat = pattern.trim_start_matches('/');
    if pat == "*" {
        return true;
    }
    if let Some(dir) = pat.strip_suffix('/') {
        return path == dir || path.starts_with(&format!("{dir}/"));
    }
    if !pat.contains('*') {
        return path == pat || path.starts_with(&format!("{pat}/"));
    }
    let mut re = String::from("^");
    for (i, part) in pat.split('*').enumerate() {
        if i > 0 {
            re.push_str("[^/]*");
        }
        re.push_str(&regex::escape(part));
    }
    re.push('$');
    Regex::new(&re).is_ok_and(|r| r.is_match(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codeowners_rules() {
        let rules =
            parse_codeowners("# owners\n*  @acme/platform\n\n/docs/ @acme/docs @alice # writers\n");
        assert_eq!(
            rules,
            vec![
                json!({"pattern": "*", "owners": ["@acme/platform"], "line": 2}),
                json!({"pattern": "/docs/", "owners": ["@acme/docs", "@alice"], "line": 4}),
            ]
        );
    }

    #[test]
    fn owners_ok_allows_teams_or_only_personal_handles() {
        assert!(owners_ok(&["@acme/platform".into()]));
        assert!(owners_ok(&["@logan".into()]));
        assert!(owners_ok(&["@alice".into(), "@bob".into()]));
        assert!(!owners_ok(&[]));
        assert!(!owners_ok(&["@acme/platform".into(), "@alice".into()]));
        assert!(!owners_ok(&["alice@acme.com".into()]));
    }

    #[test]
    fn a_catch_all_codeowners_covers_workflows_and_dependabot_gaps_are_named() {
        let root = std::env::temp_dir().join(format!("seikika-repo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".github/workflows")).unwrap();
        std::fs::write(root.join("CODEOWNERS"), "* @acme/platform\n").unwrap();
        std::fs::write(
            root.join(".github/dependabot.yml"),
            "updates:\n  - package-ecosystem: cargo\n",
        )
        .unwrap();
        std::fs::write(
            root.join(".github/workflows/codeql.yml"),
            "uses: github/codeql-action/init@v3\n",
        )
        .unwrap();
        let paths = vec![
            "CODEOWNERS".into(),
            ".github/dependabot.yml".into(),
            ".github/workflows/codeql.yml".into(),
        ];
        let (_, rec) = record(&root, &paths);
        assert_eq!(rec["workflowOwners"], true);
        assert_eq!(rec["dependabotGaps"], json!(["npm", "github-actions"]));
        assert_eq!(rec["codeScanning"], true);
        assert_eq!(rec["codeScanningPinned"], false);
        assert_eq!(rec["securityPolicy"], false);
        let _ = std::fs::remove_dir_all(&root);
    }
}
