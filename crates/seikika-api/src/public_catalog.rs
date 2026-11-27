//! The signed-out catalog is a public projection of configured repository names.
//! It never uses the live snapshot, connector records, or an authenticated principal.

use crate::routes::problem;
use crate::state::AppState;
use axum::Json;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

pub(crate) fn guest(state: &AppState, headers: &HeaderMap) -> bool {
    state.config.auth_mode == "cloudflare-access"
        && !state.config.public_repositories.is_empty()
        && !headers.contains_key(header::AUTHORIZATION)
        && !headers.contains_key("cf-access-jwt-assertion")
}

fn viewer() -> Value {
    json!({"method": "guest", "admin": false})
}

fn response(body: Value) -> Response {
    ([(header::CACHE_CONTROL, "private, no-store")], Json(body)).into_response()
}

pub(crate) fn home(state: &AppState) -> Response {
    response(json!({
        "viewer": viewer(),
        "portal": {"title": state.manifest.portal.title},
        "kinds": {"Repository": state.config.public_repositories.len()},
    }))
}

fn repository(id: &str, name: &str) -> Value {
    json!({"id": id, "kind": "Repository", "fields": {
        "title": name,
        "repositoryUrl": format!("https://github.com/{name}"),
    }})
}

pub(crate) fn list(
    state: &AppState,
    kind: Option<&str>,
    search: Option<&str>,
    cursor: Option<&str>,
    limit: Option<usize>,
) -> Response {
    let needle = search.map(str::to_ascii_lowercase);
    let limit = limit.unwrap_or(100).clamp(1, 1000);
    let items = state
        .config
        .public_repositories
        .iter()
        .filter(|(id, name)| {
            kind.is_none_or(|kind| kind.eq_ignore_ascii_case("Repository"))
                && cursor.is_none_or(|cursor| id.as_str() > cursor)
                && needle.as_ref().is_none_or(|needle| {
                    id.contains(needle) || name.to_ascii_lowercase().contains(needle)
                })
        });
    let mut items = items.peekable();
    let page: Vec<_> = items
        .by_ref()
        .take(limit)
        .map(|(id, name)| repository(id, name))
        .collect();
    let next_cursor = items
        .peek()
        .and_then(|_| page.last())
        .map(|item| &item["id"]);
    response(json!({"viewer": viewer(), "items": page, "next_cursor": next_cursor}))
}

pub(crate) fn entity(state: &AppState, id: &str, html: bool) -> Response {
    let Some(name) = state.config.public_repositories.get(id) else {
        return problem(StatusCode::NOT_FOUND, "not-found", "no public repository");
    };
    if html {
        return crate::ui::entity(name, None);
    }
    response(json!({"viewer": viewer(), "entity": repository(id, name)}))
}
