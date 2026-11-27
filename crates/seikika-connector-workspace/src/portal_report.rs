//! Import published report data without dispatching child code or following a checkout symlink.

use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::process::Command;

pub(crate) const LIMIT: u64 = 8 * 1024 * 1024;
const REPORT_PATH: &str = ".seikika/report.json";

pub(crate) fn decode(data: &[u8]) -> Result<Option<Value>, &'static str> {
    let report: Value = serde_json::from_slice(data).map_err(|_| "Invalid child report JSON")?;
    if ["catalog", "scorecards", "scores"]
        .iter()
        .any(|key| !report[*key].is_object())
    {
        return Err("The child file is not a Seikika report");
    }
    Ok(Some(report))
}

pub(crate) fn attach(record: &mut Value, result: Result<Option<Value>, &'static str>) {
    match result {
        Ok(Some(report)) => {
            record["report"] = report;
        }
        Ok(None) => {}
        Err(error) => {
            record["reportError"] = error.into();
        }
    }
}

/// Use the same immutable tree as the action metadata. The tree entry must be a regular Git blob.
pub(crate) fn committed(root: &Path, commit: &str) -> Result<Option<Value>, &'static str> {
    let git = |args: &[&str]| {
        let result = Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .map_err(|_| "Unable to read child report from Git")?;
        if result.status.success() {
            Ok(result.stdout)
        } else {
            Err("Unable to read child report from Git")
        }
    };
    let tree = git(&["ls-tree", "--full-tree", commit, "--", REPORT_PATH])?;
    if tree.is_empty() {
        return Ok(None);
    }
    let tree = std::str::from_utf8(&tree).map_err(|_| "Invalid child report tree entry")?;
    let (entry, _) = tree
        .split_once('\t')
        .ok_or("Invalid child report tree entry")?;
    let parts: Vec<_> = entry.split_whitespace().collect();
    let ["100644" | "100755", "blob", oid] = parts.as_slice() else {
        return Err("The child report must be a regular file");
    };
    let size = git(&["cat-file", "-s", oid])?;
    let size: u64 = std::str::from_utf8(&size)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .ok_or("Invalid child report size")?;
    if size > LIMIT {
        return Err("The child report exceeds the size limit");
    }
    decode(&git(&["cat-file", "blob", oid])?)
}

pub(crate) fn local(root: &Path, directory: &Path) -> Result<Option<Value>, &'static str> {
    let file = directory.join("report.json");
    let metadata = match std::fs::symlink_metadata(&file) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("Unable to read the local child report"),
    };
    if !metadata.is_file() {
        return Err("The child report must be a regular file");
    }
    if metadata.len() > LIMIT {
        return Err("The child report exceeds the size limit");
    }
    let root = root
        .canonicalize()
        .map_err(|_| "Unable to locate the portal workspace")?;
    let file = file
        .canonicalize()
        .map_err(|_| "Unable to locate the local child report")?;
    if !file.starts_with(root) {
        return Err("The child report is outside the portal workspace");
    }
    let mut contents = Vec::new();
    std::fs::File::open(&file)
        .and_then(|f| f.take(LIMIT + 1).read_to_end(&mut contents))
        .map_err(|_| "Unable to read the local child report")?;
    if contents.len() as u64 > LIMIT {
        return Err("The child report exceeds the size limit");
    }
    decode(&contents)
}
