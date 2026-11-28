//! Read organization-owned scorecard inputs from GitHub's immutable Git objects.
//! Published child snapshots remain separate from parent policy. No repository is checked out.

use crate::repo;
use reqwest::{Client, RequestBuilder, Response, StatusCode};
use seikika_connect::github_app::Installation;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::Duration;

pub(crate) const WORKFLOW_KIND: &str = "federated-workflow";
pub(crate) const KIND: &str = "federated-repo";
const API: &str = "https://api.github.com";
const API_VERSION: &str = "2022-11-28";
const JSON_LIMIT: usize = 10 * 1024 * 1024;
const README_LIMIT: usize = 1024 * 1024;
const LICENSE_LIMIT: usize = 256 * 1024;
const CODEOWNERS_LIMIT: usize = 3 * 1024 * 1024;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Config {
    organization: String,
    #[serde(default)]
    public_only: bool,
    #[serde(default)]
    pub(crate) workflows: bool,
    #[serde(default)]
    reports: bool,
    #[serde(default)]
    app: Installation,
    token: Option<String>,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
    id: Option<u64>,
    default_branch: String,
    visibility: String,
    fork: bool,
    archived: bool,
}

#[derive(Deserialize)]
struct Commit {
    sha: String,
    commit: CommitData,
}

#[derive(Deserialize)]
struct CommitData {
    tree: ObjectId,
}

#[derive(Deserialize)]
struct ObjectId {
    sha: String,
}

#[derive(Deserialize)]
struct Tree {
    truncated: bool,
    tree: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    path: String,
    mode: String,
    #[serde(rename = "type")]
    kind: String,
    sha: String,
    size: Option<usize>,
}

#[derive(Default)]
struct RepositoryTree {
    files: BTreeMap<String, Entry>,
    configured: bool,
    workflows: Option<Entry>,
    report: Option<Entry>,
}

impl Entry {
    fn directory(&self) -> bool {
        self.mode == "040000" && self.kind == "tree"
    }

    fn regular(&self) -> bool {
        matches!(self.mode.as_str(), "100644" | "100755")
            && self.kind == "blob"
            && !self.path.chars().any(char::is_control)
    }
}

fn segment(value: &str) -> bool {
    !matches!(value, "" | "." | "..")
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.'))
}

fn object_id(value: &str) -> Result<&str, String> {
    if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err("invalid Git object ID".into())
    }
}

impl Config {
    pub(crate) fn from_value(value: &Value) -> Result<Option<Self>, String> {
        if value.is_null() {
            return Ok(None);
        }
        let config: Self =
            serde_json::from_value(value.clone()).map_err(|error| format!("githubOrg: {error}"))?;
        if !segment(&config.organization) {
            return Err("githubOrg.organization must be a GitHub organization name".into());
        }
        Ok(Some(config))
    }

    pub(crate) async fn scan(&self) -> Result<BTreeMap<String, Value>, String> {
        let client = Client::builder()
            .user_agent("seikika-federation")
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(25))
            .build()
            .map_err(|_| "could not build GitHub client")?;
        self.scan_with(client, API).await
    }

    async fn scan_with(
        &self,
        client: Client,
        base: &str,
    ) -> Result<BTreeMap<String, Value>, String> {
        let token = match self.app.token(&client, base, API_VERSION).await? {
            Some(token) => Some(token),
            None => match self
                .token
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                Some(token) => Some(token.to_owned()),
                None => existing_token().await,
            },
        };
        if !self.public_only && token.is_none() {
            return Err("GitHub federation needs a GitHub App, gh auth login, GH_TOKEN or GITHUB_TOKEN to inventory visible repositories".into());
        }
        let api = Api {
            client,
            base: base.to_owned(),
            token,
        };
        api.scan(self).await
    }
}

async fn existing_token() -> Option<String> {
    for name in ["GH_TOKEN", "GITHUB_TOKEN"] {
        if let Ok(value) = std::env::var(name)
            && !value.trim().is_empty()
        {
            return Some(value.trim().to_owned());
        }
    }
    let mut command = tokio::process::Command::new("gh");
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        command
            .args(["auth", "token", "--hostname", "github.com"])
            .kill_on_drop(true)
            .output(),
    )
    .await
    .ok()?
    .ok()?;
    let token = String::from_utf8(output.stdout).ok()?;
    (output.status.success() && !token.trim().is_empty()).then(|| token.trim().to_owned())
}

#[derive(Clone)]
struct Api {
    client: Client,
    base: String,
    token: Option<String>,
}

impl Api {
    fn get(&self, path: &str, accept: &str) -> RequestBuilder {
        let request = self
            .client
            .get(format!("{}{path}", self.base))
            .header(reqwest::header::ACCEPT, accept)
            .header("X-GitHub-Api-Version", API_VERSION);
        match &self.token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }

    async fn scan(&self, config: &Config) -> Result<BTreeMap<String, Value>, String> {
        let repositories = self.inventory(config).await?;
        let mut records = BTreeMap::new();
        let mut jobs = tokio::task::JoinSet::new();
        for repository in repositories {
            let api = self.clone();
            let (workflows, reports) = (config.workflows, config.reports);
            jobs.spawn(async move {
                let result = api.read(&repository, workflows, reports).await;
                repository.record(result)
            });
            if jobs.len() == 6 {
                let (name, record) = jobs.join_next().await.unwrap().map_err(|e| e.to_string())?;
                records.insert(name, record);
            }
        }
        while let Some(result) = jobs.join_next().await {
            let (name, record) = result.map_err(|e| e.to_string())?;
            records.insert(name, record);
        }
        if config.public_only {
            // A repository may become private while an authenticated read is in flight.
            let current = self.inventory(config).await?;
            if current.len() != records.len()
                || current.iter().any(|repo| {
                    records
                        .get(&repo.full_name)
                        .is_none_or(|record| record["githubId"].as_u64() != repo.id)
                })
            {
                return Err("public GitHub inventory changed during the scan; retry".into());
            }
        }
        Ok(records)
    }

    async fn inventory(&self, config: &Config) -> Result<Vec<Repository>, String> {
        let path = format!("/orgs/{}/repos", config.organization);
        let visibility = if config.public_only { "public" } else { "all" };
        let mut repositories = BTreeMap::new();
        for page in 1.. {
            let response = send(
                self.get(&path, "application/vnd.github+json").query(&[
                    ("type", visibility),
                    ("per_page", "100"),
                    ("page", &page.to_string()),
                ]),
                "organization inventory",
            )
            .await?;
            let more = response
                .headers()
                .get(reqwest::header::LINK)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|links| links.split(',').any(|link| link.contains("rel=\"next\"")));
            let entries: Vec<Repository> = read_json(response, "organization inventory").await?;
            for repository in entries {
                let identity = repository.full_name.split_once('/');
                if !identity.is_some_and(|(org, name)| {
                    org.eq_ignore_ascii_case(&config.organization) && segment(name)
                }) || !matches!(
                    repository.visibility.as_str(),
                    "public" | "private" | "internal"
                ) {
                    return Err("GitHub returned an invalid organization inventory entry".into());
                }
                if config.public_only && repository.visibility != "public" {
                    continue;
                }
                let key = repository.full_name.to_ascii_lowercase();
                if repositories.insert(key, repository).is_some() {
                    return Err(
                        "GitHub organization inventory returned a duplicate repository".into(),
                    );
                }
            }
            if !more {
                return Ok(repositories.into_values().collect());
            }
        }
        unreachable!()
    }

    async fn default_commit(
        &self,
        repository: &Repository,
        prefix: &str,
    ) -> Result<Option<Commit>, String> {
        let response = send(
            self.get(&format!("{prefix}/commits"), "application/vnd.github+json")
                .query(&[
                    ("sha", repository.default_branch.as_str()),
                    ("per_page", "1"),
                ]),
            "default branch",
        )
        .await?;
        if response.status() == StatusCode::CONFLICT {
            let body = bytes(response, JSON_LIMIT).await?;
            let problem: Value =
                serde_json::from_slice(&body).map_err(|_| "invalid GitHub error")?;
            if problem["message"] == "Git Repository is empty." {
                return Ok(None);
            }
            return Err("default branch: HTTP 409".into());
        }
        let commits: Vec<Commit> = read_json(response, "default branch").await?;
        let commit = commits
            .into_iter()
            .next()
            .ok_or("default branch has no commit")?;
        object_id(&commit.sha)?;
        Ok(Some(commit))
    }

    async fn repository_tree(&self, prefix: &str, tree: &str) -> Result<RepositoryTree, String> {
        let mut result = RepositoryTree::default();
        for entry in self.tree(prefix, tree).await? {
            if entry.regular() {
                result.files.insert(entry.path.clone(), entry);
            } else if entry.directory()
                && matches!(
                    entry.path.as_str(),
                    ".github" | "docs" | ".seikika" | "portal"
                )
            {
                for file in self.tree(prefix, &entry.sha).await? {
                    match (entry.path.as_str(), file.path.as_str()) {
                        (".github", "workflows") if file.directory() => {
                            result.workflows = Some(file)
                        }
                        (".seikika", "report.json") if file.regular() => result.report = Some(file),
                        (".seikika" | "portal", "seikika.yaml") if file.regular() => {
                            result.configured = true
                        }
                        (".github" | "docs", "CODEOWNERS") if file.regular() => {
                            result
                                .files
                                .insert(format!("{}/CODEOWNERS", entry.path), file);
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(result)
    }

    async fn read(
        &self,
        repository: &Repository,
        workflows: bool,
        reports: bool,
    ) -> Result<(Option<String>, bool, Value), String> {
        let prefix = format!("/repos/{}", repository.full_name);
        let Some(commit) = self.default_commit(repository, &prefix).await? else {
            let facts = repo::basics(&repository.full_name, &[], |_| Ok::<_, String>(None))?;
            return Ok((None, false, facts));
        };
        let tree = self
            .repository_tree(&prefix, &commit.commit.tree.sha)
            .await?;
        let files = tree.files;
        let paths: Vec<_> = files.keys().cloned().collect();
        let mut texts = BTreeMap::new();
        for (path, limit) in [
            (repo::readme_path(&paths), README_LIMIT),
            (repo::codeowners_path(&paths), CODEOWNERS_LIMIT),
        ] {
            if let Some(path) = path
                && let Some(text) = self.text(&prefix, &files[path], limit).await?
            {
                texts.insert(path, text);
            }
        }
        for path in repo::license_paths(&paths) {
            if let Some(text) = self.text(&prefix, &files[path], LICENSE_LIMIT).await?
                && !text.trim().is_empty()
            {
                texts.insert(path, text);
                break;
            }
        }
        let mut facts = repo::basics(&repository.full_name, &paths, |path| {
            Ok::<_, String>(texts.remove(path))
        })?;
        if workflows && let Some(workflow_tree) = tree.workflows {
            match self
                .read_workflows(repository, &prefix, &commit.sha, &workflow_tree.sha)
                .await
            {
                Ok(rows) => facts["workflows"] = json!(rows),
                Err(error) => facts["workflowError"] = json!(error),
            }
        }
        if reports && let Some(report) = tree.report {
            let result = match self
                .text(&prefix, &report, crate::portal_report::LIMIT as usize)
                .await
            {
                Ok(Some(text)) => crate::portal_report::decode(text.as_bytes()),
                _ => Err("Unable to read the published child report"),
            };
            crate::portal_report::attach(&mut facts, result);
        }
        Ok((Some(commit.sha), tree.configured, facts))
    }

    async fn read_workflows(
        &self,
        repo: &Repository,
        prefix: &str,
        commit: &str,
        tree: &str,
    ) -> Result<Vec<Value>, String> {
        let mut rows = Vec::new();
        for file in self.tree(prefix, tree).await? {
            if !file.regular() || !(file.path.ends_with(".yml") || file.path.ends_with(".yaml")) {
                continue;
            }
            let path = format!(".github/workflows/{}", file.path);
            let mut source = reqwest::Url::parse("https://github.com/").expect("static GitHub URL");
            source
                .path_segments_mut()
                .expect("GitHub base URL")
                .pop_if_empty()
                .extend(repo.full_name.split('/'))
                .push("blob")
                .push(commit)
                .extend(path.split('/'));
            let audited = match self.text(prefix, &file, 512 * 1024).await {
                Ok(Some(text)) => crate::org_workflows::audit(&text)
                    .await
                    .map_err(str::to_owned),
                Ok(None) => Err("Workflow is not bounded UTF-8".into()),
                Err(error) => Err(error),
            };
            let (findings, error) = match audited {
                Ok(findings) => (Some(findings), None),
                Err(error) => (None, Some(error)),
            };
            rows.push(json!({
                "id": format!("workflow:{}/{}", repo.full_name.to_ascii_lowercase(), path),
                "title": file.path, "path": path, "repository": repo.full_name,
                "sourceUrl": source.as_str(),
                "commit": commit, "visibility": repo.visibility,
                "audited": findings.is_some(), "findings": findings.unwrap_or_default(), "auditError": error,
            }));
        }
        Ok(rows)
    }

    async fn tree(&self, prefix: &str, sha: &str) -> Result<Vec<Entry>, String> {
        let path = format!("{prefix}/git/trees/{}", object_id(sha)?);
        let response = send(self.get(&path, "application/vnd.github+json"), "Git tree").await?;
        let result: Tree = read_json(response, "Git tree").await?;
        if result.truncated {
            return Err("GitHub truncated a Git tree".into());
        }
        Ok(result.tree)
    }

    async fn text(
        &self,
        prefix: &str,
        blob: &Entry,
        limit: usize,
    ) -> Result<Option<String>, String> {
        let size = blob.size.ok_or("Git blob has no size")?;
        if size > limit {
            return Ok(None);
        }
        let path = format!("{prefix}/git/blobs/{}", object_id(&blob.sha)?);
        let response = send(
            self.get(&path, "application/vnd.github.raw+json"),
            "Git blob",
        )
        .await?;
        let body = bytes(success(response, "Git blob")?, limit).await?;
        if body.len() != size {
            return Err("Git blob length does not match the Git tree".into());
        }
        Ok(String::from_utf8(body).ok())
    }
}

impl Repository {
    fn record(self, result: Result<(Option<String>, bool, Value), String>) -> (String, Value) {
        let (commit, configured, mut facts, error) = match result {
            Ok((commit, configured, facts)) => (commit, Some(configured), facts, None),
            Err(error) => (
                None,
                None,
                repo::basics(&self.full_name, &[], |_| {
                    Ok::<_, std::convert::Infallible>(None)
                })
                .unwrap(),
                Some(error),
            ),
        };
        let url = format!("https://github.com/{}", self.full_name);
        facts["name"] = json!(self.full_name);
        facts["identity"] = json!(self.full_name.to_ascii_lowercase());
        facts["githubId"] = json!(self.id);
        facts["repositoryUrl"] = json!(url);
        facts["sourceUrl"] = json!(commit.as_ref().map(|sha| format!("{url}/tree/{sha}")));
        facts["commit"] = json!(commit);
        facts["visibility"] = json!(self.visibility);
        facts["fork"] = json!(self.fork);
        facts["archived"] = json!(self.archived);
        facts["defaultBranch"] = json!(self.default_branch);
        facts["portalPresent"] = json!(configured);
        facts["scanned"] = json!(error.is_none());
        facts["scanError"] = json!(error);
        (self.full_name, facts)
    }
}

async fn send(request: RequestBuilder, what: &str) -> Result<Response, String> {
    request.send().await.map_err(|error| {
        let reason = if error.is_timeout() {
            "timed out"
        } else {
            "request failed"
        };
        format!("{what}: {reason}")
    })
}

fn success(response: Response, what: &str) -> Result<Response, String> {
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(format!("{what}: HTTP {}", response.status().as_u16()))
    }
}

async fn read_json<T: serde::de::DeserializeOwned>(
    response: Response,
    what: &str,
) -> Result<T, String> {
    let body = bytes(success(response, what)?, JSON_LIMIT).await?;
    serde_json::from_slice(&body).map_err(|_| format!("{what}: invalid GitHub response"))
}

async fn bytes(mut response: Response, limit: usize) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|len| len > limit as u64)
    {
        return Err("GitHub response exceeds the permitted size".into());
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| "could not read GitHub response")?
    {
        if chunk.len() > limit - body.len() {
            return Err("GitHub response exceeds the permitted size".into());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    struct Reply {
        status: &'static str,
        headers: &'static str,
        body: String,
    }

    impl Reply {
        fn json(value: Value) -> Self {
            Self {
                body: value.to_string(),
                ..Self::text("")
            }
        }

        fn text(text: &str) -> Self {
            Self {
                status: "200 OK",
                headers: "",
                body: text.into(),
            }
        }
    }

    async fn fixture(
        routes: BTreeMap<String, Reply>,
    ) -> (Api, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&requests);
        let task = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    request.push(socket.read_u8().await.unwrap());
                }
                let request = String::from_utf8(request).unwrap();
                let mut request_line = request.lines().next().unwrap().split_whitespace();
                let method = request_line.next().unwrap();
                let path = request_line.next().unwrap();
                let authorization = request
                    .lines()
                    .find_map(|line| {
                        let (header, value) = line.split_once(": ")?;
                        header
                            .eq_ignore_ascii_case("authorization")
                            .then_some(value)
                    })
                    .unwrap();
                if path == "/app/installations/456/access_tokens" {
                    assert_eq!(method, "POST");
                    assert_eq!(
                        authorization
                            .strip_prefix("Bearer ")
                            .unwrap()
                            .split('.')
                            .count(),
                        3
                    );
                } else {
                    assert_eq!(method, "GET");
                    assert_eq!(authorization, "Bearer fixture");
                }
                observed.lock().unwrap().push(path.to_owned());
                let unknown = Reply {
                    status: "500 Unexpected",
                    ..Reply::text("")
                };
                let response = routes.get(path).unwrap_or(&unknown);
                let message = format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                    response.status,
                    response.body.len(),
                    response.headers,
                    response.body
                );
                socket.write_all(message.as_bytes()).await.unwrap();
            }
        });
        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        (
            Api {
                client,
                base,
                token: Some("fixture".into()),
            },
            requests,
            task,
        )
    }

    fn repository(name: &str, visibility: &str, branch: &str) -> Value {
        json!({"full_name": format!("acme/{name}"), "default_branch": branch,
               "visibility": visibility, "fork": true, "archived": false})
    }

    fn tree(entries: Vec<Value>) -> Reply {
        Reply::json(json!({"truncated": false, "tree": entries}))
    }

    fn sha(c: char) -> String {
        c.to_string().repeat(40)
    }

    fn commit() -> Reply {
        Reply::json(json!([{"sha": sha('a'), "commit": {"tree": {"sha": sha('b')}}}]))
    }

    fn item(path: &str, mode: &str, kind: &str, hash: char, size: usize) -> Value {
        json!({"path": path, "mode": mode, "type": kind, "sha": sha(hash), "size": size})
    }

    #[tokio::test]
    async fn reads_pinned_repository_facts_and_isolates_bad_downstreams() {
        let mut routes = BTreeMap::from([
            (
                "/orgs/acme/repos?type=all&per_page=100&page=1".into(),
                Reply {
                    headers: "Link: <http://other.invalid/pagination>; rel=\"next\"\r\n",
                    ..Reply::json(json!([
                        repository("claims", "public", "release/current"),
                        repository("inaccessible", "private", "main"),
                    ]))
                },
            ),
            (
                "/orgs/acme/repos?type=all&per_page=100&page=2".into(),
                Reply::json(json!([
                    repository("oversized", "public", "main"),
                    repository("empty", "private", "main"),
                ])),
            ),
            (
                "/repos/acme/claims/commits?sha=release%2Fcurrent&per_page=1".into(),
                commit(),
            ),
            (
                "/repos/acme/inaccessible/commits?sha=main&per_page=1".into(),
                commit(),
            ),
            (
                "/repos/acme/oversized/commits?sha=main&per_page=1".into(),
                commit(),
            ),
            (
                "/repos/acme/empty/commits?sha=main&per_page=1".into(),
                Reply {
                    status: "409 Conflict",
                    ..Reply::json(json!({"message": "Git Repository is empty."}))
                },
            ),
        ]);
        let readme = "literal <federation-marker> O'Reilly";
        let owners = "* @acme/platform\n";
        routes.extend([
            (
                format!("/repos/acme/claims/git/trees/{}", sha('b')),
                tree(vec![
                    item("README-\nextra-line.md", "100644", "blob", '8', 19),
                    item(
                        "README-<federation-marker>.md",
                        "100644",
                        "blob",
                        'c',
                        readme.len(),
                    ),
                    item("LICENSE", "120000", "blob", 'd', 18),
                    item(".github", "040000", "tree", 'e', 0),
                    item(".seikika", "040000", "tree", 'f', 0),
                ]),
            ),
            (
                format!("/repos/acme/claims/git/trees/{}", sha('e')),
                tree(vec![
                    item("CODEOWNERS", "100644", "blob", '1', owners.len()),
                    item("workflows", "040000", "tree", '2', 0),
                ]),
            ),
            (
                format!("/repos/acme/claims/git/trees/{}", sha('f')),
                tree(vec![
                    item("seikika.yaml", "100644", "blob", '3', 255),
                    item("results.json", "100644", "blob", '4', 255),
                ]),
            ),
            (
                format!("/repos/acme/claims/git/blobs/{}", sha('c')),
                Reply::text(readme),
            ),
            (
                format!("/repos/acme/claims/git/blobs/{}", sha('1')),
                Reply::text(owners),
            ),
            (
                format!("/repos/acme/oversized/git/trees/{}", sha('b')),
                tree(vec![
                    item("README.md", "100644", "blob", '5', README_LIMIT + 1),
                    item("CODEOWNERS", "100644", "blob", '6', CODEOWNERS_LIMIT + 1),
                    item("large-asset.bin", "100644", "blob", '7', 32 * 1024 * 1024),
                ]),
            ),
            (
                format!("/repos/acme/inaccessible/git/trees/{}", sha('b')),
                Reply::json(json!({"truncated": true, "tree": []})),
            ),
        ]);
        let (api, requests, server) = fixture(routes).await;
        let records = api
            .scan(&Config {
                organization: "acme".into(),
                public_only: false,
                workflows: false,
                reports: false,
                app: Installation::default(),
                token: None,
            })
            .await
            .unwrap();
        assert_eq!(records.len(), 4);
        let claims = &records["acme/claims"];
        assert_eq!(claims["commit"], sha('a'));
        assert_eq!(claims["portalPresent"], true);
        assert_eq!(claims["visibility"], "public");
        assert_eq!(claims["readmeWords"], 3);
        assert_eq!(claims["license"], Value::Null);
        assert_eq!(claims["licenseHasContent"], false);
        assert_eq!(claims["readmeEntrypoint"], false);
        assert_eq!(claims["owners"], json!(["@acme/platform"]));
        assert_eq!(records["acme/oversized"]["scanned"], true);
        assert_eq!(records["acme/oversized"]["readmeWords"], 0);
        assert_eq!(records["acme/oversized"]["codeowners"], Value::Null);
        assert_eq!(records["acme/empty"]["scanned"], true);
        assert_eq!(records["acme/empty"]["commit"], Value::Null);
        assert_eq!(records["acme/inaccessible"]["scanned"], false);
        assert_eq!(
            records["acme/inaccessible"]["scanError"],
            "GitHub truncated a Git tree"
        );
        let requests = requests.lock().unwrap();
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.contains("/git/blobs/"))
                .count(),
            2
        );
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.contains("/git/trees/"))
                .count(),
            5
        );
        server.abort();
    }

    #[tokio::test]
    async fn public_mode_filters_private_rows_before_reading_any_git_objects() {
        let path = "/orgs/acme/repos?type=public&per_page=100&page=1";
        let (api, requests, server) = fixture(BTreeMap::from([(
            path.into(),
            Reply::json(json!([repository("private", "private", "main")])),
        )]))
        .await;
        let records = api
            .scan(&Config {
                organization: "acme".into(),
                public_only: true,
                workflows: false,
                reports: false,
                app: Installation::default(),
                token: None,
            })
            .await
            .unwrap();
        assert!(records.is_empty());
        assert_eq!(*requests.lock().unwrap(), [path, path]);
        server.abort();
    }

    #[tokio::test]
    async fn uses_one_installation_token_per_public_scan() {
        let exchange = "/app/installations/456/access_tokens";
        let inventory = "/orgs/acme/repos?type=public&per_page=100&page=1";
        let (api, requests, server) = fixture(BTreeMap::from([
            (exchange.into(), Reply::json(json!({"token": "fixture"}))),
            (
                inventory.into(),
                Reply::json(json!([repository("private", "private", "main")])),
            ),
        ]))
        .await;
        let config = Config::from_value(&json!({
            "organization": "acme", "publicOnly": true, "token": "legacy",
            "app": {"appId": "123", "installationId": "456",
                "privateKey": include_str!("../../seikika-authz/tests/fixtures/test-rsa.pem")}
        }))
        .unwrap()
        .unwrap();
        assert!(
            config
                .scan_with(api.client, &api.base)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(*requests.lock().unwrap(), [exchange, inventory, inventory]);
        server.abort();
    }

    #[tokio::test]
    async fn invalid_inventory_does_not_yield_a_partial_snapshot() {
        let (api, _, server) = fixture(BTreeMap::from([(
            "/orgs/acme/repos?type=all&per_page=100&page=1".into(),
            Reply::json(json!([
                {"full_name": "other/foreign", "default_branch": "main",
                 "visibility": "public", "fork": false, "archived": false},
            ])),
        )]))
        .await;
        let result = api
            .scan(&Config {
                organization: "acme".into(),
                public_only: false,
                workflows: false,
                reports: false,
                app: Installation::default(),
                token: None,
            })
            .await;
        assert_eq!(
            result.unwrap_err(),
            "GitHub returned an invalid organization inventory entry"
        );
        server.abort();
    }
}
