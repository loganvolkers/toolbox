//! M0 exit criteria (docs/v1-spec.md §3): the reference portal validates cleanly, and every
//! negative case in tests/cases.json is rejected with its expected diagnostic code.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn reference() -> PathBuf {
    repo().join("examples/reference-portal")
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn scratch_copy() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "seikika-m0-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    copy_dir(&reference(), &dir);
    dir
}

/// Set (or, with `null`, remove) the value at a JSON pointer, creating the final key if needed.
fn set_pointer(doc: &mut Value, pointer: &str, value: &Value) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let parent = doc
        .pointer_mut(parent)
        .unwrap_or_else(|| panic!("no parent for {pointer}"));
    match parent {
        Value::Object(map) => {
            if value.is_null() {
                assert!(map.remove(key).is_some(), "nothing to remove at {pointer}");
            } else {
                map.insert(key.to_string(), value.clone());
            }
        }
        Value::Array(items) => items[key.parse::<usize>().unwrap()] = value.clone(),
        _ => panic!("{pointer}: parent is not a container"),
    }
}

#[test]
fn reference_portal_is_valid() {
    let report = seikika_model::validate_portal(&reference()).unwrap();
    assert!(report.ok(), "unexpected errors: {:#?}", report.errors);
    assert_eq!(report.messages, 29, "fixture message count");
}

#[test]
fn negative_cases_are_rejected() {
    let cases: Value = serde_json::from_str(include_str!("cases.json")).unwrap();
    let mut failures = Vec::new();
    for case in cases["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let code = case["code"].as_str().unwrap();
        let dir = scratch_copy();
        if let Some(sets) = case.get("set").and_then(Value::as_object) {
            let path = dir.join("seikika.yaml");
            let mut doc: Value =
                serde_norway::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            for (pointer, value) in sets {
                set_pointer(&mut doc, pointer, value);
            }
            std::fs::write(&path, serde_norway::to_string(&doc).unwrap()).unwrap();
        }
        if let Some(fx) = case.get("fixture") {
            let path = dir.join(fx["file"].as_str().unwrap());
            let mut text = std::fs::read_to_string(&path).unwrap();
            text.push_str(&serde_json::to_string(&fx["line"]).unwrap());
            text.push('\n');
            std::fs::write(&path, text).unwrap();
        }
        let report = seikika_model::validate_portal(&dir).unwrap();
        if !report.errors.iter().any(|d| d.code == code) {
            failures.push(format!("{name}: expected {code}, got {:?}", report.errors));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn include_merges_policy_files() {
    let dir = scratch_copy();
    let path = dir.join("seikika.yaml");
    let mut doc: Value = serde_norway::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let policies = doc.as_object_mut().unwrap().remove("policies").unwrap();
    doc["include"] = serde_json::json!(["policies/*.yaml"]);
    std::fs::write(&path, serde_norway::to_string(&doc).unwrap()).unwrap();
    std::fs::create_dir_all(dir.join("policies")).unwrap();
    std::fs::write(
        dir.join("policies/approved-apps.yaml"),
        serde_norway::to_string(&serde_json::json!({ "policies": policies })).unwrap(),
    )
    .unwrap();
    let report = seikika_model::validate_portal(&dir).unwrap();
    assert!(report.ok(), "{:#?}", report.errors);
    assert_eq!(
        report.included,
        vec!["policies/approved-apps.yaml".to_string()]
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn errors_in_included_files_name_the_file() {
    let dir = scratch_copy();
    let path = dir.join("seikika.yaml");
    let mut doc: Value = serde_norway::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut cards = doc.as_object_mut().unwrap().remove("scorecards").unwrap();
    // Keep the first scorecard in the root file, so the included ones land at shifted indices.
    let first = cards.as_array_mut().unwrap().remove(0);
    doc["scorecards"] = serde_json::json!([first]);
    doc["include"] = serde_json::json!(["cards/*.yaml"]);
    cards[0]["rules"][0]["check"] = serde_json::json!("entity.x >");
    std::fs::write(&path, serde_norway::to_string(&doc).unwrap()).unwrap();
    std::fs::create_dir_all(dir.join("cards")).unwrap();
    std::fs::write(
        dir.join("cards/more.yaml"),
        serde_norway::to_string(&serde_json::json!({ "scorecards": cards })).unwrap(),
    )
    .unwrap();
    let report = seikika_model::validate_portal(&dir).unwrap();
    let e = report
        .errors
        .iter()
        .find(|d| d.code == "cel.syntax")
        .expect("a CEL error");
    assert_eq!(
        (e.file.as_str(), e.path.as_str()),
        ("cards/more.yaml", "/scorecards/0/rules/0/check")
    );
    let _ = std::fs::remove_dir_all(&dir);
}
