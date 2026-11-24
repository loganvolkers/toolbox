//! CI executors (docs/v1-spec.md §7.5, docs/design/forges.md §4.4): a run handed to a CI system
//! that reports later. [`Ci`] is the executor, the same for every CI: the inputs it sends, its
//! preview, `dispatched` and the recheck that follows the run to its end. A [`CiDriver`] is what
//! differs: how a CI is started (*dispatch*), followed (*track*) and, for a federated child, how
//! its run record is read back (*collect*).
//!
//! | `type` | Driver |
//! |---|---|
//! | `github-workflow` | [`github::Workflow`]: `workflow_dispatch`, tracked by run id or `run-name` |
//! | `gitlab-pipeline` | [`gitlab::Pipeline`]: a pipeline with typed `inputs`, tracked by id, record from the `toolbox` job's artifact |
//!
//! A repo portal's own action gets each input as a string, plus `toolbox_run`. A federated
//! dispatch (`runner: {type: ci}` or `{type: github-workflow}`) gets the child action instead:
//! `action`, `inputs` (JSON), `requested_by`, `toolbox_run`, `parent_run` and `grant`.

pub mod github;
pub mod gitlab;

use crate::executor::{Context, Executor, Outcome, Preview, Rechecked};
use crate::run::{Run, now_secs};
use serde_json::{Map, Value, json};
use std::any::Any;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use toolbox_connect::hub::rfc3339;

/// A CI run, in GitHub's terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tracked {
    /// `queued`, `in_progress`, `waiting` (a manual job), `completed`.
    pub status: String,
    /// `success`, `failure`, `cancelled`, … once completed.
    pub conclusion: Option<String>,
    pub url: Option<String>,
}

/// What differs between CI systems.
pub trait CiDriver: std::fmt::Debug + Send + Sync + Any {
    /// The executor `type` a run records.
    fn kind(&self) -> &'static str;

    /// What one run is called, in messages: `workflow run`, `pipeline`.
    fn noun(&self) -> &'static str;

    /// The outputs key the run's web page is kept under.
    fn url_key(&self) -> &'static str;

    /// The preview's effect when nothing previews for it.
    fn effect(&self) -> Value;

    /// Start a run with `inputs`: the outputs a later [`CiDriver::track`] reads.
    fn dispatch(&self, inputs: Value) -> Result<Value, String>;

    /// The run `outputs` describe; `Ok(None)` until the CI lists it.
    fn track(&self, run_id: &str, outputs: &Value) -> Result<Option<Tracked>, String>;

    /// What a `waiting` run waits for, noted on the run once.
    fn waits_for(&self) -> Option<&'static str> {
        None
    }

    /// For a federated child whose CI keeps its run record ([`CiDriver::collect`]): how to say
    /// that a run kept none. `None`: this CI keeps no record.
    fn no_record(&self) -> Option<String> {
        None
    }

    /// The child's run record; `Ok(None)` when the run kept none.
    fn collect(&self, _outputs: &Value) -> Result<Option<Value>, String> {
        Ok(None)
    }
}

impl dyn CiDriver {
    /// This driver as its own type, if it is one.
    pub fn downcast<T: CiDriver>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
}

/// A run carried by a CI system.
#[derive(Debug)]
pub struct Ci {
    pub driver: Box<dyn CiDriver>,
    /// For a federated dispatch: the child action's name.
    pub federated: Option<String>,
    /// For a federated dispatch: the parent's clone of the child, where a change the child
    /// proposed is looked for on its base.
    pub clone: Option<PathBuf>,
    /// For a federated dispatch: the child's own preview, in that clone, which gives the
    /// approvers and effects the grant binds.
    pub preview: Option<Arc<dyn Executor>>,
}

impl Ci {
    /// A repo portal's own CI action.
    pub fn new(driver: Box<dyn CiDriver>) -> Self {
        Self {
            driver,
            federated: None,
            clone: None,
            preview: None,
        }
    }

    /// The inputs the CI receives. All are strings: a pipeline declares each in `spec:inputs`,
    /// and a workflow each in `workflow_dispatch.inputs`.
    pub fn inputs(&self, ctx: &Context) -> Value {
        let run = ctx.run_id.clone().unwrap_or_default();
        if let Some(action) = &self.federated {
            return json!({"action": action, "inputs": Value::Object(ctx.inputs.clone()).to_string(),
                "requested_by": ctx.actor, "toolbox_run": run,
                "parent_run": format!("{}/{run}", ctx.portal),
                "grant": ctx.grant.clone().unwrap_or_default()});
        }
        let mut m = Map::new();
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

    /// A federated child's finished run: its kept record is this run's outcome. Returns `false`
    /// when the record could not be read, so the next look tries again.
    fn record(&self, run: &mut Run, url: Option<&str>, ok: bool, concluded: &str) -> bool {
        let noun = self.driver.noun();
        let missing = self.driver.no_record().unwrap_or_default();
        match self.driver.collect(&run.outputs) {
            Ok(Some(v)) => {
                child_record(run, v, url);
                if let (Some(child), Some(clone)) = (run.child.clone(), &self.clone)
                    && child["state"] == "proposed"
                {
                    child_proposal(run, &child, clone);
                }
            }
            Ok(None) if ok => run.set(
                "unverified",
                None,
                Some(format!(
                    "the {noun} succeeded, but {missing}, so the child's own verification is unknown"
                )),
            ),
            Ok(None) => run.fail(format!("the {noun} concluded {concluded}, and {missing}")),
            Err(e) => {
                run.progress.push(
                    json!({"at": rfc3339(now_secs()), "note": format!("reading the child's run record: {e}")}),
                );
                return false;
            }
        }
        true
    }
}

impl Executor for Ci {
    fn kind(&self) -> &'static str {
        self.driver.kind()
    }

    fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        match &self.preview {
            Some(child) => child.preview(ctx),
            None => Ok(Preview {
                effects: vec![self.driver.effect()],
                ..Preview::default()
            }),
        }
    }

    fn run(&self, ctx: &Context, _: &[Value]) -> Outcome {
        match self.driver.dispatch(self.inputs(ctx)) {
            Ok(outputs) => Outcome {
                status: "dispatched".into(),
                outputs,
                error: None,
                progress: vec![],
                child: None,
            },
            Err(e) => Outcome::failed(e),
        }
    }

    fn recheck(&self, run: &mut Run, expects: bool, _: &Path) -> Rechecked {
        if run.state != "dispatched" {
            return Rechecked::Verify;
        }
        let noun = self.driver.noun();
        if let (Some(child), Some(clone)) = (run.child.clone(), &self.clone)
            && child["state"] == "proposed"
        {
            child_proposal(run, &child, clone);
            return Rechecked::Done;
        }
        match self.driver.track(&run.id, &run.outputs) {
            Ok(Some(t)) if t.status == "completed" => {
                if let (Some(u), Some(o)) = (&t.url, run.outputs.as_object_mut()) {
                    o.insert(self.driver.url_key().into(), json!(u));
                }
                let ok = t.conclusion.as_deref() == Some("success");
                let concluded = t.conclusion.as_deref().unwrap_or("unknown").to_owned();
                if self.federated.is_some() && self.driver.no_record().is_some() {
                    self.record(run, t.url.as_deref(), ok, &concluded);
                    return Rechecked::Done;
                }
                if !ok {
                    run.fail(format!("the {noun} concluded {concluded}"));
                    return Rechecked::Done;
                }
                run.set("executor_succeeded", None, t.url.clone());
                if !expects {
                    run.set("unverified", None, None);
                    return Rechecked::Done;
                }
                Rechecked::Verify
            }
            Ok(t) => {
                if run.deadline.is_some_and(|dl| now_secs() >= dl) {
                    run.set(
                        "timed_out",
                        None,
                        Some(match t {
                            None => format!("the dispatched {noun} never appeared"),
                            Some(t) => format!("the {noun} was still {}", t.status),
                        }),
                    );
                    run.error = Some(format!("the {noun} did not finish before the deadline"));
                } else if let Some(t) = t.filter(|t| t.status == "waiting")
                    && let Some(what) = self.driver.waits_for()
                    && !run.progress.iter().any(|p| p["waiting"] == true)
                {
                    run.progress
                        .push(json!({"at": rfc3339(now_secs()), "waiting": true,
                        "note": format!("the {noun} waits for {what}: {}",
                            t.url.unwrap_or_default())}));
                }
                Rechecked::Done
            }
            Err(e) => {
                run.progress
                    .push(json!({"at": rfc3339(now_secs()), "note": e}));
                Rechecked::Done
            }
        }
    }
}

/// Take a child's run record, as its CI kept it, as this run's outcome (T8: the child
/// verifies). A change the child proposed leaves this run `dispatched` until the parent sees it
/// on the base ([`child_proposal`]).
fn child_record(run: &mut Run, v: Value, pipeline: Option<&str>) {
    let state = v["state"].as_str().unwrap_or("failed").to_owned();
    let error = v["error"].as_str().unwrap_or("").to_owned();
    match state.as_str() {
        "succeeded" | "unverified" => {
            run.verification = Some(json!({"by": "child pipeline", "child_run": v["id"],
                "pipeline": pipeline, "child": v["verification"]}));
            run.set(&state, None, None);
        }
        "proposed" => {
            let review = v["outputs"]["review"]["url"]
                .as_str()
                .or(v["outputs"]["branch"].as_str())
                .unwrap_or("a branch")
                .to_owned();
            run.progress.push(json!({"at": rfc3339(now_secs()),
                "note": format!("the child proposed {review}; waiting for it on {}",
                    v["outputs"]["base"].as_str().unwrap_or("its base"))}));
        }
        "awaiting_approval" => run.fail(format!(
            "the child's run waits for approval in its pipeline, where nobody can give it: this portal's grant did not meet its rule{}",
            if error.is_empty() { String::new() } else { format!(" ({error})") }
        )),
        _ => run.fail(format!("child run {state}: {error}")),
    }
    run.child = Some(v);
}

/// A change a child proposed from its CI, looked for on its base in the parent's clone of the
/// child: merged, fast-forwarded, or squashed or rebased with its `Toolbox-Run` trailer.
fn child_proposal(run: &mut Run, child: &Value, clone: &Path) {
    let Ok(record) = serde_json::from_value::<Run>(child.clone()) else {
        run.fail("the child's run record does not parse");
        return;
    };
    match crate::executor::merged(clone, &record) {
        Ok(true) => {
            run.verification = Some(json!({"by": "parent", "merged": true,
                "child_run": record.id, "base": record.outputs["base"],
                "commit": record.outputs["commit"], "review": record.outputs["review"]}));
            run.set("succeeded", None, None);
        }
        Ok(false) => {
            if run.deadline.is_some_and(|dl| now_secs() >= dl) {
                run.fail("the change the child proposed was not merged before the deadline");
            }
        }
        Err(e) => run
            .progress
            .push(json!({"at": rfc3339(now_secs()), "note": e})),
    }
}

#[cfg(test)]
pub(crate) mod fake {
    //! A forge API double. Answers by method and path suffix; records (request line, token,
    //! body). A reply of `null` is an empty 204.

    use serde_json::{Value, json};
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    pub type Seen = Arc<Mutex<Vec<(String, String, String)>>>;

    /// `routes`: `("POST /path-suffix", status, body)` or `("/path-suffix", …)` for any method.
    /// The first match wins; anything else is a 404.
    pub fn serve(prefix: &str, routes: Vec<(&'static str, u16, Value)>) -> (String, Seen) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let api = format!("http://{}{prefix}", l.local_addr().unwrap());
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
                    if lower.starts_with("private-token:") || lower.starts_with("authorization:") {
                        let v = h.split_once(':').unwrap().1.trim();
                        token = v.strip_prefix("Bearer ").unwrap_or(v).to_owned();
                    }
                }
                let mut body = vec![0; len];
                r.read_exact(&mut body).unwrap();
                let mut words = line.split_whitespace();
                let method = words.next().unwrap_or("").to_owned();
                let path = words.next().unwrap_or("").to_owned();
                log.lock().unwrap().push((
                    line.trim().to_owned(),
                    token,
                    String::from_utf8(body).unwrap(),
                ));
                let path = path.split('?').next().unwrap();
                let (code, b) = routes
                    .iter()
                    .find(|(p, ..)| match p.split_once(' ') {
                        Some((m, p)) => m == method && path.ends_with(p),
                        None => path.ends_with(p),
                    })
                    .map_or((404, json!({"message": "404 Not Found"})), |(_, c, b)| {
                        (*c, b.clone())
                    });
                let reply = if b.is_null() {
                    format!("HTTP/1.1 {code} X\r\nconnection: close\r\n\r\n")
                } else {
                    let b = b.to_string();
                    format!(
                        "HTTP/1.1 {code} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{b}",
                        b.len()
                    )
                };
                let mut s = s;
                s.write_all(reply.as_bytes()).unwrap();
            }
        });
        (api, seen)
    }
}
