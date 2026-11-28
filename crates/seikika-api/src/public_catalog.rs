//! Signed-out access to the configured repositories and explicitly published service catalog.

use crate::routes::problem;
use crate::state::AppState;
use axum::Json;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use std::collections::BTreeMap;

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
    let mut kinds = BTreeMap::from([(
        "Repository".to_owned(),
        state.config.public_repositories.len(),
    )]);
    for item in services(state).into_values() {
        if let Some(kind) = item["kind"].as_str() {
            *kinds.entry(kind.to_owned()).or_default() += 1;
        }
    }
    response(json!({
        "viewer": viewer(),
        "portal": {"title": state.manifest.portal.title},
        "kinds": kinds,
    }))
}

fn repository_fields(name: &str) -> Value {
    json!({"title": name, "repositoryUrl": format!("https://github.com/{name}")})
}

fn repository(id: &str, name: &str) -> Value {
    json!({"id": id, "kind": "Repository", "fields": repository_fields(name)})
}

fn services(state: &AppState) -> BTreeMap<String, Value> {
    let sources = crate::report::public_sources(&state.manifest);
    if sources.is_empty() {
        return BTreeMap::new();
    }
    let Some(snapshot) = state.snapshot() else {
        return BTreeMap::new();
    };
    snapshot
        .derived
        .entities
        .keys()
        .filter_map(|id| {
            crate::report::guest_service_entity(&state.manifest, &snapshot.derived, id, &sources)
                .map(|entity| (id.clone(), entity))
        })
        .collect()
}

pub(crate) fn report(state: &AppState) -> Response {
    let repositories = &state.config.public_repositories;
    let overviews: serde_json::Map<String, Value> = repositories
        .iter()
        .map(|(id, name)| (id.clone(), repository_fields(name)))
        .collect();
    response(json!({
        "portal": state.manifest.portal.title.as_deref().unwrap_or("Seikika"),
        "publicCatalog": true,
        "catalog": {"Repository": repositories.keys().collect::<Vec<_>>()},
        "entityOverview": overviews,
        "scorecards": {},
        "scores": {},
    }))
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
    let mut public = services(state);
    if kind.is_none_or(|kind| kind.eq_ignore_ascii_case("Repository")) {
        public.extend(
            state
                .config
                .public_repositories
                .iter()
                .map(|(id, name)| (id.clone(), repository(id, name))),
        );
    }
    let items = public.into_iter().filter(|(id, item)| {
        kind.is_none_or(|kind| {
            item["kind"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(kind))
        }) && cursor.is_none_or(|cursor| id.as_str() > cursor)
            && needle.as_ref().is_none_or(|needle| {
                id.contains(needle)
                    || item["fields"]["title"]
                        .as_str()
                        .is_some_and(|title| title.to_ascii_lowercase().contains(needle))
            })
    });
    let mut items = items.peekable();
    let page: Vec<_> = items.by_ref().take(limit).map(|(_, item)| item).collect();
    let next_cursor = items
        .peek()
        .and_then(|_| page.last())
        .map(|item| &item["id"]);
    response(json!({"viewer": viewer(), "items": page, "next_cursor": next_cursor}))
}

pub(crate) fn entity(state: &AppState, id: &str, html: bool) -> Response {
    let item = state
        .config
        .public_repositories
        .get(id)
        .map(|name| repository(id, name))
        .or_else(|| {
            let snapshot = state.snapshot()?;
            let sources = crate::report::public_sources(&state.manifest);
            crate::report::guest_service_entity(&state.manifest, &snapshot.derived, id, &sources)
        });
    let Some(item) = item else {
        return problem(StatusCode::NOT_FOUND, "not-found", "no public entity");
    };
    let fields = &item["fields"];
    if html {
        return crate::ui::entity(
            fields["title"].as_str().unwrap_or(id),
            fields["owner"].as_str(),
        );
    }
    response(json!({"viewer": viewer(), "entity": item}))
}
