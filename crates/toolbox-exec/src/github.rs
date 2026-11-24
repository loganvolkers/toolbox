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
        })
    }

    fn token(&self) -> Result<String, String> {
        std::env::var(&self.token_env)
            .map_err(|_| format!("set {} to dispatch {}", self.token_env, self.repo))
    }

    /// The inputs GitHub receives: every action input as a string, and `toolbox_run`.
    pub fn dispatch_inputs(ctx: &Context) -> Value {
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
        let body = json!({"ref": self.git_ref, "inputs": Self::dispatch_inputs(ctx)});
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
