//! The Toolbox server (docs/v1-spec.md §4): snapshot loop + REST API. Later milestones add
//! authorization (M3), MCP, the UI host and connectors (M4).

pub mod routes;
pub mod state;

pub use state::{AppState, Config, Published};

use std::sync::Arc;
use toolbox_model::Manifest;
use toolbox_store::{PgStore, Store};

/// Build the shared state for a manifest and a store.
pub fn app_state(
    manifest: Manifest,
    manifest_doc: &serde_json::Value,
    store: Arc<dyn Store>,
    pg: Option<PgStore>,
    config: Config,
) -> Arc<AppState> {
    let (current, _) = tokio::sync::watch::channel(None);
    Arc::new(AppState {
        manifest: Arc::new(manifest),
        manifest_hash: state::manifest_hash(manifest_doc),
        store,
        pg,
        current,
        changed: tokio::sync::Notify::new(),
        metrics: Default::default(),
        config,
    })
}

/// Serve on `listener` until the process exits (the snapshot loop runs alongside).
pub async fn serve(state: Arc<AppState>, listener: tokio::net::TcpListener) -> std::io::Result<()> {
    tokio::spawn(state.clone().run_snapshot_loop());
    axum::serve(listener, routes::router(state)).await
}
