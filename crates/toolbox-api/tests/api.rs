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
    start_at(&portal()).await
}

async fn start_at(root: &Path) -> String {
    let src = toolbox_model::load::load(root).unwrap();
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

    // A required equality narrows candidates through the field index; `|| false` stops the
    // narrowing, so both must page through the same entities.
    let pages = |filter: &'static str| {
        let (c, url) = (c.clone(), url.clone());
        async move {
            let (mut ids, mut cursor) = (Vec::new(), None::<String>);
            loop {
                let mut q = vec![("kind", "repository".to_owned()), ("filter", filter.into())];
                q.push(("limit", "1".into()));
                q.extend(cursor.take().map(|c| ("cursor", c)));
                let r: Value = c
                    .get(format!("{url}/v1/entities"))
                    .query(&q)
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                ids.extend(
                    r["items"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|e| e["id"].clone()),
                );
                match r["next_cursor"].as_str() {
                    Some(c) => cursor = Some(c.to_owned()),
                    None => return ids,
                }
            }
        }
    };
    let narrowed = pages("entity.lifecycle == 'production'").await;
    assert!(narrowed.len() >= 2, "{narrowed:?}");
    assert_eq!(
        narrowed,
        pages("entity.lifecycle == 'production' || false").await
    );
    assert!(pages("entity.lifecycle == 'none'").await.is_empty());

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

const EDGES: &str = r#"
apiVersion: toolbox.dev/v1alpha1
portal: {name: edges, title: Edges, owner: group:maintainers, toolbox: "0.1"}
connectors:
  - {name: things, package: builtin:git-files, runtime: remote, records: [thing, other]}
kinds:
  - name: Thing
    identityFrom: [connector:things]
    fields:
      id: {type: string}
      team: {type: string}
    sources: {default: [connector:things]}
    map:
      - {from: things/thing, id: "'thing:' + record.name", set: "{'id': 'alias-' + record.name, 'team': record.team}"}
  - name: THING
    identityFrom: [connector:things]
    fields:
      team: {type: string}
    sources: {default: [connector:things]}
    map:
      - {from: things/other, id: "'other:' + record.name", set: "{'team': record.team}"}
"#;

#[tokio::test]
async fn field_index_narrowing_respects_kind_case_and_the_entity_id() {
    let dir = std::env::temp_dir().join(format!("toolbox-api-edges-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("toolbox.yaml"), EDGES).unwrap();
    let url = start_at(&dir).await;
    let c = reqwest::Client::new();
    let record = |kind: &str, name: &str| {
        serde_json::json!({"type": "record", "kind": kind, "externalId": format!("{kind}:{name}"),
                           "data": {"name": name, "team": "a"}})
    };
    let batch = vec![
        record("thing", "x"),
        record("thing", "y"),
        record("other", "z"),
    ];
    let r: Value = c
        .post(format!("{url}/v1/ingest"))
        .header("x-toolbox-connector", "things")
        .json(&batch)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let v = r["input_version"].as_u64().unwrap().to_string();
    let ids = |kind: &'static str, filter: &'static str| {
        let (c, url, v) = (c.clone(), url.clone(), v.clone());
        async move {
            let r: Value = c
                .get(format!("{url}/v1/entities"))
                .header("x-toolbox-user", "dave")
                .query(&[("kind", kind), ("filter", filter), ("min_snapshot", &v)])
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            let items = r["items"].as_array().unwrap_or_else(|| panic!("{r}"));
            items
                .iter()
                .map(|e| e["id"].as_str().unwrap().to_owned())
                .collect::<Vec<_>>()
        }
    };
    assert_eq!(
        ids("thing", "entity.team == 'a'").await,
        ["other:z", "thing:x", "thing:y"],
        "`thing` names both kinds"
    );
    assert_eq!(
        ids("Thing", "entity.id == 'thing:x'").await,
        ["thing:x"],
        "a filter sees the entity id, not the field"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn a_dev_server_does_not_take_your_word_where_the_forge_says_who_you_are() {
    let dir = std::env::temp_dir().join(format!("toolbox-api-identity-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let manifest = EDGES.to_owned()
        + "forges:\n  - {url: \"https://gitlab.example\", type: gitlab, identity: true}\n";
    std::fs::write(dir.join("toolbox.yaml"), manifest).unwrap();
    let url = start_at(&dir).await;
    let c = reqwest::Client::new();
    for path in [
        "/v1/actions/any/runs",
        "/v1/runs/run_x/approvals",
        "/v1/runs/run_x/cancel",
    ] {
        let r = c
            .post(format!("{url}{path}"))
            .header("x-toolbox-user", "dave")
            .json(&serde_json::json!({"entity": "thing:x", "inputs": {}}))
            .send()
            .await
            .unwrap();
        assert_eq!(r.status(), 403, "{path}");
        let body: Value = r.json().await.unwrap();
        assert_eq!(body["code"], "identity-from-forge", "{path}: {body}");
    }
    let r = c
        .get(format!("{url}/v1/runs"))
        .header("x-toolbox-user", "dave")
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "reads stay open");
    let _ = std::fs::remove_dir_all(&dir);
}
