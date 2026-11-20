//! Agent assessments as connector records (docs/design/scorecards-for-scorecards.md §6). An
//! assessment is one agent's or person's judgement of one entity against one rubric, written as a
//! file (`schema/assessment.schema.json`) or pushed by `toolbox assess submit`. Both paths produce
//! the same `assessment` record, so a portal maps it once.

use serde_json::{Value, json};

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
/// criteria, and the `path` it was read from (if any).
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
    }
}
