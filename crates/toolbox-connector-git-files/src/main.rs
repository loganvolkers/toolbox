//! The `git-files` connector (in-image, stdio): one `catalog-info` record per repository file.
//! `TOOLBOX_CONNECTOR_CONFIG`: `{"org", "paths": ["catalog-info.yaml"], "root"?: "/dir"}` reads
//! `<root>/<repo>/<path>` from a local checkout; without `root` it lists the org's repositories
//! and reads files through the GitHub contents API (`token`, `apiUrl` as for `github`).
//! Record: `git:<org>/<repo>/<path>` → `{repo, path, content}` (content parsed from YAML).

use serde_json::{Value, json};
use std::path::PathBuf;
use toolbox_connect::sdk::{Out, Source, run_stdio};

struct GitFiles {
    org: String,
    paths: Vec<String>,
    root: Option<PathBuf>,
    api: String,
    token: Option<String>,
    http: reqwest::Client,
}

fn ext(org: &str, repo: &str, path: &str) -> String {
    format!("git:{org}/{repo}/{path}")
}

impl GitFiles {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("TOOLBOX_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("TOOLBOX_CONNECTOR_CONFIG: {e}"))?;
        let s = |k: &str| cfg.get(k).and_then(Value::as_str).map(str::to_owned);
        Ok(GitFiles {
            org: s("org").ok_or("config needs `org`")?,
            paths: cfg["paths"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|p| p.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_else(|| vec!["catalog-info.yaml".into()]),
            root: s("root").map(PathBuf::from),
            api: s("apiUrl")
                .or_else(|| std::env::var("GITHUB_API_URL").ok())
                .unwrap_or_else(|| "https://api.github.com".into()),
            token: s("token").or_else(|| std::env::var("GITHUB_TOKEN").ok()),
            http: reqwest::Client::new(),
        })
    }

    async fn get(&self, path: &str) -> Result<Option<Value>, String> {
        let mut req = self
            .http
            .get(format!("{}{path}", self.api))
            .header("user-agent", "toolbox-connector-git-files")
            .header("accept", "application/vnd.github+json");
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        let r = req.send().await.map_err(|e| e.to_string())?;
        match r.status().as_u16() {
            404 => Ok(None),
            s if (200..300).contains(&s) => r.json().await.map(Some).map_err(|e| e.to_string()),
            s => Err(format!("GET {path}: HTTP {s}")),
        }
    }

    async fn repos(&self) -> Result<Vec<String>, String> {
        if let Some(root) = &self.root {
            let mut v: Vec<String> = std::fs::read_dir(root)
                .map_err(|e| format!("{}: {e}", root.display()))?
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect();
            v.sort();
            return Ok(v);
        }
        let mut out = Vec::new();
        for page in 1..=1000 {
            let Some(list) = self
                .get(&format!(
                    "/orgs/{}/repos?per_page=100&page={page}",
                    self.org
                ))
                .await?
            else {
                break;
            };
            let items = list.as_array().cloned().unwrap_or_default();
            out.extend(
                items
                    .iter()
                    .filter_map(|r| r["name"].as_str().map(str::to_owned)),
            );
            if items.len() < 100 {
                break;
            }
        }
        Ok(out)
    }

    async fn read(&self, repo: &str, path: &str) -> Result<Option<String>, String> {
        if let Some(root) = &self.root {
            return Ok(std::fs::read_to_string(root.join(repo).join(path)).ok());
        }
        Ok(self
            .get(&format!("/repos/{}/{repo}/contents/{path}", self.org))
            .await?
            .and_then(|f| f["content"].as_str().map(decode_base64)))
    }

    async fn emit(&self, repo: &str, path: &str, out: &mut Out) -> Result<bool, String> {
        let Some(text) = self.read(repo, path).await? else {
            return Ok(false);
        };
        match serde_norway::from_str::<Value>(&text) {
            Ok(content) => out.record(
                "catalog-info",
                &ext(&self.org, repo, path),
                json!({"repo": repo, "path": path, "content": content}),
            ),
            Err(e) => out.log("warn", &format!("{repo}/{path}: not valid YAML: {e}")),
        }
        Ok(true)
    }
}

fn decode_base64(s: &str) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let (mut out, mut buf, mut bits) = (Vec::new(), 0u32, 0u32);
    for c in s.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=') {
        let Some(v) = T.iter().position(|&t| t == c) else {
            continue;
        };
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl Source for GitFiles {
    fn name(&self) -> &str {
        "git-files"
    }

    fn capabilities(&self) -> Value {
        json!({"refresh": {"catalog-info": ["external_ids"]}, "resync": ["catalog-info"], "max_concurrency": 1})
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                for repo in self.repos().await? {
                    for path in self.paths.clone() {
                        self.emit(&repo, &path, out).await?;
                    }
                }
                out.snapshot_complete("catalog-info");
                Ok(())
            }
            Some("refresh") => {
                let prefix = format!("git:{}/", self.org);
                for id in cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    let found = match id
                        .strip_prefix(&prefix)
                        .and_then(|rest| rest.split_once('/'))
                    {
                        Some((repo, path)) => self.emit(repo, path, out).await?,
                        None => false,
                    };
                    if !found {
                        out.not_found("catalog-info", id);
                    }
                }
                Ok(())
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
}

#[tokio::main]
async fn main() {
    match GitFiles::from_env() {
        Ok(src) => run_stdio(src).await,
        Err(e) => {
            eprintln!("toolbox-connector-git-files: {e}");
            std::process::exit(2);
        }
    }
}
