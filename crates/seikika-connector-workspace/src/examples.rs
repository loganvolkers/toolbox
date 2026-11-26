//! Example and experiment write-ups (`README.md` under `examples/` and `experiments/`, plus
//! `docs/examples`), read statically (docs/design/example-standards.md).
//!
//! One record per document. Facts only: whether its relative links exist, whether it names a
//! command that checks it, and whether it points at a design doc or a decision. Thresholds live
//! in the manifest. The reference portal's `expected/` outputs are not regenerated here.

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

pub(crate) fn analyse(root: &Path, rel: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let title = text
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or("")
        .trim()
        .to_owned();
    json!({
        "path": rel,
        "title": title,
        "lines": text.lines().count() as i64,
        "commandLines": command_lines(&text),
        "missingFiles": missing_links(root, rel, &text),
        "citesDesign": cites_design(&text),
        "secretHits": secrets(rel, &text),
    })
}

fn command_lines(text: &str) -> i64 {
    let re = compile(
        r"(?i)(\b(cargo|npm|pnpm|yarn|pytest|python3?|fga)\s|\bseikika\s+(validate|test|report|plan|build|derive)|\./run)",
    );
    text.lines().filter(|l| re.is_match(l)).count() as i64
}

fn cites_design(text: &str) -> bool {
    text.contains("docs/")
        || text.contains("docs/decisions")
        || compile(r"(?i)\b(D\d+|D[0-9A-HJKMNP-TV-Z]{26})\b").is_match(text)
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
        if !root.join(&path).exists() {
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

fn secrets(rel: &str, text: &str) -> Vec<String> {
    crate::hygiene::secrets(rel, text)
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
            std::env::temp_dir().join(format!("seikika-example-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_write_up_that_links_and_names_a_check_cites_the_design() {
        let body =
            "# Reference\n\nSee [the spec](../../docs/v1-spec.md) (D2).\n\n`seikika validate`\n";
        let tmp = scratch(
            "good",
            &[
                ("examples/demo/README.md", body),
                ("docs/v1-spec.md", "# Spec\n"),
            ],
        );
        let rec = analyse(&tmp.0, "examples/demo/README.md");
        assert_eq!(rec["title"], "Reference");
        assert_eq!(rec["missingFiles"], json!([]));
        assert!(rec["commandLines"].as_i64().unwrap() >= 1);
        assert_eq!(rec["citesDesign"], true);
    }

    #[test]
    fn a_broken_link_and_a_secret_are_named() {
        let body = "# Spec\n\nSee [missing](nope.md).\nAKIAIOSFODNN7EXAMPLE\n";
        let tmp = scratch("bad", &[("docs/examples/unsafe.md", body)]);
        let rec = analyse(&tmp.0, "docs/examples/unsafe.md");
        assert_eq!(rec["commandLines"], 0);
        assert_eq!(rec["missingFiles"], json!(["nope.md"]));
        assert_eq!(rec["citesDesign"], false);
        assert_eq!(
            rec["secretHits"],
            json!(["docs/examples/unsafe.md:4 aws-access-key"])
        );
    }

    #[test]
    fn the_reference_portal_names_its_checks_and_the_executor_spec_does_not() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let portal = analyse(&root, "examples/reference-portal/README.md");
        assert_eq!(portal["missingFiles"], json!([]));
        assert!(portal["commandLines"].as_i64().unwrap() >= 1);
        assert_eq!(portal["citesDesign"], true);
        let spec = analyse(
            &root,
            "examples/reference-portal/executors/github-apps/README.md",
        );
        assert_eq!(spec["missingFiles"], json!([]));
        assert_eq!(spec["commandLines"], 0);
    }
}
