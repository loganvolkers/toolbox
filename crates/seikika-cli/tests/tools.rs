//! The box's tools, end to end through the binary (D01M800DHD0Z1RTPH7CM46GZS3B,
//! D01M801A9QNVCSBBJATYV721TCS): `seikika tools find` lists the tools a repository uses and
//! which are in the box; `seikika tools config --for mise|aqua|pypi` and
//! `seikika tools install --dry-run` derive each installer's input from the one pin.

use std::path::Path;
use std::process::Command;

fn seikika(args: &[&str]) -> (bool, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_seikika"))
        .args(args)
        .env("PATH", "/nonexistent") // no installer and no tool is found: the output is the same everywhere
        .output()
        .unwrap();
    (
        o.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        ),
    )
}

fn write(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, body).unwrap();
}

const PORTAL: &str = r#"
apiVersion: seikika.com/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, seikika: "0.1"}
connectors:
  - name: lint
    package: builtin:runner
    runtime: in-image
    config: {name: lint, jobs: [{name: ruff, command: [ruff, check, --output-format, sarif, "{files}"], parse: sarif, files: ["*.py"]}]}
    records: [ruff]
kinds:
  - name: File
    identityFrom: [connector:lint]
    fields: {title: {type: string}}
    sources: {default: [connector:lint]}
    map: [{from: lint/ruff, id: "'file:' + record.path", set: "{'title': record.path}"}]
tools:
  ruff: {version: 0.14.0, install: {pypi: ruff, mise: "pipx:ruff", aqua: astral-sh/ruff}}
  actionlint: {version: 1.7.12, install: {mise: "aqua:rhysd/actionlint", aqua: rhysd/actionlint}, versionArgs: [-version]}
"#;

fn repo(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("seikika-tools-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    write(&root, ".seikika/seikika.yaml", PORTAL);
    write(&root, "ruff.toml", "line-length = 100\n");
    write(
        &root,
        ".github/actionlint.yaml",
        "self-hosted-runner: {labels: []}\n",
    );
    write(
        &root,
        "package.json",
        r#"{"devDependencies": {"eslint": "^9.0.0"}}"#,
    );
    write(
        &root,
        ".github/workflows/ci.yml",
        "jobs:\n  lint:\n    steps:\n      - run: npx eslint .\n      - run: actionlint\n      - run: bash scripts/check-names.sh\n",
    );
    write(&root, "scripts/check-names.sh", "#!/bin/sh\n");
    root
}

#[test]
fn find_lists_tools_in_and_out_of_the_box() {
    let root = repo("find");
    let (ok, out) = seikika(&["tools", "find", root.to_str().unwrap(), "--format", "json"]);
    assert!(ok, "{out}");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let status = |tool: &str| {
        v["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["tool"] == tool)
            .map(|t| t["status"].as_str().unwrap().to_owned())
    };
    // ruff: a connector runs it. actionlint: pinned, nothing runs it. eslint: used, not in the box.
    assert_eq!(status("ruff").as_deref(), Some("in box"));
    assert_eq!(status("actionlint").as_deref(), Some("pinned"));
    assert_eq!(status("eslint").as_deref(), Some("candidate"));
    assert_eq!(status("semgrep"), None);
    assert_eq!(v["scripts"], serde_json::json!(["scripts/check-names.sh"]));
    let (_, text) = seikika(&["tools", "find", root.to_str().unwrap()]);
    assert!(
        text.contains("[unchecked: run it and read the SARIF]"),
        "{text}"
    );
    assert!(text.contains("evidence, not a verdict"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn config_and_install_take_the_one_pin() {
    let root = repo("pin");
    let portal = root.join(".seikika");
    let p = portal.to_str().unwrap();
    let (ok, mise) = seikika(&["tools", "config", p, "--for", "mise"]);
    assert!(ok, "{mise}");
    assert!(
        mise.contains(r#""aqua:rhysd/actionlint" = "1.7.12""#),
        "{mise}"
    );
    assert!(mise.contains(r#""pipx:ruff" = "0.14.0""#), "{mise}");
    let (_, pypi) = seikika(&["tools", "config", p, "--for", "pypi"]);
    assert!(pypi.contains("ruff==0.14.0"), "{pypi}");
    assert!(
        pypi.contains("note: actionlint declares no pypi id"),
        "{pypi}"
    );
    let (_, aqua) = seikika(&["tools", "config", p, "--for", "aqua"]);
    assert!(aqua.contains("- name: rhysd/actionlint@v1.7.12"), "{aqua}");
    // Nothing is on the PATH: no installer is available, so every tool is skipped and it fails.
    let (ok, dry) = seikika(&["tools", "install", p, "--dry-run"]);
    assert!(!ok);
    assert!(
        dry.contains("SKIP    ruff 0.14.0: no installer available"),
        "{dry}"
    );
    let (_, via) = seikika(&["tools", "install", p, "--via", "pip", "--dry-run"]);
    assert!(
        via.contains("install ruff 0.14.0 via pip: python3 -m pip install ruff==0.14.0"),
        "{via}"
    );
    assert!(
        via.contains("SKIP    actionlint 1.7.12: declares no pip id"),
        "{via}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
