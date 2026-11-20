//! `toolbox serve` and `toolbox push`.

use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use toolbox_store::{SharedMem, Store};

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
    let manifest =
        crate::portal::manifest_from(&src.document, &src.manifest_file).expect("validated above");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async move {
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
        let mode = if pg.is_some() {
            "postgres"
        } else {
            "in-memory"
        };
        let state = toolbox_api::app_state(manifest, &src.document, store, pg, config_from_env());
        let listener = match tokio::net::TcpListener::bind(listen).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("error: listen {listen}: {e}");
                return ExitCode::from(2);
            }
        };
        eprintln!(
            "toolbox: serving {} on http://{listen} ({mode} store, manifest {})",
            src.manifest_file, state.manifest_hash
        );
        tokio::select! {
            r = toolbox_api::serve(state, listener) => {
                if let Err(e) = r { eprintln!("error: {e}"); return ExitCode::from(1); }
            }
            _ = tokio::signal::ctrl_c() => eprintln!("toolbox: shutting down"),
        }
        ExitCode::SUCCESS
    })
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
