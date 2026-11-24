//! The Toolbox server (docs/v1-spec.md §4): snapshot loop + REST API. Later milestones add
//! authorization (M3), MCP, the UI host and connectors (M4).

pub mod access;
pub mod actions;
pub mod artifacts;
pub mod assess;
pub mod confine;
pub mod connectors;
pub mod http_trace;
pub mod mcp;
pub mod reads;
pub mod routes;
pub mod state;
pub mod ui;

pub use state::{AppState, Config, Published};

use std::sync::Arc;
use toolbox_model::Manifest;
use toolbox_store::{PgStore, Store};

/// Build the shared state for a manifest and a store, with dev authentication and no Authorizer.
pub fn app_state(
    manifest: Manifest,
    manifest_doc: &serde_json::Value,
    store: Arc<dyn Store>,
    pg: Option<PgStore>,
    config: Config,
) -> Arc<AppState> {
    let authn = toolbox_authz::authn::Authenticator::new(
        toolbox_authz::authn::AuthConfig::from_env(toolbox_authz::authn::Mode::Dev),
        None,
    );
    app_state_with(manifest, manifest_doc, store, pg, config, authn, None)
}

/// Build the shared state with an explicit authenticator and Authorizer.
pub fn app_state_with(
    manifest: Manifest,
    manifest_doc: &serde_json::Value,
    store: Arc<dyn Store>,
    pg: Option<PgStore>,
    config: Config,
    authn: toolbox_authz::authn::Authenticator,
    authz: Option<Arc<dyn toolbox_authz::Authorizer>>,
) -> Arc<AppState> {
    let (current, _) = tokio::sync::watch::channel(None);
    Arc::new(AppState {
        hub: toolbox_connect::Hub::new(),
        authn,
        authz,
        manifest: Arc::new(manifest),
        manifest_hash: state::manifest_hash(manifest_doc),
        store,
        pg,
        current,
        changed: tokio::sync::Notify::new(),
        config,
        runs: Default::default(),
    })
}

/// Serve on `listener` until the process exits (the snapshot loop runs alongside).
pub async fn serve(state: Arc<AppState>, listener: tokio::net::TcpListener) -> std::io::Result<()> {
    mcp::trace_declare();
    tokio::spawn(state.clone().run_snapshot_loop());
    tokio::spawn(connectors::freshness_loop(state.clone()));
    tokio::spawn(actions::runs_loop(state.clone()));
    connectors::spawn_in_image(&state);
    axum::serve(listener, routes::router(state)).await
}
