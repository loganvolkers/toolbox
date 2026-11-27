//! Audit a pinned workflow as data and validate zizmor's SARIF before scoring it.

use serde_json::{Value, json};
use std::time::Duration;

pub(crate) async fn audit(text: &str) -> Result<Vec<Value>, &'static str> {
    // Only this blob exists in the audit directory. There is no repository, action code,
    // child-owned config, or GitHub credential for the tool to discover.
    let root = tempfile::tempdir().map_err(|_| "Unable to stage workflow for audit")?;
    let dir = root.path().join(".github/workflows");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|_| "Unable to stage workflow for audit")?;
    let path = ".github/workflows/workflow.yml";
    tokio::fs::write(root.path().join(path), text)
        .await
        .map_err(|_| "Unable to stage workflow for audit")?;
    let mut command = tokio::process::Command::new("zizmor");
    command
        .args([
            "--offline",
            "--no-config",
            "--no-ignores",
            "--no-exit-codes",
            "--strict-collection",
            "--format",
            "sarif",
            path,
        ])
        .current_dir(root.path())
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .kill_on_drop(true);
    let result = tokio::time::timeout(Duration::from_secs(15), command.output())
        .await
        .map_err(|_| "zizmor audit timed out")?
        .map_err(|_| "zizmor is unavailable")?;
    if !result.status.success() {
        return Err("zizmor could not audit this workflow");
    }
    let sarif = std::str::from_utf8(&result.stdout).map_err(|_| "Invalid zizmor SARIF")?;
    findings(sarif)
}

fn findings(sarif: &str) -> Result<Vec<Value>, &'static str> {
    const INVALID: &str = "Invalid zizmor SARIF";
    let doc: Value = serde_json::from_str(sarif).map_err(|_| INVALID)?;
    let [run] = doc["runs"].as_array().map(Vec::as_slice).ok_or(INVALID)? else {
        return Err(INVALID);
    };
    if doc["version"] != "2.1.0" || run["tool"]["driver"]["name"] != "zizmor" {
        return Err(INVALID);
    }
    if let Some(invocations) = run.get("invocations")
        && !invocations
            .as_array()
            .is_some_and(|rows| rows.iter().all(|row| row["executionSuccessful"] == true))
    {
        return Err(INVALID);
    }
    run["results"]
        .as_array()
        .ok_or(INVALID)?
        .iter()
        .map(|result| {
            let severity = result["properties"]["zizmor/severity"]
                .as_str()
                .ok_or(INVALID)?
                .to_ascii_lowercase();
            if !matches!(
                severity.as_str(),
                "high" | "medium" | "low" | "informational" | "unknown"
            ) {
                return Err(INVALID);
            }
            let rule = result["ruleId"]
                .as_str()
                .and_then(|rule| rule.strip_prefix("zizmor/"))
                .filter(|s| !s.is_empty())
                .ok_or(INVALID)?;
            let message = result["message"]["text"].as_str().ok_or(INVALID)?;
            let line = result["locations"][0]["physicalLocation"]["region"]["startLine"]
                .as_u64()
                .filter(|line| *line > 0)
                .ok_or(INVALID)?;
            Ok(json!({"rule": rule, "severity": severity, "line": line, "message": message}))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_requires_a_completed_audit_with_classified_results() {
        let report = |results| {
            json!({"version": "2.1.0", "runs": [{
                "tool": {"driver": {"name": "zizmor"}}, "results": results
            }]})
            .to_string()
        };
        assert_eq!(findings(&report(json!([]))).unwrap(), Vec::<Value>::new());
        for sarif in [
            "{}".into(),
            report(json!(null)),
            report(json!([{"level":"error"}])),
        ] {
            assert!(findings(&sarif).is_err());
        }
        let mut failed: Value = serde_json::from_str(&report(json!([]))).unwrap();
        failed["runs"][0]["invocations"] = json!([{"executionSuccessful": false}]);
        assert!(findings(&failed.to_string()).is_err());
        let low = report(json!([{
            "ruleId": "zizmor/self-repository",
            "message": {"text": "use a local checkout path"},
            "properties": {"zizmor/severity": "Low"},
            "locations": [{"physicalLocation": {"region": {"startLine": 3}}}]
        }]));
        assert_eq!(findings(&low).unwrap()[0]["severity"], "low");
    }
}
