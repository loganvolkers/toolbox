//! Forge (mock Jira Cloud) stdio connector.
//!
//! `SEIKIKA_CONNECTOR_CONFIG`: `{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}`.
//! Record kinds: `project`, `issue` (phase 1). External ids: `forge:<key>`.
//! With `FORGE_WRITE_ENTITIES=1`, a resync also writes mapped catalog entities to
//! `OUT/entities.ndjson` (or `FORGE_ENTITIES_PATH`), sorted by kind then id.

use serde_json::{Value, json};
use std::path::PathBuf;
use seikika_connect::sdk::{Out, Source, run_stdio};

struct Forge {
    base_url: String,
    token: String,
    http: reqwest::Client,
    /// Accumulated catalog entity lines for optional NDJSON export (`kind` order + id).
    entities: Vec<(u8, String, String)>, // (kind_rank, id, line)
}

impl Forge {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("SEIKIKA_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("SEIKIKA_CONNECTOR_CONFIG: {e}"))?;
        let s = |k: &str| cfg.get(k).and_then(Value::as_str).map(str::to_owned);
        Ok(Forge {
            base_url: s("baseUrl")
                .ok_or("config needs `baseUrl`")?
                .trim_end_matches('/')
                .to_owned(),
            token: s("token").ok_or("config needs `token`")?,
            http: reqwest::Client::new(),
            entities: Vec::new(),
        })
    }

    async fn get(&self, path: &str) -> Result<Value, String> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .header("accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("GET {path}: {e}"))?;
        let status = resp.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(format!("GET {path}: HTTP {status}"));
        }
        resp.json()
            .await
            .map_err(|e| format!("GET {path}: {e}"))
    }

    /// Paginate Forge list endpoints that use `startAt` / `maxResults` / `isLast`.
    async fn page(
        &self,
        path: &str,
        values_key: &str,
        page_size: u64,
    ) -> Result<Vec<Value>, String> {
        let mut out = Vec::new();
        let mut start = 0u64;
        loop {
            let sep = if path.contains('?') { '&' } else { '?' };
            let body = self
                .get(&format!(
                    "{path}{sep}startAt={start}&maxResults={page_size}"
                ))
                .await?;
            let items = body[values_key]
                .as_array()
                .cloned()
                .unwrap_or_default();
            let n = items.len() as u64;
            out.extend(items);
            let is_last = body["isLast"].as_bool().unwrap_or(n < page_size);
            if is_last || n == 0 {
                break;
            }
            start += n;
        }
        Ok(out)
    }

    async fn projects(&self) -> Result<Vec<Value>, String> {
        self.page("/rest/api/3/project", "values", 50).await
    }

    async fn issues(&self) -> Result<Vec<Value>, String> {
        // Search has no `isLast`; stop when a page is short or startAt past total.
        let mut out = Vec::new();
        let mut start = 0u64;
        let page_size = 50u64;
        loop {
            let body = self
                .get(&format!(
                    "/rest/api/3/search?startAt={start}&maxResults={page_size}"
                ))
                .await?;
            let items = body["issues"].as_array().cloned().unwrap_or_default();
            let n = items.len() as u64;
            out.extend(items);
            let total = body["total"].as_u64().unwrap_or(start + n);
            start += n;
            if n == 0 || start >= total {
                break;
            }
        }
        Ok(out)
    }

    fn map_project(p: &Value) -> Option<(String, Value, String)> {
        let key = p["key"].as_str()?.to_owned();
        let name = p["name"].as_str().unwrap_or(&key);
        let ty = p["projectTypeKey"].as_str().unwrap_or("software");
        let data = json!({"key": &key, "name": name, "type": ty});
        // Fixed key order for OUT/entities.ndjson (kind, id, data; data: key, name, type).
        let line = format!(
            "{{\"kind\":\"project\",\"id\":{},\"data\":{{\"key\":{},\"name\":{},\"type\":{}}}}}",
            serde_json::to_string(&key).unwrap(),
            serde_json::to_string(&key).unwrap(),
            serde_json::to_string(name).unwrap(),
            serde_json::to_string(ty).unwrap(),
        );
        Some((format!("forge:{key}"), data, line))
    }

    fn map_issue(i: &Value) -> Option<(String, Value, String)> {
        let key = i["key"].as_str()?.to_owned();
        let fields = &i["fields"];
        let summary = fields["summary"].as_str().unwrap_or("");
        let status = fields["status"]["name"].as_str().unwrap_or("");
        let project = fields["project"]["key"].as_str().unwrap_or("");
        let assignee = fields["assignee"]
            .as_object()
            .and_then(|a| a.get("accountId"))
            .and_then(Value::as_str);
        let data = json!({
            "key": &key,
            "summary": summary,
            "status": status,
            "project": project,
            "assignee": assignee,
        });
        let assignee_json = match assignee {
            Some(a) => serde_json::to_string(a).unwrap(),
            None => "null".to_owned(),
        };
        let line = format!(
            "{{\"kind\":\"issue\",\"id\":{},\"data\":{{\"key\":{},\"summary\":{},\"status\":{},\"project\":{},\"assignee\":{}}}}}",
            serde_json::to_string(&key).unwrap(),
            serde_json::to_string(&key).unwrap(),
            serde_json::to_string(summary).unwrap(),
            serde_json::to_string(status).unwrap(),
            serde_json::to_string(project).unwrap(),
            assignee_json,
        );
        Some((format!("forge:{key}"), data, line))
    }

    fn remember(&mut self, rank: u8, id: &str, line: String) {
        self.entities.retain(|e| !(e.0 == rank && e.1 == id));
        self.entities.push((rank, id.to_owned(), line));
    }

    fn write_entities_file(&mut self) -> Result<(), String> {
        if std::env::var("FORGE_WRITE_ENTITIES").ok().as_deref() != Some("1") {
            return Ok(());
        }
        self.entities
            .sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        let path = std::env::var("FORGE_ENTITIES_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("OUT/entities.ndjson"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let mut body = String::new();
        for (_, _, line) in &self.entities {
            body.push_str(line);
            body.push('\n');
        }
        std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(())
    }

    async fn emit_projects(
        &mut self,
        ids: Option<&[String]>,
        out: &mut Out,
    ) -> Result<(), String> {
        let want = |ext: &str| ids.is_none_or(|ids| ids.iter().any(|i| i == ext));
        let mut seen = Vec::new();
        for p in self.projects().await? {
            let Some((ext, data, line)) = Self::map_project(&p) else {
                continue;
            };
            if want(&ext) {
                let id = data["key"].as_str().unwrap_or("").to_owned();
                out.record("project", &ext, data);
                self.remember(0, &id, line);
                seen.push(ext);
            }
        }
        if let Some(ids) = ids {
            for id in ids {
                if !seen.iter().any(|s| s == id) {
                    out.not_found("project", id);
                }
            }
        }
        Ok(())
    }

    async fn emit_issues(
        &mut self,
        ids: Option<&[String]>,
        out: &mut Out,
    ) -> Result<(), String> {
        let want = |ext: &str| ids.is_none_or(|ids| ids.iter().any(|i| i == ext));
        let mut seen = Vec::new();
        for i in self.issues().await? {
            let Some((ext, data, line)) = Self::map_issue(&i) else {
                continue;
            };
            if want(&ext) {
                let id = data["key"].as_str().unwrap_or("").to_owned();
                out.record("issue", &ext, data);
                self.remember(1, &id, line);
                seen.push(ext);
            }
        }
        if let Some(ids) = ids {
            for id in ids {
                if !seen.iter().any(|s| s == id) {
                    out.not_found("issue", id);
                }
            }
        }
        Ok(())
    }

    /// Ensure OUT has both kinds even when conformance resyncs one kind at a time.
    async fn ensure_entities_snapshot(&mut self) -> Result<(), String> {
        if std::env::var("FORGE_WRITE_ENTITIES").ok().as_deref() != Some("1") {
            return Ok(());
        }
        self.entities.clear();
        for p in self.projects().await? {
            if let Some((_, data, line)) = Self::map_project(&p) {
                let id = data["key"].as_str().unwrap_or("").to_owned();
                self.remember(0, &id, line);
            }
        }
        for i in self.issues().await? {
            if let Some((_, data, line)) = Self::map_issue(&i) {
                let id = data["key"].as_str().unwrap_or("").to_owned();
                self.remember(1, &id, line);
            }
        }
        self.write_entities_file()
    }
}

impl Source for Forge {
    fn name(&self) -> &str {
        "forge"
    }

    fn capabilities(&self) -> Value {
        json!({
            "refresh": {
                "project": ["external_ids"],
                "issue": ["external_ids"]
            },
            "resync": ["project", "issue"],
            "max_concurrency": 1
        })
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                let kinds: Vec<String> = cmd["kinds"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|k| k.as_str().map(str::to_owned))
                    .collect();
                for kind in &kinds {
                    match kind.as_str() {
                        "project" => self.emit_projects(None, out).await?,
                        "issue" => self.emit_issues(None, out).await?,
                        other => return Err(format!("unsupported kind {other}")),
                    }
                    out.snapshot_complete(kind);
                }
                self.ensure_entities_snapshot().await?;
                Ok(())
            }
            Some("refresh") => {
                let kind = cmd["kind"].as_str().unwrap_or_default();
                let ids: Vec<String> = cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect();
                match kind {
                    "project" => self.emit_projects(Some(&ids), out).await?,
                    "issue" => self.emit_issues(Some(&ids), out).await?,
                    other => return Err(format!("unsupported kind {other}")),
                }
                Ok(())
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
}

#[tokio::main]
async fn main() {
    match Forge::from_env() {
        Ok(src) => run_stdio(src).await,
        Err(e) => {
            eprintln!("forge-connector: {e}");
            std::process::exit(2);
        }
    }
}
