//! Shared SARIF ingestion for tools whose findings are keyed by file.

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// A finding, reduced to what scorecards and agents need.
fn finding(rule: &str, severity: &str, line: u64, message: &str) -> Value {
    json!({"rule": rule, "severity": severity, "line": line, "message": message})
}

fn normalise(path: &str) -> String {
    path.trim_start_matches("./").to_owned()
}

/// A SARIF artifact URI as a path relative to the checkout: `file://` and the root stripped.
fn sarif_path(uri: &str, root: &std::path::Path) -> String {
    let uri = uri.strip_prefix("file://").unwrap_or(uri);
    let abs = std::fs::canonicalize(root).ok();
    let rel = abs
        .as_ref()
        .and_then(|a| uri.strip_prefix(a.to_string_lossy().as_ref()))
        .map(|r| r.trim_start_matches('/'))
        .unwrap_or(uri);
    normalise(rel)
}

/// SARIF 2.1.0, the standard slot: every run's results by file. A finding has the rule's short
/// name (`rule`, after the last `/` or `.` of `ruleId`) and its `ruleId`, SARIF's `level`, a
/// `severity` (the tool's own `…severity` result property when it has one, as zizmor's
/// `zizmor/severity`; else the level), the first location's line, the message, `suppressed`
/// (SARIF `suppressions` not rejected, e.g. semgrep's `nosemgrep`) and the result's
/// `properties`. Files listed in `artifacts` are keys even with no results.
pub fn by_file(text: &str, root: &std::path::Path) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let doc: Value = serde_json::from_str(text).map_err(|e| format!("SARIF: {e}"))?;
    let mut by_file: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for run in doc["runs"].as_array().into_iter().flatten() {
        for a in run["artifacts"].as_array().into_iter().flatten() {
            if let Some(uri) = a["location"]["uri"].as_str() {
                by_file.entry(sarif_path(uri, root)).or_default();
            }
        }
        for r in run["results"].as_array().into_iter().flatten() {
            let loc = &r["locations"][0]["physicalLocation"];
            let path = sarif_path(loc["artifactLocation"]["uri"].as_str().unwrap_or(""), root);
            let id = r["ruleId"].as_str().unwrap_or("?");
            let short = id.rsplit('/').next().unwrap_or(id);
            let short = short.rsplit('.').next().unwrap_or(short);
            let level = r["level"].as_str().unwrap_or("warning");
            let props = r["properties"].clone();
            let severity = props
                .as_object()
                .and_then(|o| {
                    o.iter()
                        .find(|(k, v)| k.ends_with("severity") && v.is_string())
                })
                .and_then(|(_, v)| v.as_str())
                .unwrap_or(level)
                .to_lowercase();
            let suppressed = r["suppressions"]
                .as_array()
                .is_some_and(|s| s.iter().any(|x| x["status"] != "rejected"));
            let mut f = finding(
                short,
                &severity,
                loc["region"]["startLine"].as_u64().unwrap_or(0),
                r["message"]["text"].as_str().unwrap_or(""),
            );
            f["ruleId"] = json!(id);
            f["level"] = json!(level);
            f["suppressed"] = json!(suppressed);
            if !props.is_null() {
                f["properties"] = props;
            }
            by_file.entry(path).or_default().push(f);
        }
    }
    Ok(by_file)
}
