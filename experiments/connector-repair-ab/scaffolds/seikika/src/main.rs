//! Forge (mock Jira Cloud) stdio connector.
//! Config: `SEIKIKA_CONNECTOR_CONFIG` JSON `{"baseUrl","token"}`.
//! Kinds: `project`, `issue`, `component`.

use serde_json::{Value, json};
use seikika_connect::sdk::{Out, Source, run_stdio};

struct Forge {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl Forge {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("SEIKIKA_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("SEIKIKA_CONNECTOR_CONFIG: {e}"))?;
        let s = |k: &str| cfg.get(k).and_then(Value::as_str).map(str::to_owned);
        Ok(Forge {
            base_url: s("baseUrl").ok_or("config needs `baseUrl`")?,
            token: s("token").ok_or("config needs `token`")?,
            http: reqwest::Client::new(),
        })
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{path}", self.base_url.trim_end_matches('/'));
        let r = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = r.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(format!("GET {path}: HTTP {status}"));
        }
        r.json().await.map_err(|e| e.to_string())
    }

    async fn resync_projects(&self, out: &mut Out) -> Result<(), String> {
        let mut start_at = 0u64;
        loop {
            let page = self
                .get(&format!(
                    "/rest/api/3/project?startAt={start_at}&maxResults=50"
                ))
                .await?;
            let values = page["values"].as_array().cloned().unwrap_or_default();
            for p in &values {
                let key = p["key"].as_str().unwrap_or_default();
                if key.is_empty() {
                    continue;
                }
                out.record(
                    "project",
                    &format!("project:{key}"),
                    json!({
                        "key": key,
                        "name": p["name"].as_str().unwrap_or(""),
                        "type": p["projectTypeKey"].as_str().unwrap_or(""),
                    }),
                );
            }
            if page["isLast"].as_bool() == Some(true) || values.is_empty() {
                break;
            }
            start_at += values.len() as u64;
        }
        out.snapshot_complete("project");
        Ok(())
    }

    async fn resync_issues(&self, out: &mut Out) -> Result<(), String> {
        let mut start_at = 0u64;
        loop {
            let page = self
                .get(&format!(
                    "/rest/api/3/search?startAt={start_at}&maxResults=50"
                ))
                .await?;
            let issues = page["issues"].as_array().cloned().unwrap_or_default();
            for issue in &issues {
                let key = issue["key"].as_str().unwrap_or_default();
                if key.is_empty() {
                    continue;
                }
                let fields = &issue["fields"];
                let assignee: Option<String> = None;
                out.record(
                    "issue",
                    &format!("issue:{key}"),
                    json!({
                        "key": key,
                        "summary": fields["summary"].as_str().unwrap_or(""),
                        "status": fields["status"]["name"].as_str().unwrap_or(""),
                        "project": fields["project"]["key"].as_str().unwrap_or(""),
                        "assignee": assignee,
                    }),
                );
            }
            let total = page["total"].as_u64().unwrap_or(0);
            start_at += issues.len() as u64;
            if issues.is_empty() || start_at >= total {
                break;
            }
        }
        out.snapshot_complete("issue");
        Ok(())
    }

    /// Collect project keys (paged) for per-project component fetches.
    async fn project_keys(&self) -> Result<Vec<String>, String> {
        let mut keys = Vec::new();
        let mut start_at = 0u64;
        loop {
            let page = self
                .get(&format!(
                    "/rest/api/3/project?startAt={start_at}&maxResults=50"
                ))
                .await?;
            let values = page["values"].as_array().cloned().unwrap_or_default();
            for p in &values {
                if let Some(key) = p["key"].as_str().filter(|k| !k.is_empty()) {
                    keys.push(key.to_owned());
                }
            }
            if page["isLast"].as_bool() == Some(true) || values.is_empty() {
                break;
            }
            start_at += values.len() as u64;
        }
        Ok(keys)
    }

    async fn resync_components(&self, out: &mut Out) -> Result<(), String> {
        // Components are a bare JSON array per project, not a global paged list.
        // Catalog id is `{project}:{name}`; Forge numeric id stays in data.id only.
        for project_key in self.project_keys().await? {
            let comps = self
                .get(&format!("/rest/api/3/project/{project_key}/components"))
                .await?;
            let arr = comps.as_array().cloned().unwrap_or_default();
            for c in &arr {
                let id = c["id"].as_str().unwrap_or_default();
                let name = c["name"].as_str().unwrap_or("");
                if id.is_empty() || name.is_empty() {
                    continue;
                }
                out.record(
                    "component",
                    &format!("component:{name}"),
                    json!({
                        "id": id,
                        "name": name,
                        "project": project_key,
                        "description": c["description"].as_str().unwrap_or(""),
                    }),
                );
            }
        }
        out.snapshot_complete("component");
        Ok(())
    }
}

impl Source for Forge {
    fn name(&self) -> &str {
        "forge"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn capabilities(&self) -> Value {
        json!({
            "refresh": {
                "project": ["external_ids"],
                "issue": ["external_ids"],
                "component": ["external_ids"]
            },
            "resync": ["project", "issue", "component"],
            "max_concurrency": 1
        })
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                let kinds: Vec<&str> = cmd["kinds"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect();
                // Empty kinds ⇒ all capabilities.resync kinds.
                let all = kinds.is_empty();
                if all || kinds.contains(&"project") {
                    self.resync_projects(out).await?;
                }
                if all || kinds.contains(&"issue") {
                    self.resync_issues(out).await?;
                }
                if all || kinds.contains(&"component") {
                    self.resync_components(out).await?;
                }
                Ok(())
            }
            Some("refresh") => {
                let kind = cmd["kind"].as_str().unwrap_or("");
                for ext in cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    match kind {
                        "project" => {
                            let key = ext.strip_prefix("project:").unwrap_or(ext);
                            let page = self
                                .get(&format!(
                                    "/rest/api/3/project?startAt=0&maxResults=100"
                                ))
                                .await?;
                            let found = page["values"].as_array().into_iter().flatten().find(|p| {
                                p["key"].as_str() == Some(key)
                            });
                            if let Some(p) = found {
                                out.record(
                                    "project",
                                    ext,
                                    json!({
                                        "key": key,
                                        "name": p["name"].as_str().unwrap_or(""),
                                        "type": p["projectTypeKey"].as_str().unwrap_or(""),
                                    }),
                                );
                            } else {
                                out.not_found("project", ext);
                            }
                        }
                        "issue" => {
                            // Search is the list API; filter by key in-memory for refresh.
                            let want = ext.strip_prefix("issue:").unwrap_or(ext);
                            let mut found = None;
                            let mut start_at = 0u64;
                            loop {
                                let page = self
                                    .get(&format!(
                                        "/rest/api/3/search?startAt={start_at}&maxResults=50"
                                    ))
                                    .await?;
                                let issues =
                                    page["issues"].as_array().cloned().unwrap_or_default();
                                if let Some(issue) =
                                    issues.iter().find(|i| i["key"].as_str() == Some(want))
                                {
                                    found = Some(issue.clone());
                                    break;
                                }
                                let total = page["total"].as_u64().unwrap_or(0);
                                start_at += issues.len() as u64;
                                if issues.is_empty() || start_at >= total {
                                    break;
                                }
                            }
                            if let Some(issue) = found {
                                let fields = &issue["fields"];
                                let assignee: Option<String> = None;
                                out.record(
                                    "issue",
                                    ext,
                                    json!({
                                        "key": want,
                                        "summary": fields["summary"].as_str().unwrap_or(""),
                                        "status": fields["status"]["name"].as_str().unwrap_or(""),
                                        "project": fields["project"]["key"].as_str().unwrap_or(""),
                                        "assignee": assignee,
                                    }),
                                );
                            } else {
                                out.not_found("issue", ext);
                            }
                        }
                        "component" => {
                            // externalId is component:{project}:{name}; catalog id keeps project:name.
                            let want = ext.strip_prefix("component:").unwrap_or(ext);
                            let (want_project, want_name) = match want.split_once(':') {
                                Some((p, n)) => (p, n),
                                None => {
                                    out.not_found("component", ext);
                                    continue;
                                }
                            };
                            let mut found = None;
                            for project_key in self.project_keys().await? {
                                if project_key != want_project {
                                    continue;
                                }
                                let comps = self
                                    .get(&format!(
                                        "/rest/api/3/project/{project_key}/components"
                                    ))
                                    .await?;
                                if let Some(c) = comps
                                    .as_array()
                                    .into_iter()
                                    .flatten()
                                    .find(|c| c["name"].as_str() == Some(want_name))
                                {
                                    found = Some((project_key, c.clone()));
                                    break;
                                }
                            }
                            if let Some((project_key, c)) = found {
                                out.record(
                                    "component",
                                    ext,
                                    json!({
                                        "id": c["id"].as_str().unwrap_or(""),
                                        "name": c["name"].as_str().unwrap_or(""),
                                        "project": project_key,
                                        "description": c["description"].as_str().unwrap_or(""),
                                    }),
                                );
                            } else {
                                out.not_found("component", ext);
                            }
                        }
                        other => return Err(format!("unsupported refresh kind: {other}")),
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
    let src = match Forge::from_env() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    run_stdio(src).await;
}
