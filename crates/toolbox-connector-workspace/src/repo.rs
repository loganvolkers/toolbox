//! The `repo` record: one per checkout, with the files that repository standards look at (README,
//! LICENSE, CODEOWNERS and the `.github` directory), CODEOWNERS parsed into rules, and every owner
//! it names (sorted, unique). It is the entity
//! that repo-level scorecards (e.g. examples/repo-portal-starter) score.

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
    let name = name(root);
    (
        name.clone(),
        json!({
            "owners": owners,
            "name": name,
            "files": top,
            "readme": readme,
            "readmeWords": readme_words,
            "license": license,
            "codeowners": codeowners,
        }),
    )
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
}
