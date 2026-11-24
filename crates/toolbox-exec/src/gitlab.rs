//! The GitLab driver (docs/design/forges.md §4.4): *dispatch* creates a pipeline with typed
//! `inputs`, never pipeline variables, which gitlab.com refuses on new projects by default;
//! *track* follows it by the id the create call returned; *collect* reads the child's run record
//! from the artifact of the job named `toolbox`. The token comes from the environment variable
//! the forge names (default `GITLAB_TOKEN`), is sent as `PRIVATE-TOKEN`, and is never written to
//! a run record.
//!
//! The `gitlab-pipeline` executor is the same pipeline for a repo portal's own action: each
//! action input as a string input, plus `toolbox_run`.

use crate::executor::{Context, Outcome};
use crate::github::{Tracked, block};
use serde_json::{Value, json};
use std::path::PathBuf;
use toolbox_model::forge::{self, Kind};
use toolbox_model::manifest::Forge;

/// The job whose artifact holds the child's run record.
pub const JOB: &str = "toolbox";
/// The run record's file name in that artifact.
pub const RECORD: &str = "toolbox-run.json";

/// A child project's pipeline, started on its forge with the parent's token.
#[derive(Debug, Clone)]
pub struct Pipeline {
    /// `group/subgroup/project`.
    pub project: String,
    pub git_ref: String,
    pub api: String,
    pub token_env: String,
    /// For a federated dispatch: the child action's name. The pipeline then gets `action`,
    /// `inputs` (JSON), `requested_by`, `toolbox_run`, `parent_run` and `grant`.
    pub federated: Option<String>,
    /// For a federated dispatch: the parent's clone of the child, where a change the child
    /// proposed is looked for on its base.
    pub clone: Option<PathBuf>,
    /// For a federated dispatch: the child's own preview, in that clone, which gives the
    /// approvers and effects the grant binds.
    pub preview: Option<Box<crate::executor::Executor>>,
}

/// A project path in a URL path segment: `acme/pay` is `acme%2Fpay`.
fn encode(project: &str) -> String {
    project
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

impl Pipeline {
    /// A `gitlab-pipeline` executor: `project`, `ref` (default main), and the forge's `url` as
    /// `forges:` lists it (default https://gitlab.com), whose API and token it uses.
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
            federated: None,
            clone: None,
            preview: None,
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

    /// The inputs the pipeline receives. All are strings: a pipeline declares each in
    /// `spec:inputs`, and GitLab refuses one it doesn't declare.
    pub fn inputs(&self, ctx: &Context) -> Value {
        let run = ctx.run_id.clone().unwrap_or_default();
        if let Some(action) = &self.federated {
            return json!({"action": action, "inputs": Value::Object(ctx.inputs.clone()).to_string(),
                "requested_by": ctx.actor, "toolbox_run": run,
                "parent_run": format!("{}/{run}", ctx.portal),
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
        m.insert("toolbox_run".into(), run.into());
        Value::Object(m)
    }

    fn get(&self, path: &str) -> Result<Option<Value>, String> {
        let (url, token) = (format!("{}{path}", self.base()), self.token()?);
        block(async move {
            let r = reqwest::Client::new()
                .get(&url)
                .header("private-token", token)
                .header("user-agent", "toolbox")
                .send()
                .await
                .map_err(|e| e.to_string())?;
            match r.status().as_u16() {
                404 => Ok(None),
                s if (200..300).contains(&s) => {
                    let text = r.text().await.map_err(|e| e.to_string())?;
                    serde_json::from_str(&text)
                        .map(Some)
                        .map_err(|e| format!("GET {url}: {e}"))
                }
                s => Err(format!(
                    "GET {url}: HTTP {s}: {}",
                    r.text().await.unwrap_or_default()
                )),
            }
        })?
    }

    /// Starts the child's pipeline with the run's six inputs; `dispatched` carries its id.
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
        let url = format!("{}/pipeline", self.base());
        let body = json!({"ref": self.git_ref, "inputs": self.inputs(ctx)});
        let dispatched_at = crate::run::now_secs();
        let res = block(async move {
            let r = reqwest::Client::new()
                .post(url)
                .header("private-token", token)
                .header("user-agent", "toolbox")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let status = r.status();
            Ok::<_, String>((status, r.text().await.unwrap_or_default()))
        });
        match res {
            Ok(Ok((st, text))) if st.is_success() => {
                let p: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                let Some(id) = p["id"].as_u64() else {
                    return fail(format!(
                        "GitLab created a pipeline but returned no id: {text}"
                    ));
                };
                Outcome {
                    status: "dispatched".into(),
                    outputs: json!({"project": self.project, "ref": self.git_ref,
                        "pipeline_id": id, "pipeline": p["web_url"], "dispatched_at": dispatched_at}),
                    error: None,
                    progress: vec![],
                    child: None,
                }
            }
            Ok(Ok((st, text))) => fail(format!(
                "GitLab refused the pipeline for {} on {} ({st}): {}",
                self.project,
                self.git_ref,
                pipeline_error(&text)
            )),
            Ok(Err(e)) | Err(e) => fail(format!("creating a pipeline in {}: {e}", self.project)),
        }
    }

    /// The pipeline's state, in the GitHub driver's terms: `completed` with a conclusion, or
    /// `queued`, `in_progress` or `waiting` (a manual job).
    pub fn track(&self, outputs: &Value) -> Result<Option<Tracked>, String> {
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

    /// The child's run record from the `toolbox` job's artifact; `Ok(None)` when the pipeline has
    /// no such job, or the job kept no record.
    pub fn collect(&self, outputs: &Value) -> Result<Option<Value>, String> {
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
}

/// Open a merge request for a branch already pushed, through the API with the forge's token:
/// `{forge, url, iid}`, or `{forge, error}` with the branch left pushed.
pub fn open_merge(
    f: &forge::Resolved,
    source: &str,
    target: &str,
    title: &str,
    body: &str,
) -> Value {
    let Ok(token) = std::env::var(&f.token) else {
        return json!({"forge": "gitlab", "error": format!(
            "set {} to open a merge request; the branch is pushed", f.token)});
    };
    let url = format!(
        "{}/projects/{}/merge_requests",
        f.api.trim_end_matches('/'),
        encode(&f.project)
    );
    let req = json!({"source_branch": source, "target_branch": target, "title": title,
        "description": body, "remove_source_branch": true});
    let post = || {
        let (url, token, req) = (url.clone(), token.clone(), req.clone());
        block(async move {
            let r = reqwest::Client::new()
                .post(url)
                .header("private-token", token)
                .header("user-agent", "toolbox")
                .json(&req)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let st = r.status();
            Ok::<_, String>((st, r.text().await.unwrap_or_default()))
        })
    };
    // GitLab takes in a push before it knows the branch: until then it says the source branch
    // does not exist.
    let mut res = post();
    for _ in 0..20 {
        match &res {
            Ok(Ok((st, text))) if st.as_u16() == 400 && text.contains("source_branch") => {
                std::thread::sleep(std::time::Duration::from_millis(500));
                res = post();
            }
            _ => break,
        }
    }
    match res {
        Ok(Ok((st, text))) if st.is_success() => {
            let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            json!({"forge": "gitlab", "url": v["web_url"], "iid": v["iid"]})
        }
        Ok(Ok((st, text))) => json!({"forge": "gitlab",
            "error": format!("GitLab refused the merge request ({st}): {}", pipeline_error(&text))}),
        Ok(Err(e)) | Err(e) => json!({"forge": "gitlab", "error": e}),
    }
}

/// GitLab's reason, from `{"message": …}` or `{"error": …}`, else the body.
fn pipeline_error(text: &str) -> String {
    let v: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    match v.get("message").or_else(|| v.get("error")) {
        Some(Value::String(s)) => s.clone(),
        Some(m) => m.to_string(),
        None => text.trim().to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    type Seen = Arc<Mutex<Vec<(String, String, String)>>>;

    /// A GitLab API double. Answers by path suffix; records (request line, token, body).
    fn fake(routes: Vec<(&'static str, u16, Value)>) -> (String, Seen) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let api = format!("http://{}/api/v4", l.local_addr().unwrap());
        let seen: Seen = Arc::new(Mutex::new(vec![]));
        let log = seen.clone();
        std::thread::spawn(move || {
            for s in l.incoming().flatten() {
                let mut r = BufReader::new(s.try_clone().unwrap());
                let mut line = String::new();
                r.read_line(&mut line).unwrap();
                let (mut len, mut token) = (0, String::new());
                loop {
                    let mut h = String::new();
                    r.read_line(&mut h).unwrap();
                    if h.trim().is_empty() {
                        break;
                    }
                    let lower = h.to_ascii_lowercase();
                    if let Some(v) = lower.strip_prefix("content-length:") {
                        len = v.trim().parse().unwrap();
                    }
                    if lower.starts_with("private-token:") {
                        token = h.split_once(':').unwrap().1.trim().to_owned();
                    }
                }
                let mut body = vec![0; len];
                r.read_exact(&mut body).unwrap();
                let path = line.split_whitespace().nth(1).unwrap_or("").to_owned();
                log.lock().unwrap().push((
                    line.trim().to_owned(),
                    token,
                    String::from_utf8(body).unwrap(),
                ));
                let (code, b) = routes
                    .iter()
                    .find(|(p, ..)| path.split('?').next().unwrap().ends_with(p))
                    .map_or((404, json!({"message": "404 Not Found"})), |(_, c, b)| {
                        (*c, b.clone())
                    });
                let b = b.to_string();
                let reply = format!(
                    "HTTP/1.1 {code} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{b}",
                    b.len()
                );
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
        let p = Pipeline::parse(
            &json!({"project": "acme/platform/pay", "forge": "https://gitlab.acme.dev", "ref": "trunk"}),
            &forges(&api),
        )
        .unwrap();
        let out = p.dispatch(&ctx());
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
        let t = p.track(&out.outputs).unwrap().unwrap();
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
        let mut p = Pipeline::parse(
            &json!({"project": "acme/pay", "forge": "https://gitlab.acme.dev"}),
            &forges(&api),
        )
        .unwrap();
        p.federated = Some("set-owners".into());
        let out = p.dispatch(&ctx());
        let body: Value = serde_json::from_str(&seen.lock().unwrap()[0].2).unwrap();
        assert_eq!(body["inputs"]["action"], "set-owners");
        assert_eq!(body["inputs"]["parent_run"], "org/run_7");
        assert_eq!(body["inputs"]["grant"], "g");
        assert_eq!(p.collect(&out.outputs).unwrap(), Some(record));
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
        let out = p.dispatch(&ctx());
        assert_eq!(out.status, "failed");
        assert!(out.error.unwrap().contains("Unknown input: replicas"));
        let p = Pipeline {
            token_env: "TOOLBOX_TEST_NO_SUCH_VAR".into(),
            ..p
        };
        let n = seen.lock().unwrap().len();
        assert!(
            p.dispatch(&ctx())
                .error
                .unwrap()
                .contains("TOOLBOX_TEST_NO_SUCH_VAR")
        );
        assert_eq!(seen.lock().unwrap().len(), n);
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
