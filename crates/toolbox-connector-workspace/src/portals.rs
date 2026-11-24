//! Child portals (`portals` globs, usually `**/.toolbox/toolbox.yaml`), read statically for a
//! parent portal that federates them (docs/design/federated-actions.md §5). One `portal` record
//! per child manifest: its identity, the actions it declares (with their own `include` files), how
//! a parent reaches it (`federation.runner`) and its scorecards. Actions travel up as data; the
//! parent dispatches runs back to the child, which previews, runs and verifies them itself.
//!
//! No other tool reads a Toolbox manifest, so this is the one place the fact can come from.
//! Actions contributed by packages (`include: [{package}]`) are not resolved here: the record
//! lists the package specs, and the child's own `toolbox run` sees the full set.

use serde_json::{Value, json};
use std::path::Path;

fn yaml(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_norway::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The action as a parent needs it: what it does, what it takes, who may run and approve it, and
/// whether its completion is observed. Executor details stay in the child.
fn summary(a: &Value) -> Value {
    json!({
        "name": a["name"],
        "kind": a["kind"],
        "scope": a.get("scope").cloned().unwrap_or(json!("day2")),
        "description": a["description"],
        "inputs": a.get("inputs").cloned().unwrap_or(json!({"type": "object"})),
        "permission": a["permission"],
        "approval": a.get("approval").cloned().unwrap_or(Value::Null),
        "executor": a["executor"]["type"].as_str().unwrap_or("command"),
        "verified": a.get("expect").is_some_and(Value::is_object),
        "postcondition": a["expect"]["postcondition"],
        "annotations": a.get("annotations").cloned().unwrap_or(Value::Null),
    })
}

/// Included YAML files (string entries of `include`), matched under the manifest's directory.
fn included(dir: &Path, doc: &Value) -> (Vec<Value>, Vec<String>) {
    let mut docs = Vec::new();
    let mut packages = Vec::new();
    let files = toolbox_connect::files::walk(dir);
    for inc in doc["include"].as_array().into_iter().flatten() {
        if let Some(g) = inc.as_str() {
            let Ok(re) = toolbox_connect::files::glob(g) else {
                continue;
            };
            for rel in files.iter().filter(|p| re.is_match(p)) {
                if let Ok(v) = yaml(&dir.join(rel)) {
                    docs.push(v);
                }
            }
        } else if let Some(p) = inc["package"].as_str() {
            packages.push(p.to_owned());
        }
    }
    (docs, packages)
}

pub(crate) fn analyse(root: &Path, rel: &str) -> Value {
    let file = root.join(rel);
    let dir = file.parent().unwrap_or(root);
    let path = Path::new(rel)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| ".".into());
    let abs = dir
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let doc = match yaml(&file) {
        Ok(d) => d,
        Err(e) => {
            return json!({"name": path, "path": path, "dir": abs, "file": rel, "error": e,
                "actions": [], "scorecards": [], "packages": [], "runner": null});
        }
    };
    let (extra, packages) = included(dir, &doc);
    let all = std::iter::once(&doc).chain(extra.iter());
    let mut actions = Vec::new();
    let mut scorecards = Vec::new();
    for d in all {
        actions.extend(d["actions"].as_array().into_iter().flatten().map(summary));
        scorecards.extend(
            d["scorecards"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|s| s["name"].as_str().map(str::to_owned)),
        );
    }
    json!({
        "name": doc["portal"]["name"].as_str().unwrap_or(&path),
        "title": doc["portal"]["title"],
        "owner": doc["portal"]["owner"],
        "path": path,
        "dir": abs,
        "file": rel,
        "actions": actions,
        "runner": doc["federation"]["runner"],
        "acceptApprovalsFrom": doc["federation"]
            .get("acceptApprovalsFrom")
            .cloned()
            .unwrap_or(json!([])),
        "scorecards": scorecards,
        "packages": packages,
        "error": null,
    })
}

/// The record id: `portal:<manifest path>`.
pub(crate) fn id(rel: &str) -> String {
    format!("portal:{rel}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_actions_from_the_manifest_and_its_includes() {
        let root = std::env::temp_dir().join(format!("portals-test-{}", std::process::id()));
        let dir = root.join("svc/.toolbox");
        std::fs::create_dir_all(dir.join("actions")).unwrap();
        std::fs::write(
            dir.join("toolbox.yaml"),
            "portal: {name: svc, owner: group:a}\ninclude: [actions/*.yaml, {package: github:o/r@v1}]\nfederation: {runner: {type: local}}\nactions:\n  - {name: release, kind: Repository, description: Cut a release, permission: can_run_actions, executor: {type: command, run: x}, expect: {postcondition: 'true', within: 1m}}\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("actions/more.yaml"),
            "actions:\n  - {name: rotate, kind: Repository, description: Rotate, permission: can_run_actions, executor: {type: change, run: y}, unverified: true}\n",
        )
        .unwrap();
        let r = analyse(&root, "svc/.toolbox/toolbox.yaml");
        assert_eq!(r["name"], "svc");
        assert_eq!(r["path"], "svc/.toolbox");
        let names: Vec<&str> = r["actions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["release", "rotate"]);
        assert_eq!(r["actions"][0]["verified"], true);
        assert_eq!(r["actions"][1]["executor"], "change");
        assert_eq!(r["packages"], json!(["github:o/r@v1"]));
        assert_eq!(r["runner"]["type"], "local");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_broken_manifest_is_a_record_with_an_error() {
        let root = std::env::temp_dir().join(format!("portals-bad-{}", std::process::id()));
        std::fs::create_dir_all(root.join("x")).unwrap();
        std::fs::write(root.join("x/toolbox.yaml"), "portal: [").unwrap();
        let r = analyse(&root, "x/toolbox.yaml");
        assert!(r["error"].as_str().is_some());
        std::fs::remove_dir_all(&root).ok();
    }
}
