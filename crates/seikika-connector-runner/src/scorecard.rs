//! OpenSSF Scorecard as a tool in the box (docs/design/openssf-template.md): a fitting for its
//! `--format json --show-details` output, one record per check.
//!
//! Why a fitting and not a slot (AGENTS.md): Scorecard's SARIF output needs a policy file and an
//! experimental flag (`ENABLE_SARIF`), and a check scored 0–10 isn't a SARIF concept; its probe
//! format has no scores. Its JSON is the tool's own stable output, read here with no change.

use serde_json::{Value, json};

/// One record per check: `check`, `score` (null when Scorecard reports -1, inconclusive),
/// `reason`, `details`, `documentation`, and the run's `aggregate` and `scorecardVersion`.
pub fn checks(stdout: &str) -> Result<Vec<(String, Value)>, String> {
    let doc: Value = serde_json::from_str(stdout).map_err(|e| format!("scorecard JSON: {e}"))?;
    let version = doc["scorecard"]["version"].clone();
    let aggregate = doc["score"].clone();
    let checks = doc["checks"]
        .as_array()
        .ok_or("scorecard JSON: no `checks` (run it with --format json)")?;
    Ok(checks
        .iter()
        .filter_map(|c| {
            let name = c["name"].as_str()?.to_owned();
            let score = c["score"].as_i64().filter(|s| *s >= 0);
            Some((
                name.clone(),
                json!({
                    "check": name,
                    "score": score,
                    "inconclusive": score.is_none(),
                    "reason": c["reason"],
                    "details": c["details"].as_array().cloned().unwrap_or_default(),
                    "documentation": c["documentation"]["url"],
                    "aggregate": aggregate,
                    "scorecardVersion": version,
                    "via": "scorecard",
                }),
            ))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_record_per_check_and_inconclusive_is_null() {
        let out = r#"{"scorecard": {"version": "v5.7.0"}, "score": 7.2, "checks": [
            {"name": "SAST", "score": 10, "reason": "SAST tool detected: CodeQL", "details": null,
             "documentation": {"url": "https://github.com/ossf/scorecard/blob/main/docs/checks.md#sast"}},
            {"name": "Fuzzing", "score": -1, "reason": "internal error", "details": ["Warn: x"]}]}"#;
        let r = checks(out).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(
            (r[0].0.as_str(), r[0].1["score"].as_i64()),
            ("SAST", Some(10))
        );
        assert_eq!(r[1].1["score"], Value::Null);
        assert_eq!(r[1].1["inconclusive"], true);
        assert_eq!(r[1].1["details"][0], "Warn: x");
        assert!(checks("{}").is_err());
    }
}
