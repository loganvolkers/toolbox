//! Exercise `seikika init` and load its output with the normal portal commands.

use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(seikika_exec::run::new_id("seikika-init"));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn command(&self, args: &[&str], input: &str) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_seikika"))
            .args(args)
            .current_dir(&self.0)
            .env("SEIKIKA_CACHE_DIR", self.0.join("cache"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn ok(&self, args: &[&str], input: &str) -> String {
        let output = self.command(args, input);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn init_defaults_are_scriptable_and_refuse_to_replace_an_existing_portal() {
    let temp = Scratch::new();
    temp.ok(&["init", "--yes"], "");
    let path = temp.0.join(".seikika/seikika.yaml");
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("builtin:workspace"));
    assert!(contents.contains("links-resolve"));
    temp.ok(&["validate", ".seikika"], "");

    let output = temp.command(&["init", "--yes"], "");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
    assert_eq!(fs::read_to_string(path).unwrap(), contents);
}

#[test]
fn init_can_start_empty_and_cancel_without_writing() {
    let temp = Scratch::new();
    let output = temp.command(&["init", "canceled"], "portal\ngroup:platform\n");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("setup canceled"));
    assert!(!temp.0.join("canceled").exists());

    temp.ok(&["init", "blank"], "\n\nnone\nnone\n");
    temp.ok(&["validate", "blank"], "");
    let manifest = temp.0.join("blank/seikika.yaml");
    let alternate = temp.0.join("blank/seikika.yml");
    fs::rename(&manifest, &alternate).unwrap();
    let output = temp.command(&["init", "blank", "--yes"], "");
    assert_eq!(output.status.code(), Some(2));
    assert!(alternate.is_file());
    assert!(!manifest.exists());
}

#[test]
fn init_combines_connectors_and_packages_into_an_installable_portal() {
    let temp = Scratch::new();
    let answers = concat!(
        "demo\ngroup:platform\n",
        "unlisted\n2, gitlab, git-files, runner, scim, 2\n",
        "1,2,3,4,5,6,7,8,9\n",
        "[\"src/**/*.rs\"]\n-1\n120\n",       // file-size
        "acme\ndemo\n\n",                     // github-actions
        "[\"src/**/*.rs\"]\n[\nlegacy_api\n", // pattern-count
        "py-lint\nPythonFile\nruff\n0.14.0\n[\"ruff\",\"check\",\"--output-format\",\"sarif\"]\n",
        ".semgrep/rules\n\n",                          // semgrep
        "checkout\ngroup:platform\n[\"readiness\"]\n", // sre
        "acme\nacme/platform\nacme\n\n[\"checks\"]\n", // connector settings
    );
    let output = temp.ok(&["init", "new portal"], answers);
    assert!(output.contains("Choose from the options above"));
    assert!(output.contains("Enter a positive whole number"));
    assert!(output.contains("Enter a valid Rust regular expression"));
    assert!(output.contains("Adding Workspace for github-actions"));
    assert!(output.contains("seikika install -- 'new portal'"));

    let text = fs::read_to_string(temp.0.join("new portal/seikika.yaml")).unwrap();
    let manifest: Value = serde_norway::from_str(&text).unwrap();
    assert_eq!(manifest["portal"]["owner"], "group:platform");
    let connectors = manifest["connectors"].as_array().unwrap();
    assert_eq!(connectors.len(), 6);
    assert_eq!(connectors[0]["config"]["token"], "${env:GITHUB_TOKEN}");
    let workspace = connectors
        .iter()
        .find(|c| c["name"] == "workspace")
        .unwrap();
    assert_eq!(
        workspace["records"],
        json!([
            "doc",
            "workflow",
            "action-ref",
            "js-workspace",
            "expo-app",
            "storybook"
        ])
    );
    assert_eq!(manifest["include"][0]["with"]["limit"], 120);

    // Seed the standard package cache with the workspace's package files so install and validate
    // run normally without contacting GitHub. The wizard itself needs neither Git nor a cache.
    let packages = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages");
    for package in manifest["include"].as_array().unwrap() {
        let spec = seikika_model::package::parse_git(package["package"].as_str().unwrap()).unwrap();
        assert_eq!((&*spec.owner, &*spec.repo), ("seikika", "seikika"));
        assert!(matches!(
            seikika_model::package::git_ref(&spec.version),
            seikika_model::package::GitRef::Commit(_)
        ));
        let subdir = spec.subdir.unwrap();
        let source = packages.parent().unwrap().join(&subdir);
        let cached = temp
            .0
            .join("cache/packages/seikika/seikika")
            .join(spec.version)
            .join(subdir);
        fs::create_dir_all(cached.join("fragments")).unwrap();
        fs::copy(
            source.join("seikika-package.yaml"),
            cached.join("seikika-package.yaml"),
        )
        .unwrap();
        for entry in fs::read_dir(source.join("fragments")).unwrap() {
            let entry = entry.unwrap();
            fs::copy(
                entry.path(),
                cached.join("fragments").join(entry.file_name()),
            )
            .unwrap();
        }
    }
    temp.ok(&["install", "new portal"], "");
    temp.ok(&["install", "new portal", "--frozen"], "");
    temp.ok(&["validate", "new portal"], "");
}
