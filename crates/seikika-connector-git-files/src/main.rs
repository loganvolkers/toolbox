//! The `git-files` connector (in-image, stdio): one `catalog-info` record per repository file.
//! `SEIKIKA_CONNECTOR_CONFIG`: `{"org", "paths": ["catalog-info.yaml"], "root"?: "/dir"}` reads
//! `<root>/<repo>/<path>` from a local checkout; without `root` it lists the org's repositories
//! and reads files through the GitHub contents API (`token`, `apiUrl` as for `github`).
//! Record: `git:<org>/<repo>/<path>` → `{repo, path, content}` (content parsed from YAML).
//!
//! `splitKinds: true` emits one record per Backstage document (`component`, `api`, `system`,
//! `domain`, `resource`) so a multi-document `catalog-info.yaml` becomes one entity per kind.
//! `files` (for example `["README.md"]`) emits `file` records `{repo, path, present: true}`.
//! The external id of a split document is `git:<org>/<repo>/<path>#<kind>:<name>`.

use seikika_connect::sdk::{Out, Source, run_stdio};
use serde_json::{Value, json};
use std::path::PathBuf;

struct GitFiles {
    org: String,
    paths: Vec<String>,
    files: Vec<String>,
    split: bool,
    root: Option<PathBuf>,
    api: String,
    token: Option<String>,
    http: reqwest::Client,
}

fn ext(org: &str, repo: &str, path: &str) -> String {
    format!("git:{org}/{repo}/{path}")
}

/// YAML documents in a catalog file. A leading `---` and a line that is exactly `---` separate
/// documents, which is how Backstage writes several entities in one `catalog-info.yaml`.
fn yaml_documents(text: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for part in text.split("\n---") {
        let doc = part.trim().trim_start_matches("---").trim();
        if doc.is_empty() {
            continue;
        }
        if let Ok(value) = serde_norway::from_str::<Value>(doc)
            && value.is_object()
        {
            out.push(value);
        }
    }
    out
}

/// The connector record kind for one Backstage entity, when it is one of the five catalog kinds.
fn catalog_kind(doc: &Value) -> Option<String> {
    let kind = doc.get("kind")?.as_str()?.to_ascii_lowercase();
    let name = doc.pointer("/metadata/name")?.as_str()?;
    if name.is_empty() {
        return None;
    }
    match kind.as_str() {
        "component" | "api" | "system" | "domain" | "resource" => Some(kind),
        _ => None,
    }
}

impl GitFiles {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("SEIKIKA_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("SEIKIKA_CONNECTOR_CONFIG: {e}"))?;
        let s = |k: &str| cfg.get(k).and_then(Value::as_str).map(str::to_owned);
        let list = |k: &str| -> Option<Vec<String>> {
            cfg[k].as_array().map(|a| {
                a.iter()
                    .filter_map(|p| p.as_str().map(str::to_owned))
                    .collect()
            })
        };
        Ok(GitFiles {
            org: s("org").ok_or("config needs `org`")?,
            paths: list("paths").unwrap_or_else(|| vec!["catalog-info.yaml".into()]),
            files: list("files").unwrap_or_default(),
            split: cfg
                .get("splitKinds")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            root: s("root").map(PathBuf::from),
            api: s("apiUrl")
                .or_else(|| std::env::var("GITHUB_API_URL").ok())
                .unwrap_or_else(|| "https://api.github.com".into()),
            token: s("token").or_else(|| std::env::var("GITHUB_TOKEN").ok()),
            http: reqwest::Client::new(),
        })
    }

    fn kinds(&self) -> Vec<String> {
        if !self.split {
            return vec!["catalog-info".into()];
        }
        let mut kinds = vec![
            "component".into(),
            "api".into(),
            "system".into(),
            "domain".into(),
            "resource".into(),
        ];
        if !self.files.is_empty() {
            kinds.push("file".into());
        }
        kinds
    }

    async fn get(&self, path: &str) -> Result<Option<Value>, String> {
        let mut req = self
            .http
            .get(format!("{}{path}", self.api))
            .header("user-agent", "seikika-connector-git-files")
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

    async fn emit_docs(
        &self,
        repo: &str,
        path: &str,
        kinds: &[String],
        out: &mut Out,
    ) -> Result<(), String> {
        let Some(text) = self.read(repo, path).await? else {
            return Ok(());
        };
        for doc in yaml_documents(&text) {
            let Some(kind) = catalog_kind(&doc) else {
                continue;
            };
            if !kinds.iter().any(|k| k == &kind) {
                continue;
            }
            let name = doc["metadata"]["name"].as_str().unwrap_or_default();
            out.record(
                &kind,
                &format!("git:{}/{repo}/{path}#{kind}:{name}", self.org),
                json!({"repo": repo, "path": path, "content": doc}),
            );
        }
        Ok(())
    }

    async fn emit_file(&self, repo: &str, path: &str, out: &mut Out) -> Result<bool, String> {
        if self.read(repo, path).await?.is_none() {
            return Ok(false);
        }
        out.record(
            "file",
            &ext(&self.org, repo, path),
            json!({"repo": repo, "path": path, "present": true}),
        );
        Ok(true)
    }

    fn file_path(id_path: &str) -> &str {
        id_path.split('#').next().unwrap_or(id_path)
    }

    async fn refresh_one(&self, kind: &str, id: &str, out: &mut Out) -> Result<bool, String> {
        let prefix = format!("git:{}/", self.org);
        let Some((repo, raw)) = id
            .strip_prefix(&prefix)
            .and_then(|rest| rest.split_once('/'))
        else {
            return Ok(false);
        };
        let path = Self::file_path(raw);
        if self.split && kind == "file" {
            return Ok(self.emit_file(repo, path, out).await? && id == ext(&self.org, repo, path));
        }
        if self.split {
            let Some(text) = self.read(repo, path).await? else {
                return Ok(false);
            };
            let mut hit = false;
            for doc in yaml_documents(&text) {
                let Some(doc_kind) = catalog_kind(&doc) else {
                    continue;
                };
                if doc_kind != kind {
                    continue;
                }
                let name = doc["metadata"]["name"].as_str().unwrap_or_default();
                let ext_id = format!("git:{}/{repo}/{path}#{doc_kind}:{name}", self.org);
                out.record(
                    &doc_kind,
                    &ext_id,
                    json!({"repo": repo, "path": path, "content": doc}),
                );
                if ext_id == id {
                    hit = true;
                }
            }
            return Ok(hit);
        }
        if kind == "catalog-info" {
            return self.emit(repo, path, out).await;
        }
        Ok(false)
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

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn capabilities(&self) -> Value {
        if !self.split {
            return json!({"refresh": {"catalog-info": ["external_ids"]}, "resync": ["catalog-info"], "max_concurrency": 1});
        }
        let kinds = self.kinds();
        let mut refresh = serde_json::Map::new();
        for kind in &kinds {
            refresh.insert(kind.clone(), json!(["external_ids"]));
        }
        json!({"refresh": refresh, "resync": kinds, "max_concurrency": 1})
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                let requested: Vec<String> = cmd["kinds"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect();
                let kinds = if requested.is_empty() {
                    self.kinds()
                } else {
                    requested
                };
                let repos = self.repos().await?;
                if self.split {
                    let catalog: Vec<String> = kinds
                        .iter()
                        .filter(|k| k.as_str() != "file")
                        .cloned()
                        .collect();
                    for repo in &repos {
                        if !catalog.is_empty() {
                            for path in self.paths.clone() {
                                self.emit_docs(repo, &path, &catalog, out).await?;
                            }
                        }
                        if kinds.iter().any(|k| k == "file") {
                            for path in self.files.clone() {
                                self.emit_file(repo, &path, out).await?;
                            }
                        }
                    }
                    for kind in &kinds {
                        out.snapshot_complete(kind);
                    }
                } else {
                    for repo in &repos {
                        for path in self.paths.clone() {
                            self.emit(repo, &path, out).await?;
                        }
                    }
                    out.snapshot_complete("catalog-info");
                }
                Ok(())
            }
            Some("refresh") => {
                let kind = cmd["kind"].as_str().unwrap_or("catalog-info");
                for id in cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    if !self.refresh_one(kind, id, out).await? {
                        out.not_found(kind, id);
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
            eprintln!("seikika-connector-git-files: {e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_document_catalog_keeps_each_kind() {
        let text = "\
---
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: payments-api
spec:
  owner: group:payments
---
apiVersion: backstage.io/v1alpha1
kind: API
metadata:
  name: payments
spec:
  owner: group:payments
";
        let docs = yaml_documents(text);
        assert_eq!(docs.len(), 2);
        assert_eq!(catalog_kind(&docs[0]).as_deref(), Some("component"));
        assert_eq!(catalog_kind(&docs[1]).as_deref(), Some("api"));
        assert_eq!(docs[0]["metadata"]["name"], "payments-api");
    }

    #[test]
    fn external_ids_name_org_repo_and_path() {
        assert_eq!(
            ext("acme", "web", "catalog-info.yaml"),
            "git:acme/web/catalog-info.yaml"
        );
    }

    #[test]
    fn contents_api_base64_decodes_across_line_breaks_and_padding() {
        // GitHub wraps base64 content at 60 characters and pads with `=`.
        assert_eq!(decode_base64("b3duZXI6\nIHRlYW0tYQ=="), "owner: team-a");
        assert_eq!(decode_base64(""), "");
        assert_eq!(decode_base64("YQ"), "a");
    }
}
