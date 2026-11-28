//! The `repo` record: one per checkout, with the files that repository standards look at (README,
//! LICENSE, CODEOWNERS and the `.github` directory), CODEOWNERS parsed into rules, and every owner
//! it names (sorted, unique). It is the entity
//! that repo-level scorecards (e.g. examples/repo-portal-starter) score.

use regex::Regex;
use serde_json::{Value, json};
use std::{path::Path, sync::LazyLock};

/// GitHub's CODEOWNERS locations, in the order GitHub looks.
pub(crate) const CODEOWNERS: &[&str] = &[".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS"];

fn readme_entrypoint(path: &str) -> bool {
    matches!(
        path.to_ascii_lowercase().as_str(),
        "readme" | "readme.md" | "readme.markdown" | "readme.rst" | "readme.txt" | "readme.adoc"
    )
}

pub(crate) fn readme_path(paths: &[String]) -> Option<&str> {
    paths
        .iter()
        .find(|p| readme_entrypoint(p))
        .or_else(|| {
            [".github/", "docs/"].into_iter().find_map(|dir| {
                paths
                    .iter()
                    .find(|p| p.strip_prefix(dir).is_some_and(readme_entrypoint))
            })
        })
        .or_else(|| {
            paths
                .iter()
                .find(|p| !p.contains('/') && p.to_ascii_lowercase().starts_with("readme"))
        })
        .map(String::as_str)
}

fn readme_words(path: &str, text: &str) -> usize {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let extension = Path::new(path).extension().and_then(|ext| ext.to_str());
    if extension.is_some_and(|ext| {
        !["md", "markdown"]
            .iter()
            .any(|md| ext.eq_ignore_ascii_case(md))
    }) {
        return text.split_whitespace().count();
    }

    let mut ignored = 0;
    let mut words = 0;
    for event in Parser::new_ext(text, Options::ENABLE_YAML_STYLE_METADATA_BLOCKS) {
        match event {
            Event::Start(Tag::CodeBlock(_) | Tag::Image { .. }) => ignored += 1,
            Event::End(TagEnd::CodeBlock | TagEnd::Image) => ignored -= 1,
            Event::Text(prose) if ignored == 0 => {
                words += prose
                    .split_whitespace()
                    .filter(|part| part.chars().any(char::is_alphanumeric))
                    .count();
            }
            _ => {}
        }
    }
    words
}

pub(crate) fn license_paths(paths: &[String]) -> impl Iterator<Item = &str> {
    paths.iter().map(String::as_str).filter(|path| {
        !path.contains('/')
            && ["license", "licence", "copying"].iter().any(|prefix| {
                path.to_ascii_lowercase()
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(['.', '-', '_']))
            })
    })
}

pub(crate) fn codeowners_path(paths: &[String]) -> Option<&'static str> {
    CODEOWNERS
        .iter()
        .copied()
        .find(|c| paths.iter().any(|p| p == c))
}

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

static HANDLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^@[a-zA-Z0-9]+(?:-[a-zA-Z0-9]+)*(?:/[a-zA-Z0-9][a-zA-Z0-9_-]*)?$").unwrap()
});
static EMAIL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[^@\s]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)+$").unwrap());

fn valid_handle(owner: &str) -> bool {
    HANDLE.is_match(owner)
        && owner
            .split('/')
            .next()
            .is_some_and(|login| login.len() <= 40)
}

/// CODEOWNERS rules with raw owners and a count of recognizable owner identifiers.
pub fn parse_codeowners(text: &str) -> Vec<Value> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let mut parts = line
                .split_whitespace()
                .take_while(|part| !part.starts_with('#'));
            let pattern = parts.next()?;
            let owners: Vec<&str> = parts.collect();
            let valid = owners
                .iter()
                .filter(|o| valid_handle(o) || EMAIL.is_match(o))
                .count();
            Some(json!({"pattern": pattern, "owners": owners, "validOwners": valid, "line": i + 1}))
        })
        .collect()
}

/// The `repo` record for the checkout at `root`; `paths` are its files (relative, sorted).
pub fn record(root: &Path, paths: &[String]) -> (String, Value) {
    let name = name(root);
    let mut facts = basics(&name, paths, |path| {
        Ok::<_, std::convert::Infallible>(Some(
            std::fs::read_to_string(root.join(path)).unwrap_or_default(),
        ))
    })
    .unwrap();
    let supply = supply_chain(root, paths, facts.get("codeowners"));
    facts["dependabotGaps"] = json!(supply.dependabot_gaps);
    facts["codeScanning"] = json!(supply.code_scanning);
    facts["codeScanningPinned"] = json!(supply.code_scanning_pinned);
    facts["securityPolicy"] = json!(supply.security_policy);
    facts["workflowOwners"] = json!(supply.workflow_owners);
    (name, facts)
}

/// The repository basics used by both checkout and GitHub scans. The GitHub reader can
/// distinguish a rejected file (`None`) from an API failure (`Err`).
pub(crate) fn basics<E>(
    name: &str,
    paths: &[String],
    mut read: impl FnMut(&str) -> Result<Option<String>, E>,
) -> Result<Value, E> {
    let top: Vec<&String> = paths
        .iter()
        .filter(|p| !p.contains('/') || p.starts_with(".github/"))
        .collect();
    let readme = readme_path(paths);
    let mut license = None;
    let mut license_has_content = false;
    for path in license_paths(paths) {
        license.get_or_insert(path);
        if read(path)?.is_some_and(|text| !text.trim().is_empty()) {
            license = Some(path);
            license_has_content = true;
            break;
        }
    }
    let codeowners = match codeowners_path(paths) {
        Some(path) => {
            read(path)?.map(|text| json!({"path": path, "rules": parse_codeowners(&text)}))
        }
        None => None,
    };
    let readme_words = match readme {
        Some(path) => read(path)?.map_or(0, |text| readme_words(path, &text)),
        None => 0,
    };
    let mut owners: Vec<String> = codeowners
        .iter()
        .flat_map(|c| c["rules"].as_array().cloned().unwrap_or_default())
        .flat_map(|r| r["owners"].as_array().cloned().unwrap_or_default())
        .filter_map(|o| o.as_str().map(str::to_owned))
        .collect();
    owners.sort();
    owners.dedup();
    let owners_ok = owners_ok(&owners);
    Ok(json!({
        "owners": owners,
        "ownersOk": owners_ok,
        "name": name,
        "files": top,
        "readme": readme,
        "readmeWords": readme_words,
        "readmeEntrypoint": readme.is_some_and(readme_entrypoint),
        "license": license,
        "licenseHasContent": license_has_content,
        "codeowners": codeowners,
    }))
}

/// True when every owner is `@org/team`, or every owner is a personal `@user` (D01M7Y57FVX0XYNGMX0A0XJ4GK9).
fn owners_ok(owners: &[String]) -> bool {
    if owners.is_empty() || !owners.iter().all(|o| valid_handle(o)) {
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
                json!({"pattern": "*", "owners": ["@acme/platform"], "validOwners": 1, "line": 2}),
                json!({"pattern": "/docs/", "owners": ["@acme/docs", "@alice"], "validOwners": 2, "line": 4}),
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
        assert!(!owners_ok(&["@foo-".into()]));
        assert!(!owners_ok(&["@foo--bar".into()]));
        assert_eq!(parse_codeowners("* nobody")[0]["validOwners"], 0);
        assert_eq!(
            parse_codeowners("/private/\t# @org/team")[0]["validOwners"],
            0
        );
        assert_eq!(
            parse_codeowners("* @org/team\t# explanation")[0]["validOwners"],
            1
        );
        assert!(!owners_ok(&[format!("@{}", "a".repeat(40))]));
    }

    #[test]
    fn readme_fallback_counts_markdown_prose_without_badges_or_code() {
        let files = vec!["docs/README.md".into(), "README-notes.md".into()];
        let facts = basics("example", &files, |path| {
            assert_eq!(path, "docs/README.md");
            Ok::<_, ()>(Some(
                "# Getting started\n\nRun the application.\n\n[![Build status](badge.svg)](build)\n\n<!-- internal note -->\n\n```sh\nrun this example\n```\n".into(),
            ))
        })
        .unwrap();
        assert_eq!(facts["readme"], "docs/README.md");
        assert_eq!(facts["readmeWords"], 5);
        assert_eq!(facts["readmeEntrypoint"], false);
    }

    #[test]
    fn empty_copying_does_not_mask_the_license() {
        let files = vec!["COPYING".into(), "LICENCE".into()];
        let facts = basics("example", &files, |path| {
            Ok::<_, ()>(Some(
                if path == "LICENCE" {
                    "reuse terms"
                } else {
                    " "
                }
                .into(),
            ))
        })
        .unwrap();
        assert_eq!(facts["license"], "LICENCE");
        assert_eq!(facts["licenseHasContent"], true);
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
