//! The DSL converter matches `fga model transform`, and the reference portal's `.fga.yaml` passes
//! through the Authorizer (M3 exit criterion). Needs the `openfga` binary (TOOLBOX_OPENFGA_BIN or
//! PATH); skips with a message when it isn't available.

use std::path::Path;
use toolbox_authz::{fga_dsl, fgatest};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn dsl_matches_fga_cli_for_the_reference_model() {
    let dsl = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/reference-portal/authz/model.fga"),
    )
    .unwrap();
    let ours = fga_dsl::to_json(&dsl).unwrap();
    let cli: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture("reference-model.json")).unwrap())
            .unwrap();
    assert_eq!(ours, cli);
}

#[test]
fn dsl_matches_fga_cli_for_all_operators() {
    let ours = fga_dsl::to_json(&std::fs::read_to_string(fixture("operators-model.fga")).unwrap())
        .unwrap();
    let cli: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture("operators-model.json")).unwrap())
            .unwrap();
    assert_eq!(ours, cli);
}

#[test]
fn dsl_rejects_unsupported_features() {
    assert!(
        fga_dsl::to_json(
            "model\n  schema 1.1\ntype a\n  relations\n    define r: [user with cond]\n"
        )
        .is_err()
    );
    assert!(
        fga_dsl::to_json("model\n  schema 1.1\ntype a\n  relations\n    define r: x or y and z\n")
            .is_err()
    );
}

fn openfga_available() -> bool {
    std::process::Command::new(toolbox_authz::openfga::binary())
        .arg("version")
        .output()
        .is_ok()
}

#[tokio::test]
async fn reference_fga_tests_pass_through_the_authorizer() {
    if !openfga_available() {
        eprintln!("skipping: openfga binary not found (set TOOLBOX_OPENFGA_BIN)");
        return;
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/reference-portal/authz/model.fga.yaml");
    let r = fgatest::run_file(&path).await.unwrap();
    assert!(r.ok(), "{:#?}", r.failures);
    assert_eq!((r.tests, r.checks, r.list_objects), (3, 16, 3));
}
