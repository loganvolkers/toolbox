//! Direct library dependencies of a Cargo workspace, read statically
//! (docs/design/dependency-standards.md).
//!
//! One record per external crate named by a workspace member's normal dependency. Versions and
//! sources come from `Cargo.lock`. The version requirement and who uses it come from
//! `cargo metadata --no-deps`. Dev-dependencies are not included. Thresholds live in the manifest.

use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn records(root: &Path, meta: &Value) -> Vec<Value> {
    let members: BTreeSet<&str> = meta["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["name"].as_str())
        .collect();
    let mut used: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut reqs: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in meta["packages"].as_array().into_iter().flatten() {
        let crate_name = package["name"].as_str().unwrap_or_default();
        for dep in package["dependencies"].as_array().into_iter().flatten() {
            if !dep["kind"].is_null() {
                continue;
            }
            let Some(name) = dep["name"].as_str() else {
                continue;
            };
            if members.contains(name) {
                continue;
            }
            used.entry(name.to_owned())
                .or_default()
                .insert(crate_name.to_owned());
            reqs.entry(name.to_owned())
                .or_default()
                .insert(dep["req"].as_str().unwrap_or("").to_owned());
        }
    }
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).unwrap_or_default();
    let locked = lock_packages(&lock);
    let inherited =
        workspace_deps(&std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default());
    used.into_iter()
        .map(|(name, users)| {
            let requirements: Vec<String> =
                reqs.get(&name).into_iter().flatten().cloned().collect();
            let (versions, sources) = locked.get(&name).cloned().unwrap_or_default();
            json!({
                "name": name,
                "ecosystem": "cargo",
                "requirements": requirements,
                "usedBy": users.into_iter().collect::<Vec<_>>(),
                "versions": versions,
                "sources": sources,
                "git": sources.iter().any(|s| s.starts_with("git+")),
                "wildcard": requirements.iter().any(|r| unbounded(r)),
                "inherited": inherited.contains(&name),
            })
        })
        .collect()
}

fn unbounded(req: &str) -> bool {
    let req = req.trim();
    req.is_empty() || req == "*" || req.starts_with('>') || req.starts_with('*')
}

fn workspace_deps(toml: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    let mut inside = false;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == "[workspace.dependencies]";
            continue;
        }
        if inside
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && let Some((key, _)) = trimmed.split_once('=')
        {
            keys.insert(key.trim().to_owned());
        }
    }
    keys
}

fn lock_packages(text: &str) -> BTreeMap<String, (Vec<String>, Vec<String>)> {
    let mut versions: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut sources: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for block in text.split("[[package]]").skip(1) {
        let Some(name) = field(block, "name") else {
            continue;
        };
        if let Some(version) = field(block, "version") {
            versions.entry(name.clone()).or_default().insert(version);
        }
        if let Some(source) = field(block, "source") {
            sources.entry(name).or_default().insert(source);
        }
    }
    versions
        .into_iter()
        .map(|(name, vers)| {
            let src = sources.remove(&name).unwrap_or_default();
            (
                name,
                (vers.into_iter().collect(), src.into_iter().collect()),
            )
        })
        .collect()
}

fn field(block: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    for line in block.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(&prefix) else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        return Some(rest[..end].to_owned());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn meta() -> Value {
        json!({
            "packages": [
                {
                    "name": "app",
                    "dependencies": [
                        {"name": "serde", "req": "^1", "kind": null},
                        {"name": "leftpad", "req": "*", "kind": null},
                        {"name": "app-core", "req": "*", "kind": null},
                        {"name": "tempfile", "req": "^3", "kind": "dev"}
                    ]
                },
                {
                    "name": "app-core",
                    "dependencies": [
                        {"name": "serde", "req": "^1", "kind": null}
                    ]
                }
            ]
        })
    }

    #[test]
    fn direct_deps_carry_the_lock_and_skip_workspace_members() {
        let root = std::env::temp_dir().join(format!("seikika-deps-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace.dependencies]\nserde = \"1\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("Cargo.lock"),
            r#"
[[package]]
name = "serde"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"

[[package]]
name = "leftpad"
version = "0.1.0"
source = "git+https://example.invalid/leftpad"

[[package]]
name = "tempfile"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#,
        )
        .unwrap();
        let recs = records(&root, &meta());
        let by: BTreeMap<_, _> = recs
            .iter()
            .map(|r| (r["name"].as_str().unwrap().to_owned(), r.clone()))
            .collect();
        assert!(
            !by.contains_key("app-core"),
            "workspace members are not deps"
        );
        assert!(
            !by.contains_key("tempfile"),
            "dev-dependencies are not deps"
        );
        assert_eq!(by["serde"]["inherited"], true);
        assert_eq!(by["serde"]["versions"], json!(["1.0.1"]));
        assert_eq!(by["serde"]["git"], false);
        assert_eq!(by["serde"]["wildcard"], false);
        assert_eq!(by["serde"]["usedBy"], json!(["app", "app-core"]));
        assert_eq!(by["leftpad"]["git"], true);
        assert_eq!(by["leftpad"]["wildcard"], true);
        assert_eq!(by["leftpad"]["inherited"], false);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn the_workspace_lock_resolves_every_direct_dep() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let out = std::process::Command::new("cargo")
            .args(["metadata", "--no-deps", "--format-version", "1"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(out.status.success());
        let meta: Value = serde_json::from_slice(&out.stdout).unwrap();
        let recs = records(&root, &meta);
        assert!(recs.len() >= 15, "{}", recs.len());
        // sqlx 0.9 collapsed duplicate sha2 lines; syn 2+3 remain after the connector's syn 3 bump.
        let syn = recs.iter().find(|r| r["name"] == "syn").unwrap();
        assert!(syn["versions"].as_array().unwrap().len() > 1);
        let regex = recs.iter().find(|r| r["name"] == "regex").unwrap();
        assert_eq!(regex["inherited"], true);
        assert!(regex["usedBy"].as_array().unwrap().len() > 1);
        assert!(recs.iter().all(|r| r["git"] == false));
    }
}
