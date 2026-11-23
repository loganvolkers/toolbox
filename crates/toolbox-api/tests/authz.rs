//! M3 exit criteria with an in-memory OpenFGA (tuples synced from each snapshot): A3/A4 facet
//! shaping, A10 action authorization, admin-only operator endpoints, and revocation within 60 s.
//! Needs the `openfga` binary (TOOLBOX_OPENFGA_BIN or PATH); skips otherwise.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use toolbox_store::SharedMem;

fn portal() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference-portal")
}

async fn start() -> Option<String> {
    if std::process::Command::new(toolbox_authz::openfga::binary())
        .arg("version")
        .output()
        .is_err()
    {
        // CI sets TOOLBOX_REQUIRE_OPENFGA, so a missing binary fails there instead of passing.
        assert!(
            std::env::var_os("TOOLBOX_REQUIRE_OPENFGA").is_none(),
            "openfga binary not found and TOOLBOX_REQUIRE_OPENFGA is set"
        );
        eprintln!("skipping: openfga binary not found");
        return None;
    }
    let src = toolbox_model::load::load(&portal()).unwrap();
    let (m, _) = toolbox_model::validate_manifest(&src.document, "toolbox.yaml");
    let m = m.unwrap();
    let dsl = std::fs::read_to_string(portal().join("authz/model.fga")).unwrap();
    let fga = toolbox_authz::OpenFga::start(
        toolbox_authz::Datastore::Memory,
        "toolbox-acme",
        toolbox_authz::fga_dsl::to_json(&dsl).unwrap(),
    )
    .await
    .unwrap();
    let store = Arc::new(SharedMem(Mutex::new(
        toolbox_store::load_fixtures(&portal()).unwrap(),
    )));
    let config = toolbox_api::Config {
        debounce: Duration::from_millis(50),
        ..Default::default()
    };
    let authn = toolbox_authz::authn::Authenticator::new(
        toolbox_authz::authn::AuthConfig::from_env(toolbox_authz::authn::Mode::Dev),
        None,
    );
    let state = toolbox_api::app_state_with(
        m,
        &src.document,
        store,
        None,
        config,
        authn,
        Some(Arc::new(fga)),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(toolbox_api::serve(state, listener));
    let c = reqwest::Client::new();
    for _ in 0..100 {
        if c.get(format!("{url}/readyz"))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
        {
            return Some(url);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("server not ready");
}

async fn get(url: &str, user: &str) -> (u16, Value) {
    let r = reqwest::Client::new()
        .get(url)
        .header("x-toolbox-user", user)
        .send()
        .await
        .unwrap();
    (r.status().as_u16(), r.json().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn facets_actions_and_revocation() {
    let Some(url) = start().await else { return };

    // A3: erin (no team) sees levels, not rule details, with a marker naming who can.
    let (_, r) = get(
        &format!("{url}/v1/entities/repository:checkout-web/scorecards"),
        "erin",
    )
    .await;
    assert_eq!(r["scorecards"]["ownership"]["level"], "bronze");
    assert_eq!(r["scorecards"]["github-app-hygiene"]["level"], "none");
    assert_eq!(r["scorecards"]["ownership"]["rules"]["redacted"], true);
    assert_eq!(
        r["scorecards"]["ownership"]["rules"]["requires"],
        "can_view_details"
    );
    assert_eq!(
        r["scorecards"]["ownership"]["rules"]["visible_to"],
        "members of group:checkout and portal admins"
    );

    // A4: bob (checkout) sees the failing rules and their fixes.
    let (_, r) = get(
        &format!("{url}/v1/entities/repository:checkout-web/scorecards"),
        "bob",
    )
    .await;
    assert_eq!(
        r["scorecards"]["ownership"]["rules"]["branch-protected"]["outcome"],
        "fail"
    );
    assert!(
        r["scorecards"]["ownership"]["rules"]["branch-protected"]["fix"]
            .as_str()
            .unwrap()
            .contains("ruleset")
    );
    assert_eq!(
        r["scorecards"]["github-app-hygiene"]["rules"]["only-approved-apps"]["outcome"],
        "fail"
    );

    // Contact facet (User.email) is owner-only: hidden from erin for other users, but admins see it.
    let (_, r) = get(&format!("{url}/v1/entities/user:alice"), "erin").await;
    assert_eq!(r["fields"]["email"]["redacted"], true);
    assert_eq!(r["fields"]["displayName"], "Alice Ng");
    let (_, r) = get(&format!("{url}/v1/entities/user:alice"), "dave").await;
    assert_eq!(r["fields"]["email"], "alice@acme.example");

    // Unverified ownership confers nothing: alice can't see ledger's details; dave (admin) can.
    let (_, r) = get(
        &format!("{url}/v1/entities/repository:ledger/scorecards"),
        "alice",
    )
    .await;
    assert_eq!(r["scorecards"]["ownership"]["rules"]["redacted"], true);
    assert_eq!(
        r["scorecards"]["ownership"]["rules"]["visible_to"],
        "portal admins"
    );
    let (_, r) = get(
        &format!("{url}/v1/entities/repository:ledger/scorecards"),
        "dave",
    )
    .await;
    assert_eq!(
        r["scorecards"]["ownership"]["rules"]["has-owner"]["outcome"],
        "fail"
    );

    // A10: erin can't preview install-approved-app; the 403 names the missing relation.
    let c = reqwest::Client::new();
    let body = json!({"entity": "repository:checkout-web", "inputs": {"app": "snyk-security", "justification": "Need scanning before the audit"}});
    let r = c
        .post(format!("{url}/v1/actions/install-approved-app/previews"))
        .header("x-toolbox-user", "erin")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
    assert_eq!(
        r.headers()["x-toolbox-missing-relation"],
        "can_install_apps"
    );
    assert!(r.text().await.unwrap().contains("can_install_apps"));
    // bob administers checkout-web in GitHub: the Toolbox preview passes validation.
    let r = c
        .post(format!("{url}/v1/actions/install-approved-app/previews"))
        .header("x-toolbox-user", "bob")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let p: Value = r.json().await.unwrap();
    assert_eq!(p["approval"]["required"], false);
    assert_eq!(p["related"]["installation"][0]["id"], "installation:4413");
    // An option not offered for this repository is rejected.
    let bad = json!({"entity": "repository:checkout-web", "inputs": {"app": "renovate", "justification": "Already there, but try"}});
    let r = c
        .post(format!("{url}/v1/actions/install-approved-app/previews"))
        .header("x-toolbox-user", "bob")
        .json(&bad)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    let (_, actions) = get(
        &format!("{url}/v1/actions?entity=repository:checkout-web"),
        "erin",
    )
    .await;
    assert_eq!(actions["actions"][0]["allowed"], false);

    // Operator export is admin-only.
    assert_eq!(get(&format!("{url}/v1/ops/derived"), "erin").await.0, 403);
    assert_eq!(get(&format!("{url}/v1/ops/derived"), "dave").await.0, 200);

    // Revocation: remove alice from payments through the IdP feed; details disappear within 60 s.
    let (_, r) = get(
        &format!("{url}/v1/entities/repository:payments-api/scorecards"),
        "alice",
    )
    .await;
    assert_eq!(
        r["scorecards"]["ownership"]["rules"]["claim-verified"]["outcome"],
        "pass"
    );
    let group = json!({"type": "record", "kind": "group", "externalId": "idp:group/payments",
        "data": {"slug": "payments", "displayName": "Payments", "parent": "engineering", "githubTeam": "acme/payments", "members": []}});
    let started = Instant::now();
    let r = c
        .post(format!("{url}/v1/ingest"))
        .header("x-toolbox-connector", "idp")
        .json(&vec![group])
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 202);
    loop {
        let (_, r) = get(
            &format!("{url}/v1/entities/repository:payments-api/scorecards"),
            "alice",
        )
        .await;
        if r["scorecards"]["ownership"]["rules"]["redacted"] == true {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(60),
            "revocation took longer than 60 s"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    eprintln!("revocation enforced after {:?}", started.elapsed());
}
