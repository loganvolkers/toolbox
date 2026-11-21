//! Findings per file, for the box (docs/design/toolbox-and-tools.md): the SARIF 2.1.0 slot, which
//! any tool with SARIF output fits without code here, and fittings for tools without it: zizmor
//! (`--format json`, v1), actionlint (`-format '{{json .}}'`) and semgrep (`--json`). Every file
//! the job covers gets a record, so a clean file is distinguishable from one that wasn't scanned
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
pub fn sarif(text: &str, root: &std::path::Path) -> Result<BTreeMap<String, Vec<Value>>, String> {
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

/// The record for one file: its findings (sorted by line), counts per severity, how many are
/// suppressed inline (`suppressed`) and how many not (`active`), `command`: the command that
/// reproduces this file's findings with the tool alone (take-it-out), and `via`: the slot or
/// fitting that read its output (`sarif`, `zizmor`, …).
pub fn record(path: &str, command: &str, via: &str, mut findings: Vec<Value>) -> Value {
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
        "note",
        "none",
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
        "findings": findings,
        "command": command,
        "via": via
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
        let rec = record(
            ".github/workflows/ci.yml",
            "zizmor ci.yml",
            "zizmor",
            f.clone(),
        );
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
        let rec = record("app/a.py", "semgrep", "semgrep", r["app/a.py"].clone());
        assert_eq!(rec["findings"][0]["rule"], "no-orm");
        assert_eq!(rec["counts"]["warning"], 3);
        assert_eq!(
            (rec["suppressed"].as_u64(), rec["active"].as_u64()),
            (Some(2), Some(1))
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn sarif_results_by_file_with_tool_severity_and_suppressions() {
        // The shapes zizmor 1.32.0 (`--format sarif`) and semgrep 1.163.0 (`--sarif`) print, trimmed.
        let zizmor = r#"{"version": "2.1.0", "runs": [{"tool": {"driver": {"name": "zizmor"}}, "results": [
            {"ruleId": "zizmor/self-repository", "level": "note", "message": {"text": "self"},
             "properties": {"zizmor/severity": "Low", "zizmor/confidence": "High"},
             "locations": [{"physicalLocation": {"artifactLocation": {"uri": ".github/workflows/ci.yml"},
               "region": {"startLine": 12}}}]}]}]}"#;
        let root = std::path::Path::new(".");
        let r = sarif(zizmor, root).unwrap();
        let f = &r[".github/workflows/ci.yml"][0];
        assert_eq!(
            (
                f["rule"].as_str(),
                f["severity"].as_str(),
                f["line"].as_u64()
            ),
            (Some("self-repository"), Some("low"), Some(12))
        );
        assert_eq!(f["suppressed"], false);
        let semgrep = r#"{"version": "2.1.0", "runs": [{"tool": {"driver": {"name": "Semgrep OSS"}},
            "artifacts": [{"location": {"uri": "app/clean.py"}}], "results": [
            {"ruleId": "semgrep.rules.no-orm", "level": "warning", "message": {"text": "use the helpers"},
             "suppressions": [{"kind": "inSource"}],
             "locations": [{"physicalLocation": {"artifactLocation": {"uri": "app/a.py"}, "region": {"startLine": 3}}}]},
            {"ruleId": "semgrep.rules.no-orm", "level": "warning", "message": {"text": "use the helpers"},
             "suppressions": [{"kind": "inSource", "status": "rejected"}],
             "locations": [{"physicalLocation": {"artifactLocation": {"uri": "app/a.py"}, "region": {"startLine": 9}}}]}]}]}"#;
        let r = sarif(semgrep, root).unwrap();
        assert!(r["app/clean.py"].is_empty());
        let rec = record(
            "app/a.py",
            "semgrep --sarif app/a.py",
            "sarif",
            r["app/a.py"].clone(),
        );
        assert_eq!(rec["findings"][0]["rule"], "no-orm");
        assert_eq!(
            (rec["suppressed"].as_u64(), rec["active"].as_u64()),
            (Some(1), Some(1))
        );
        assert_eq!(rec["counts"]["warning"], 2);
        assert_eq!(rec["command"], "semgrep --sarif app/a.py");
    }

    #[test]
    fn actionlint_errors_and_clean_output() {
        let out = r#"[{"message":"unexpected key \"branch\"","filepath":".github/workflows/a.yml","line":3,"column":5,"kind":"syntax-check"}]"#;
        let r = actionlint(out).unwrap();
        assert_eq!(r[".github/workflows/a.yml"][0]["rule"], "syntax-check");
        assert!(actionlint("null\n").unwrap().is_empty());
        assert!(actionlint("").unwrap().is_empty());
        assert_eq!(
            record("x.yml", "actionlint x.yml", "actionlint", vec![])["total"],
            0
        );
    }
}
