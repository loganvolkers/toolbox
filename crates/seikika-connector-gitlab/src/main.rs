//! The `gitlab` connector (in-image, stdio): a GitLab group's subgroups with their members, and
//! its projects, so an org on GitLab manages people in one place (docs/design/forges.md §4.6).
//! Configuration arrives as JSON in `SEIKIKA_CONNECTOR_CONFIG` (secrets already resolved):
//! `{"group": "acme", "url"?: "https://gitlab.acme.dev", "apiUrl"?, "token"?, "minAccess"?}`.
//! `url` defaults to `https://gitlab.com` and `apiUrl` to `<url>/api/v4`; the token defaults to
//! `GITLAB_TOKEN` and needs `read_api`. `minAccess` (default 30, Developer) is the lowest access
//! level counted as a member. Declare both kinds (`records: [group, project]`): the conformance
//! kit checks the advertised capabilities against them.
//!
//! Record kinds:
//! - `group` `gitlab:group/<full path>`: slug (the full path), name, parent (the parent's full
//!   path, absent at the top), members (usernames of direct members at `minAccess` or above,
//!   sorted), access (username → access level). GitLab's inheritance runs down: a parent's
//!   members are also members of its subgroups, so a model that wants that maps `parent`.
//! - `project` `gitlab:project/<id>`: path (with its namespace), name, default_branch, archived,
//!   web_url.
//!
//! Endpoints: `GET /groups/:id`, `/groups/:id/descendant_groups`, `/groups/:id/members`,
//! `/groups/:id/projects?include_subgroups=true`, `/projects/:id` (`doc/api` of GitLab 19.6).

use seikika_connect::sdk::{Out, Source, run_stdio};
use serde_json::{Value, json};

struct GitLab {
    http: reqwest::Client,
    api: String,
    group: String,
    token: Option<String>,
    min_access: u64,
}

/// A path in one URL segment: `acme/platform` is `acme%2Fplatform`.
fn encode(path: &str) -> String {
    path.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

impl GitLab {
    fn from_config(cfg: &Value, env_token: Option<String>) -> Result<GitLab, String> {
        let s = |k: &str| cfg.get(k).and_then(Value::as_str).map(str::to_owned);
        let url = s("url").unwrap_or_else(|| "https://gitlab.com".into());
        Ok(GitLab {
            http: reqwest::Client::new(),
            api: s("apiUrl").unwrap_or_else(|| format!("{}/api/v4", url.trim_end_matches('/'))),
            group: s("group").ok_or("config needs `group` (a GitLab group's full path)")?,
            token: s("token").filter(|t| !t.is_empty()).or(env_token),
            min_access: cfg.get("minAccess").and_then(Value::as_u64).unwrap_or(30),
        })
    }

    /// GET a path; `Ok(None)` on 404.
    async fn get(&self, path: &str) -> Result<Option<Value>, String> {
        let mut req = self.http.get(format!("{}{path}", self.api));
        if let Some(t) = &self.token {
            req = req.header("PRIVATE-TOKEN", t);
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
    async fn list(&self, path: &str) -> Result<Vec<Value>, String> {
        let mut out = Vec::new();
        for page in 1..=1000 {
            let sep = if path.contains('?') { '&' } else { '?' };
            let Some(Value::Array(items)) = self
                .get(&format!("{path}{sep}per_page=100&page={page}"))
                .await?
            else {
                break;
            };
            let n = items.len();
            out.extend(items);
            if n < 100 {
                break;
            }
        }
        Ok(out)
    }

    async fn group_record(&self, g: &Value) -> Result<Value, String> {
        let path = g["full_path"].as_str().unwrap_or_default();
        let members = self
            .list(&format!("/groups/{}/members", encode(path)))
            .await?;
        Ok(group_data(g, &members, self.min_access))
    }

    async fn emit(&self, kind: &str, ids: Option<&[String]>, out: &mut Out) -> Result<(), String> {
        let top = encode(&self.group);
        match (kind, ids) {
            ("group", None) => {
                let root = self.get(&format!("/groups/{top}")).await?.ok_or_else(|| {
                    format!("no GitLab group {} (or the token can't see it)", self.group)
                })?;
                let mut groups = vec![root];
                groups.extend(
                    self.list(&format!("/groups/{top}/descendant_groups"))
                        .await?,
                );
                for g in &groups {
                    let ext = format!(
                        "gitlab:group/{}",
                        g["full_path"].as_str().unwrap_or_default()
                    );
                    out.record("group", &ext, self.group_record(g).await?);
                }
            }
            ("group", Some(ids)) => {
                for ext in ids {
                    let path = ext.strip_prefix("gitlab:group/").unwrap_or(ext);
                    match self.get(&format!("/groups/{}", encode(path))).await? {
                        Some(g) => out.record("group", ext, self.group_record(&g).await?),
                        None => out.not_found("group", ext),
                    }
                }
            }
            ("project", None) => {
                for p in self
                    .list(&format!(
                        "/groups/{top}/projects?include_subgroups=true&archived=false"
                    ))
                    .await?
                {
                    out.record(
                        "project",
                        &format!("gitlab:project/{}", p["id"]),
                        project_data(&p),
                    );
                }
            }
            ("project", Some(ids)) => {
                for ext in ids {
                    let id = ext.rsplit('/').next().unwrap_or_default();
                    match self.get(&format!("/projects/{}", encode(id))).await? {
                        Some(p) => out.record("project", ext, project_data(&p)),
                        None => out.not_found("project", ext),
                    }
                }
            }
            (other, _) => return Err(format!("unknown record kind {other:?}")),
        }
        Ok(())
    }
}

/// A group record from GitLab's group and its direct members.
fn group_data(g: &Value, members: &[Value], min_access: u64) -> Value {
    let path = g["full_path"].as_str().unwrap_or_default();
    let mut access = serde_json::Map::new();
    let mut names: Vec<String> = Vec::new();
    for m in members {
        let (Some(u), Some(level)) = (m["username"].as_str(), m["access_level"].as_u64()) else {
            continue;
        };
        if m["state"].as_str().is_some_and(|s| s != "active") {
            continue;
        }
        access.insert(u.into(), json!(level));
        if level >= min_access {
            names.push(u.into());
        }
    }
    names.sort();
    let mut data = json!({"slug": path, "name": g["name"], "members": names, "access": access});
    if let Some((parent, _)) = path.rsplit_once('/') {
        data["parent"] = json!(parent);
    }
    data
}

fn project_data(p: &Value) -> Value {
    json!({"path": p["path_with_namespace"], "name": p["name"], "default_branch": p["default_branch"],
           "archived": p["archived"].as_bool().unwrap_or(false), "web_url": p["web_url"]})
}

impl Source for GitLab {
    fn name(&self) -> &str {
        "gitlab"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn capabilities(&self) -> Value {
        json!({"refresh": {"group": ["external_ids"], "project": ["external_ids"]},
               "resync": ["group", "project"], "max_concurrency": 1})
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
                    self.emit(kind, None, out).await?;
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
                self.emit(kind, ids.as_deref(), out).await
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
}

#[tokio::main]
async fn main() {
    let cfg: Value =
        match std::env::var("SEIKIKA_CONNECTOR_CONFIG").map(|c| serde_json::from_str(&c)) {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => {
                eprintln!("seikika-connector-gitlab: SEIKIKA_CONNECTOR_CONFIG: {e}");
                std::process::exit(2);
            }
            Err(_) => json!({}),
        };
    match GitLab::from_config(&cfg, std::env::var("GITLAB_TOKEN").ok()) {
        Ok(src) => run_stdio(src).await,
        Err(e) => {
            eprintln!("seikika-connector-gitlab: {e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_lists_active_members_at_the_minimum_access_with_its_parent() {
        let g = json!({"full_path": "acme/release", "name": "Release"});
        let members = [
            json!({"username": "erin", "access_level": 30, "state": "active"}),
            json!({"username": "dave", "access_level": 40, "state": "active"}),
            json!({"username": "guest", "access_level": 10, "state": "active"}),
            json!({"username": "gone", "access_level": 50, "state": "blocked"}),
        ];
        assert_eq!(
            group_data(&g, &members, 30),
            json!({"slug": "acme/release", "name": "Release", "members": ["dave", "erin"],
                   "access": {"erin": 30, "dave": 40, "guest": 10}, "parent": "acme"})
        );
        let top = group_data(&json!({"full_path": "acme", "name": "Acme"}), &[], 30);
        assert!(top.get("parent").is_none());
    }

    #[test]
    fn config_defaults_to_gitlab_com_and_the_environment_token() {
        let c = GitLab::from_config(&json!({"group": "acme"}), Some("t".into())).unwrap();
        assert_eq!(
            (c.api.as_str(), c.token.as_deref(), c.min_access),
            ("https://gitlab.com/api/v4", Some("t"), 30)
        );
        let c = GitLab::from_config(
            &json!({"group": "acme", "url": "http://127.0.0.1:8929/", "token": "x"}),
            None,
        )
        .unwrap();
        assert_eq!(
            (c.api.as_str(), c.token.as_deref()),
            ("http://127.0.0.1:8929/api/v4", Some("x"))
        );
        assert!(GitLab::from_config(&json!({}), None).is_err());
        assert_eq!(encode("acme/platform"), "acme%2Fplatform");
    }
}
