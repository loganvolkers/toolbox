//! GitHub Actions (docs/v1-spec.md §7.5): `workflow_dispatch` with the run's inputs, asking for
//! the run's id (`return_run_details`). The run is then tracked by that id. A server that answers
//! 204 with no id (GitHub Enterprise Server before the feature) falls back to the `run-name`,
//! which the workflow sets to include `seikika_run` (`run-name: seikika ${{ inputs.seikika_run
//! }}`): exactly one run of that name, on the dispatched ref, created after the dispatch.
//!
//! The token comes from the environment variable named by `token` (default `GITHUB_TOKEN`) and is
//! never written to a run record. `api` points at GitHub Enterprise Server or a test double.

use super::{Ci, CiDriver, Tracked};
use crate::executor::Executor;
use crate::forge::ForgeDriver;
use seikika_model::ci::{CiType, Hosts};
use serde_json::{Value, json};
use std::sync::Arc;

/// A `github-workflow` executor: dispatches a GitHub Actions workflow and follows its run.
#[derive(Debug, Clone)]
pub struct Workflow {
    pub repo: String,
    pub workflow: String,
    pub git_ref: String,
    pub api: String,
    pub token_env: String,
}

/// A `github-workflow` executor.
pub(crate) fn parse(v: &Value, hosts: &Hosts) -> Result<Arc<dyn Executor>, String> {
    Ok(Arc::new(Ci::new(Box::new(Workflow::parse_in(v, hosts)?))))
}

impl Workflow {
    /// `repo`, `workflow`, `ref` (default main), `api` (default api.github.com) and `token` (the
    /// variable holding it, default `GITHUB_TOKEN`).
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

    /// As [`Workflow::parse`], with `ci`, a `github-actions` host, supplying `api` and `token`.
    pub fn parse_in(v: &Value, hosts: &Hosts) -> Result<Self, String> {
        let mut w = Self::parse(v)?;
        if let Some(name) = v.get("ci").and_then(Value::as_str) {
            let t = super::host_target(hosts, name, CiType::GithubActions, &w.repo)?;
            w.api = t.api;
            w.token_env = t.token.unwrap_or_default();
        }
        Ok(w)
    }

    fn token(&self) -> Result<String, String> {
        std::env::var(&self.token_env)
            .map_err(|_| format!("set {} to dispatch {}", self.token_env, self.repo))
    }

    fn request(
        &self,
        method: reqwest::Method,
        url: String,
        token: String,
    ) -> reqwest::RequestBuilder {
        crate::forge::github::Github.auth(crate::http::client().request(method, url), token)
    }
}

impl CiDriver for Workflow {
    fn kind(&self) -> &'static str {
        "github-workflow"
    }

    fn noun(&self) -> &'static str {
        "workflow run"
    }

    fn url_key(&self) -> &'static str {
        "workflow_run"
    }

    fn effect(&self) -> Value {
        json!({"resource": format!("github:{}/actions/workflows/{}", self.repo, self.workflow),
            "operation": "dispatch", "ref": self.git_ref})
    }

    fn dispatch(&self, inputs: Value) -> Result<Value, String> {
        let token = self.token()?;
        let url = format!(
            "{}/repos/{}/actions/workflows/{}/dispatches",
            self.api.trim_end_matches('/'),
            self.repo,
            self.workflow
        );
        let dispatched_at = crate::run::now_secs();
        let post = |body: &Value| {
            crate::http::send(
                self.request(reqwest::Method::POST, url.clone(), token.clone())
                    .json(body),
            )
        };
        let body = json!({"ref": self.git_ref, "inputs": inputs});
        let mut asked = body.clone();
        asked["return_run_details"] = json!(true);
        let mut res = post(&asked);
        if let Ok((st, text)) = &res
            && st.as_u16() == 422
            && text.contains("return_run_details")
        {
            res = post(&body);
        }
        match res {
            Ok((st, text)) if st.is_success() => {
                let details: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                let mut outputs = json!({"repo": self.repo, "workflow": self.workflow,
                    "ref": self.git_ref, "dispatched_at": dispatched_at});
                if let Some(id) = details["workflow_run_id"].as_u64() {
                    outputs["workflow_run_id"] = json!(id);
                }
                if let Some(u) = details["html_url"].as_str() {
                    outputs["workflow_run"] = json!(u);
                }
                Ok(outputs)
            }
            Ok((st, text)) => Err(format!("workflow_dispatch answered {st}: {text}")),
            Err(e) => Err(format!("workflow_dispatch: {e}")),
        }
    }

    /// Find the run a dispatch started: by the id GitHub returned, else the one run whose name
    /// carries `run_id`, on the dispatched ref, created after the dispatch. `Ok(None)` until
    /// GitHub lists it; an error when more than one run matches.
    fn track(&self, run_id: &str, outputs: &Value) -> Result<Option<Tracked>, String> {
        let token = self.token()?;
        let api = self.api.trim_end_matches('/');
        let get = |url: String| -> Result<Value, String> {
            let (st, text) =
                crate::http::send(self.request(reqwest::Method::GET, url, token.clone()))?;
            if st.as_u16() == 404 {
                return Ok(Value::Null);
            }
            serde_json::from_str(&text).map_err(|e| e.to_string())
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
                    .and_then(seikika_connect::hub::parse_rfc3339)
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

    /// Cancel the run by the id the dispatch returned; a run found by name has none recorded.
    fn cancel(&self, outputs: &Value) -> Result<String, String> {
        let id = outputs["workflow_run_id"]
            .as_u64()
            .ok_or("GitHub returned no workflow run id, so cancel the run on GitHub")?;
        let url = format!(
            "{}/repos/{}/actions/runs/{id}/cancel",
            self.api.trim_end_matches('/'),
            self.repo
        );
        match crate::http::send(self.request(reqwest::Method::POST, url, self.token()?))? {
            (st, _) if st.is_success() => Ok(format!("cancelled workflow run {id}")),
            (st, text) => Err(format!(
                "cancelling workflow run {id} answered {st}: {text}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::fake;
    use crate::executor::Context;
    use serde_json::Map;

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

    /// A GitHub API double: 204 for a dispatch, the given runs for a list. With `details`, a
    /// dispatch answers them (200), and a run by id is the first run.
    fn fake(runs: Value, details: Option<Value>) -> (String, fake::Seen) {
        let dispatch = details.unwrap_or(Value::Null);
        let code = if dispatch.is_null() { 204 } else { 200 };
        fake::serve(
            "",
            vec![
                ("POST /dispatches", code, dispatch),
                ("/actions/runs/42", 200, runs["workflow_runs"][0].clone()),
                ("/runs", 200, runs),
            ],
        )
    }

    fn workflow(api: &str) -> Ci {
        Ci::new(Box::new(
            Workflow::parse(
                &json!({"repo": "acme/alpha", "workflow": "deploy.yml", "api": api, "token": "PATH"}),
            )
            .unwrap(),
        ))
    }

    #[test]
    fn dispatches_with_string_inputs_and_tracks_by_run_name() {
        let (api, seen) = fake(
            json!({"workflow_runs": [
            {"name": "seikika run_6", "status": "completed", "conclusion": "failure"},
            {"display_title": "seikika run_7", "status": "completed", "conclusion": "success",
             "html_url": "https://example.test/runs/7"}]}),
            None,
        );
        let w = workflow(&api);
        let out = w.run(&ctx(), &[]);
        assert_eq!(out.status, "dispatched", "{:?}", out.error);
        let (line, token, body) = seen.lock().unwrap()[0].clone();
        assert_eq!(
            line,
            "POST /repos/acme/alpha/actions/workflows/deploy.yml/dispatches HTTP/1.1"
        );
        assert_eq!(token, std::env::var("PATH").unwrap());
        let body: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(
            body,
            json!({"ref": "main", "return_run_details": true,
                "inputs": {"env": "staging", "replicas": "2", "seikika_run": "run_7"}})
        );
        let t = w.driver.track("run_7", &out.outputs).unwrap().unwrap();
        assert_eq!(t.conclusion.as_deref(), Some("success"));
        assert_eq!(t.url.as_deref(), Some("https://example.test/runs/7"));
        assert!(w.driver.track("run_8", &out.outputs).unwrap().is_none());
    }

    #[test]
    fn a_returned_run_id_is_tracked_by_id() {
        let (api, seen) = fake(
            json!({"workflow_runs": [{"id": 42, "status": "in_progress"}]}),
            Some(json!({"workflow_run_id": 42, "html_url": "https://example.test/runs/42"})),
        );
        let w = workflow(&api);
        let out = w.run(&ctx(), &[]);
        assert_eq!(out.outputs["workflow_run_id"], 42);
        let t = w.driver.track("run_7", &out.outputs).unwrap().unwrap();
        assert_eq!(t.status, "in_progress");
        assert_eq!(
            seen.lock().unwrap()[1].0,
            "GET /repos/acme/alpha/actions/runs/42 HTTP/1.1"
        );
    }

    #[test]
    fn name_matching_refuses_old_runs_other_refs_and_ambiguity() {
        let old = json!({"name": "seikika run_7", "status": "completed", "conclusion": "success",
            "head_branch": "main", "created_at": "2020-01-01T00:00:00Z"});
        let other_ref = json!({"name": "seikika run_7", "status": "completed",
            "conclusion": "success", "head_branch": "evil"});
        let (api, _) = fake(json!({"workflow_runs": [old, other_ref]}), None);
        let w = Workflow::parse(
            &json!({"repo": "acme/alpha", "workflow": "deploy.yml", "api": api, "token": "PATH"}),
        )
        .unwrap();
        let outputs = json!({"dispatched_at": crate::run::now_secs()});
        assert!(w.track("run_7", &outputs).unwrap().is_none());
        let twin = json!({"name": "seikika run_7", "status": "queued", "head_branch": "main"});
        let (api, _) = fake(json!({"workflow_runs": [twin.clone(), twin]}), None);
        let w = Workflow { api, ..w };
        assert!(
            w.track("run_7", &outputs)
                .unwrap_err()
                .contains("refusing to guess")
        );
    }

    #[test]
    fn a_federated_dispatch_sends_the_child_action_and_who_asked() {
        let (api, seen) = fake(json!({"workflow_runs": []}), None);
        let mut w = Ci::new(Box::new(
            Workflow::parse(
                &json!({"repo": "acme/beta", "workflow": "seikika.yml", "ref": "trunk", "api": api, "token": "PATH"}),
            )
            .unwrap(),
        ));
        w.federated = Some("set-owners".into());
        assert_eq!(w.run(&ctx(), &[]).status, "dispatched");
        let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].2).unwrap();
        assert_eq!(body["ref"], "trunk");
        assert_eq!(body["inputs"]["action"], "set-owners");
        assert_eq!(body["inputs"]["requested_by"], "user:ann");
        let inputs: Value =
            serde_json::from_str(body["inputs"]["inputs"].as_str().unwrap()).unwrap();
        assert_eq!(inputs, json!({"env": "staging", "replicas": 2}));
    }

    #[test]
    fn cancel_posts_to_the_recorded_run_and_refuses_without_one() {
        let (api, seen) = fake::serve("", vec![("POST /actions/runs/42/cancel", 202, json!({}))]);
        let w = workflow(&api);
        assert_eq!(
            w.driver.cancel(&json!({"workflow_run_id": 42})),
            Ok("cancelled workflow run 42".into())
        );
        assert_eq!(
            seen.lock().unwrap()[0].0,
            "POST /repos/acme/alpha/actions/runs/42/cancel HTTP/1.1"
        );
        assert!(
            w.driver
                .cancel(&json!({}))
                .unwrap_err()
                .contains("on GitHub")
        );
        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    #[test]
    fn a_missing_token_fails_before_any_request() {
        let w = Ci::new(Box::new(
            Workflow::parse(
                &json!({"repo": "a/b", "workflow": "w.yml", "token": "SEIKIKA_TEST_NO_SUCH_VAR"}),
            )
            .unwrap(),
        ));
        let out = w.run(&ctx(), &[]);
        assert_eq!(out.status, "failed");
        assert!(out.error.unwrap().contains("SEIKIKA_TEST_NO_SUCH_VAR"));
    }
}
