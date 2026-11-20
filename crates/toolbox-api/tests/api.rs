//! REST API against the reference portal with the in-memory store: ingest, read-your-writes,
//! CEL filters (acceptance A5), projections, pagination and problem responses.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use toolbox_store::SharedMem;

fn portal() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference-portal")
}

async fn start() -> String {
    let src = toolbox_model::load::load(&portal()).unwrap();
    let (m, errors) = toolbox_model::validate_manifest(&src.document, "toolbox.yaml");
    assert!(errors.is_empty());
    let store = Arc::new(SharedMem(Mutex::new(toolbox_store::MemStore::new())));
    let config = toolbox_api::Config {
        debounce: std::time::Duration::from_millis(20),
        ..Default::default()
    };
    let state = toolbox_api::app_state(m.unwrap(), &src.document, store, None, config);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(toolbox_api::serve(state, listener));
    url
}

#[tokio::test]
async fn ingest_then_read_with_filters() {
    let url = start().await;
    let mut h = reqwest::header::HeaderMap::new();
    h.insert("x-toolbox-user", "dave".parse().unwrap());
    let c = reqwest::Client::builder()
        .default_headers(h)
        .build()
        .unwrap();
    let (batches, native) = toolbox_store::read_fixtures(&portal()).unwrap();
    let mut last = 0;
    for (connector, batch) in &batches {
        let body = serde_json::to_string(batch).unwrap();
        let r = c
            .post(format!("{url}/v1/ingest"))
            .header("x-toolbox-connector", connector)
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 202);
        last = r.json::<Value>().await.unwrap()["input_version"]
            .as_u64()
            .unwrap();
    }
    // Native write needs the entity: read-your-writes first.
    let r = c
        .get(format!(
            "{url}/v1/entities/repository:platform-infra?min_snapshot={last}"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    assert!(r.headers().contains_key("x-toolbox-snapshot"));
    let n = &native[0];
    let r = c
        .put(format!("{url}/v1/entities/{}/native/{}", n.entity, n.field))
        .header("x-toolbox-user", "dave")
        .json(&serde_json::json!({"value": n.value, "reason": n.reason}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 202);
    let v = r.json::<Value>().await.unwrap()["input_version"]
        .as_u64()
        .unwrap();

    // A5: repositories whose ownership isn't verified.
    let filter = "entity.ownership.status != 'verified'";
    let r: Value = c
        .get(format!("{url}/v1/entities"))
        .query(&[
            ("kind", "Repository"),
            ("filter", filter),
            ("fields", "ownership"),
            ("min_snapshot", &v.to_string()),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let ids: Vec<&str> = r["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "repository:ledger",
            "repository:legacy-tool",
            "repository:platform-infra"
        ]
    );
    assert!(
        r["items"][0]["fields"].get("title").is_none(),
        "fields projection"
    );

    // Native value applied.
    let r: Value = c
        .get(format!(
            "{url}/v1/entities/repository:platform-infra?min_snapshot={v}"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(r["fields"]["tier"], "1");
    assert_eq!(r["provenance"]["tier"], "native");

    // Pagination.
    let page: Value = c
        .get(format!("{url}/v1/entities?kind=User&limit=2"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    let cursor = page["next_cursor"].as_str().unwrap().to_owned();
    let rest: Value = c
        .get(format!("{url}/v1/entities"))
        .query(&[("kind", "User"), ("cursor", cursor.as_str())])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rest["items"].as_array().unwrap().len(), 3);

    // Scorecards and problems.
    let r: Value = c
        .get(format!(
            "{url}/v1/entities/repository:checkout-web/scorecards"
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(r["scorecards"]["ownership"]["level"], "bronze");
    let r = c
        .get(format!("{url}/v1/entities?filter=entity.("))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 400);
    assert_eq!(r.headers()["content-type"], "application/problem+json");
    let r = c
        .put(format!("{url}/v1/entities/repository:ledger/native/title"))
        .json(&serde_json::json!({"value": "x"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 409, "title is not a native field");
    let r = c
        .post(format!("{url}/v1/ingest"))
        .header("x-toolbox-connector", "github")
        .body(r#"[{"type":"record","kind":"pull-request","externalId":"x:1","data":{}}]"#)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422, "undeclared record kind");
    let r = c
        .post(format!("{url}/v1/ingest"))
        .body("[]")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 401);
}
