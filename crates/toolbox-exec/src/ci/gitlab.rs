//! GitLab CI (docs/design/forges.md §4.4): *dispatch* creates a pipeline with typed `inputs`,
//! never pipeline variables, which gitlab.com refuses on new projects by default; *track* follows
//! it by the id the create call returned; *collect* reads a federated child's run record from the
//! artifact of the job named `toolbox`. The token comes from the environment variable the forge
//! names (default `GITLAB_TOKEN`), is sent as `PRIVATE-TOKEN`, and is never written to a run
//! record.

use super::{Ci, CiDriver, Tracked};
use crate::executor::Executor;
use crate::forge::ForgeDriver;
use crate::forge::gitlab::{Gitlab, encode, reason};
use serde_json::{Value, json};
use std::sync::Arc;
use toolbox_model::forge::{self, Kind};
use toolbox_model::manifest::Forge;

/// The job whose artifact holds the child's run record.
pub const JOB: &str = "toolbox";
/// The run record's file name in that artifact.
pub const RECORD: &str = "toolbox-run.json";

/// A project's pipeline, started on its forge with the forge's token.
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// `group/subgroup/project`.
    pub project: String,
    pub git_ref: String,
    pub api: String,
    pub token_env: String,
}

/// A `gitlab-pipeline` executor: `project`, `ref` (default main), and the forge's `url` as
/// `forges:` lists it (default https://gitlab.com), whose API and token it uses.
pub(crate) fn parse(v: &Value, forges: &[Forge]) -> Result<Arc<dyn Executor>, String> {
    Ok(Arc::new(Ci::new(Box::new(Pipeline::parse(v, forges)?))))
}

impl Pipeline {
    /// Reads a `gitlab-pipeline` executor, resolving its `forge:` against the manifest's forges.
    pub fn parse(v: &Value, forges: &[Forge]) -> Result<Self, String> {
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
        let project = s("project").ok_or("executor.project (group/project) is required")?;
        let url = s("forge").unwrap_or_else(|| "https://gitlab.com".into());
        let git = format!("{}/{project}.git", url.trim_end_matches('/'));
        let f = forge::resolve(forges, &git)
            .filter(|f| f.kind == Kind::Gitlab)
            .ok_or_else(|| {
                format!("{url} is not a GitLab forge: list it under forges: with type: gitlab")
            })?;
        Ok(Self {
            project: f.project,
            git_ref: s("ref").unwrap_or_else(|| "main".into()),
            api: f.api,
            token_env: f.token,
        })
    }

    fn token(&self) -> Result<String, String> {
        std::env::var(&self.token_env).map_err(|_| {
            format!(
                "set {} to start pipelines in {}",
                self.token_env, self.project
            )
        })
    }

    fn base(&self) -> String {
        format!(
            "{}/projects/{}",
            self.api.trim_end_matches('/'),
            encode(&self.project)
        )
    }

    fn get(&self, path: &str) -> Result<Option<Value>, String> {
        let url = format!("{}{path}", self.base());
        let req = Gitlab.auth(crate::http::client().get(&url), self.token()?);
        let (st, text) = crate::http::send(req)?;
        match st.as_u16() {
            404 => Ok(None),
            s if (200..300).contains(&s) => serde_json::from_str(&text)
                .map(Some)
                .map_err(|e| format!("GET {url}: {e}")),
            s => Err(format!("GET {url}: HTTP {s}: {text}")),
        }
    }
}

impl CiDriver for Pipeline {
    fn kind(&self) -> &'static str {
        "gitlab-pipeline"
    }

    fn noun(&self) -> &'static str {
        "pipeline"
    }

    fn url_key(&self) -> &'static str {
        "pipeline"
    }

    fn effect(&self) -> Value {
        json!({"resource": format!("gitlab:{}/pipelines", self.project),
            "operation": "dispatch", "ref": self.git_ref})
    }

    /// Starts the pipeline; the outputs carry its id.
    fn dispatch(&self, inputs: Value) -> Result<Value, String> {
        let token = self.token()?;
        let url = format!("{}/pipeline", self.base());
        let body = json!({"ref": self.git_ref, "inputs": inputs});
        let dispatched_at = crate::run::now_secs();
        match crate::http::send(
            Gitlab
                .auth(crate::http::client().post(url), token)
                .json(&body),
        ) {
            Ok((st, text)) if st.is_success() => {
                let p: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                let Some(id) = p["id"].as_u64() else {
                    return Err(format!(
                        "GitLab created a pipeline but returned no id: {text}"
                    ));
                };
                Ok(json!({"project": self.project, "ref": self.git_ref,
                    "pipeline_id": id, "pipeline": p["web_url"], "dispatched_at": dispatched_at}))
            }
            Ok((st, text)) => Err(format!(
                "GitLab refused the pipeline for {} on {} ({st}): {}",
                self.project,
                self.git_ref,
                reason(&text)
            )),
            Err(e) => Err(format!("creating a pipeline in {}: {e}", self.project)),
        }
    }

    /// The pipeline's state, in GitHub's terms: `completed` with a conclusion, or `queued`,
    /// `in_progress` or `waiting` (a manual job).
    fn track(&self, _: &str, outputs: &Value) -> Result<Option<Tracked>, String> {
        let id = outputs["pipeline_id"]
            .as_u64()
            .ok_or("the run records no pipeline id")?;
        let Some(p) = self.get(&format!("/pipelines/{id}"))? else {
            return Ok(None);
        };
        let st = p["status"].as_str().unwrap_or("created");
        let (status, conclusion) = match st {
            "success" => ("completed", Some("success")),
            "failed" => ("completed", Some("failure")),
            "canceled" | "canceling" => ("completed", Some("cancelled")),
            "skipped" => ("completed", Some("skipped")),
            "manual" | "scheduled" => ("waiting", None),
            "running" => ("in_progress", None),
            _ => ("queued", None),
        };
        Ok(Some(Tracked {
            status: status.into(),
            conclusion: conclusion.map(str::to_owned),
            url: p["web_url"].as_str().map(str::to_owned),
        }))
    }

    fn waits_for(&self) -> Option<&'static str> {
        Some("someone to start a manual job")
    }

    fn no_record(&self) -> Option<String> {
        Some(format!("no job named {JOB} kept {RECORD}"))
    }

    /// The child's run record from the `toolbox` job's artifact; `Ok(None)` when the pipeline has
    /// no such job, or the job kept no record.
    fn collect(&self, outputs: &Value) -> Result<Option<Value>, String> {
        let id = outputs["pipeline_id"]
            .as_u64()
            .ok_or("the run records no pipeline id")?;
        let jobs = self
            .get(&format!("/pipelines/{id}/jobs?per_page=100"))?
            .unwrap_or(Value::Null);
        let Some(job) = jobs
            .as_array()
            .into_iter()
            .flatten()
            .filter(|j| j["name"] == JOB)
            .max_by_key(|j| j["id"].as_u64())
            .and_then(|j| j["id"].as_u64())
        else {
            return Ok(None);
        };
        self.get(&format!("/jobs/{job}/artifacts/{RECORD}"))
    }

    fn cancel(&self, outputs: &Value) -> Result<String, String> {
        let id = outputs["pipeline_id"]
            .as_u64()
            .ok_or("the run records no pipeline id")?;
        let url = format!("{}/pipelines/{id}/cancel", self.base());
        match crate::http::send(Gitlab.auth(crate::http::client().post(url), self.token()?))? {
            (st, _) if st.is_success() => Ok(format!("cancelled pipeline {id}")),
            (st, text) => Err(format!(
                "cancelling pipeline {id} ({st}): {}",
                reason(&text)
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

    fn fake(routes: Vec<(&'static str, u16, Value)>) -> (String, fake::Seen) {
        fake::serve("/api/v4", routes)
    }

    fn ctx() -> Context {
        Context {
            run_id: Some("run_7".into()),
            portal: "org".into(),
            action: "deploy".into(),
            action_version: "v1".into(),
            entity: json!({"id": "portal:pay"}),
            inputs: serde_json::from_value(json!({"env": "staging", "replicas": 2})).unwrap(),
            actor: "user:ann".into(),
            approved_by: vec![],
            related: Map::new(),
            grant: Some("g".into()),
            cwd: std::env::temp_dir(),
        }
    }

    fn forges(api: &str) -> Vec<Forge> {
        vec![Forge {
            url: "https://gitlab.acme.dev".into(),
            kind: Some("gitlab".into()),
            api: Some(api.into()),
            token: Some("PATH".into()),
            projects: None,
            identity: false,
        }]
    }

    #[test]
    fn creates_a_pipeline_with_inputs_and_tracks_it_by_id() {
        let (api, seen) = fake(vec![
            (
                "/projects/acme%2Fplatform%2Fpay/pipeline",
                201,
                json!({"id": 9, "status": "created", "web_url": "https://gitlab.acme.dev/acme/platform/pay/-/pipelines/9"}),
            ),
            (
                "/projects/acme%2Fplatform%2Fpay/pipelines/9",
                200,
                json!({"id": 9, "status": "running"}),
            ),
        ]);
        let p = Ci::new(Box::new(
            Pipeline::parse(
                &json!({"project": "acme/platform/pay", "forge": "https://gitlab.acme.dev", "ref": "trunk"}),
                &forges(&api),
            )
            .unwrap(),
        ));
        let out = p.run(&ctx(), &[]);
        assert_eq!(out.status, "dispatched", "{:?}", out.error);
        assert_eq!(out.outputs["pipeline_id"], 9);
        let (line, token, body) = seen.lock().unwrap()[0].clone();
        assert_eq!(
            line,
            "POST /api/v4/projects/acme%2Fplatform%2Fpay/pipeline HTTP/1.1"
        );
        assert_eq!(token, std::env::var("PATH").unwrap());
        assert_eq!(
            serde_json::from_str::<Value>(&body).unwrap(),
            json!({"ref": "trunk", "inputs": {"env": "staging", "replicas": "2", "toolbox_run": "run_7"}})
        );
        let t = p.driver.track("run_7", &out.outputs).unwrap().unwrap();
        assert_eq!((t.status.as_str(), t.conclusion), ("in_progress", None));
    }

    #[test]
    fn a_federated_pipeline_gets_the_child_action_and_its_record_is_collected() {
        let record = json!({"id": "run_c1", "state": "succeeded", "verification": {"by": "base"}});
        let (api, seen) = fake(vec![
            ("/projects/acme%2Fpay/pipeline", 201, json!({"id": 3})),
            (
                "/projects/acme%2Fpay/pipelines/3/jobs",
                200,
                json!([{"id": 11, "name": "lint"}, {"id": 12, "name": "toolbox"}]),
            ),
            (
                "/projects/acme%2Fpay/jobs/12/artifacts/toolbox-run.json",
                200,
                record.clone(),
            ),
        ]);
        let mut p = Ci::new(Box::new(
            Pipeline::parse(
                &json!({"project": "acme/pay", "forge": "https://gitlab.acme.dev"}),
                &forges(&api),
            )
            .unwrap(),
        ));
        p.federated = Some("set-owners".into());
        let out = p.run(&ctx(), &[]);
        let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].2).unwrap();
        assert_eq!(body["inputs"]["action"], "set-owners");
        assert_eq!(body["inputs"]["parent_run"], "org/run_7");
        assert_eq!(body["inputs"]["grant"], "g");
        assert_eq!(p.driver.collect(&out.outputs).unwrap(), Some(record));
    }

    #[test]
    fn a_refused_pipeline_says_why_and_a_missing_token_sends_nothing() {
        let (api, seen) = fake(vec![(
            "/projects/acme%2Fpay/pipeline",
            400,
            json!({"message": {"base": ["Unknown input: replicas"]}}),
        )]);
        let p = Pipeline::parse(
            &json!({"project": "acme/pay", "forge": "https://gitlab.acme.dev"}),
            &forges(&api),
        )
        .unwrap();
        let out = Ci::new(Box::new(p.clone())).run(&ctx(), &[]);
        assert_eq!(out.status, "failed");
        assert!(out.error.unwrap().contains("Unknown input: replicas"));
        let p = Pipeline {
            token_env: "TOOLBOX_TEST_NO_SUCH_VAR".into(),
            ..p
        };
        let n = seen.lock().unwrap().len();
        assert!(
            Ci::new(Box::new(p))
                .run(&ctx(), &[])
                .error
                .unwrap()
                .contains("TOOLBOX_TEST_NO_SUCH_VAR")
        );
        assert_eq!(seen.lock().unwrap().len(), n);
    }

    #[test]
    fn cancel_posts_to_the_recorded_pipeline() {
        let (api, seen) = fake(vec![(
            "POST /projects/acme%2Fpay/pipelines/3/cancel",
            200,
            json!({"id": 3, "status": "canceled"}),
        )]);
        let p = Pipeline::parse(
            &json!({"project": "acme/pay", "forge": "https://gitlab.acme.dev"}),
            &forges(&api),
        )
        .unwrap();
        assert_eq!(
            p.cancel(&json!({"pipeline_id": 3})),
            Ok("cancelled pipeline 3".into())
        );
        assert_eq!(
            seen.lock().unwrap()[0].0,
            "POST /api/v4/projects/acme%2Fpay/pipelines/3/cancel HTTP/1.1"
        );
        assert!(p.cancel(&json!({})).is_err());
    }

    #[test]
    fn only_a_gitlab_forge_takes_a_pipeline() {
        assert!(Pipeline::parse(&json!({"project": "a/b"}), &[]).is_ok());
        let e = Pipeline::parse(
            &json!({"project": "a/b", "forge": "https://git.example"}),
            &[],
        )
        .unwrap_err();
        assert!(e.contains("forges:"));
        let e = Pipeline::parse(
            &json!({"project": "a/b", "forge": "https://github.com"}),
            &[],
        )
        .unwrap_err();
        assert!(e.contains("not a GitLab forge"));
    }
}
