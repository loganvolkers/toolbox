//! Parsers for CI linters whose findings are reported per file: zizmor (`--format json`, v1) and
//! actionlint (`-format '{{json .}}'`). Every file the job covers gets a record, so a clean file is
//! distinguishable from one that wasn't scanned (docs/design/github-actions-standards.md §4).

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// A finding, reduced to what scorecards and agents need.
fn finding(rule: &str, severity: &str, line: u64, message: &str) -> Value {
    json!({"rule": rule, "severity": severity, "line": line, "message": message})
}

fn normalise(path: &str) -> String {
    path.trim_start_matches("./").to_owned()
}

/// zizmor JSON v1: a flat array of findings. Locations carry 0-based rows; the first location
/// (the primary one when marked) names the file.
pub fn zizmor(stdout: &str) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let all: Vec<Value> = serde_json::from_str(stdout).map_err(|e| format!("zizmor JSON: {e}"))?;
    let mut by_file: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for f in all {
        let locs = f["locations"].as_array().cloned().unwrap_or_default();
        let loc = locs
            .iter()
            .find(|l| l["symbolic"]["kind"] == "Primary")
            .or(locs.first())
            .cloned()
            .unwrap_or(Value::Null);
        let path = loc["symbolic"]["key"]["Local"]["verbatim_path"]
            .as_str()
            .map(normalise)
            .unwrap_or_default();
        let row = loc["concrete"]["location"]["start_point"]["row"]
            .as_u64()
            .unwrap_or(0);
        by_file.entry(path).or_default().push(finding(
            f["ident"].as_str().unwrap_or("?"),
            &f["determinations"]["severity"]
                .as_str()
                .unwrap_or("Unknown")
                .to_lowercase(),
            row + 1,
            f["desc"].as_str().unwrap_or(""),
        ));
    }
    Ok(by_file)
}

/// actionlint JSON: an array of `{message, filepath, line, column, kind, …}`; `null` or empty when
/// clean. Every actionlint error counts as `error` severity.
pub fn actionlint(stdout: &str) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let text = stdout.trim();
    if text.is_empty() || text == "null" {
        return Ok(BTreeMap::new());
    }
    let all: Vec<Value> =
        serde_json::from_str(text).map_err(|e| format!("actionlint JSON: {e}"))?;
    let mut by_file: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for e in all {
        by_file
            .entry(normalise(e["filepath"].as_str().unwrap_or("")))
            .or_default()
            .push(finding(
                e["kind"].as_str().unwrap_or("?"),
                "error",
                e["line"].as_u64().unwrap_or(0),
                e["message"].as_str().unwrap_or(""),
            ));
    }
    Ok(by_file)
}

/// The record for one file: its findings (sorted by line) and counts per severity.
pub fn record(path: &str, mut findings: Vec<Value>) -> Value {
    findings.sort_by_key(|f| (f["line"].as_u64(), f["rule"].as_str().map(str::to_owned)));
    let mut counts = Map::new();
    for sev in ["high", "medium", "low", "informational", "unknown", "error"] {
        counts.insert(sev.into(), json!(0));
    }
    for f in &findings {
        let s = f["severity"].as_str().unwrap_or("unknown").to_owned();
        let n = counts.get(&s).and_then(Value::as_u64).unwrap_or(0) + 1;
        counts.insert(s, json!(n));
    }
    json!({"path": path, "total": findings.len(), "counts": counts, "findings": findings})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zizmor_findings_group_by_file_with_one_based_lines() {
        // The shape from zizmor's docs (docs/usage.md, "JSON"), trimmed.
        let out = r#"[{"ident": "template-injection", "desc": "code injection via template expansion",
            "determinations": {"confidence": "High", "severity": "High", "persona": "Regular"},
            "locations": [
              {"symbolic": {"key": {"Local": {"verbatim_path": "./.github/workflows/ci.yml"}}, "kind": "Hidden"},
               "concrete": {"location": {"start_point": {"row": 6, "column": 8}}}},
              {"symbolic": {"key": {"Local": {"verbatim_path": "./.github/workflows/ci.yml"}}, "kind": "Primary"},
               "concrete": {"location": {"start_point": {"row": 7, "column": 13}}}}]}]"#;
        let r = zizmor(out).unwrap();
        let f = &r[".github/workflows/ci.yml"];
        assert_eq!(f[0]["rule"], "template-injection");
        assert_eq!(
            (f[0]["severity"].as_str(), f[0]["line"].as_u64()),
            (Some("high"), Some(8))
        );
        let rec = record(".github/workflows/ci.yml", f.clone());
        assert_eq!(
            (rec["counts"]["high"].as_u64(), rec["total"].as_u64()),
            (Some(1), Some(1))
        );
    }

    #[test]
    fn actionlint_errors_and_clean_output() {
        let out = r#"[{"message":"unexpected key \"branch\"","filepath":".github/workflows/a.yml","line":3,"column":5,"kind":"syntax-check"}]"#;
        let r = actionlint(out).unwrap();
        assert_eq!(r[".github/workflows/a.yml"][0]["rule"], "syntax-check");
        assert!(actionlint("null\n").unwrap().is_empty());
        assert!(actionlint("").unwrap().is_empty());
        assert_eq!(record("x.yml", vec![])["total"], 0);
    }
}
