//! Agent instruction files (`AGENTS.md` and the aliases harnesses still read), read statically
//! (docs/design/agents-md-standards.md).
//!
//! One record per file. Facts only: whether it is the canonical `AGENTS.md` or an alias that
//! points at one, whether its relative links exist, whether a canonical file says how to test
//! and where decisions live, whether those check commands appear in a workflow, and a size.
//! Thresholds live in the manifest.

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn analyse(root: &Path, rel: &str, workflows: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let name = Path::new(rel)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let canonical = name == "AGENTS.md";
    json!({
        "path": rel,
        "name": name,
        "canonical": canonical,
        "lines": text.lines().count() as i64,
        "commandLines": command_lines(&text),
        "pointsAtAgents": text.contains("AGENTS.md"),
        "mentionsDecisions": text.contains("decisions.md"),
        "ciMissingCommands": ci_missing(&text, workflows),
        "missingFiles": missing_links(root, rel, &text),
        "hiddenChars": hidden_count(&text),
        "secretHits": secrets(rel, &text),
    })
}

/// Backtick invocations (`cargo test`, `toolbox validate`, `python scripts/…`) whose tool and
/// subcommand do not appear in `workflows`. Table rows are the track index, not the check
/// procedure, so a command that exists only there is not required to run in CI.
fn ci_missing(text: &str, workflows: &str) -> Vec<String> {
    let tick = compile(r"`([^`]{1,400})`");
    let inv = compile(
        r"(?i)^(?:(?:\./)?(?:target/debug/)?)?(cargo|npm|pnpm|yarn|pytest|python3?|fga|toolbox)\s+([A-Za-z][A-Za-z0-9-]*)",
    );
    let low = workflows.to_lowercase();
    let mut missing = Vec::new();
    for cap in tick.captures_iter(text) {
        let start = cap.get(0).map(|m| m.start()).unwrap_or(0);
        let line_at = text[..start].rfind('\n').map_or(0, |i| i + 1);
        let line = text[line_at..].lines().next().unwrap_or("");
        if line.trim_start().starts_with('|') {
            continue;
        }
        let body = cap
            .get(1)
            .map(|m| m.as_str().trim().lines().next().unwrap_or("").trim())
            .unwrap_or("");
        let Some(hit) = inv.captures(body) else {
            continue;
        };
        let needle = format!("{} {}", hit[1].to_lowercase(), hit[2].to_lowercase());
        if !low.contains(&needle) {
            missing.push(needle);
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

fn command_lines(text: &str) -> i64 {
    let re = compile(r"(?i)\b(cargo|npm|pnpm|yarn|pytest|toolbox)\b");
    text.lines().filter(|l| re.is_match(l)).count() as i64
}

fn missing_links(root: &Path, rel: &str, text: &str) -> Vec<String> {
    let re = compile(r"\]\(([^)\s#]+)(?:#[^)]*)?\)");
    let mut missing = Vec::new();
    for cap in re.captures_iter(text) {
        let Some(target) = cap.get(1).map(|m| m.as_str()) else {
            continue;
        };
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            continue;
        }
        let path = resolve(rel, target);
        if !root.join(&path).is_file() {
            missing.push(target.to_owned());
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

fn resolve(file: &str, target: &str) -> String {
    let base = Path::new(file).parent().unwrap_or(Path::new(""));
    let mut parts: Vec<&str> = base
        .components()
        .map(|c| c.as_os_str().to_str().unwrap_or(""))
        .filter(|s| !s.is_empty())
        .collect();
    for part in target.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn hidden_count(text: &str) -> i64 {
    text.chars()
        .filter(|c| {
            let u = *c as u32;
            (0x200B..=0x200F).contains(&u)
                || u == 0xFEFF
                || (0x202A..=0x202E).contains(&u)
                || (0x2066..=0x2069).contains(&u)
        })
        .count() as i64
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
        let root =
            std::env::temp_dir().join(format!("toolbox-agents-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_canonical_file_that_links_and_says_how_to_test() {
        let body = "# Agents\n\nSee [decisions](docs/decisions.md).\n\nRun `cargo test`.\n";
        let tmp = scratch(
            "good",
            &[("AGENTS.md", body), ("docs/decisions.md", "# Decisions\n")],
        );
        let rec = analyse(&tmp.0, "AGENTS.md", "run: cargo test\n");
        assert_eq!(rec["canonical"], true);
        assert_eq!(rec["missingFiles"], json!([]));
        assert!(rec["commandLines"].as_i64().unwrap() >= 1);
        assert_eq!(rec["mentionsDecisions"], true);
        assert_eq!(rec["secretHits"], json!([]));
    }

    #[test]
    fn an_alias_must_point_at_agents_and_a_broken_link_is_named() {
        let tmp = scratch(
            "alias",
            &[(
                "CLAUDE.md",
                "See [missing](docs/nope.md).\nAKIAIOSFODNN7EXAMPLE\n",
            )],
        );
        let rec = analyse(&tmp.0, "CLAUDE.md", "");
        assert_eq!(rec["canonical"], false);
        assert_eq!(rec["pointsAtAgents"], false);
        assert_eq!(rec["missingFiles"], json!(["docs/nope.md"]));
        assert_eq!(rec["secretHits"], json!(["CLAUDE.md:2 aws-access-key"]));
    }

    #[test]
    fn the_files_in_this_repo_are_canonical_or_a_short_pointer() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let workflows = std::fs::read_dir(root.join(".github/workflows"))
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| std::fs::read_to_string(e.path()).ok())
            .collect::<Vec<_>>()
            .join("\n");
        let agents = analyse(&root, "AGENTS.md", &workflows);
        assert_eq!(agents["canonical"], true);
        assert_eq!(agents["missingFiles"], json!([]));
        assert!(agents["commandLines"].as_i64().unwrap() >= 1);
        assert_eq!(agents["mentionsDecisions"], true);
        assert_eq!(agents["ciMissingCommands"], json!([]));
        assert!(agents["lines"].as_i64().unwrap() <= 500);
        let claude = analyse(&root, "CLAUDE.md", &workflows);
        assert_eq!(claude["canonical"], false);
        assert_eq!(claude["pointsAtAgents"], true);
        assert_eq!(claude["missingFiles"], json!([]));
        assert!(claude["lines"].as_i64().unwrap() <= 30);
    }

    #[test]
    fn a_check_command_missing_from_workflows_is_named_and_a_cursor_rule_is_an_alias() {
        let body =
            "# Agents\n\nRun `cargo test` and `toolbox nope`.\n| Operator | `toolbox dev` |\n";
        let tmp = scratch(
            "ci",
            &[("AGENTS.md", body), (".cursorrules", "See AGENTS.md\n")],
        );
        let agents = analyse(&tmp.0, "AGENTS.md", "cargo test\n");
        assert_eq!(agents["ciMissingCommands"], json!(["toolbox nope"]));
        let alias = analyse(&tmp.0, ".cursorrules", "");
        assert_eq!(alias["canonical"], false);
        assert_eq!(alias["pointsAtAgents"], true);
    }
}
