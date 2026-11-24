//! A `ci:` host of `type: http` (docs/design/forges.md §4.7): a service that speaks the executor
//! protocol and bridges to a CI Toolbox has no driver for (Buildkite, Jenkins, Tekton, …).
//! *Dispatch* is a `run` message carrying the same string inputs every CI gets, with the project
//! and ref on its `entity`; the service answers `accepted` with an `executor_run_id`, or a
//! `result` at once. *Track* is `status`. *Collect* asks `status` again and reads the child's run
//! record from the result's `outputs.record`, so a service answers `status` for a finished run
//! with its result each time it is asked. *Cancel* is `cancel`. The bearer token comes from the
//! variable the host names, and is never written to a run record.

use super::{CiDriver, Tracked};
use crate::executor::protocol::post;
use serde_json::{Value, json};

/// A project's runs on a protocol service.
#[derive(Debug, Clone)]
pub struct Service {
    /// The `ci:` host's name.
    pub host: String,
    pub url: String,
    pub token_env: Option<String>,
    pub project: String,
    pub git_ref: String,
}

impl Service {
    fn send(&self, msg: &Value) -> Result<Vec<Value>, String> {
        post(&self.url, self.token_env.as_deref(), msg)
    }

    fn ask(&self, ty: &str, run_id: &str, outputs: &Value) -> Result<Vec<Value>, String> {
        let mut msg = json!({"type": ty, "run_id": run_id});
        if let Some(id) = outputs["executor_run_id"].as_str() {
            msg["executor_run_id"] = json!(id);
        }
        self.send(&msg)
    }

    /// The run's `result`: the one dispatch returned, or what `status` answers now.
    fn result(&self, outputs: &Value) -> Result<Option<Value>, String> {
        if outputs["result"].is_object() {
            return Ok(Some(outputs["result"].clone()));
        }
        let run_id = outputs["run_id"].as_str().unwrap_or_default();
        Ok(self
            .ask("status", run_id, outputs)?
            .into_iter()
            .find(|m| m["type"] == "result"))
    }
}

impl CiDriver for Service {
    fn kind(&self) -> &'static str {
        "http"
    }

    fn noun(&self) -> &'static str {
        "CI run"
    }

    fn url_key(&self) -> &'static str {
        "ci_run"
    }

    fn effect(&self) -> Value {
        json!({"resource": format!("ci:{}/{}", self.host, self.project),
            "operation": "dispatch", "ref": self.git_ref})
    }

    fn dispatch(&self, inputs: Value) -> Result<Value, String> {
        let s = |k: &str| inputs[k].as_str().unwrap_or_default().to_owned();
        let run_id = s("toolbox_run");
        let actor = s("requested_by");
        let identity = if actor.starts_with("user:") {
            json!({"mode": "user", "principal": actor})
        } else {
            json!({"mode": "service", "principal": "toolbox"})
        };
        let key = match s("parent_run") {
            k if k.is_empty() => run_id.clone(),
            k => k,
        };
        let msg = json!({"type": "run", "run_id": run_id, "idempotency_key": key,
            "confirmed_effects": [], "action": s("action"), "action_version": "",
            "entity": {"id": format!("project:{}", self.project), "project": self.project,
                "ref": self.git_ref, "ci": self.host},
            "inputs": inputs, "identity": identity});
        let replies = self.send(&msg)?;
        let mut out = json!({"run_id": run_id, "project": self.project, "ref": self.git_ref,
            "dispatched_at": toolbox_connect::hub::rfc3339(crate::run::now_secs())});
        if let Some(r) = replies.iter().find(|m| m["type"] == "result") {
            out["result"] = r.clone();
        } else if let Some(id) = replies
            .iter()
            .find(|m| m["type"] == "accepted")
            .and_then(|m| m["executor_run_id"].as_str())
        {
            out["executor_run_id"] = json!(id);
        } else {
            return Err(format!(
                "{} answered the run with neither a result nor accepted with an executor_run_id",
                self.url
            ));
        }
        Ok(out)
    }

    fn track(&self, _: &str, outputs: &Value) -> Result<Option<Tracked>, String> {
        Ok(Some(match self.result(outputs)? {
            Some(r) => Tracked {
                status: "completed".into(),
                conclusion: Some(
                    if r["status"] == "succeeded" {
                        "success"
                    } else {
                        "failure"
                    }
                    .into(),
                ),
                url: r["outputs"]["url"].as_str().map(str::to_owned),
            },
            None => Tracked {
                status: "in_progress".into(),
                conclusion: None,
                url: None,
            },
        }))
    }

    fn no_record(&self) -> Option<String> {
        Some(format!(
            "ci {}'s result carried no outputs.record",
            self.host
        ))
    }

    fn collect(&self, outputs: &Value) -> Result<Option<Value>, String> {
        Ok(self
            .result(outputs)?
            .map(|r| r["outputs"]["record"].clone())
            .filter(Value::is_object))
    }

    fn cancel(&self, outputs: &Value) -> Result<String, String> {
        let run_id = outputs["run_id"].as_str().unwrap_or_default();
        self.ask("cancel", run_id, outputs)?;
        Ok(format!("asked ci {} to cancel the run", self.host))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::{Ci, fake};
    use crate::executor::{Context, Executor, Rechecked};
    use crate::run::Run;
    use serde_json::Map;

    fn service(url: String) -> Service {
        Service {
            host: "buildkite".into(),
            url,
            token_env: Some("PATH".into()),
            project: "acme/web".into(),
            git_ref: "main".into(),
        }
    }

    fn ctx() -> Context {
        Context {
            run_id: Some("run_7".into()),
            portal: "org".into(),
            action: "deploy".into(),
            action_version: "v1".into(),
            entity: json!({"id": "portal:web"}),
            inputs: serde_json::from_value(json!({"env": "staging"})).unwrap(),
            actor: "user:ann".into(),
            approved_by: vec![],
            related: Map::new(),
            grant: Some("g".into()),
            cwd: std::env::temp_dir(),
        }
    }

    fn valid(body: &str) {
        let v: Value = serde_json::from_str(body).unwrap();
        let errs: Vec<String> = toolbox_model::schema::executor_messages()
            .iter_errors(&v)
            .map(|e| e.to_string())
            .collect();
        assert!(errs.is_empty(), "{v}: {errs:?}");
    }

    #[test]
    fn a_federated_run_is_dispatched_tracked_and_read_back_from_the_result() {
        let record = json!({"id": "run_c1", "state": "succeeded", "verification": {"ok": true}});
        let (url, seen) = fake::serve(
            "/toolbox",
            vec![(
                "POST /toolbox",
                200,
                json!([{"type": "accepted", "run_id": "run_7", "executor_run_id": "bk-42"},
                    {"type": "progress", "run_id": "run_7", "message": "queued"}]),
            )],
        );
        let ci = Ci {
            driver: Box::new(service(url.clone())),
            federated: Some("deploy".into()),
            clone: None,
            preview: None,
        };
        let out = ci.run(&ctx(), &[]);
        assert_eq!(out.status, "dispatched", "{:?}", out.error);
        assert_eq!(out.outputs["executor_run_id"], "bk-42");
        let mut run: Run = serde_json::from_value(json!({
            "id": "run_7", "action": "deploy", "action_version": "v1", "entity": "portal:web",
            "inputs": {}, "requested_by": "user:ann", "via": "cli", "portal": "org",
            "executor": "http", "snapshot": "s", "effects": [],
            "approval": {"required": false, "approvers": []}, "state": "dispatched",
            "events": [], "outputs": out.outputs}))
        .unwrap();

        let (url2, seen2) = fake::serve(
            "/toolbox",
            vec![(
                "POST /toolbox",
                200,
                json!({"type": "result", "run_id": "run_7", "status": "succeeded",
                    "outputs": {"url": "https://bk.acme.dev/builds/42", "record": record}}),
            )],
        );
        let ci = Ci {
            driver: Box::new(service(url2)),
            ..ci
        };
        assert_eq!(
            ci.recheck(&mut run, false, &std::env::temp_dir()),
            Rechecked::Done
        );
        assert_eq!(run.state, "succeeded", "{:?}", run.error);
        assert_eq!(run.outputs["ci_run"], "https://bk.acme.dev/builds/42");
        assert_eq!(run.child.as_ref().unwrap()["id"], "run_c1");

        let sent = seen.lock().unwrap();
        let (_, token, body) = &sent[0];
        assert_eq!(token, &std::env::var("PATH").unwrap());
        valid(body);
        let msg: Value = serde_json::from_str(body).unwrap();
        assert_eq!(
            (
                &msg["entity"]["project"],
                &msg["inputs"]["action"],
                &msg["identity"]["principal"]
            ),
            (&json!("acme/web"), &json!("deploy"), &json!("user:ann"))
        );
        let asked = seen2.lock().unwrap();
        assert!(asked.len() >= 2, "track and collect each ask status");
        for (_, _, body) in asked.iter() {
            valid(body);
            assert!(body.contains("\"executor_run_id\":\"bk-42\""), "{body}");
        }
    }

    #[test]
    fn a_result_at_once_a_missing_record_and_cancel() {
        let (url, seen) = fake::serve(
            "/toolbox",
            vec![(
                "POST /toolbox",
                200,
                json!({"type": "result", "run_id": "run_7", "status": "failed",
                    "error": {"code": "build_failed", "message": "red"}}),
            )],
        );
        let s = service(url);
        let out = s.dispatch(json!({"toolbox_run": "run_7"})).unwrap();
        let t = s.track("run_7", &out).unwrap().unwrap();
        assert_eq!(
            (t.status.as_str(), t.conclusion.as_deref()),
            ("completed", Some("failure"))
        );
        assert_eq!(s.collect(&out).unwrap(), None);
        assert!(s.cancel(&out).unwrap().contains("buildkite"));
        let sent = seen.lock().unwrap();
        assert_eq!(sent.len(), 2, "the kept result is not asked for again");
        valid(&sent[0].2);
        valid(&sent[1].2);
        assert!(sent[1].2.contains("\"type\":\"cancel\""));
    }

    #[test]
    fn a_service_that_neither_accepts_nor_answers_fails_the_dispatch() {
        let (url, _) = fake::serve("/toolbox", vec![("POST /toolbox", 200, json!([]))]);
        let e = service(url)
            .dispatch(json!({"toolbox_run": "run_7"}))
            .unwrap_err();
        assert!(e.contains("neither"), "{e}");
    }
}
