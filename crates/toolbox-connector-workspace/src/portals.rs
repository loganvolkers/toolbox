//! Child portals (`portals` globs, usually `**/.toolbox/toolbox.yaml`), read statically for a
//! parent portal that federates them (docs/design/federated-actions.md §5). One `portal` record
//! per child manifest: its identity, the actions it declares (with their own `include` files), how
//! a parent reaches it (`federation.runner`) and its scorecards. Actions travel up as data; the
//! parent dispatches runs back to the child, which previews, runs and verifies them itself.
//!
//! No other tool reads a Toolbox manifest, so this is the one place the fact can come from.
//! The manifest is loaded as the child loads it, packages included, so an action an org package
//! ships to its repos is offered by each repo that includes it. A package that is not installed
//! here (`toolbox install`) leaves `packageError` set and the record lists the child's own actions.
//! A manifest that does not load for any other reason offers nothing.
//!
//! A child is identified by where it was found (`key`: its directory, without `/.toolbox`), not by
//! the name it gives itself, so one repo can't take over another's entity by copying its name.

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
        "land": a["executor"]["land"],
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
    let key = path
        .strip_suffix("/.toolbox")
        .or_else(|| path.strip_suffix(".toolbox"))
        .filter(|k| !k.is_empty())
        .unwrap_or(&path)
        .to_owned();
    let abs = dir
        .canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let broken = |e: String| {
        json!({"name": key, "key": key, "path": path, "dir": abs, "file": rel, "error": e,
            "actions": [], "scorecards": [], "packages": [], "runner": null})
    };
    let doc = match yaml(&file) {
        Ok(d) => d,
        Err(e) => return broken(e),
    };
    let (extra, packages) = included(dir, &doc);
    let (merged, package_error) = match toolbox_model::load::load(&file) {
        Ok(src) => {
            let (_, errors) = toolbox_model::validate::validate_source(&src);
            if let Some(e) = errors.first() {
                return broken(format!(
                    "the manifest is invalid ({} error(s); toolbox validate shows them all): {e}",
                    errors.len()
                ));
            }
            (vec![src.document], None)
        }
        Err(e) if packages.is_empty() => return broken(e.to_string()),
        Err(e) => (
            std::iter::once(doc.clone()).chain(extra).collect(),
            Some(e.to_string()),
        ),
    };
    let all = merged.iter();
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
        "name": doc["portal"]["name"].as_str().unwrap_or(&key),
        "key": key,
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
        "packageError": package_error,
        "error": null,
    })
}

/// Two child portals with one name are still two entities (each has its own `key`), but a person
/// choosing between them by name could pick the wrong one: each gets a warning naming the others.
pub(crate) fn flag_duplicates<K>(found: Vec<(K, Value)>) -> Vec<(K, Value)> {
    let mut files: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (_, r) in &found {
        if let (Some(n), Some(f)) = (r["name"].as_str(), r["file"].as_str()) {
            files.entry(n.to_owned()).or_default().push(f.to_owned());
        }
    }
    found
        .into_iter()
        .map(|(k, mut r)| {
            let name = r["name"].as_str().unwrap_or_default().to_owned();
            if let Some(fs) = files.get(&name).filter(|fs| fs.len() > 1) {
                r["warning"] = json!(format!(
                    "portal name {name} is declared by {}; they are told apart by key",
                    fs.join(" and ")
                ));
            }
            (k, r)
        })
        .collect()
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
        assert_eq!(r["key"], "svc");
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
        assert!(r["packageError"].as_str().is_some(), "{r}");
        assert_eq!(r["runner"]["type"], "local");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn package_actions_are_offered_by_the_repo_that_includes_them() {
        let root = std::env::temp_dir().join(format!("portals-pkg-{}", std::process::id()));
        let pkg = root.join("org-actions");
        std::fs::create_dir_all(pkg.join("fragments")).unwrap();
        std::fs::write(
            pkg.join("toolbox-package.yaml"),
            "apiVersion: toolbox.dev/package/v0\nname: org-actions\nfragments: [fragments/*.yaml]\n",
        )
        .unwrap();
        std::fs::write(
            pkg.join("fragments/fix.yaml"),
            "actions:\n  - {name: add-codeowners, kind: Repository, scope: day2, description: Add CODEOWNERS, inputs: {type: object}, permission: can_run_actions, executor: {type: change, run: z}, expect: {postcondition: has(entity.codeowners), within: 1h}}\n",
        )
        .unwrap();
        let dir = root.join("svc/.toolbox");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("toolbox.yaml"),
            concat!(
                "apiVersion: toolbox.dev/v1alpha1\n",
                "portal: {name: svc, owner: group:a, toolbox: \"0.1\"}\n",
                "include: [{package: ../../org-actions}]\n",
                "connectors:\n  - {name: workspace, package: builtin:workspace, runtime: in-image, config: {root: .}, records: [repository]}\n",
                "kinds:\n  - {name: Repository, identityFrom: [connector:workspace], fields: {codeowners: {type: string}}, sources: {default: [connector:workspace]}, map: [{from: workspace/repository, id: \"'repository:x'\", set: \"{}\"}]}\n",
            ),
        )
        .unwrap();
        let r = analyse(&root, "svc/.toolbox/toolbox.yaml");
        assert_eq!(r["packageError"], Value::Null, "{r}");
        assert_eq!(r["actions"][0]["name"], "add-codeowners");
        assert_eq!(r["actions"][0]["executor"], "change");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn a_broken_manifest_is_a_record_with_an_error() {
        let root = std::env::temp_dir().join(format!("portals-bad-{}", std::process::id()));
        std::fs::create_dir_all(root.join("x")).unwrap();
        std::fs::write(root.join("x/toolbox.yaml"), "portal: [").unwrap();
        let r = analyse(&root, "x/toolbox.yaml");
        assert!(r["error"].as_str().is_some());
        std::fs::write(
            root.join("x/toolbox.yaml"),
            "portal: {name: x, owner: group:a}\nactions:\n  - {name: Bad Name, kind: Repository, description: d, permission: p, executor: {type: command, run: x}, unverified: true}\n",
        )
        .unwrap();
        let r = analyse(&root, "x/toolbox.yaml");
        assert!(r["error"].as_str().is_some(), "{r}");
        assert_eq!(r["actions"], json!([]));
        assert_eq!(r["key"], "x");
        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn two_portals_with_one_name_are_told_apart_by_key() {
        let rec = |name: &str, file: &str| json!({"name": name, "file": file, "actions": [{"name": "release"}], "error": null});
        let out = flag_duplicates(vec![
            (1, rec("svc", "a/.toolbox/toolbox.yaml")),
            (2, rec("svc", "b/.toolbox/toolbox.yaml")),
            (3, rec("web", "c/.toolbox/toolbox.yaml")),
        ]);
        for (_, r) in &out[..2] {
            assert_eq!(r["actions"][0]["name"], "release");
            assert_eq!(r["error"], Value::Null);
            let e = r["warning"].as_str().unwrap();
            assert!(
                e.contains("a/.toolbox/toolbox.yaml and b/.toolbox/toolbox.yaml"),
                "{e}"
            );
        }
        assert_eq!(out[2].1["warning"], Value::Null);
        assert_eq!(out[2].1["actions"][0]["name"], "release");
    }
}
