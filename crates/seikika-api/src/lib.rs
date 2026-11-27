//! The Seikika server (docs/v1-spec.md §4): snapshot loop + REST API. Later milestones add
//! authorization (M3), MCP, the UI host and connectors (M4).

pub mod access;
pub mod actions;
pub use seikika_model::artifacts;
pub mod assess;
pub mod confine;
pub mod connectors;
pub mod http_trace;
pub mod mcp;
mod public_catalog;
pub mod reads;
pub mod report;
pub mod routes;
pub mod state;
pub mod ui;

pub use state::{AppState, Config, Published};

use seikika_model::Manifest;
use seikika_store::{PgStore, Store};
use std::sync::Arc;

/// Build the shared state for a manifest and a store, with dev authentication and no Authorizer.
pub fn app_state(
    manifest: Manifest,
    manifest_doc: &serde_json::Value,
    store: Arc<dyn Store>,
    pg: Option<PgStore>,
    config: Config,
) -> Arc<AppState> {
    let authn = seikika_authz::authn::Authenticator::new(
        seikika_authz::authn::AuthConfig::from_env(seikika_authz::authn::Mode::Dev),
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
    authn: seikika_authz::authn::Authenticator,
    authz: Option<Arc<dyn seikika_authz::Authorizer>>,
) -> Arc<AppState> {
    let (current, _) = tokio::sync::watch::channel(None);
    Arc::new(AppState {
        hub: seikika_connect::Hub::new(),
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
        confinement: Default::default(),
    })
}

/// Serve on `listener` until the process exits (the snapshot loop runs alongside).
pub async fn serve(state: Arc<AppState>, listener: tokio::net::TcpListener) -> std::io::Result<()> {
    mcp::trace_declare();
    tokio::spawn(state.clone().run_snapshot_loop());
    tokio::spawn(connectors::freshness_loop(state.clone()));
    match actions::restore(&state).await {
        Ok(0) => {}
        Ok(n) => tracing::info!(runs = n, "restored action runs"),
        Err(e) => return Err(std::io::Error::other(format!("action runs: {e}"))),
    }
    tokio::spawn(actions::runs_loop(state.clone()));
    if let Some(k) = actions::approval_key(&state) {
        tracing::info!(
            approval_key = %k,
            "children accept this portal's approvals by pinning {{portal: {}, key: {k}}} in federation.acceptApprovalsFrom",
            state.manifest.portal.name
        );
    }
    connectors::spawn_in_image(&state);
    axum::serve(listener, routes::router(state)).await
}
