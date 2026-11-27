//! M1 exit criteria (docs/v1-spec.md §3): the Rust pipeline reproduces the reference model's
//! outputs byte for byte, every hand-written expectation passes, and plan diffs report changes.

use seikika_derive::{DerivedState, FullRecompute, plan};
use seikika_model::Manifest;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn portal() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference-portal")
}

fn manifest(doc: serde_json::Value) -> Manifest {
    let (m, errors) = seikika_model::validate_manifest(&doc, "seikika.yaml");
    assert!(errors.is_empty(), "{errors:#?}");
    m.unwrap()
}

fn reference() -> (Manifest, seikika_store::InputSnapshot) {
    let src = seikika_model::load::load(&portal()).unwrap();
    let store = seikika_store::load_fixtures(&portal()).unwrap();
    (manifest(src.document), store.snapshot())
}

#[test]
fn outputs_are_byte_identical_to_the_reference_model() {
    let (m, snap) = reference();
    let d = FullRecompute.recompute(&m, &snap);
    assert!(d.diagnostics.is_empty(), "{:#?}", d.diagnostics);
    let mut files = d.reference_files();
    files.sort_unstable_by_key(|(rel, _)| *rel);
    assert_eq!(
        files.iter().map(|(rel, _)| *rel).collect::<Vec<_>>(),
        [
            "authz/tuples.expected.json",
            "expected/entities.json",
            "expected/options.json",
            "expected/orphans.json",
            "expected/scorecards.json",
        ]
    );
    for (rel, text) in files {
        let expected = std::fs::read_to_string(portal().join(rel)).unwrap();
        assert!(
            text == expected,
            "{rel} differs from the reference model's output"
        );
    }
}

#[test]
fn hand_written_expectations_pass() {
    let (m, snap) = reference();
    let d = Arc::new(FullRecompute.recompute(&m, &snap));
    let results = seikika_derive::testing::run(&portal(), &m.tests, d).unwrap();
    assert_eq!(results.len(), 20);
    let failed: Vec<_> = results.iter().filter(|r| !r.passed).collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn output_does_not_depend_on_arrival_order() {
    let (m, _) = reference();
    let (batches, native) = seikika_store::read_fixtures(&portal()).unwrap();
    let mut forward = seikika_store::MemStore::new();
    let mut backward = seikika_store::MemStore::new();
    for (c, b) in &batches {
        forward.ingest(c, b);
    }
    for (c, b) in batches.iter().rev() {
        let mut rev = b.clone();
        rev.reverse();
        backward.ingest(c, &rev);
    }
    forward.put_native(native.clone());
    backward.put_native(native);
    let (a, b) = (
        FullRecompute.recompute(&m, &forward.snapshot()),
        FullRecompute.recompute(&m, &backward.snapshot()),
    );
    assert_eq!(a.reference_files(), b.reference_files());
    let no_changes = plan::diff(&m, &a, &m, &b);
    assert!(no_changes.is_empty(), "{no_changes:#?}");
}

#[test]
fn plan_lists_field_level_tuple_and_option_changes() {
    let (old_m, snap) = reference();
    let mut doc = serde_json::to_value(&old_m).unwrap();
    // Candidate: protect nothing about tier (drop the native source), approve sketchy-bot.
    doc["kinds"][2]["sources"]["fields"]["tier"] = serde_json::json!(["git:catalog-info"]);
    doc["policies"]["approvedApps"]["sketchy-bot"] = serde_json::json!({"steward": "group:platform", "reviewed": "2026-11-20",
        "permissions": {"contents": "write", "metadata": "read"}, "eligible": "true", "approval": "'none'"});
    let new_m = manifest(strip_nulls(doc));
    let (a, b) = (
        FullRecompute.recompute(&old_m, &snap),
        FullRecompute.recompute(&new_m, &snap),
    );
    let p = plan::diff(&old_m, &a, &new_m, &b);
    assert!(
        p.fields
            .iter()
            .any(|f| f.entity == "repository:platform-infra"
                && f.field == "tier"
                && f.after.is_none())
    );
    assert!(p.scores.iter().any(|s| s.scorecard == "ownership"
        && s.entity == "repository:platform-infra"
        && s.rules.contains_key("has-tier")));
    assert!(p.scores.iter().any(|s| s.scorecard == "github-app-hygiene"
        && s.entity == "repository:checkout-web"
        && s.level_after.as_deref() == Some("bronze")));
    assert!(
        p.options.iter().any(
            |o| o.entity == "repository:ledger" && o.after.contains(&"sketchy-bot".to_string())
        )
    );
    assert!(p.markdown().contains("Scorecard changes"));

    // Ownership change through the manifest: drop CODEOWNERS authority input → tuples lost.
    let mut doc = serde_json::to_value(&old_m).unwrap();
    doc["authz"]["tuples"][1]["emit"] = serde_json::json!("[]");
    let no_members = manifest(strip_nulls(doc));
    let c = FullRecompute.recompute(&no_members, &snap);
    let p = plan::diff(&old_m, &a, &no_members, &c);
    assert!(
        p.tuples_removed
            .iter()
            .any(|t| t["user"] == "user:alice" && t["object"] == "group:payments")
    );
}

/// Serialising the typed manifest writes `null` for absent options; the schema rejects those.
fn strip_nulls(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(m) => serde_json::Value::Object(
            m.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, strip_nulls(v)))
                .collect(),
        ),
        serde_json::Value::Array(a) => {
            serde_json::Value::Array(a.into_iter().map(strip_nulls).collect())
        }
        other => other,
    }
}
