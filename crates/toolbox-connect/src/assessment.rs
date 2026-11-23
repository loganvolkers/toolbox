//! Agent assessments as connector records (docs/design/scorecards-for-scorecards.md §6). An
//! assessment is one agent's or person's judgement of one entity against one rubric, written as a
//! file (`schema/assessment.schema.json`) or pushed by `toolbox assess submit`. Both paths produce
//! the same `assessment` record, so a portal maps it once.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

/// Something a reader can check. Without a repository (`root` is `None`, e.g. `assess submit`
/// before the files are at hand), the shape alone: a path with a directory or a file extension, a
/// decision or question id (`D71`, `Q10`, a `D01…` ULID), or a URL. With the repository, each
/// path or file name must exist, and each id must be one the repository has, so "good and/or bad"
/// or "in 2026" cites nothing.
fn cites(reason: &str, root: Option<&std::path::Path>) -> bool {
    static PATH: OnceLock<regex::Regex> = OnceLock::new();
    static NAME: OnceLock<regex::Regex> = OnceLock::new();
    static ID: OnceLock<regex::Regex> = OnceLock::new();
    let path = PATH.get_or_init(|| {
        regex::Regex::new(r"(?:\.{0,2}/)?[\w.-]+(?:/[\w.<>*-]+)+").expect("valid regex")
    });
    let name = NAME.get_or_init(|| {
        regex::Regex::new(r"\b[\w-]+\.(?:md|mdx|rs|ya?ml|json|ndjson|toml|sh|py|ts|tsx|fga)\b")
            .expect("valid regex")
    });
    let id = ID.get_or_init(|| {
        regex::Regex::new(r"\b(?:D01[0-9A-Z]{20,}|D\d{1,3}|Q\d{1,3})\b").expect("valid regex")
    });
    if reason.contains("https://") || reason.contains("http://") {
        return true;
    }
    let Some(root) = root else {
        return path
            .find_iter(reason)
            .any(|m| m.as_str().contains('.') || m.as_str().matches('/').count() >= 2)
            || name.is_match(reason)
            || id.is_match(reason);
    };
    let exists = |p: &str| {
        let p = p
            .trim_start_matches("./")
            .trim_end_matches(['.', ',', ')', ':']);
        !p.is_empty() && root.join(p).exists()
    };
    path.find_iter(reason).any(|m| exists(m.as_str()))
        || name
            .find_iter(reason)
            .any(|m| file_names(root).contains(m.as_str()))
        || id.find_iter(reason).any(|m| known_id(root, m.as_str()))
}

/// File names in the repository (outside build output and dependencies), read once per root.
fn file_names(root: &std::path::Path) -> std::collections::HashSet<String> {
    static CACHE: OnceLock<
        std::sync::Mutex<
            std::collections::HashMap<std::path::PathBuf, std::collections::HashSet<String>>,
        >,
    > = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().expect("cache lock");
    cache
        .entry(root.to_path_buf())
        .or_insert_with(|| {
            walkdir::WalkDir::new(root)
                .into_iter()
                .filter_entry(|e| {
                    let n = e.file_name().to_string_lossy();
                    !matches!(n.as_ref(), "target" | "node_modules" | ".git" | "dist")
                })
                .flatten()
                .filter(|e| e.file_type().is_file())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .clone()
}

/// Whether the repository has this decision (a file in docs/decisions, or a backfilled file whose
/// `legacy:` field is the short id) or question (in docs/questions.md).
fn known_id(root: &std::path::Path, id: &str) -> bool {
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap_or_default();
    let word = |text: &str| {
        text.match_indices(id).any(|(i, _)| {
            let after = text[i + id.len()..].chars().next();
            !after.is_some_and(|c| c.is_ascii_alphanumeric())
        })
    };
    if id.starts_with("D01") {
        return root.join(format!("docs/decisions/{id}.md")).exists();
    }
    if id.starts_with('D') {
        let legacy = format!("legacy: {id}");
        return std::fs::read_dir(root.join("docs/decisions"))
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .unwrap_or_default()
                    .lines()
                    .any(|l| l.trim_end() == legacy)
            });
    }
    word(&read("docs/questions.md"))
}

/// The first 12 hex digits of the SHA-256 of the assessment as written: what an assessment of
/// this assessment is versioned by.
pub fn hash(doc: &Value) -> String {
    let text = serde_json::to_string(doc).unwrap_or_default();
    Sha256::digest(text.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The external id: one current assessment per (rubric, entity).
pub fn external_id(doc: &Value) -> String {
    format!(
        "assessment:{}/{}",
        doc["rubric"].as_str().unwrap_or_default(),
        doc["entity"].as_str().unwrap_or_default()
    )
}

/// The record data: the assessment as written, plus its overall `verdict` (`fail` when any
/// criterion fails, else `unsure` when any is unsure, else `pass`), the `failing` and `unsure`
/// criteria, the criteria whose fail or unsure verdict has no `suggestion` (`unsuggested`) or whose
/// reason cites nothing checkable (`uncited`), its content `hash`, and the `path` it was read from.
pub fn record(doc: &Value, path: Option<&str>) -> Value {
    record_in(doc, path, None)
}

/// [`record`], resolving each reason's citations against the repository at `root`.
pub fn record_in(doc: &Value, path: Option<&str>, root: Option<&std::path::Path>) -> Value {
    let by = |v: &str| -> Vec<String> {
        doc["verdicts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["verdict"] == v)
            .filter_map(|c| c["criterion"].as_str().map(str::to_owned))
            .collect()
    };
    let (failing, unsure) = (by("fail"), by("unsure"));
    let verdicts = doc["verdicts"].as_array().cloned().unwrap_or_default();
    let names = |keep: &dyn Fn(&Value) -> bool| -> Vec<String> {
        verdicts
            .iter()
            .filter(|v| keep(v))
            .filter_map(|v| v["criterion"].as_str().map(str::to_owned))
            .collect()
    };
    let unsuggested = names(&|v| {
        v["verdict"] != "pass" && v["suggestion"].as_str().is_none_or(|s| s.trim().is_empty())
    });
    let uncited = names(&|v| !cites(v["reason"].as_str().unwrap_or_default(), root));
    let verdict = if !failing.is_empty() {
        "fail"
    } else if !unsure.is_empty() {
        "unsure"
    } else {
        "pass"
    };
    let mut data = json!({
        "entity": doc["entity"], "rubric": doc["rubric"], "subjectHash": doc["subjectHash"],
        "assessor": doc["assessor"], "at": doc["at"], "verdicts": doc["verdicts"],
        "summary": doc.get("summary").cloned().unwrap_or(Value::Null),
        "verdict": verdict, "failing": failing, "unsure": unsure,
        "unsuggested": unsuggested, "uncited": uncited, "hash": hash(doc),
    });
    if let Some(p) = path {
        data["path"] = json!(p);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_rolls_up() {
        let doc = json!({"entity": "rule:p/c/r", "rubric": "rule-quality", "verdicts": [
            {"criterion": "a", "verdict": "pass"}, {"criterion": "b", "verdict": "unsure"}]});
        let r = record(&doc, Some("assessments/x.yaml"));
        assert_eq!(r["verdict"], "unsure");
        assert_eq!(r["unsure"], json!(["b"]));
        assert_eq!(external_id(&doc), "assessment:rule-quality/rule:p/c/r");
        let failed = json!({"verdicts": [{"criterion": "b", "verdict": "unsure"},
                                         {"criterion": "a", "verdict": "fail"}]});
        assert_eq!(record(&failed, None)["verdict"], "fail");
        assert_eq!(record(&failed, None)["unsuggested"], json!(["b", "a"]));
    }

    #[test]
    fn citations_are_checkable_references() {
        for yes in [
            "see docs/plan.md",
            "per D71",
            "crates/toolbox-api/tests",
            "lazy.rs",
            "Q10",
        ] {
            assert!(cites(yes, None), "{yes}");
        }
        for no in [
            "it seems fine",
            "good and/or bad",
            "in 2026",
            "20% failed",
            "yes/no",
        ] {
            assert!(!cites(no, None), "{no}");
        }
    }

    #[test]
    fn with_the_repository_a_citation_must_resolve() {
        let root = std::env::temp_dir().join(format!("toolbox-cites-{}", std::process::id()));
        std::fs::create_dir_all(root.join("docs/decisions")).unwrap();
        std::fs::write(root.join("docs/plan.md"), "").unwrap();
        std::fs::write(
            root.join("docs/decisions/D01M7VZ2DPS0000000000000000.md"),
            "---\nid: D01M7VZ2DPS0000000000000000\nlegacy: D71\n---\n",
        )
        .unwrap();
        std::fs::write(root.join("docs/questions.md"), "## Q10 a question").unwrap();
        let r = Some(root.as_path());
        for yes in [
            "docs/plan.md:12",
            "see plan.md",
            "per D71",
            "open in Q10",
            "https://x.dev",
        ] {
            assert!(cites(yes, r), "{yes}");
        }
        for no in ["docs/gone.md", "per D7", "Q1", "good and/or bad", "gone.rs"] {
            assert!(!cites(no, r), "{no}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
