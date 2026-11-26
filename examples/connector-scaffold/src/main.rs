//! Minimal stdio connector. Config: `SEIKIKA_CONNECTOR_CONFIG` JSON `{"seed":"…"}`.
//! Emits one `example-item` record per resync so `seikika connector test` / `drive` work.

use serde_json::{Value, json};
use seikika_connect::sdk::{Out, Source, run_stdio};

struct Example {
    seed: String,
}

impl Example {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("SEIKIKA_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("SEIKIKA_CONNECTOR_CONFIG: {e}"))?;
        Ok(Example {
            seed: cfg
                .get("seed")
                .and_then(Value::as_str)
                .unwrap_or("alpha")
                .to_owned(),
        })
    }
}

impl Source for Example {
    fn name(&self) -> &str {
        "example"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn capabilities(&self) -> Value {
        json!({
            "refresh": {"example-item": ["external_ids"]},
            "resync": ["example-item"],
            "max_concurrency": 1
        })
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                // externalId must match ^[a-z0-9-]+:.+$ (kind-ish prefix + id).
                let id = format!("example:{}", self.seed);
                out.record(
                    "example-item",
                    &id,
                    json!({"id": self.seed, "title": format!("Item {}", self.seed)}),
                );
                out.snapshot_complete("example-item");
                Ok(())
            }
            Some("refresh") => {
                for ext in cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    let want = format!("example:{}", self.seed);
                    if ext == want {
                        out.record(
                            "example-item",
                            ext,
                            json!({"id": self.seed, "title": format!("Item {}", self.seed)}),
                        );
                    } else {
                        out.not_found("example-item", ext);
                    }
                }
                Ok(())
            }
            other => Err(format!("unsupported command: {other:?}")),
        }
    }
}

#[tokio::main]
async fn main() {
    let src = match Example::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    run_stdio(src).await;
}
