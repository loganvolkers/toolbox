//! Read bounded Cloudflare Workers rollout history through the Seikika stdio SDK.
//!
//! `SEIKIKA_CONNECTOR_CONFIG` carries `accountId`, `token`, optional `apiUrl`, and
//! `targets` mapping service/environment pairs to Worker scripts.
//! - `worker-deployments`: five recent rollouts with version IDs and traffic shares.

use reqwest::{Client, Url};
use seikika_connect::sdk::{Out, Source, run_stdio};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KIND: &str = "worker-deployments";
const DEPLOYMENTS_PER_WORKER: u8 = 5;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Config {
    account_id: Option<String>,
    api_url: Option<String>,
    token: Option<String>,
    #[serde(default)]
    targets: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct Envelope {
    success: bool,
    result: Option<History>,
}

#[derive(Deserialize)]
struct History {
    deployments: Vec<Deployment>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Deployment {
    id: String,
    created_on: String,
    versions: Vec<Version>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
struct Version {
    version_id: String,
    percentage: f64,
}

struct Cloudflare {
    http: Client,
    api: Url,
    config: Config,
}

impl Cloudflare {
    fn new(config: Config) -> Result<Self, String> {
        if !config.targets.is_empty()
            && !config
                .account_id
                .as_deref()
                .is_some_and(|id| id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("targets require a 32-digit hex accountId".into());
        }
        if config.targets.iter().any(|(id, worker)| {
            !id.split_once('@').is_some_and(|(service, environment)| {
                valid_slug(service) && valid_slug(environment)
            }) || !valid_slug(worker)
        }) {
            return Err("targets must map <service>@<environment> to a Worker script name".into());
        }
        let api = Url::parse(
            config
                .api_url
                .as_deref()
                .unwrap_or("https://api.cloudflare.com/client/v4"),
        )
        .map_err(|_| "invalid Cloudflare apiUrl")?;
        if !matches!(api.scheme(), "http" | "https")
            || api.host_str().is_none()
            || !api.username().is_empty()
            || api.password().is_some()
            || api.query().is_some()
            || api.fragment().is_some()
        {
            return Err(
                "apiUrl must be an HTTP(S) base URL without credentials, query or fragment".into(),
            );
        }
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "could not initialize HTTP client")?;
        Ok(Self { http, api, config })
    }

    async fn read_worker(&self, worker: &str) -> Result<Value, String> {
        let account = self
            .config
            .account_id
            .as_deref()
            .ok_or("accountId required")?;
        let token = self
            .config
            .token
            .as_deref()
            .filter(|t| !t.is_empty())
            .ok_or("Cloudflare Workers deployment read token required")?;
        let mut url = self.api.clone();
        url.path_segments_mut()
            .map_err(|_| "invalid Cloudflare API path")?
            .pop_if_empty()
            .extend([
                "accounts",
                account,
                "workers",
                "scripts",
                worker,
                "deployments",
            ]);
        let response = self
            .http
            .get(url)
            .bearer_auth(token)
            .query(&[("page", 1), ("per_page", DEPLOYMENTS_PER_WORKER)])
            .send()
            .await
            .map_err(|_| "Cloudflare deployment request failed")?;
        if !response.status().is_success() {
            return Err(format!(
                "Cloudflare deployment request returned HTTP {}",
                response.status().as_u16()
            ));
        }
        let result = response
            .json::<Envelope>()
            .await
            .map_err(|_| "invalid Cloudflare deployment response")?;
        let history = recent_deployments(result)?;
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system time precedes the Unix epoch")?
            .as_secs();
        Ok(json!({"worker": worker, "observedAt": observed_at, "recent": history}))
    }

    async fn record(&self, target: &str, worker: &str) -> Result<Value, String> {
        let mut record = self.read_worker(worker).await?;
        record["target"] = json!(target);
        Ok(record)
    }
}

fn recent_deployments(result: Envelope) -> Result<Vec<Deployment>, String> {
    let mut history = result
        .result
        .filter(|_| result.success)
        .ok_or("Cloudflare deployment read was unsuccessful")?;
    // Cloudflare defines the first entry as the deployment currently serving traffic.
    history.deployments.truncate(DEPLOYMENTS_PER_WORKER.into());
    Ok(history.deployments)
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl Source for Cloudflare {
    fn name(&self) -> &str {
        "cloudflare"
    }
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    fn capabilities(&self) -> Value {
        json!({"refresh": {(KIND): ["external_ids"]}, "resync": [KIND], "max_concurrency": 1})
    }

    async fn handle(&mut self, command: &Value, out: &mut Out) -> Result<(), String> {
        match command["command"].as_str() {
            Some("resync")
                if command["kinds"]
                    .as_array()
                    .is_some_and(|kinds| kinds.contains(&json!(KIND))) =>
            {
                let mut records = Vec::new();
                for (target, worker) in &self.config.targets {
                    records.push((
                        format!("{KIND}:{target}"),
                        self.record(target, worker).await?,
                    ));
                }
                for (id, record) in records {
                    out.record(KIND, &id, record);
                }
                out.snapshot_complete(KIND);
            }
            Some("refresh") if command["kind"] == KIND => {
                let ids = command["external_ids"]
                    .as_array()
                    .ok_or("external_ids required")?;
                for id in ids {
                    let id = id.as_str().ok_or("expected a string external ID")?;
                    if let Some((target, worker)) = id
                        .strip_prefix("worker-deployments:")
                        .and_then(|target| self.config.targets.get_key_value(target))
                    {
                        out.record(KIND, id, self.record(target, worker).await?);
                    } else {
                        out.not_found(KIND, id);
                    }
                }
            }
            _ => return Err("unsupported command or record kind".into()),
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = std::env::var("SEIKIKA_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into());
    let result = serde_json::from_str(&config)
        .map_err(|_| "invalid SEIKIKA_CONNECTOR_CONFIG for Cloudflare".into())
        .and_then(Cloudflare::new);
    match result {
        Ok(source) => run_stdio(source).await,
        Err(error) => {
            eprintln!("seikika-connector-cloudflare: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_the_active_deployment_and_bounds_public_history() -> Result<(), String> {
        let upstream = json!({"success": true, "result": {"deployments": (0..7).map(|id| json!({
            "id": id.to_string(),
            "created_on": format!("2026-11-28T02:00:0{id}Z"),
            "versions": [{"version_id": "version", "percentage": 100}],
            "annotations": {"workers/message": "operator details"},
            "author_email": "author@example.test"
        })).collect::<Vec<_>>()}});
        let response = serde_json::from_value(upstream).map_err(|error| error.to_string())?;
        let recent = recent_deployments(response)?;
        assert_eq!(recent.len(), 5);
        assert_eq!(
            json!(recent[0]),
            json!({
                "id": "0", "createdOn": "2026-11-28T02:00:00Z",
                "versions": [{"versionId": "version", "percentage": 100.0}]
            })
        );
        Ok(())
    }
}
