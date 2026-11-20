//! Agent assessments as connector records (docs/design/scorecards-for-scorecards.md §6). An
//! assessment is one agent's or person's judgement of one entity against one rubric, written as a
//! file (`schema/assessment.schema.json`) or pushed by `toolbox assess submit`. Both paths produce
//! the same `assessment` record, so a portal maps it once.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::OnceLock;

/// Something a reader can check: a path, a file name, a decision, question, lesson or finding id
/// (`D71`, `Q10`, `L9`, `F14`), a section (`§4`), a percentage or a number of two or more digits.
fn cites(reason: &str) -> bool {
    static RE: OnceLock<regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(
            r"[\w.-]+/[\w./<>-]+|\b[\w-]+\.(?:md|rs|ya?ml|json|toml|sh|py|ts|tsx)\b|\b[DQLF]\d+\b|§\s?\d|\d+%|\b\d{2,}\b",
        )
        .expect("valid regex")
    })
    .is_match(reason)
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
    let uncited = names(&|v| !cites(v["reason"].as_str().unwrap_or_default()));
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
            "§4 says",
            "20% failed",
            "lazy.rs",
        ] {
            assert!(cites(yes), "{yes}");
        }
        for no in ["it seems fine", "the check is good", "one test"] {
            assert!(!cites(no), "{no}");
        }
    }
}
