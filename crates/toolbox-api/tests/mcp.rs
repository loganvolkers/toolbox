//! MCP contract (v1-spec §7.3): every built-in tool called over `POST /mcp` with `tools/call`
//! against the reference portal fixtures (in-memory store, no Authorizer, so every caller may see
//! and do everything). Each call asserts on the `structuredContent` and the Markdown `content`.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use toolbox_store::SharedMem;

fn portal() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reference-portal")
}

async fn start() -> String {
    start_with(|_| {}).await
}

/// Start a server on the reference portal with its manifest document changed by `edit`.
async fn start_with(edit: impl FnOnce(&mut Value)) -> String {
    let mut src = toolbox_model::load::load(&portal()).unwrap();
    edit(&mut src.document);
    let (m, errors) = toolbox_model::validate_manifest(&src.document, "toolbox.yaml");
    assert!(errors.is_empty());
    let store = Arc::new(SharedMem(Mutex::new(
        toolbox_store::load_fixtures(&portal()).unwrap(),
    )));
    let config = toolbox_api::Config {
        debounce: Duration::from_millis(20),
        ..Default::default()
    };
    let state = toolbox_api::app_state(m.unwrap(), &src.document, store, None, config);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(toolbox_api::serve(state, listener));
    for _ in 0..100 {
        if reqwest::get(format!("{url}/readyz"))
            .await
            .is_ok_and(|r| r.status().is_success())
        {
            return url;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("server not ready");
}

/// One JSON-RPC request to `/mcp` as `dave`; checks the envelope and returns its `result`.
async fn rpc(url: &str, method: &str, params: Value) -> Value {
    let r: Value = reqwest::Client::new()
        .post(format!("{url}/mcp"))
        .header("x-toolbox-user", "dave")
        .json(&json!({"jsonrpc": "2.0", "id": 7, "method": method, "params": params}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!((&r["jsonrpc"], &r["id"]), (&json!("2.0"), &json!(7)), "{r}");
    r["result"].clone()
}

/// `tools/call` with `params` (`{"name": …, "arguments": …}`); returns (structuredContent,
/// Markdown text, isError).
async fn call(url: &str, params: Value) -> (Value, String, bool) {
    let r = rpc(url, "tools/call", params).await;
    assert_eq!(r["content"][0]["type"], "text", "{r}");
    let text = r["content"][0]["text"].as_str().unwrap().to_owned();
    (r["structuredContent"].clone(), text, r["isError"] == true)
}

async fn ok(url: &str, params: Value) -> (Value, String) {
    let (v, text, is_error) = call(url, params).await;
    assert!(!is_error, "{text}");
    (v, text)
}

#[tokio::test]
async fn tools_list_describes_every_input() {
    let url = start().await;
    let tools = rpc(&url, "tools/list", json!({})).await["tools"].clone();
    let names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "search",
            "get",
            "query",
            "graph",
            "owner_of",
            "scorecard_status",
            "scorecard_report",
            "assessment_brief",
            "explain",
            "refresh_entity",
            "list_actions",
            "action_options",
            "preview_action"
        ]
    );
    for t in tools.as_array().unwrap() {
        assert!(t["annotations"]["readOnlyHint"].is_boolean(), "{t}");
        for (prop, schema) in t["inputSchema"]["properties"].as_object().unwrap() {
            assert!(
                schema["description"].is_string(),
                "{}.{prop} has no description",
                t["name"]
            );
        }
    }
}

#[tokio::test]
async fn read_tools_over_tools_call() {
    let url = start().await;

    let (v, text) = ok(
        &url,
        json!({"name": "search", "arguments": {"query": "LED", "kind": "repository", "limit": 5}}),
    )
    .await;
    assert_eq!(v["items"].as_array().unwrap().len(), 1);
    assert_eq!(v["items"][0]["id"], "repository:ledger");
    assert!(
        text.starts_with("1 result(s):\n- `repository:ledger` ledger"),
        "{text}"
    );

    let (v, text) = ok(
        &url,
        json!({"name": "get", "arguments": {"id": "repository:checkout-web"}}),
    )
    .await;
    assert_eq!(v["kind"], "Repository");
    assert_eq!(v["fields"]["ownership"]["owner"], "group:checkout");
    assert_eq!(v["provenance"]["ownership"], "authority:codeowners");
    assert!(
        text.contains("- title: Checkout web app _(from git:catalog-info)_"),
        "{text}"
    );

    // A5 through MCP: repositories whose ownership isn't verified, projected to one field.
    let (v, text) = ok(
        &url,
        json!({"name": "query", "arguments": {"kind": "Repository",
            "filter": "entity.ownership.status != 'verified'", "fields": "ownership"}}),
    )
    .await;
    let ids: Vec<&str> = v["items"]
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
    assert!(v["items"][0]["fields"].get("title").is_none(), "projection");
    assert!(
        text.contains("- `repository:platform-infra`  — owner group:platform (inferred)"),
        "{text}"
    );

    let (v, text) = ok(
        &url,
        json!({"name": "graph", "arguments": {"id": "repository:checkout-web",
            "relation": "installedOn", "direction": "in", "depth": 1}}),
    )
    .await;
    assert_eq!(v["root"], "repository:checkout-web");
    assert_eq!(
        v["edges"],
        json!([
            {"from": "installation:4412", "relation": "installedOn", "to": "repository:checkout-web"},
            {"from": "installation:4414", "relation": "installedOn", "to": "repository:checkout-web"}
        ])
    );
    assert!(
        text.starts_with("- `installation:4412` —installedOn→ `repository:checkout-web`"),
        "{text}"
    );
    let (v, text) = ok(
        &url,
        json!({"name": "graph", "arguments": {"id": "repository:checkout-web", "direction": "out"}}),
    )
    .await;
    assert_eq!(
        v["edges"],
        json!([
            {"from": "repository:checkout-web", "relation": "ownedBy", "to": "group:checkout"},
            {"from": "repository:checkout-web", "relation": "ownerClaim", "to": "group:checkout"}
        ]),
        "installedOn edges point in, so only ownership goes out"
    );
    assert!(
        text.contains("- `repository:checkout-web` —ownedBy→ `group:checkout`"),
        "{text}"
    );
    let (v, text) = ok(
        &url,
        json!({"name": "graph", "arguments": {"id": "repository:checkout-web", "relation": "nope"}}),
    )
    .await;
    assert_eq!(v["edges"], json!([]));
    assert!(text.starts_with("No visible neighbours."), "{text}");

    let (v, text) = ok(
        &url,
        json!({"name": "owner_of", "arguments": {"id": "repository:ledger"}}),
    )
    .await;
    assert_eq!(v["field"], "ownership");
    assert_eq!(v["authority"]["status"], "unverified");
    assert!(
        text.starts_with("**repository:ledger**: no applied owner — ownership **unverified**."),
        "{text}"
    );
    assert!(
        text.contains("\nFix: "),
        "an unverified claim comes with a fix: {text}"
    );

    let (v, text) = ok(
        &url,
        json!({"name": "scorecard_status", "arguments": {"id": "repository:checkout-web", "scorecard": "ownership"}}),
    )
    .await;
    assert_eq!(v["scorecards"]["ownership"]["level"], "bronze");
    assert_eq!(
        v["scorecards"]["ownership"]["rules"]["branch-protected"]["outcome"],
        "fail"
    );
    assert!(
        text.starts_with("**ownership**: bronze\n- branch-protected: fail — fix: "),
        "{text}"
    );
    assert!(
        !text.contains("github-app-hygiene"),
        "only the requested scorecard: {text}"
    );
    assert_eq!(
        v["scorecards"].as_object().unwrap().len(),
        1,
        "the structured result is narrowed too: {v}"
    );

    let (v, text) = ok(
        &url,
        json!({"name": "explain", "arguments": {"id": "repository:checkout-web",
            "rule": "github-app-hygiene/only-approved-apps"}}),
    )
    .await;
    assert_eq!(v["outcome"], "fail");
    assert_eq!(
        v["observed"]["installedApps"],
        json!(["renovate", "sketchy-bot"])
    );
    assert!(
        text.contains("- observed installedApps: [\"renovate\",\"sketchy-bot\"]"),
        "{text}"
    );
    let (v, text) = ok(
        &url,
        json!({"name": "explain", "arguments": {"id": "repository:checkout-web", "field": "tier"}}),
    )
    .await;
    assert_eq!(
        (&v["value"], &v["source"]),
        (&json!("2"), &json!("git:catalog-info"))
    );
    assert!(
        text.starts_with("**tier** on `repository:checkout-web`: 2"),
        "{text}"
    );

    // Errors come back as tool results (isError) with the problem, not as JSON-RPC errors.
    let (v, text, is_error) = call(
        &url,
        json!({"name": "get", "arguments": {"id": "repository:nope"}}),
    )
    .await;
    assert!(is_error);
    assert_eq!(v["status"], 404);
    assert!(text.starts_with("**not-found** (404)"), "{text}");
}

#[tokio::test]
async fn action_tools_over_tools_call() {
    let url = start().await;

    let (v, text) = ok(
        &url,
        json!({"name": "list_actions", "arguments": {"id": "repository:checkout-web"}}),
    )
    .await;
    let names: Vec<&str> = v["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["install-approved-app", "remove-app"]);
    assert_eq!(v["actions"][0]["allowed"], true);
    assert!(
        text.contains("- `remove-app`: Remove this repository"),
        "{text}"
    );

    // expected/options.json: only snyk-security is approved and not yet installed.
    let (v, text) = ok(
        &url,
        json!({"name": "action_options", "arguments": {"action": "install-approved-app",
            "input": "app", "id": "repository:checkout-web"}}),
    )
    .await;
    assert_eq!(v["options"].as_array().unwrap().len(), 1);
    assert_eq!(v["options"][0]["value"], "snyk-security");
    assert!(text.starts_with("- snyk-security\n"), "{text}");

    let (v, text) = ok(
        &url,
        json!({"name": "preview_action", "arguments": {"action": "install-approved-app",
            "id": "repository:checkout-web",
            "inputs": {"app": "snyk-security", "justification": "dependency scanning for checkout"}}}),
    )
    .await;
    assert_eq!(v["blockers"], json!([]));
    assert_eq!(v["approval"]["required"], false);
    assert_eq!(v["related"]["installation"][0]["id"], "installation:4413");
    assert!(
        text.contains("- blockers: none\n- approval: none"),
        "{text}"
    );
    // An app outside the options fails input validation, reported as a tool error.
    let (v, text, is_error) = call(
        &url,
        json!({"name": "preview_action", "arguments": {"action": "install-approved-app",
            "id": "repository:checkout-web",
            "inputs": {"app": "sketchy-bot", "justification": "please let me install it"}}}),
    )
    .await;
    assert!(is_error);
    assert_eq!(v["status"], 422);
    assert!(
        text.starts_with("**invalid-inputs** (422): app=\"sketchy-bot\" is not an option"),
        "{text}"
    );
}

/// `refresh_entity` against a remote connector played by the test: the tool waits for the
/// command to complete and reports the snapshot that holds the fresh data.
#[tokio::test]
async fn refresh_entity_over_tools_call() {
    let url = start().await;
    let c = reqwest::Client::new();
    let ingest = |body: Value| {
        c.post(format!("{url}/v1/ingest"))
            .header("x-toolbox-connector", "idp")
            .json(&body)
            .send()
    };

    // Before idp registers, nothing can refresh the user.
    let (v, text, is_error) = call(
        &url,
        json!({"name": "refresh_entity", "arguments": {"id": "user:dave"}}),
    )
    .await;
    assert!(is_error);
    assert_eq!(
        (&v["status"], &v["code"]),
        (&json!(409), &json!("not-refreshable"))
    );
    assert!(text.starts_with("**not-refreshable** (409)"), "{text}");

    let r = ingest(
        json!([{"type": "register", "connector": "idp", "version": "test", "instance": "idp-test",
        "capabilities": {"refresh": {"user": ["external_ids"]}}}]),
    )
    .await
    .unwrap();
    assert_eq!(r.status(), 202);
    let connector = {
        let (c, url) = (c.clone(), url.clone());
        tokio::spawn(async move {
            let cmds: Value = c
                .get(format!("{url}/v1/connectors/idp/commands?wait=20"))
                .header("x-toolbox-connector", "idp")
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            let cmd = &cmds["commands"][0];
            assert_eq!(
                (&cmd["command"], &cmd["kind"]),
                (&json!("refresh"), &json!("user"))
            );
            assert_eq!(cmd["external_ids"], json!(["idp:user/dave"]));
            let id = &cmd["command_id"];
            let r = c
                .post(format!("{url}/v1/ingest"))
                .header("x-toolbox-connector", "idp")
                .json(&json!([
                    {"type": "ack", "command_id": id},
                    {"type": "record", "kind": "user", "externalId": "idp:user/dave", "command_id": id,
                     "data": {"login": "dave", "name": "Dave Kim-Park", "email": "dave@acme.example"}},
                    {"type": "complete", "command_id": id, "status": "succeeded", "counts": {"records": 1}}
                ]))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status(), 202);
        })
    };

    let (v, text) = ok(
        &url,
        json!({"name": "refresh_entity", "arguments": {"id": "user:dave"}}),
    )
    .await;
    connector.await.unwrap();
    assert_eq!(v["entity"], "user:dave");
    let done = &v["commands"][0];
    assert_eq!(
        (&done["status"], &done["connector"]),
        (&json!("succeeded"), &json!("idp"))
    );
    let snapshot = done["snapshot"]
        .as_str()
        .expect("the snapshot with the results");
    assert!(
        text.contains(&format!(
            "refresh user via idp: succeeded → snapshot {snapshot}"
        )),
        "{text}"
    );

    // What changed: the entity's fields, over the snapshot range the refresh covered.
    assert_eq!(v["changes"]["fields"], json!(["displayName"]), "{v}");
    assert_eq!(v["changes"]["to"], json!(snapshot));
    assert!(
        v["changes"]["entities"]
            .as_array()
            .unwrap()
            .contains(&json!("user:dave"))
    );
    assert!(
        text.starts_with("`user:dave`: changed displayName (snapshot "),
        "{text}"
    );

    // "Returns once the fresh data is visible."
    let (v, _) = ok(
        &url,
        json!({"name": "get", "arguments": {"id": "user:dave"}}),
    )
    .await;
    assert_eq!(v["fields"]["displayName"], "Dave Kim-Park");
}

#[tokio::test]
async fn scorecard_report_over_tools_call() {
    let url = start().await;
    // Without a scorecard: every scorecard with its level counts.
    let (v, text) = ok(&url, json!({"name": "scorecard_report", "arguments": {}})).await;
    let names: Vec<&str> = v["scorecards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["ownership", "github-app-hygiene", "app-approval"]);
    assert_eq!(
        v["scorecards"][0]["by_level"],
        json!({"bronze": 2, "none": 1, "silver": 1, "skipped": 1})
    );
    assert!(
        text.contains("- **ownership** (Repository; levels bronze < silver): bronze: 2"),
        "{text}"
    );

    // One rule's failures: a work list (the same entities as expected/scorecards.json).
    let (v, text) = ok(
        &url,
        json!({"name": "scorecard_report", "arguments": {"scorecard": "ownership", "rule": "has-owner"}}),
    )
    .await;
    let ids: Vec<&str> = v["entities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["repository:ledger"]);
    assert_eq!(v["entities"][0]["failing"]["has-owner"]["outcome"], "fail");
    assert!(v["entities"][0]["failing"]["has-owner"]["fix"].is_string());
    assert!(
        text.starts_with("**ownership**: 1 entities (none: 1)\n- `repository:ledger`: none\n"),
        "{text}"
    );
    assert!(text.contains("\n  - has-owner: fail\n"), "{text}");

    // By level, and passing outcomes.
    let (v, _) = ok(
        &url,
        json!({"name": "scorecard_report", "arguments": {"scorecard": "ownership", "level": "silver"}}),
    )
    .await;
    assert_eq!(v["entities"][0]["id"], "repository:payments-api");
    assert_eq!(v["entities"][0]["failing"], json!({}));
    let (v, _) = ok(
        &url,
        json!({"name": "scorecard_report", "arguments": {"scorecard": "ownership", "rule": "branch-protected", "outcome": "pass"}}),
    )
    .await;
    assert_eq!(v["count"], 3);

    // Unknown scorecard or rule: a tool error, not an empty list.
    let (_, text, is_error) = call(
        &url,
        json!({"name": "scorecard_report", "arguments": {"scorecard": "ownership", "rule": "nope"}}),
    )
    .await;
    assert!(
        is_error && text.contains("ownership has no rule nope"),
        "{text}"
    );
}

#[tokio::test]
async fn query_filters_on_scores_and_get_projects_fields() {
    let url = start().await;
    let (v, _) = ok(
        &url,
        json!({"name": "query", "arguments": {"kind": "Repository",
            "filter": "scores['ownership'].rules['branch-protected'] == 'fail'", "fields": "title"}}),
    )
    .await;
    let ids: Vec<&str> = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["repository:checkout-web"]);
    let (v, _) = ok(
        &url,
        json!({"name": "get", "arguments": {"id": "repository:checkout-web", "fields": "title,tier"}}),
    )
    .await;
    let fields: Vec<&String> = v["fields"].as_object().unwrap().keys().collect();
    assert_eq!(fields, ["tier", "title"]);
    assert_eq!(
        v["outbound"],
        json!({"ownedBy": ["group:checkout"], "ownerClaim": ["group:checkout"]}),
        "outgoing references next to inbound relations: {v}"
    );
}

#[tokio::test]
async fn unknown_tool_is_a_tool_error() {
    let url = start().await;
    let (_, text, is_error) = call(&url, json!({"name": "drop_tables", "arguments": {}})).await;
    assert!(is_error);
    assert_eq!(text, "Unknown tool `drop_tables`.");
}

#[tokio::test]
async fn assessment_brief_over_tools_call() {
    // The reference portal has no rubrics: add one, versioned by `title` (any string field).
    let url = start_with(|doc| {
        doc["rubrics"] = json!([{"name": "repo-review", "kind": "Repository", "subject": "title",
            "description": "Is this repository ready for production?",
            "criteria": [{"name": "owned", "question": "Does a real team own it?", "guidance": "Check CODEOWNERS."},
                         {"name": "documented", "question": "Can a newcomer run it from its README?"}]}]);
    })
    .await;
    let (v, text) = ok(
        &url,
        json!({"name": "assessment_brief", "arguments": {"id": "repository:checkout-web"}}),
    )
    .await;
    assert_eq!(v["rubric"]["name"], "repo-review");
    assert_eq!(v["subjectHash"], v["facts"]["title"]);
    assert_eq!(
        v["file"],
        "assessments/repo-review/repository/checkout-web.yaml"
    );
    assert!(
        v["template"]
            .as_str()
            .unwrap()
            .contains("criterion: documented")
    );
    assert!(
        text.starts_with("# Assess `repository:checkout-web` against `repo-review`"),
        "{text}"
    );
    assert!(
        text.contains("- **owned**: Does a real team own it?\n  Check CODEOWNERS."),
        "{text}"
    );
    let (_, text, is_error) = call(
        &url,
        json!({"name": "assessment_brief", "arguments": {"id": "repository:checkout-web", "rubric": "nope"}}),
    )
    .await;
    assert!(is_error);
    assert_eq!(text, "no rubric nope for kind Repository");
}
