//! The runner works in any repository, not only a Cargo workspace: only per-crate jobs ask Cargo
//! for crates (docs/design/repo-value.md R2 found the `github-actions` package's lint connector
//! failing on every non-Rust checkout).

use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

/// Resync the runner in `dir` with `jobs`; return the messages it printed.
fn resync(dir: &std::path::Path, jobs: Value) -> Vec<Value> {
    let cfg = serde_json::json!({"name": "t", "root": dir, "jobs": jobs});
    let mut child = Command::new(env!("CARGO_BIN_EXE_toolbox-connector-runner"))
        .current_dir(dir)
        .env("TOOLBOX_CONNECTOR_CONFIG", cfg.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("runner starts");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"type\":\"command\",\"command_id\":\"c1\",\"command\":\"resync\"}\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

#[test]
fn ndjson_and_lint_jobs_run_outside_a_cargo_workspace() {
    let dir = std::env::temp_dir().join(format!("toolbox-runner-any-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let jobs = serde_json::json!([
        {"name": "notes", "kinds": ["note"], "command": ["sh", "-c",
            "echo '{\"type\":\"record\",\"kind\":\"note\",\"externalId\":\"note:1\",\"data\":{}}'"]},
        // A lint tool that isn't installed is optional: its rules become unknown, the resync succeeds.
        {"name": "actionlint", "parse": "actionlint", "optional": true,
         "command": ["toolbox-no-such-linter"], "files": [".github/workflows/*.yml"]}
    ]);
    let msgs = resync(&dir, jobs);
    let done = msgs
        .iter()
        .find(|m| m["type"] == "complete")
        .expect("a completion");
    assert_eq!(done["status"], "succeeded", "{msgs:#?}");
    assert!(
        msgs.iter()
            .any(|m| m["type"] == "record" && m["externalId"] == "note:1"),
        "{msgs:#?}"
    );
    // `{files}` with nothing matching: the linter isn't run (it would reject an empty input).
    let msgs = resync(
        &dir,
        serde_json::json!([{"name": "zizmor", "parse": "zizmor",
            "command": ["sh", "-c", "exit 3", "{files}"], "files": [".github/workflows/*.y*ml"]}]),
    );
    let done = msgs.iter().find(|m| m["type"] == "complete").unwrap();
    assert_eq!(done["status"], "succeeded", "{msgs:#?}");
    // With a matching file, `{files}` becomes its path: the tool sees it as its last argument.
    std::fs::create_dir_all(dir.join(".github/workflows")).unwrap();
    std::fs::write(dir.join(".github/workflows/ci.yml"), "on: push\n").unwrap();
    let msgs = resync(
        &dir,
        serde_json::json!([{"name": "zizmor", "parse": "zizmor",
            "command": ["sh", "-c", "test \"$0\" = .github/workflows/ci.yml && echo []", "{files}"],
            "files": [".github/workflows/*.y*ml"]}]),
    );
    let done = msgs.iter().find(|m| m["type"] == "complete").unwrap();
    assert_eq!(done["status"], "succeeded", "{msgs:#?}");
    assert!(
        msgs.iter()
            .any(|m| m["externalId"] == "zizmor:.github/workflows/ci.yml"),
        "{msgs:#?}"
    );
    // A per-crate job still needs a Cargo workspace, and says so.
    let msgs = resync(
        &dir,
        serde_json::json!([{"name": "tests", "per": "crate", "parse": "cargo-test", "command": ["true"]}]),
    );
    let done = msgs.iter().find(|m| m["type"] == "complete").unwrap();
    assert_eq!(done["status"], "failed");
    assert!(done.to_string().contains("cargo metadata"));
    let _ = std::fs::remove_dir_all(&dir);
}
