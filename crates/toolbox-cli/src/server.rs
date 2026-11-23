//! `toolbox serve` and `toolbox push`.

use std::path::Path;

/// Resolve the listen address. An explicit `--listen` or `TOOLBOX_LISTEN` wins. Otherwise
/// `PORT` binds every interface, which is what Cloud Run and `docker run -e PORT` inject.
/// The laptop default stays loopback so a bare `toolbox serve` is not exposed.
pub fn listen_addr(explicit: Option<&str>, port: Option<&str>) -> String {
    if let Some(addr) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        return addr.to_owned();
    }
    if let Some(port) = port.map(str::trim).filter(|s| !s.is_empty()) {
        return format!("0.0.0.0:{port}");
    }
    "127.0.0.1:8080".to_owned()
}
use std::process::ExitCode;
use std::sync::Arc;
use toolbox_authz::authn::{AuthConfig, Authenticator, Mode, PatLookup};
use toolbox_store::{SharedMem, Store};

struct PgPats(toolbox_store::PgStore);

#[async_trait::async_trait]
impl PatLookup for PgPats {
    async fn subject_for(&self, hash: &str) -> Option<String> {
        self.0.token_subject(hash).await
    }
}

/// Start OpenFGA for the portal's model: on Postgres with the tuple view, else in memory.
async fn start_authz(
    root: &Path,
    manifest: &toolbox_model::Manifest,
    pg: Option<&toolbox_store::PgStore>,
    database_url: Option<&str>,
) -> Result<Option<Arc<dyn toolbox_authz::Authorizer>>, String> {
    let Some(authz) = &manifest.authz else {
        return Ok(None);
    };
    let dsl = std::fs::read_to_string(root.join(&authz.model))
        .map_err(|e| format!("{}: {e}", authz.model))?;
    let model = toolbox_authz::fga_dsl::to_json(&dsl)?;
    let datastore = match database_url {
        Some(url) => {
            let uri = std::env::var("TOOLBOX_OPENFGA_DATASTORE_URI").unwrap_or_else(|_| {
                if url.contains("sslmode=") {
                    url.to_owned()
                } else if url.contains('?') {
                    format!("{url}&sslmode=disable")
                } else {
                    format!("{url}?sslmode=disable")
                }
            });
            toolbox_authz::Datastore::Postgres(uri)
        }
        None => toolbox_authz::Datastore::Memory,
    };
    let fga = toolbox_authz::OpenFga::start(
        datastore,
        &format!("toolbox-{}", manifest.portal.name),
        model,
    )
    .await?;
    if let Some(pg) = pg {
        toolbox_authz::openfga::install_tuple_view(&pg.pool, &fga.store).await?;
    }
    Ok(Some(Arc::new(fga)))
}

fn config_from_env() -> toolbox_api::Config {
    let mut c = toolbox_api::Config::default();
    if let Ok(mode) = std::env::var("TOOLBOX_AUTH_MODE") {
        c.auth_mode = mode;
    }
    if let Ok(tokens) = std::env::var("TOOLBOX_CONNECTOR_TOKENS") {
        c.connector_tokens = tokens
            .split(',')
            .filter_map(|p| {
                p.split_once('=')
                    .map(|(n, t)| (n.trim().to_owned(), t.trim().to_owned()))
            })
            .collect();
    }
    c
}

fn init_serve_telemetry() -> Result<toolbox_telemetry::TelemetryGuard, ExitCode> {
    toolbox_telemetry::init(toolbox_telemetry::TelemetryOptions {
        mode: None,
        service_name: None,
        prometheus: Some(true),
        log_format: None,
    })
    .map_err(|e| {
        eprintln!("error: telemetry: {e}");
        ExitCode::from(2)
    })
}

pub fn serve(path: &Path, listen: &str, database_url: Option<String>, fixtures: bool) -> ExitCode {
    let report = match toolbox_model::validate_portal(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    if !report.ok() {
        for d in &report.errors {
            eprintln!("ERROR {d}");
        }
        return ExitCode::from(1);
    }
    let src = toolbox_model::load::load(path).expect("validated above");
    let manifest = crate::portal::manifest_from(&src).expect("validated above");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let database_url_for_fga = database_url.clone();
    rt.block_on(async move {
        // Init after the runtime exists (OTLP batch exporters need it) and before listen.
        let _telemetry = match init_serve_telemetry() {
            Ok(g) => g,
            Err(code) => return code,
        };
        let (store, pg): (Arc<dyn Store>, _) = match database_url {
            Some(url) => match toolbox_store::PgStore::connect(&url).await {
                Ok(pg) => (Arc::new(pg.clone()), Some(pg)),
                Err(e) => {
                    eprintln!("error: Postgres: {e}");
                    return ExitCode::from(2);
                }
            },
            None => {
                let mem = if fixtures {
                    match toolbox_store::load_fixtures(&src.root) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("error: {e}");
                            return ExitCode::from(2);
                        }
                    }
                } else {
                    toolbox_store::MemStore::new()
                };
                (Arc::new(SharedMem(std::sync::Mutex::new(mem))), None)
            }
        };
        let mode = if pg.is_some() { "postgres" } else { "in-memory" };
        let config = config_from_env();
        let auth_mode = match Mode::parse(&config.auth_mode) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(2);
            }
        };
        let authn = Authenticator::new(
            AuthConfig::from_env(auth_mode.clone()),
            pg.clone().map(|p| Box::new(PgPats(p)) as Box<dyn PatLookup>),
        );
        let openfga_available = std::process::Command::new(toolbox_authz::openfga::binary()).arg("version").output().is_ok();
        let authz = if openfga_available {
            match start_authz(&src.root, &manifest, pg.as_ref(), database_url_for_fga.as_deref()).await {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("error: authorization: {e}");
                    return ExitCode::from(2);
                }
            }
        } else if auth_mode == Mode::Dev {
            tracing::warn!(
                "no `openfga` binary (TOOLBOX_OPENFGA_BIN); running WITHOUT authorization (dev mode only)"
            );
            None
        } else {
            eprintln!("error: authorization needs the `openfga` binary (TOOLBOX_OPENFGA_BIN); refusing to start in {} mode without it", config.auth_mode);
            return ExitCode::from(2);
        };
        let authz_label = if authz.is_some() { "OpenFGA" } else { "none" };
        let state = toolbox_api::app_state_with(manifest, &src.document, store, pg, config, authn, authz);
        let listener = match tokio::net::TcpListener::bind(listen).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: listen {listen}: {e}");
                return ExitCode::from(2);
            }
        };
        // Keep the literal `authorization: …` in the message: m3-acceptance greps server.log for it.
        tracing::info!(
            "toolbox: serving {} on http://{listen} ({mode} store, {auth:?} auth, authorization: {authorization}, manifest {manifest})",
            src.manifest_file,
            listen = listen,
            mode = mode,
            auth = format!("{auth_mode:?}"),
            authorization = authz_label,
            manifest = state.manifest_hash.as_str(),
        );
        tokio::select! {
            r = toolbox_api::serve(state, listener) => {
                if let Err(e) = r { eprintln!("error: {e}"); return ExitCode::from(1); }
            }
            _ = shutdown_signal() => tracing::info!("shutting down"),
        }
        ExitCode::SUCCESS
    })
}

/// Ctrl-C, or SIGTERM on Unix (how containers and scripts stop a server). Exiting this way drops
/// the supervised children, so the OpenFGA co-process is killed rather than left running.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = term.recv() => {}
                }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

pub fn push(path: &Path, url: &str) -> ExitCode {
    let root = match toolbox_model::load::resolve(path) {
        Ok((root, _)) => root,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let (batches, native) = match toolbox_store::read_fixtures(&root) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async move {
        let client = reqwest::Client::new();
        let mut last = 0u64;
        for (connector, batch) in &batches {
            let body = batch
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            let resp = client
                .post(format!("{url}/v1/ingest"))
                .header("x-toolbox-connector", connector)
                .header("content-type", "application/x-ndjson")
                .body(body)
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() => {
                    let v: serde_json::Value = r.json().await.unwrap_or_default();
                    last = v["input_version"].as_u64().unwrap_or(last);
                    println!(
                        "{connector}: {} message(s) → input version {last}",
                        batch.len()
                    );
                }
                Ok(r) => {
                    let status = r.status();
                    eprintln!(
                        "{connector}: {status} {}",
                        r.text().await.unwrap_or_default()
                    );
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("{connector}: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        // Native writes need the entity to exist: wait for the snapshot with the records first.
        let snap = client
            .get(format!("{url}/v1/snapshot?min_snapshot={last}"))
            .send()
            .await;
        if !snap.map(|r| r.status().is_success()).unwrap_or(false) {
            eprintln!("snapshot {last} was not published");
            return ExitCode::from(1);
        }
        for n in &native {
            let r = client
                .put(format!("{url}/v1/entities/{}/native/{}", n.entity, n.field))
                .header(
                    "x-toolbox-user",
                    n.author
                        .as_deref()
                        .unwrap_or("user:push")
                        .trim_start_matches("user:"),
                )
                .json(&serde_json::json!({"value": n.value, "reason": n.reason}))
                .send()
                .await;
            match r {
                Ok(r) if r.status().is_success() => {
                    let v: serde_json::Value = r.json().await.unwrap_or_default();
                    last = v["input_version"].as_u64().unwrap_or(last);
                    println!("native {}.{} → input version {last}", n.entity, n.field);
                }
                Ok(r) => {
                    eprintln!(
                        "native {}.{}: {}",
                        n.entity,
                        n.field,
                        r.text().await.unwrap_or_default()
                    );
                    return ExitCode::from(1);
                }
                Err(e) => {
                    eprintln!("native: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        match client
            .get(format!("{url}/v1/snapshot?min_snapshot={last}"))
            .send()
            .await
        {
            Ok(r) if r.status().is_success() => {
                let v: serde_json::Value = r.json().await.unwrap_or_default();
                println!(
                    "snapshot {} published ({} entities)",
                    v["snapshot"].as_str().unwrap_or("?"),
                    v["entities"]
                );
                ExitCode::SUCCESS
            }
            _ => {
                eprintln!("snapshot {last} was not published within 10 s");
                ExitCode::from(1)
            }
        }
    })
}

pub fn create_token(user: &str, label: Option<&str>, database_url: &str) -> ExitCode {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async move {
        let pg = match toolbox_store::PgStore::connect(database_url).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error: Postgres: {e}");
                return ExitCode::from(2);
            }
        };
        let token = match toolbox_authz::authn::new_pat() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("error: {e}");
                return ExitCode::from(2);
            }
        };
        match pg
            .create_token(&toolbox_authz::authn::pat_hash(&token), user, label)
            .await
        {
            Ok(()) => {
                println!("{token}");
                eprintln!("personal access token for {user} (shown once; only its hash is stored)");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(1)
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::listen_addr;

    #[test]
    fn port_binds_every_interface_unless_listen_is_set() {
        assert_eq!(listen_addr(None, Some("8080")), "0.0.0.0:8080");
        assert_eq!(listen_addr(None, Some(" 9090 ")), "0.0.0.0:9090");
        assert_eq!(
            listen_addr(Some("127.0.0.1:8080"), Some("9090")),
            "127.0.0.1:8080"
        );
        assert_eq!(listen_addr(None, None), "127.0.0.1:8080");
        assert_eq!(listen_addr(None, Some("")), "127.0.0.1:8080");
        assert_eq!(listen_addr(Some(""), Some("8080")), "0.0.0.0:8080");
    }
}
