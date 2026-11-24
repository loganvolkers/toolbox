//! The `github-workflow` executor (docs/v1-spec.md §7.5): `workflow_dispatch` with the action's
//! inputs as strings plus `toolbox_run`, asking for the run's id (`return_run_details`). The run
//! is then tracked by that id. A server that answers 204 with no id (GitHub Enterprise Server
//! before the feature) falls back to the `run-name`, which the workflow sets to include
//! `toolbox_run` (`run-name: toolbox ${{ inputs.toolbox_run }}`): exactly one run of that name,
//! on the dispatched ref, created after the dispatch.
//!
//! The token comes from the environment variable named by `token` (default `GITHUB_TOKEN`) and is
//! never written to a run record. `api` points at GitHub Enterprise Server or a test double.

use crate::executor::{Context, Outcome};
use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub struct Workflow {
    pub repo: String,
    pub workflow: String,
    pub git_ref: String,
    pub api: String,
    pub token_env: String,
    /// For a federated dispatch: the child action's name. The workflow then gets `action`,
    /// `inputs` (JSON) and `toolbox_run`, and runs `toolbox run` itself.
    pub federated: Option<String>,
    /// For a federated dispatch: the child's own preview, in this portal's clone of it, which
    /// gives the approvers and effects the grant binds.
    pub preview: Option<Box<crate::executor::Executor>>,
}

/// A tracked workflow run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tracked {
    /// `queued`, `in_progress`, `completed`, …
    pub status: String,
    /// `success`, `failure`, `cancelled`, … once completed.
    pub conclusion: Option<String>,
    pub url: Option<String>,
}

pub(crate) fn block<T: Send + 'static>(
    f: impl std::future::Future<Output = T> + Send + 'static,
) -> Result<T, String> {
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map(|rt| rt.block_on(f))
            .map_err(|e| e.to_string())
    })
    .join()
    .map_err(|_| "http thread panicked".to_string())?
}

impl Workflow {
    pub fn parse(v: &Value) -> Result<Self, String> {
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
        Ok(Self {
            repo: s("repo").ok_or("executor.repo (owner/name) is required")?,
            workflow: s("workflow").ok_or("executor.workflow (file name or id) is required")?,
            git_ref: s("ref").unwrap_or_else(|| "main".into()),
            api: s("api").unwrap_or_else(|| "https://api.github.com".into()),
            token_env: s("token").unwrap_or_else(|| "GITHUB_TOKEN".into()),
            federated: None,
            preview: None,
        })
    }

    fn token(&self) -> Result<String, String> {
        std::env::var(&self.token_env)
            .map_err(|_| format!("set {} to dispatch {}", self.token_env, self.repo))
    }

    /// The inputs GitHub receives: every action input as a string, and `toolbox_run`; for a
    /// federated dispatch, `action`, `inputs` as JSON, `requested_by`, `toolbox_run`, `parent_run`
    /// and `grant` (the parent's signed approvals, empty when there are none).
    pub fn dispatch_inputs(&self, ctx: &Context) -> Value {
        if let Some(action) = &self.federated {
            return json!({"action": action, "inputs": Value::Object(ctx.inputs.clone()).to_string(),
                "requested_by": ctx.actor, "toolbox_run": ctx.run_id.clone().unwrap_or_default(),
                "parent_run": format!("{}/{}", ctx.portal, ctx.run_id.clone().unwrap_or_default()),
                "grant": ctx.grant.clone().unwrap_or_default()});
        }
        let mut m = serde_json::Map::new();
        for (k, v) in &ctx.inputs {
            m.insert(
                k.clone(),
                v.as_str()
                    .map_or_else(|| v.to_string(), str::to_owned)
                    .into(),
            );
        }
        m.insert(
            "toolbox_run".into(),
            ctx.run_id.clone().unwrap_or_default().into(),
        );
        Value::Object(m)
    }

    pub fn dispatch(&self, ctx: &Context) -> Outcome {
        let fail = |e: String| Outcome {
            status: "failed".into(),
            outputs: json!({}),
            error: Some(e),
            progress: vec![],
            child: None,
        };
        let token = match self.token() {
            Ok(t) => t,
            Err(e) => return fail(e),
        };
        let url = format!(
            "{}/repos/{}/actions/workflows/{}/dispatches",
            self.api.trim_end_matches('/'),
            self.repo,
            self.workflow
        );
        let dispatched_at = crate::run::now_secs();
        let post = |body: Value| {
            let (url, token) = (url.clone(), token.clone());
            block(async move {
                let r = reqwest::Client::new()
                    .post(url)
                    .bearer_auth(token)
                    .header("accept", "application/vnd.github+json")
                    .header("user-agent", "toolbox")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                let status = r.status();
                let text = r.text().await.unwrap_or_default();
                Ok::<_, String>((status, text))
            })
        };
        let body = json!({"ref": self.git_ref, "inputs": self.dispatch_inputs(ctx)});
        let mut asked = body.clone();
        asked["return_run_details"] = json!(true);
        let mut res = post(asked);
        if let Ok(Ok((st, text))) = &res
            && st.as_u16() == 422
            && text.contains("return_run_details")
        {
            res = post(body);
        }
        match res {
            Ok(Ok((st, text))) if st.is_success() => {
                let details: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                let mut outputs = json!({"repo": self.repo, "workflow": self.workflow,
                    "ref": self.git_ref, "dispatched_at": dispatched_at});
                if let Some(id) = details["workflow_run_id"].as_u64() {
                    outputs["workflow_run_id"] = json!(id);
                }
                if let Some(u) = details["html_url"].as_str() {
                    outputs["workflow_run"] = json!(u);
                }
                Outcome {
                    status: "dispatched".into(),
                    outputs,
                    error: None,
                    progress: vec![],
                    child: None,
                }
            }
            Ok(Ok((st, text))) => fail(format!("workflow_dispatch answered {st}: {text}")),
            Ok(Err(e)) | Err(e) => fail(format!("workflow_dispatch: {e}")),
        }
    }

    /// Find the run a dispatch started: by the id GitHub returned, else the one run whose name
    /// carries `run_id`, on the dispatched ref, created after the dispatch. `Ok(None)` until
    /// GitHub lists it; an error when more than one run matches.
    pub fn track(&self, run_id: &str, outputs: &Value) -> Result<Option<Tracked>, String> {
        let token = self.token()?;
        let api = self.api.trim_end_matches('/');
        let get = |url: String| {
            let token = token.clone();
            block(async move {
                let r = reqwest::Client::new()
                    .get(url)
                    .bearer_auth(token)
                    .header("accept", "application/vnd.github+json")
                    .header("user-agent", "toolbox")
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                if r.status().as_u16() == 404 {
                    return Ok(Value::Null);
                }
                r.json::<Value>().await.map_err(|e| e.to_string())
            })?
        };
        let tracked = |r: &Value| Tracked {
            status: r["status"].as_str().unwrap_or("queued").into(),
            conclusion: r["conclusion"].as_str().map(str::to_owned),
            url: r["html_url"].as_str().map(str::to_owned),
        };
        if let Some(id) = outputs["workflow_run_id"].as_u64() {
            let r = get(format!("{api}/repos/{}/actions/runs/{id}", self.repo))?;
            return Ok((!r.is_null()).then(|| tracked(&r)));
        }
        let since = outputs["dispatched_at"]
            .as_u64()
            .unwrap_or(0)
            .saturating_sub(60);
        let v = get(format!(
            "{api}/repos/{}/actions/workflows/{}/runs?event=workflow_dispatch&per_page=50",
            self.repo, self.workflow
        ))?;
        let found: Vec<&Value> = v["workflow_runs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| {
                ["name", "display_title"]
                    .iter()
                    .any(|k| r[k].as_str().is_some_and(|n| n.contains(run_id)))
            })
            .filter(|r| r["head_branch"].as_str().is_none_or(|b| b == self.git_ref))
            .filter(|r| {
                r["created_at"]
                    .as_str()
                    .and_then(toolbox_connect::hub::parse_rfc3339)
                    .is_none_or(|t| t >= since)
            })
            .collect();
        match found.as_slice() {
            [] => Ok(None),
            [r] => Ok(Some(tracked(r))),
            _ => Err(format!(
                "{} workflow runs are named for {run_id}; refusing to guess which is this run's",
                found.len()
            )),
        }
    }
}

/// Open a pull request for a pushed branch with the forge's token: `{forge, url}`, or `{forge,
/// error}` when it can't (the branch stays pushed, and the run stays `proposed`).
pub fn open_pull(
    f: &toolbox_model::forge::Resolved,
    head: &str,
    base: &str,
    title: &str,
    body: &str,
) -> Value {
    let Ok(token) = std::env::var(&f.token) else {
        return json!({"forge": "github", "error": format!(
            "set {} to open a pull request; the branch is pushed", f.token)});
    };
    let url = format!("{}/repos/{}/pulls", f.api.trim_end_matches('/'), f.project);
    let req = json!({"title": title, "head": head, "base": base, "body": body});
    let res = block(async move {
        let r = reqwest::Client::new()
            .post(url)
            .bearer_auth(token)
            .header("accept", "application/vnd.github+json")
            .header("user-agent", "toolbox")
            .json(&req)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let st = r.status();
        Ok::<_, String>((st, r.json::<Value>().await.unwrap_or(Value::Null)))
    });
    match res {
        Ok(Ok((st, v))) if st.is_success() => {
            json!({"forge": "github", "url": v["html_url"], "number": v["number"]})
        }
        Ok(Ok((st, v))) => json!({"forge": "github",
            "error": format!("GitHub refused the pull request ({st}): {}", v["message"])}),
        Ok(Err(e)) | Err(e) => json!({"forge": "github", "error": e}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    type Seen = Arc<Mutex<Vec<(String, String)>>>;

    /// A GitHub API double: 204 for a dispatch, the given runs for a list. Records each request.
    fn fake(runs: Value) -> (String, Seen) {
        fake_with(runs, None)
    }

    /// As `fake`, answering a dispatch with `details` (200) when given.
    fn fake_with(runs: Value, details: Option<Value>) -> (String, Seen) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let api = format!("http://{}", l.local_addr().unwrap());
        let seen = Arc::new(Mutex::new(vec![]));
        let log = seen.clone();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let mut r = BufReader::new(s.try_clone().unwrap());
                let mut line = String::new();
                r.read_line(&mut line).unwrap();
                let mut len = 0;
                loop {
                    let mut h = String::new();
                    r.read_line(&mut h).unwrap();
                    if h.trim().is_empty() {
                        break;
                    }
                    if let Some(v) = h.to_ascii_lowercase().strip_prefix("content-length:") {
                        len = v.trim().parse().unwrap();
                    }
                }
                let mut body = vec![0; len];
                r.read_exact(&mut body).unwrap();
                log.lock()
                    .unwrap()
                    .push((line.trim().to_owned(), String::from_utf8(body).unwrap()));
                let reply = if line.starts_with("POST") && details.is_none() {
                    "HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n".to_owned()
                } else {
                    let b = if line.starts_with("POST") {
                        details.clone().unwrap().to_string()
                    } else if line.contains("/actions/runs/") {
                        runs["workflow_runs"][0].to_string()
                    } else {
                        runs.to_string()
                    };
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{b}",
                        b.len()
                    )
                };
                let mut s = s;
                s.write_all(reply.as_bytes()).unwrap();
            }
        });
        (api, seen)
    }

    fn ctx() -> Context {
        Context {
            run_id: Some("run_7".into()),
            portal: "org".into(),
            action: "deploy".into(),
            action_version: "v1".into(),
            entity: json!({"id": "repository:alpha"}),
            inputs: serde_json::from_value(json!({"env": "staging", "replicas": 2})).unwrap(),
            actor: "user:ann".into(),
            approved_by: vec![],
            related: Map::new(),
            grant: None,
            cwd: std::env::temp_dir(),
        }
    }

    #[test]
    fn dispatches_with_string_inputs_and_tracks_by_run_name() {
        let (api, seen) = fake(json!({"workflow_runs": [
            {"name": "toolbox run_6", "status": "completed", "conclusion": "failure"},
            {"display_title": "toolbox run_7", "status": "completed", "conclusion": "success",
             "html_url": "https://example.test/runs/7"}]}));
        let w = Workflow::parse(
            &json!({"repo": "acme/alpha", "workflow": "deploy.yml", "api": api, "token": "PATH"}),
        )
        .unwrap();
        let out = w.dispatch(&ctx());
        assert_eq!(out.status, "dispatched", "{:?}", out.error);
        let (line, body) = seen.lock().unwrap()[0].clone();
        assert_eq!(
            line,
            "POST /repos/acme/alpha/actions/workflows/deploy.yml/dispatches HTTP/1.1"
        );
        let body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            body,
            json!({"ref": "main", "return_run_details": true,
                "inputs": {"env": "staging", "replicas": "2", "toolbox_run": "run_7"}})
        );
        let t = w.track("run_7", &out.outputs).unwrap().unwrap();
        assert_eq!(t.conclusion.as_deref(), Some("success"));
        assert_eq!(t.url.as_deref(), Some("https://example.test/runs/7"));
        assert!(w.track("run_8", &out.outputs).unwrap().is_none());
    }

    #[test]
    fn a_returned_run_id_is_tracked_by_id() {
        let (api, seen) = fake_with(
            json!({"workflow_runs": [{"id": 42, "status": "in_progress"}]}),
            Some(json!({"workflow_run_id": 42, "html_url": "https://example.test/runs/42"})),
        );
        let w = Workflow::parse(
            &json!({"repo": "acme/alpha", "workflow": "deploy.yml", "api": api, "token": "PATH"}),
        )
        .unwrap();
        let out = w.dispatch(&ctx());
        assert_eq!(out.outputs["workflow_run_id"], 42);
        let t = w.track("run_7", &out.outputs).unwrap().unwrap();
        assert_eq!(t.status, "in_progress");
        assert_eq!(
            seen.lock().unwrap()[1].0,
            "GET /repos/acme/alpha/actions/runs/42 HTTP/1.1"
        );
    }

    #[test]
    fn name_matching_refuses_old_runs_other_refs_and_ambiguity() {
        let old = json!({"name": "toolbox run_7", "status": "completed", "conclusion": "success",
            "head_branch": "main", "created_at": "2020-01-01T00:00:00Z"});
        let other_ref = json!({"name": "toolbox run_7", "status": "completed",
            "conclusion": "success", "head_branch": "evil"});
        let (api, _) = fake(json!({"workflow_runs": [old, other_ref]}));
        let w = Workflow::parse(
            &json!({"repo": "acme/alpha", "workflow": "deploy.yml", "api": api, "token": "PATH"}),
        )
        .unwrap();
        let outputs = json!({"dispatched_at": crate::run::now_secs()});
        assert!(w.track("run_7", &outputs).unwrap().is_none());
        let twin = json!({"name": "toolbox run_7", "status": "queued", "head_branch": "main"});
        let (api, _) = fake(json!({"workflow_runs": [twin.clone(), twin]}));
        let w = Workflow { api, ..w };
        assert!(
            w.track("run_7", &outputs)
                .unwrap_err()
                .contains("refusing to guess")
        );
    }

    #[test]
    fn a_federated_dispatch_sends_the_child_action_and_who_asked() {
        let (api, seen) = fake(json!({"workflow_runs": []}));
        let mut w = Workflow::parse(
            &json!({"repo": "acme/beta", "workflow": "toolbox.yml", "ref": "trunk", "api": api, "token": "PATH"}),
        )
        .unwrap();
        w.federated = Some("set-owners".into());
        assert_eq!(w.dispatch(&ctx()).status, "dispatched");
        let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].1).unwrap();
        assert_eq!(body["ref"], "trunk");
        assert_eq!(body["inputs"]["action"], "set-owners");
        assert_eq!(body["inputs"]["requested_by"], "user:ann");
        let inputs: Value =
            serde_json::from_str(body["inputs"]["inputs"].as_str().unwrap()).unwrap();
        assert_eq!(inputs, json!({"env": "staging", "replicas": 2}));
    }

    #[test]
    fn a_missing_token_fails_before_any_request() {
        let w = Workflow::parse(
            &json!({"repo": "a/b", "workflow": "w.yml", "token": "TOOLBOX_TEST_NO_SUCH_VAR"}),
        )
        .unwrap();
        let out = w.dispatch(&ctx());
        assert_eq!(out.status, "failed");
        assert!(out.error.unwrap().contains("TOOLBOX_TEST_NO_SUCH_VAR"));
    }
}
