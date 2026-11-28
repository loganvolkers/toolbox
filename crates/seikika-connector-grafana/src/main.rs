//! Read configured instant Prometheus queries and dashboard links through Grafana.
//!
//! `SEIKIKA_CONNECTOR_CONFIG` carries `url`, optional `token` and `publicUrl`,
//! `datasourceUid`, and the configured service/environment `targets`.
//! - `grafana-observation`: Grafana identifiers, dashboard link, and instant metric results.

use reqwest::{Client, Method, RequestBuilder, Url};
use seikika_connect::sdk::{Out, Source, run_stdio};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const KIND: &str = "grafana-observation";
const SERVICE: &str = "{{service}}";
const ENVIRONMENT: &str = "{{environment}}";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Config {
    url: Option<String>,
    public_url: Option<String>,
    token: Option<String>,
    datasource_uid: Option<String>,
    #[serde(default)]
    targets: BTreeMap<String, Target>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Target {
    service_name: String,
    dashboard_uid: String,
    metrics: Vec<Metric>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metric {
    id: String,
    label: String,
    unit: String,
    expr: String,
}

struct Grafana {
    http: Client,
    base: Url,
    public: Url,
    config: Config,
}

impl Grafana {
    fn new(config: Config) -> Result<Self, String> {
        if !config.targets.is_empty()
            && (config.url.is_none() || !config.datasource_uid.as_deref().is_some_and(valid_uid))
        {
            return Err("targets require url and datasourceUid".into());
        }
        for (key, target) in &config.targets {
            target.validate(key)?;
        }
        let base = base_url(config.url.as_deref().unwrap_or("http://127.0.0.1:3000"))?;
        let public = config
            .public_url
            .as_deref()
            .map(base_url)
            .transpose()?
            .unwrap_or_else(|| base.clone());
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| "could not initialize HTTP client")?;
        Ok(Self {
            http,
            base,
            public,
            config,
        })
    }

    fn request(&self, method: Method, path: &[&str]) -> Result<RequestBuilder, String> {
        let mut url = self.base.clone();
        url.path_segments_mut()
            .map_err(|_| "invalid Grafana URL path")?
            .pop_if_empty()
            .extend(path);
        let request = self.http.request(method, url);
        Ok(match &self.config.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        })
    }

    async fn read(&self, request: RequestBuilder, resource: &str) -> Result<Option<Value>, String> {
        let response = request
            .send()
            .await
            .map_err(|_| format!("Grafana {resource} request failed"))?;
        match response.status().as_u16() {
            404 if resource == "dashboard" => Ok(None),
            // Grafana can send 400 with per-query results when a query fails.
            status if response.status().is_success() || (resource == "query" && status == 400) => {
                response
                    .json()
                    .await
                    .map(Some)
                    .map_err(|_| format!("invalid Grafana {resource} response"))
            }
            status => Err(format!("Grafana {resource} returned HTTP {status}")),
        }
    }

    async fn dashboard(&self, uid: &str) -> Result<Option<String>, String> {
        let request = self.request(Method::GET, &["api", "dashboards", "uid", uid])?;
        let Some(body) = self.read(request, "dashboard").await? else {
            return Ok(None);
        };
        if body["dashboard"]["uid"].as_str() != Some(uid) {
            return Err("Grafana returned a different dashboard UID".into());
        }
        // Build the link from the configured browser base. Grafana resolves a UID-only URL
        // to its current slug, and a response cannot override the link's host.
        let mut url = self.public.clone();
        url.path_segments_mut()
            .map_err(|_| "invalid Grafana browser URL path")?
            .pop_if_empty()
            .extend(["d", uid]);
        Ok(Some(url.to_string()))
    }

    async fn record(&self, key: &str, target: &Target) -> Result<Value, String> {
        let dashboard = self.dashboard(&target.dashboard_uid).await?;
        let results = self.query(key, target).await?;
        let observed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system time precedes the Unix epoch")?
            .as_secs();
        let metrics: Vec<_> = target
            .metrics
            .iter()
            .map(|metric| {
                let (status, value) = sample(&results[&metric.id]);
                json!({"id": metric.id, "label": metric.label, "unit": metric.unit,
                   "status": status, "value": value})
            })
            .collect();
        Ok(json!({
            "target": key,
            "serviceName": target.service_name,
            "dashboardUid": target.dashboard_uid,
            "dashboardUrl": dashboard,
            "observedAt": observed_at,
            "metrics": metrics,
        }))
    }

    async fn query(&self, key: &str, target: &Target) -> Result<Value, String> {
        let environment = key
            .split_once('@')
            .map(|(_, env)| env)
            .ok_or("target environment required")?;
        let queries: Vec<_> = target
            .metrics
            .iter()
            .map(|metric| {
                json!({
                    "refId": metric.id,
                    "datasource": {"uid": self.config.datasource_uid, "type": "prometheus"},
                    "expr": metric.expr.replace(SERVICE, &json!(target.service_name).to_string())
                        .replace(ENVIRONMENT, &json!(environment).to_string()),
                    "instant": true,
                    "range": false,
                    "format": "time_series"
                })
            })
            .collect();
        let request = self
            .request(Method::POST, &["api", "ds", "query"])?
            .json(&json!({"from": "now-5m", "to": "now", "queries": queries}));
        let body = self
            .read(request, "query")
            .await?
            .ok_or("missing Grafana query response")?;
        if !body["results"].is_object() {
            return Err("Grafana response is missing query results".into());
        }
        Ok(body["results"].clone())
    }
}

impl Target {
    fn validate(&self, key: &str) -> Result<(), String> {
        let slug = |value: &str| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        if !key
            .split_once('@')
            .is_some_and(|(service, env)| slug(service) && slug(env))
            || self.service_name.trim().is_empty()
            || !valid_uid(&self.dashboard_uid)
            || !(1..=12).contains(&self.metrics.len())
        {
            return Err("targets need a <service>@<environment> key, serviceName, dashboardUid and 1–12 metrics".into());
        }
        let mut ids = BTreeSet::new();
        if self.metrics.iter().any(|m| {
            !valid_uid(&m.id)
                || !ids.insert(&m.id)
                || m.label.trim().is_empty()
                || !m.expr.contains(SERVICE)
        }) {
            return Err(
                "metrics need unique IDs, labels and a PromQL expression scoped with {{service}}"
                    .into(),
            );
        }
        Ok(())
    }
}

fn valid_uid(value: &str) -> bool {
    (1..=40).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}

fn base_url(value: &str) -> Result<Url, String> {
    let url = Url::parse(value).map_err(|_| "invalid Grafana base URL")?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "Grafana URLs must be HTTP(S) base URLs without credentials, query or fragment".into(),
        );
    }
    Ok(url)
}

fn sample(result: &Value) -> (&'static str, Option<f64>) {
    if result.is_null()
        || result["error"].is_string()
        || result["status"]
            .as_u64()
            .is_some_and(|status| status >= 400)
    {
        return ("unavailable", None);
    }
    let Some(frames) = result["frames"].as_array() else {
        return ("unavailable", None);
    };
    if frames.is_empty() {
        return ("no-data", None);
    }
    if frames.len() != 1 {
        return ("unavailable", None);
    }
    let frame = &frames[0];
    let Some(fields) = frame["schema"]["fields"].as_array() else {
        return ("unavailable", None);
    };
    let mut numbers = fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field["type"] == "number");
    let Some((index, _)) = numbers.next() else {
        return if frame["data"]["values"].as_array().is_some_and(|columns| {
            columns
                .iter()
                .all(|col| col.as_array().is_some_and(Vec::is_empty))
        }) {
            ("no-data", None)
        } else {
            ("unavailable", None)
        };
    };
    if numbers.next().is_some() {
        return ("unavailable", None);
    }
    match frame["data"]["values"][index].as_array().map(Vec::as_slice) {
        Some([value]) if value.is_number() => ("ok", value.as_f64()),
        Some([] | [Value::Null]) => ("no-data", None),
        _ => ("unavailable", None),
    }
}

impl Source for Grafana {
    fn name(&self) -> &str {
        "grafana"
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
                for (key, target) in &self.config.targets {
                    records.push((format!("{KIND}:{key}"), self.record(key, target).await?));
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
                    if let Some((key, target)) = id
                        .strip_prefix("grafana-observation:")
                        .and_then(|key| self.config.targets.get_key_value(key))
                    {
                        out.record(KIND, id, self.record(key, target).await?);
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
        .map_err(|_| "invalid SEIKIKA_CONNECTOR_CONFIG for Grafana".into())
        .and_then(Grafana::new);
    match result {
        Ok(source) => run_stdio(source).await,
        Err(error) => {
            eprintln!("seikika-connector-grafana: {error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instant_query_requires_one_scalar_and_preserves_zero() {
        let frame = |values: Value| {
            json!({"schema": {"fields": [
            {"type": "number"}, {"type": "time"}
        ]}, "data": {"values": [values, [10]]}})
        };
        let result = |values| json!({"frames": [frame(values)], "status": 200});
        assert_eq!(sample(&result(json!([0]))), ("ok", Some(0.0)));
        assert_eq!(sample(&result(json!([null]))), ("no-data", None));
        assert_eq!(sample(&result(json!([3, 4]))), ("unavailable", None));
        assert_eq!(sample(&json!({"frames": []})), ("no-data", None));
        assert_eq!(
            sample(&json!({"status": 400, "error": "query error"})),
            ("unavailable", None)
        );
        assert_eq!(
            sample(&json!({"frames": [frame(json!([3])), frame(json!([4]))]})),
            ("unavailable", None)
        );
    }
}
