//! The `github-workflow` executor (docs/v1-spec.md §7.5): `workflow_dispatch` with the action's
//! inputs as strings plus `toolbox_run`, then the run is found by its `run-name`, which the
//! workflow sets to include `toolbox_run` (`run-name: toolbox ${{ inputs.toolbox_run }}`).
//! GitHub's dispatch API answers 204 with no run id, so the name is the only join.
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

fn block<T: Send + 'static>(
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
        })
    }

    fn token(&self) -> Result<String, String> {
        std::env::var(&self.token_env)
            .map_err(|_| format!("set {} to dispatch {}", self.token_env, self.repo))
    }

    /// The inputs GitHub receives: every action input as a string, and `toolbox_run`; for a
    /// federated dispatch, `action`, `inputs` as JSON, `requested_by` and `toolbox_run`.
    pub fn dispatch_inputs(&self, ctx: &Context) -> Value {
        if let Some(action) = &self.federated {
            return json!({"action": action, "inputs": Value::Object(ctx.inputs.clone()).to_string(),
                "requested_by": ctx.actor, "toolbox_run": ctx.run_id.clone().unwrap_or_default()});
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
        let body = json!({"ref": self.git_ref, "inputs": self.dispatch_inputs(ctx)});
        let res = block(async move {
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
        });
        match res {
            Ok(Ok((st, _))) if st.is_success() => Outcome {
                status: "dispatched".into(),
                outputs: json!({"repo": self.repo, "workflow": self.workflow, "ref": self.git_ref}),
                error: None,
                progress: vec![],
                child: None,
            },
            Ok(Ok((st, text))) => fail(format!("workflow_dispatch answered {st}: {text}")),
            Ok(Err(e)) | Err(e) => fail(format!("workflow_dispatch: {e}")),
        }
    }

    /// Find the dispatched run whose name carries `run_id`. `Ok(None)` until GitHub lists it.
    pub fn track(&self, run_id: &str) -> Result<Option<Tracked>, String> {
        let token = self.token()?;
        let url = format!(
            "{}/repos/{}/actions/workflows/{}/runs?event=workflow_dispatch&per_page=50",
            self.api.trim_end_matches('/'),
            self.repo,
            self.workflow
        );
        let v: Value = block(async move {
            reqwest::Client::new()
                .get(url)
                .bearer_auth(token)
                .header("accept", "application/vnd.github+json")
                .header("user-agent", "toolbox")
                .send()
                .await
                .map_err(|e| e.to_string())?
                .json::<Value>()
                .await
                .map_err(|e| e.to_string())
        })??;
        Ok(v["workflow_runs"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|r| {
                ["name", "display_title"]
                    .iter()
                    .any(|k| r[k].as_str().is_some_and(|n| n.contains(run_id)))
            })
            .map(|r| Tracked {
                status: r["status"].as_str().unwrap_or("queued").into(),
                conclusion: r["conclusion"].as_str().map(str::to_owned),
                url: r["html_url"].as_str().map(str::to_owned),
            }))
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
                let reply = if line.starts_with("POST") {
                    "HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n".to_owned()
                } else {
                    let b = runs.to_string();
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
            related: Map::new(),
            approvals: vec![],
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
            json!({"ref": "main", "inputs": {"env": "staging", "replicas": "2", "toolbox_run": "run_7"}})
        );
        let t = w.track("run_7").unwrap().unwrap();
        assert_eq!(t.conclusion.as_deref(), Some("success"));
        assert_eq!(t.url.as_deref(), Some("https://example.test/runs/7"));
        assert!(w.track("run_8").unwrap().is_none());
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
