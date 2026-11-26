//! M3 exit criteria with an in-memory OpenFGA (tuples synced from each snapshot): A3/A4 facet
//! shaping, A10 action authorization, admin-only operator endpoints, and revocation within 60 s.
//! Needs the `openfga` binary (SEIKIKA_OPENFGA_BIN or PATH); skips otherwise.

use seikika_store::SharedMem;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn portal() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference-portal")
}

async fn start() -> Option<String> {
    if std::process::Command::new(seikika_authz::openfga::binary())
        .arg("version")
        .output()
        .is_err()
    {
        // CI sets SEIKIKA_REQUIRE_OPENFGA, so a missing binary fails there instead of passing.
        assert!(
            std::env::var_os("SEIKIKA_REQUIRE_OPENFGA").is_none(),
            "openfga binary not found and SEIKIKA_REQUIRE_OPENFGA is set"
        );
        eprintln!("skipping: openfga binary not found");
        return None;
    }
    let src = seikika_model::load::load(&portal()).unwrap();
    let (m, _) = seikika_model::validate_manifest(&src.document, "seikika.yaml");
    let m = m.unwrap();
    let dsl = std::fs::read_to_string(portal().join("authz/model.fga")).unwrap();
    let fga = seikika_authz::OpenFga::start(
        seikika_authz::Datastore::Memory,
        "seikika-acme",
        seikika_authz::fga_dsl::to_json(&dsl).unwrap(),
    )
    .await
    .unwrap();
    let store = Arc::new(SharedMem(Mutex::new(
        seikika_store::load_fixtures(&portal()).unwrap(),
    )));
    let config = seikika_api::Config {
        debounce: Duration::from_millis(50),
        ..Default::default()
    };
    let authn = seikika_authz::authn::Authenticator::new(
        seikika_authz::authn::AuthConfig::from_env(seikika_authz::authn::Mode::Dev),
        None,
    );
    let state = seikika_api::app_state_with(
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
    tokio::spawn(seikika_api::serve(state, listener));
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
        .header("x-seikika-user", user)
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
        .header("x-seikika-user", "erin")
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 403);
    assert_eq!(
        r.headers()["x-seikika-missing-relation"],
        "can_install_apps"
    );
    assert!(r.text().await.unwrap().contains("can_install_apps"));
    // bob administers checkout-web in GitHub: the Seikika preview passes validation.
    let r = c
        .post(format!("{url}/v1/actions/install-approved-app/previews"))
        .header("x-seikika-user", "bob")
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
        .header("x-seikika-user", "bob")
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
        .header("x-seikika-connector", "idp")
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

#[tokio::test]
async fn a_filter_cannot_probe_a_hidden_facet() {
    let Some(url) = start().await else { return };
    let list = |user: &'static str| {
        let url = url.clone();
        async move {
            let r = reqwest::Client::new()
                .get(format!("{url}/v1/entities"))
                .query(&[
                    ("kind", "User"),
                    ("filter", "entity.email == 'alice@acme.example'"),
                ])
                .header("x-seikika-user", user)
                .send()
                .await
                .unwrap();
            let v: Value = r.json().await.unwrap();
            v["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|i| i["id"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        }
    };
    // User.email is owner-only: erin can't see alice's, so her filter on it matches nothing.
    assert!(list("erin").await.is_empty());
    // A portal admin sees the facet, so the same filter finds alice.
    assert_eq!(list("dave").await, vec!["user:alice".to_owned()]);
}

#[tokio::test]
async fn a_decision_never_outlives_its_snapshot() {
    let Some(url) = start().await else { return };
    let c = reqwest::Client::new();
    let members = |m: &[&str]| {
        json!([{"type": "record", "kind": "group", "externalId": "idp:group/payments",
            "data": {"slug": "payments", "displayName": "Payments", "parent": "engineering",
                     "githubTeam": "acme/payments", "members": m}}])
    };
    let details = |version: u64| {
        let url = url.clone();
        async move {
            let (_, r) = get(
                &format!(
                    "{url}/v1/entities/repository:payments-api/scorecards?min_snapshot={version}"
                ),
                "alice",
            )
            .await;
            r["scorecards"]["ownership"]["rules"]["redacted"] != true
        }
    };
    let ingest = |body: Value| {
        let (c, url) = (c.clone(), url.clone());
        async move {
            let r = c
                .post(format!("{url}/v1/ingest"))
                .header("x-seikika-connector", "idp")
                .json(&body)
                .send()
                .await
                .unwrap();
            r.json::<Value>().await.unwrap()["input_version"]
                .as_u64()
                .unwrap()
        }
    };
    // Read twice so the second answer can come from a kept decision.
    assert!(details(0).await && details(0).await);
    // The first read at the next snapshot already reflects each change: no TTL to wait out.
    let v = ingest(members(&[])).await;
    assert!(!details(v).await, "revoked membership still allowed");
    let v = ingest(members(&["alice"])).await;
    assert!(details(v).await, "restored membership still denied");
}
