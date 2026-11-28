//! The `github` connector (in-image, stdio). Configuration arrives as JSON in
//! `SEIKIKA_CONNECTOR_CONFIG` (secrets already resolved by the server):
//! `{"org", "apiVersion"?, "token"? | "appId"+"privateKey"+"installationId", "apiUrl"?}`.
//!
//! Record kinds (shapes match examples/reference-portal/fixtures/github.ndjson):
//! - `repository` `github:repo/<id>`: id, name, archived, default_branch_protected, admins
//! - `codeowners` `github:codeowners/<repo id>`: repo, rules [{pattern, owners}]
//! - `installation` `github:installation/<id>`: id, app_slug, repository_selection, repositories
//! - `app` `github:app/<slug>`: slug, owner, permissions
//! - `workflow-runs` `github:workflow-runs/<repo>/<path>` (with `actions: {repos, branch?, runs?}`):
//!   per workflow, failure rate, reruns, failing streak and run times over its recent runs on one
//!   branch (`runs::stats`)
//!
//! Endpoints are from GitHub's REST description (github/rest-api-description). Listing another
//! app's installation repositories needs a suitable user token (Q14).

mod runs;

use seikika_connect::sdk::{Out, Source, run_stdio};
use serde_json::{Value, json};

struct GitHub {
    http: reqwest::Client,
    api: String,
    org: String,
    version: String,
    token: Option<String>,
    app: Option<(String, String, String)>, // (app id, private key PEM, installation id)
    /// `actions`: repositories whose workflow runs to summarise, the branch, and how many runs.
    actions: Option<(Vec<String>, String, u64)>,
}

impl GitHub {
    fn from_env() -> Result<GitHub, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("SEIKIKA_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("SEIKIKA_CONNECTOR_CONFIG: {e}"))?;
        let s = |k: &str| cfg.get(k).and_then(Value::as_str).map(str::to_owned);
        Ok(GitHub {
            http: reqwest::Client::new(),
            api: s("apiUrl")
                .or_else(|| std::env::var("GITHUB_API_URL").ok())
                .unwrap_or_else(|| "https://api.github.com".into()),
            org: s("org").ok_or("config needs `org`")?,
            version: s("apiVersion").unwrap_or_else(|| "2022-11-28".into()),
            token: s("token").or_else(|| std::env::var("GITHUB_TOKEN").ok()),
            actions: cfg.get("actions").map(|a| {
                let repos = a["repos"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| r.as_str().map(str::to_owned))
                    .collect();
                let branch = a["branch"].as_str().unwrap_or("main").to_owned();
                (
                    repos,
                    branch,
                    a["runs"].as_u64().unwrap_or(50).clamp(1, 100),
                )
            }),
            app: match (s("appId"), s("privateKey"), s("installationId")) {
                (Some(a), Some(k), Some(i)) => Some((a, k, i)),
                _ => None,
            },
        })
    }

    /// An access token: the configured token, or a GitHub App installation token.
    async fn token(&self) -> Result<Option<String>, String> {
        if let Some(t) = &self.token {
            return Ok(Some(t.clone()));
        }
        let Some((app_id, pem, inst)) = &self.app else {
            return Ok(None);
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let claims = json!({"iat": now - 60, "exp": now + 540, "iss": app_id});
        let key = jsonwebtoken::EncodingKey::from_rsa_pem(pem.as_bytes())
            .map_err(|e| format!("privateKey: {e}"))?;
        let jwt = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &key,
        )
        .map_err(|e| e.to_string())?;
        let response = self
            .http
            .post(format!(
                "{}/app/installations/{inst}/access_tokens",
                self.api
            ))
            .bearer_auth(jwt)
            .header("accept", "application/vnd.github+json")
            .header("user-agent", "seikika-connector-github")
            .send()
            .await
            .map_err(|e| format!("request installation token: {e}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "request installation token: HTTP {}",
                response.status().as_u16()
            ));
        }
        let r: Value = response
            .json()
            .await
            .map_err(|e| format!("decode installation token response: {e}"))?;
        let token = r["token"]
            .as_str()
            .filter(|token| !token.is_empty())
            .ok_or("installation token response did not contain a token")?;
        Ok(Some(token.to_owned()))
    }

    /// GET a path; `Ok(None)` on 404.
    async fn get(&self, path: &str) -> Result<Option<Value>, String> {
        let mut req = self
            .http
            .get(format!("{}{path}", self.api))
            .header("accept", "application/vnd.github+json")
            .header("x-github-api-version", &self.version)
            .header("user-agent", "seikika-connector-github");
        if let Some(t) = self.token().await? {
            req = req.bearer_auth(t);
        }
        let resp = req.send().await.map_err(|e| format!("GET {path}: {e}"))?;
        match resp.status().as_u16() {
            404 => Ok(None),
            s if (200..300).contains(&s) => resp
                .json()
                .await
                .map(Some)
                .map_err(|e| format!("GET {path}: {e}")),
            s => Err(format!("GET {path}: HTTP {s}")),
        }
    }

    /// GET every page of a list endpoint (`per_page=100`).
    async fn list(&self, path: &str, key: Option<&str>) -> Result<Vec<Value>, String> {
        let mut out = Vec::new();
        for page in 1..=1000 {
            let sep = if path.contains('?') { '&' } else { '?' };
            let Some(v) = self
                .get(&format!("{path}{sep}per_page=100&page={page}"))
                .await?
            else {
                break;
            };
            let items = match key {
                Some(k) => v[k].as_array().cloned().unwrap_or_default(),
                None => v.as_array().cloned().unwrap_or_default(),
            };
            let n = items.len();
            out.extend(items);
            if n < 100 {
                break;
            }
        }
        Ok(out)
    }

    async fn repository(&self, repo: &Value) -> Result<Value, String> {
        let name = repo["name"].as_str().unwrap_or_default();
        let branch = repo["default_branch"].as_str().unwrap_or("main");
        let protected = self
            .get(&format!("/repos/{}/{name}/branches/{branch}", self.org))
            .await?
            .and_then(|b| b["protected"].as_bool())
            .unwrap_or(false);
        let admins: Vec<Value> = self
            .list(
                &format!(
                    "/repos/{}/{name}/collaborators?permission=admin&affiliation=direct",
                    self.org
                ),
                None,
            )
            .await?
            .into_iter()
            .filter_map(|c| c.get("login").cloned())
            .collect();
        Ok(
            json!({"id": repo["id"], "name": name, "archived": repo["archived"].as_bool().unwrap_or(false),
                  "default_branch_protected": protected, "admins": admins}),
        )
    }

    async fn codeowners(&self, repo: &Value) -> Result<Option<Value>, String> {
        let name = repo["name"].as_str().unwrap_or_default();
        for path in [".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS"] {
            if let Some(file) = self
                .get(&format!("/repos/{}/{name}/contents/{path}", self.org))
                .await?
            {
                let text = decode_base64(file["content"].as_str().unwrap_or_default());
                return Ok(Some(
                    json!({"repo": name, "rules": parse_codeowners(&text)}),
                ));
            }
        }
        Ok(None)
    }

    async fn installations(&self) -> Result<Vec<Value>, String> {
        let mut out = Vec::new();
        for inst in self
            .list(
                &format!("/orgs/{}/installations", self.org),
                Some("installations"),
            )
            .await?
        {
            let id = inst["id"].as_u64().unwrap_or_default();
            let repos: Vec<Value> = self
                .list(
                    &format!("/user/installations/{id}/repositories"),
                    Some("repositories"),
                )
                .await?
                .into_iter()
                .filter_map(|r| r.get("name").cloned())
                .collect();
            out.push(json!({"id": id, "app_slug": inst["app_slug"], "repository_selection": inst["repository_selection"], "repositories": repos}));
        }
        Ok(out)
    }

    async fn app(&self, slug: &str) -> Result<Option<Value>, String> {
        Ok(self.get(&format!("/apps/{slug}")).await?.map(|a| json!({"slug": a["slug"], "owner": a["owner"]["login"], "permissions": a["permissions"]})))
    }

    /// Per workflow of each configured repository: run statistics on the configured branch.
    async fn workflow_runs(&self) -> Result<Vec<(String, Value)>, String> {
        let Some((repos, branch, n)) = &self.actions else {
            return Ok(vec![]);
        };
        let mut out = Vec::new();
        for repo in repos {
            let base = format!("/repos/{}/{repo}/actions", self.org);
            let workflows = self
                .get(&format!("{base}/workflows?per_page=100"))
                .await?
                .map(|v| v["workflows"].as_array().cloned().unwrap_or_default())
                .unwrap_or_default();
            for w in workflows {
                let path = w["path"].as_str().unwrap_or_default().to_owned();
                let runs = self
                    .get(&format!(
                        "{base}/workflows/{}/runs?branch={branch}&per_page={n}",
                        w["id"]
                    ))
                    .await?
                    .map(|v| v["workflow_runs"].as_array().cloned().unwrap_or_default())
                    .unwrap_or_default();
                let mut data = runs::stats(&runs);
                data["repo"] = json!(repo);
                data["path"] = json!(path);
                data["name"] = w["name"].clone();
                data["state"] = w["state"].clone();
                data["branch"] = json!(branch);
                out.push((format!("github:workflow-runs/{repo}/{path}"), data));
            }
        }
        Ok(out)
    }

    async fn emit_kind(
        &self,
        kind: &str,
        ids: Option<&[String]>,
        selector: Option<&Value>,
        out: &mut Out,
    ) -> Result<(), String> {
        let want = |ext: &str| ids.is_none_or(|ids| ids.iter().any(|i| i == ext));
        let mut seen = Vec::new();
        match kind {
            "repository" | "codeowners" => {
                let repos: Vec<Value> = match ids {
                    // Scoped refresh: fetch only the requested repositories by id.
                    Some(ids) => {
                        let mut v = Vec::new();
                        for ext in ids {
                            let id = ext.rsplit('/').next().unwrap_or_default();
                            if let Some(r) = self.get(&format!("/repositories/{id}")).await? {
                                v.push(r);
                            }
                        }
                        v
                    }
                    None => {
                        self.list(&format!("/orgs/{}/repos", self.org), None)
                            .await?
                    }
                };
                for repo in repos {
                    let id = repo["id"].as_u64().unwrap_or_default();
                    if kind == "repository" {
                        let ext = format!("github:repo/{id}");
                        if want(&ext) {
                            out.record("repository", &ext, self.repository(&repo).await?);
                            seen.push(ext);
                        }
                    } else {
                        let ext = format!("github:codeowners/{id}");
                        if want(&ext)
                            && let Some(data) = self.codeowners(&repo).await?
                        {
                            out.record("codeowners", &ext, data);
                            seen.push(ext);
                        }
                    }
                }
            }
            "installation" | "app" => {
                let insts = self.installations().await?;
                if kind == "installation" {
                    for inst in insts {
                        let ext = format!("github:installation/{}", inst["id"]);
                        let selected = selector.is_none_or(|sel| {
                            sel.as_object().is_none_or(|m| {
                                m.iter().all(|(k, v)| {
                                    let field = if k == "app" { "app_slug" } else { k.as_str() };
                                    &inst[field] == v
                                })
                            })
                        });
                        if want(&ext) && selected {
                            out.record("installation", &ext, inst);
                            seen.push(ext);
                        }
                    }
                } else {
                    let mut slugs: Vec<String> = insts
                        .iter()
                        .filter_map(|i| i["app_slug"].as_str().map(str::to_owned))
                        .collect();
                    if let Some(ids) = ids {
                        slugs.extend(
                            ids.iter()
                                .filter_map(|e| e.strip_prefix("github:app/").map(str::to_owned)),
                        );
                    }
                    slugs.sort();
                    slugs.dedup();
                    for slug in slugs {
                        let ext = format!("github:app/{slug}");
                        if want(&ext)
                            && let Some(data) = self.app(&slug).await?
                        {
                            out.record("app", &ext, data);
                            seen.push(ext);
                        }
                    }
                }
            }
            "workflow-runs" => {
                for (ext, data) in self.workflow_runs().await? {
                    if want(&ext) {
                        out.record("workflow-runs", &ext, data);
                        seen.push(ext);
                    }
                }
            }
            other => return Err(format!("unknown record kind {other}")),
        }
        if let Some(ids) = ids {
            for ext in ids.iter().filter(|e| !seen.contains(e)) {
                out.not_found(kind, ext);
            }
        }
        Ok(())
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

/// CODEOWNERS lines → [{pattern, owners}], in file order (the last match wins when evaluating).
fn parse_codeowners(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or_default().trim())
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let mut parts = l.split_whitespace();
            let pattern = parts.next()?;
            let owners: Vec<&str> = parts.collect();
            Some(json!({"pattern": pattern, "owners": owners}))
        })
        .collect()
}

impl Source for GitHub {
    fn name(&self) -> &str {
        "github"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn capabilities(&self) -> Value {
        // `workflow-runs` only when configured (`actions`): capabilities name only kinds this
        // instance can produce, and the conformance kit checks them against the manifest.
        let mut caps = json!({"refresh": {"repository": ["external_ids"], "codeowners": ["external_ids"],
                                          "installation": ["external_ids", "selector"], "app": ["external_ids"]},
                              "resync": ["repository", "codeowners", "installation", "app"], "max_concurrency": 1});
        if self.actions.is_some() {
            caps["refresh"]["workflow-runs"] = json!(["external_ids"]);
            if let Some(r) = caps["resync"].as_array_mut() {
                r.push(json!("workflow-runs"));
            }
        }
        caps
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                for kind in cmd["kinds"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    self.emit_kind(kind, None, None, out).await?;
                    out.snapshot_complete(kind);
                }
                Ok(())
            }
            Some("refresh") => {
                let kind = cmd["kind"].as_str().ok_or("refresh needs kind")?;
                let ids: Option<Vec<String>> = cmd["external_ids"].as_array().map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                });
                self.emit_kind(kind, ids.as_deref(), cmd.get("selector"), out)
                    .await
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
}

#[tokio::main]
async fn main() {
    match GitHub::from_env() {
        Ok(src) => run_stdio(src).await,
        Err(e) => {
            eprintln!("seikika-connector-github: {e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codeowners_and_base64() {
        let rules = parse_codeowners(
            "# owners\n*  @acme/checkout\n/catalog-info.yaml @acme/checkout @acme/platform # infra\n",
        );
        assert_eq!(
            rules,
            vec![
                json!({"pattern": "*", "owners": ["@acme/checkout"]}),
                json!({"pattern": "/catalog-info.yaml", "owners": ["@acme/checkout", "@acme/platform"]})
            ]
        );
        assert_eq!(
            decode_base64("KiBAYWNtZS9wYXltZW50cwo="),
            "* @acme/payments\n"
        );
    }
}
