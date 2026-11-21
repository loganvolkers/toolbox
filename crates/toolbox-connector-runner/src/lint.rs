//! Parsers for linters whose findings are reported per file: zizmor (`--format json`, v1),
//! actionlint (`-format '{{json .}}'`) and semgrep (`--json`). Every file the job covers gets a
//! record, so a clean file is distinguishable from one that wasn't scanned
//! (docs/design/github-actions-standards.md §4).

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

/// semgrep `--json`: `{results: [{check_id, path, start: {line}, extra: {message, severity}}],
/// paths: {scanned}}`. Every scanned file is a key (clean ones with no findings). Run it with
/// `--disable-nosem` to count suppressed findings too: semgrep's JSON doesn't mark them, so a
/// finding is `suppressed` when its line, or the line above, has a `nosemgrep` comment (read from
/// the file under `root`). The suppressions are then a standard's baseline, and a campaign can pay
/// them down.
pub fn semgrep(
    stdout: &str,
    root: &std::path::Path,
) -> Result<BTreeMap<String, Vec<Value>>, String> {
    let doc: Value = serde_json::from_str(stdout).map_err(|e| format!("semgrep JSON: {e}"))?;
    let mut by_file: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for p in doc["paths"]["scanned"].as_array().into_iter().flatten() {
        if let Some(p) = p.as_str() {
            by_file.entry(normalise(p)).or_default();
        }
    }
    let mut sources: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in doc["results"].as_array().into_iter().flatten() {
        let path = normalise(r["path"].as_str().unwrap_or(""));
        let line = r["start"]["line"].as_u64().unwrap_or(0);
        let text = sources.entry(path.clone()).or_insert_with(|| {
            std::fs::read_to_string(root.join(&path))
                .map(|t| t.lines().map(str::to_owned).collect())
                .unwrap_or_default()
        });
        let at = |n: u64| {
            n.checked_sub(1)
                .and_then(|i| text.get(i as usize))
                .is_some_and(|l| l.contains("nosemgrep"))
        };
        let id = r["check_id"].as_str().unwrap_or("?");
        let mut f = finding(
            id.rsplit('.').next().unwrap_or(id),
            &r["extra"]["severity"]
                .as_str()
                .unwrap_or("unknown")
                .to_lowercase(),
            line,
            r["extra"]["message"].as_str().unwrap_or(""),
        );
        f["suppressed"] = json!(at(line) || at(line.saturating_sub(1)));
        by_file.entry(path).or_default().push(f);
    }
    Ok(by_file)
}

/// The record for one file: its findings (sorted by line), counts per severity, and how many are
/// suppressed inline (`suppressed`) and how many not (`active`).
pub fn record(path: &str, mut findings: Vec<Value>) -> Value {
    findings.sort_by_key(|f| (f["line"].as_u64(), f["rule"].as_str().map(str::to_owned)));
    let mut counts = Map::new();
    for sev in [
        "high",
        "medium",
        "low",
        "informational",
        "unknown",
        "error",
        "warning",
        "info",
    ] {
        counts.insert(sev.into(), json!(0));
    }
    for f in &findings {
        let s = f["severity"].as_str().unwrap_or("unknown").to_owned();
        let n = counts.get(&s).and_then(Value::as_u64).unwrap_or(0) + 1;
        counts.insert(s, json!(n));
    }
    let suppressed = findings.iter().filter(|f| f["suppressed"] == true).count();
    json!({
        "path": path,
        "total": findings.len(),
        "suppressed": suppressed,
        "active": findings.len() - suppressed,
        "counts": counts,
        "findings": findings
    })
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
    fn semgrep_findings_with_suppressions_and_clean_files() {
        // The shape semgrep 1.163.0 prints with `--json --disable-nosem` (trimmed).
        let dir = std::env::temp_dir().join(format!("semgrep-parse-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("app")).unwrap();
        std::fs::write(
            dir.join("app/a.py"),
            "x = Person.objects.get(pk=1)  # nosemgrep: no-orm\n# nosemgrep: no-orm\ny = Person.objects.all()\nz = Person.objects.none()\n",
        )
        .unwrap();
        let out = r#"{"results": [
            {"check_id": "semgrep.rules.no-orm", "path": "app/a.py", "start": {"line": 1, "col": 5},
             "extra": {"message": "use the helpers", "severity": "WARNING"}},
            {"check_id": "semgrep.rules.no-orm", "path": "app/a.py", "start": {"line": 3, "col": 5},
             "extra": {"message": "use the helpers", "severity": "WARNING"}},
            {"check_id": "semgrep.rules.no-orm", "path": "app/a.py", "start": {"line": 4, "col": 5},
             "extra": {"message": "use the helpers", "severity": "WARNING"}}],
          "errors": [], "paths": {"scanned": ["app/a.py", "app/clean.py"]}}"#;
        let r = semgrep(out, &dir).unwrap();
        assert!(r["app/clean.py"].is_empty());
        let rec = record("app/a.py", r["app/a.py"].clone());
        assert_eq!(rec["findings"][0]["rule"], "no-orm");
        assert_eq!(rec["counts"]["warning"], 3);
        assert_eq!(
            (rec["suppressed"].as_u64(), rec["active"].as_u64()),
            (Some(2), Some(1))
        );
        let _ = std::fs::remove_dir_all(dir);
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
