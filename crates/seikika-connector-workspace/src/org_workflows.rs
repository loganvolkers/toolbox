//! Audit a pinned workflow as data with the same SARIF slot as repository portals.

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
    let findings =
        seikika_connect::sarif::by_file(sarif, root.path()).map_err(|_| "Invalid zizmor SARIF")?;
    Ok(findings
        .into_values()
        .flatten()
        .map(|finding| {
            json!({
                "rule": finding["rule"], "severity": finding["severity"],
                "line": finding["line"], "message": finding["message"],
            })
        })
        .collect())
}
