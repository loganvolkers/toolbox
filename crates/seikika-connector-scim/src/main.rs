//! The SCIM connector (remote): an identity provider provisions users and groups into it with
//! SCIM 2.0 (RFC 7644), and it pushes each change to Seikika as connector records, so membership
//! removals reach authorization within seconds (the revocation fast path, §9).
//!
//! Usage: `seikika-connector-scim --seikika URL --listen ADDR [--connector idp]`
//! Env: `SEIKIKA_SCIM_TOKEN` (bearer token the IdP sends), `SEIKIKA_CONNECTOR_TOKEN` (for Seikika).
//!
//! Records (shapes match examples/reference-portal/fixtures/idp.ndjson):
//! - `user` `idp:user/<userName>`: login, name, email
//! - `group` `idp:group/<slug>`: slug, displayName, parent?, githubTeam?, members (logins);
//!   slug/parent/githubTeam come from the `urn:ietf:params:scim:schemas:extension:seikika:2.0:Group`
//!   extension, defaulting the slug to the lower-cased display name.
//!
//! SCIM is push-only, so the connector registers without refresh or resync capabilities.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use seikika_connect::sdk::{Out, Source, push, run_remote};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const EXT: &str = "urn:ietf:params:scim:schemas:extension:seikika:2.0:Group";

struct App {
    seikika: String,
    connector: String,
    token: Option<String>,
    scim_token: Option<String>,
    users: Mutex<HashMap<String, Value>>, // SCIM id → user resource
    groups: Mutex<HashMap<String, Value>>, // SCIM id → group resource
}

fn authorized(app: &App, h: &HeaderMap) -> bool {
    match &app.scim_token {
        None => true,
        Some(t) => {
            h.get("authorization").and_then(|v| v.to_str().ok()) == Some(&format!("Bearer {t}"))
        }
    }
}

fn user_record(u: &Value) -> Value {
    let login = u["userName"].as_str().unwrap_or_default();
    let name = u["name"]["formatted"]
        .as_str()
        .or(u["displayName"].as_str())
        .unwrap_or(login);
    let email = u["emails"]
        .as_array()
        .and_then(|e| e.first())
        .and_then(|e| e["value"].as_str())
        .unwrap_or_default();
    json!({"type": "record", "kind": "user", "externalId": format!("idp:user/{login}"), "data": {"login": login, "name": name, "email": email}})
}

fn group_record(app: &App, g: &Value) -> Value {
    let display = g["displayName"].as_str().unwrap_or_default();
    let ext = &g[EXT];
    let slug = ext["slug"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| display.to_lowercase().replace(' ', "-"));
    let users = app.users.lock().unwrap();
    let members: Vec<Value> = g["members"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["value"].as_str())
        .filter_map(|id| {
            users
                .get(id)
                .and_then(|u| u["userName"].as_str())
                .map(|s| json!(s))
        })
        .collect();
    let mut data = json!({"slug": slug, "displayName": display, "members": members});
    for k in ["parent", "githubTeam"] {
        if let Some(v) = ext[k].as_str() {
            data[k] = json!(v);
        }
    }
    json!({"type": "record", "kind": "group", "externalId": format!("idp:group/{slug}"), "data": data})
}

async fn send(app: &App, msgs: Vec<Value>) -> Result<(), Response> {
    push(&app.seikika, &app.connector, app.token.as_deref(), msgs)
        .await
        .map(|_| ())
        .map_err(|e| (StatusCode::BAD_GATEWAY, Json(json!({"schemas": ["urn:ietf:params:scim:api:messages:2.0:Error"], "detail": e}))).into_response())
}

async fn upsert(
    State(app): State<Arc<App>>,
    h: HeaderMap,
    Path((resource, id)): Path<(String, String)>,
    Json(mut body): Json<Value>,
) -> Response {
    if !authorized(&app, &h) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    body["id"] = json!(id);
    let msg = match resource.as_str() {
        "Users" => {
            app.users.lock().unwrap().insert(id.clone(), body.clone());
            user_record(&body)
        }
        "Groups" => {
            app.groups.lock().unwrap().insert(id.clone(), body.clone());
            group_record(&app, &body)
        }
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    if let Err(r) = send(&app, vec![msg]).await {
        return r;
    }
    (StatusCode::OK, Json(body)).into_response()
}

async fn create(
    State(app): State<Arc<App>>,
    h: HeaderMap,
    Path(resource): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    let id = body["externalId"]
        .as_str()
        .or(body["userName"].as_str())
        .or(body["displayName"].as_str())
        .unwrap_or("x")
        .to_owned();
    let r = upsert(State(app), h, Path((resource, id)), Json(body)).await;
    if r.status() == StatusCode::OK {
        (StatusCode::CREATED, r.into_body()).into_response()
    } else {
        r
    }
}

async fn remove(
    State(app): State<Arc<App>>,
    h: HeaderMap,
    Path((resource, id)): Path<(String, String)>,
) -> Response {
    if !authorized(&app, &h) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let msg = match resource.as_str() {
        "Users" => app.users.lock().unwrap().remove(&id).map(|u| json!({"type": "delete", "kind": "user", "externalId": user_record(&u)["externalId"]})),
        "Groups" => {
            let g = app.groups.lock().unwrap().remove(&id);
            g.map(|g| json!({"type": "delete", "kind": "group", "externalId": group_record(&app, &g)["externalId"]}))
        }
        _ => None,
    };
    match msg {
        Some(m) => match send(&app, vec![m]).await {
            Ok(()) => StatusCode::NO_CONTENT.into_response(),
            Err(r) => r,
        },
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn fetch(
    State(app): State<Arc<App>>,
    Path((resource, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    if !authorized(&app, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let found = match resource.as_str() {
        "Users" => app.users.lock().unwrap().get(&id).cloned(),
        "Groups" => app.groups.lock().unwrap().get(&id).cloned(),
        _ => None,
    };
    found.map_or(StatusCode::NOT_FOUND.into_response(), |v| {
        Json(v).into_response()
    })
}

struct Registrar {
    connector: String,
}

impl Source for Registrar {
    fn name(&self) -> &str {
        &self.connector
    }
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    fn capabilities(&self) -> Value {
        json!({"max_concurrency": 1})
    }
    async fn handle(&mut self, _cmd: &Value, _out: &mut Out) -> Result<(), String> {
        Err("SCIM is push-only: the identity provider sends changes; nothing to refresh".into())
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let Some(seikika) = arg("--seikika") else {
        eprintln!("usage: seikika-connector-scim --seikika URL --listen ADDR [--connector idp]");
        std::process::exit(2);
    };
    let listen = arg("--listen").unwrap_or_else(|| "127.0.0.1:9100".into());
    let connector = arg("--connector").unwrap_or_else(|| "idp".into());
    let app = Arc::new(App {
        seikika: seikika.clone(),
        connector: connector.clone(),
        token: std::env::var("SEIKIKA_CONNECTOR_TOKEN").ok(),
        scim_token: std::env::var("SEIKIKA_SCIM_TOKEN").ok(),
        users: Mutex::new(HashMap::new()),
        groups: Mutex::new(HashMap::new()),
    });
    let token = app.token.clone();
    tokio::spawn(async move {
        if let Err(e) = run_remote(Registrar { connector }, &seikika, token.as_deref()).await {
            eprintln!("seikika-connector-scim: control channel: {e}");
        }
    });
    let router = Router::new()
        .route("/scim/v2/{resource}", axum::routing::post(create))
        .route(
            "/scim/v2/{resource}/{id}",
            get(fetch).put(upsert).patch(upsert).delete(remove),
        )
        .with_state(app);
    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .expect("listen");
    eprintln!("seikika-connector-scim: SCIM on http://{listen}/scim/v2");
    axum::serve(listener, router).await.expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(scim_token: Option<&str>) -> App {
        App {
            seikika: String::new(),
            connector: "idp".into(),
            token: None,
            scim_token: scim_token.map(str::to_owned),
            users: Mutex::new(HashMap::new()),
            groups: Mutex::new(HashMap::new()),
        }
    }

    #[test]
    fn only_the_configured_bearer_token_is_authorized() {
        let mut h = HeaderMap::new();
        assert!(authorized(&app(None), &h));
        assert!(!authorized(&app(Some("s3")), &h));
        h.insert("authorization", "Bearer s3".parse().unwrap());
        assert!(authorized(&app(Some("s3")), &h));
        h.insert("authorization", "Bearer other".parse().unwrap());
        assert!(!authorized(&app(Some("s3")), &h));
    }

    #[test]
    fn user_records_fall_back_from_formatted_name_to_display_name_to_login() {
        let r = user_record(&json!({"userName": "ada", "emails": [{"value": "ada@example.com"}]}));
        assert_eq!(r["externalId"], "idp:user/ada");
        assert_eq!(
            r["data"],
            json!({"login": "ada", "name": "ada", "email": "ada@example.com"})
        );
        let r = user_record(&json!({"userName": "ada", "displayName": "Ada L"}));
        assert_eq!(r["data"]["name"], "Ada L");
    }

    #[tokio::test]
    async fn reads_require_the_configured_bearer_token() {
        let app = Arc::new(app(Some("test-token")));
        app.users
            .lock()
            .unwrap()
            .insert("u1".into(), json!({"userName": "ada"}));
        let path = || Path(("Users".into(), "u1".into()));
        let mut headers = HeaderMap::new();
        let unauthenticated = fetch(State(app.clone()), path(), headers.clone()).await;
        assert_eq!(unauthenticated.status(), StatusCode::UNAUTHORIZED);

        headers.insert("authorization", "Bearer test-token".parse().unwrap());
        let authenticated = fetch(State(app), path(), headers).await;
        assert_eq!(authenticated.status(), StatusCode::OK);
    }

    #[test]
    fn group_records_resolve_members_to_logins_and_read_the_extension() {
        let a = app(None);
        a.users
            .lock()
            .unwrap()
            .insert("u1".into(), json!({"userName": "ada"}));
        let g = json!({
            "displayName": "Platform Team",
            "members": [{"value": "u1"}, {"value": "gone"}],
            EXT: {"parent": "eng", "githubTeam": "acme/platform"}
        });
        let r = group_record(&a, &g);
        assert_eq!(r["externalId"], "idp:group/platform-team");
        assert_eq!(r["data"]["members"], json!(["ada"]));
        assert_eq!(r["data"]["parent"], "eng");
        assert_eq!(r["data"]["githubTeam"], "acme/platform");
        let r = group_record(&a, &json!({"displayName": "X", EXT: {"slug": "core"}}));
        assert_eq!(r["externalId"], "idp:group/core");
    }
}
