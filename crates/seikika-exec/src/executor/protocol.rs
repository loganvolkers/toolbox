//! Executor protocol v1 (`schema/executor-messages.schema.json`, docs/design/action-runs.md §5),
//! over either transport: `in-image`, a program spoken to over stdio, and `http`, a service
//! POSTed the same bodies. Both get `preview` and `run`; a run the executor `accepted` with an
//! `executor_run_id` and no `result` stays `dispatched` and is asked after with `status`; a
//! dispatched run is sent `cancel`; and an executor that declares `verify: true` is sent `verify`
//! when the action has no postcondition.

use super::{Context, Executor, Outcome, Preview, Rechecked, argv, clean_env, names, note};
use crate::run::Run;
use seikika_model::ci::Hosts;
use serde_json::{Map, Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

/// How messages reach the executor.
#[derive(Debug, Clone)]
pub enum Transport {
    /// A child process: one message on stdin, the replies on stdout, one per line.
    Stdio {
        command: Vec<String>,
        env: Vec<String>,
        /// The released package the program comes from, when it does.
        package: Option<String>,
    },
    /// One POST per message; the response body holds the replies.
    Http {
        url: String,
        /// The environment variable holding a bearer token, if one is sent.
        token_env: Option<String>,
    },
}

/// An `in-image` or `http` executor.
#[derive(Debug, Clone)]
pub struct Protocol {
    pub transport: Transport,
    /// The executor answers `verify`.
    pub verify: bool,
}

/// An `in-image` executor: `command`, or a released `package` whose binary gets `command` as
/// its arguments.
pub(crate) fn parse_in_image(v: &Value, _: &Hosts) -> Result<Arc<dyn Executor>, String> {
    let package = v.get("package").and_then(Value::as_str).map(str::to_owned);
    let command = match &package {
        Some(p) => {
            let spec = seikika_model::artifacts::parse(p).ok_or_else(|| {
                format!("executor.package {p} is not github:owner/repo[/app]@version")
            })?;
            let mut c = vec![spec.installed().display().to_string()];
            if v.get("command").is_some() {
                c.extend(argv(v.get("command"), "executor.command")?);
            }
            c
        }
        None => argv(v.get("command"), "executor.command")?,
    };
    Ok(Arc::new(Protocol {
        transport: Transport::Stdio {
            command,
            env: names(v, "secrets"),
            package,
        },
        verify: v.get("verify") == Some(&Value::Bool(true)),
    }))
}

/// An `http` executor: `url`, and `token`, the variable holding a bearer token.
pub(crate) fn parse_http(v: &Value, _: &Hosts) -> Result<Arc<dyn Executor>, String> {
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
    Ok(Arc::new(Protocol {
        transport: Transport::Http {
            url: s("url").ok_or("executor.url is required")?,
            token_env: s("token"),
        },
        verify: v.get("verify") == Some(&Value::Bool(true)),
    }))
}

impl Executor for Protocol {
    fn kind(&self) -> &'static str {
        match self.transport {
            Transport::Stdio { .. } => "in-image",
            Transport::Http { .. } => "http",
        }
    }

    /// A program that is not installed here (a checkout, not the image) previews the Seikika
    /// layer only, with a warning that says so.
    fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        let mut msg = context(ctx);
        msg.insert("type".into(), "preview".into());
        msg.insert("request_id".into(), "req_preview".into());
        let msgs = match self.session(&ctx.cwd, &Value::Object(msg)) {
            Err(e) if e.starts_with("cannot start ") => {
                let hint = match &self.transport {
                    Transport::Stdio {
                        package: Some(_), ..
                    } => "run seikika install",
                    _ => "the executor is not installed here",
                };
                return Ok(Preview {
                    warnings: vec![format!(
                        "{e}: {hint}, so this preview checks inputs, preconditions and approvals only"
                    )],
                    ..Preview::default()
                });
            }
            other => other?,
        };
        let r = msgs
            .iter()
            .find(|m| m["type"] == "preview_result")
            .ok_or("the executor ended without a preview_result")?;
        let strings = |k: &str| -> Vec<String> {
            r[k].as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        };
        Ok(Preview {
            effects: r["effects"].as_array().cloned().unwrap_or_default(),
            warnings: strings("warnings"),
            blockers: strings("blockers"),
            executed: true,
            child: None,
        })
    }

    fn run(&self, ctx: &Context, confirmed: &[Value]) -> Outcome {
        let msgs = match self.session(&ctx.cwd, &run_message(ctx, confirmed)) {
            Ok(m) => m,
            Err(e) => return Outcome::failed(e),
        };
        let progress = of_type(&msgs, "progress");
        if let Some(r) = msgs.iter().find(|m| m["type"] == "result") {
            let (status, outputs, error) = result(r, &json!({}));
            return Outcome {
                status,
                outputs,
                error,
                progress,
                child: None,
            };
        }
        match msgs
            .iter()
            .find(|m| m["type"] == "accepted")
            .and_then(|a| a["executor_run_id"].as_str())
        {
            Some(id) => Outcome {
                status: "dispatched".into(),
                outputs: json!({"executor_run_id": id}),
                error: None,
                progress,
                child: None,
            },
            None => Outcome {
                progress,
                ..Outcome::failed("the executor ended without a result")
            },
        }
    }

    /// A dispatched run: ask the executor for its `status`.
    fn recheck(&self, run: &mut Run, _: bool, cwd: &Path) -> Rechecked {
        if run.state != "dispatched" {
            return Rechecked::Verify;
        }
        let msgs = match self.session(cwd, &status_message(run, "status")) {
            Ok(m) => m,
            Err(e) => {
                note(
                    run,
                    format!("asking the executor for the run's status: {e}"),
                );
                return Rechecked::Done;
            }
        };
        for p in of_type(&msgs, "progress") {
            if !run.progress.contains(&p) {
                run.progress.push(p);
            }
        }
        let Some(r) = msgs.iter().find(|m| m["type"] == "result") else {
            super::timed_out(
                run,
                "the executor was still running the run".into(),
                "the executor did not finish before the deadline".into(),
            );
            return Rechecked::Done;
        };
        let (status, outputs, error) = result(r, &run.outputs);
        run.outputs = outputs;
        if status != "succeeded" {
            run.fail(error.unwrap_or_else(|| "the executor failed".into()));
            return Rechecked::Done;
        }
        run.set("executor_succeeded", None, None);
        Rechecked::Verify
    }

    fn cancel(&self, run: &Run, cwd: &Path) -> Option<Result<String, String>> {
        if run.state != "dispatched" {
            return None;
        }
        Some(
            self.session(cwd, &status_message(run, "cancel"))
                .map(|msgs| match msgs.iter().find(|m| m["type"] == "result") {
                    Some(r) => format!(
                        "the executor stopped the run: {}",
                        r["status"].as_str().unwrap_or("no status")
                    ),
                    None => "the executor was asked to stop the run".into(),
                }),
        )
    }

    fn verify(&self, run: &Run, cwd: &Path) -> Option<Result<(bool, Value), String>> {
        if !self.verify {
            return None;
        }
        let mut msg = run_context(run);
        msg.insert("type".into(), "verify".into());
        msg.insert("run_id".into(), run.id.clone().into());
        msg.insert("outputs".into(), run.outputs.clone());
        if let Some(id) = run.outputs["executor_run_id"].as_str() {
            msg.insert("executor_run_id".into(), id.into());
        }
        Some(self.session(cwd, &Value::Object(msg)).and_then(|msgs| {
            msgs.iter()
                .find(|m| m["type"] == "verify_result")
                .map(|r| {
                    (
                        r["ok"] == true,
                        r.get("observed").cloned().unwrap_or(json!({})),
                    )
                })
                .ok_or_else(|| "the executor ended without a verify_result".into())
        }))
    }
}

impl Protocol {
    /// Send one message and return the replies.
    fn session(&self, cwd: &Path, msg: &Value) -> Result<Vec<Value>, String> {
        match &self.transport {
            Transport::Stdio { command, env, .. } => stdio(command, env, cwd, msg),
            Transport::Http { url, token_env } => post(url, token_env.as_deref(), msg),
        }
    }
}

fn stdio(
    command: &[String],
    env: &[String],
    cwd: &Path,
    msg: &Value,
) -> Result<Vec<Value>, String> {
    let mut cmd = Command::new(&command[0]);
    clean_env(&mut cmd, env);
    let mut child = cmd
        .args(&command[1..])
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", command[0]))?;
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(format!("{msg}\n").as_bytes())
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for line in BufReader::new(child.stdout.take().ok_or("no stdout")?).lines() {
        let line = line.map_err(|e| e.to_string())?;
        let Ok(m) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let last = matches!(
            m["type"].as_str(),
            Some("preview_result" | "result" | "verify_result" | "error")
        );
        out.push(m);
        if last {
            break;
        }
    }
    let _ = child.kill();
    let _ = child.wait();
    Ok(out)
}

pub(crate) fn post(url: &str, token_env: Option<&str>, msg: &Value) -> Result<Vec<Value>, String> {
    let mut req = crate::http::client().post(url).json(msg);
    if let Some(var) = token_env {
        let token = std::env::var(var).map_err(|_| format!("set {var} to reach {url}"))?;
        req = req.bearer_auth(token);
    }
    let (st, text) = crate::http::send(req).map_err(|e| format!("{url}: {e}"))?;
    if !st.is_success() {
        return Err(format!("{url} answered {st}: {}", text.trim()));
    }
    Ok(replies(&text))
}

/// A response body's messages: one JSON value (an array is a list of them), or NDJSON.
fn replies(text: &str) -> Vec<Value> {
    let flat = |v: Value| match v {
        Value::Array(a) => a,
        v => vec![v],
    };
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return flat(v);
    }
    text.lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .flat_map(flat)
        .collect()
}

fn of_type(msgs: &[Value], ty: &str) -> Vec<Value> {
    msgs.iter().filter(|m| m["type"] == ty).cloned().collect()
}

/// A `result`'s status, its outputs over `before`, and its error message.
fn result(r: &Value, before: &Value) -> (String, Value, Option<String>) {
    let mut outputs = before.as_object().cloned().unwrap_or_default();
    if let Some(Value::Object(o)) = r.get("outputs") {
        outputs.extend(o.clone());
    }
    let status = if r["status"] == "succeeded" {
        "succeeded"
    } else {
        "failed"
    };
    let error = r.get("error").map(|e| {
        e.get("message")
            .and_then(Value::as_str)
            .map_or_else(|| e.to_string(), str::to_owned)
    });
    (status.into(), Value::Object(outputs), error)
}

fn identity(actor: &str) -> Value {
    let mode = if actor.starts_with("user:") {
        "user"
    } else {
        "service"
    };
    json!({"mode": mode, "principal": actor})
}

fn context(ctx: &Context) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("action".into(), ctx.action.clone().into());
    m.insert("action_version".into(), ctx.action_version.clone().into());
    m.insert("entity".into(), ctx.entity.clone());
    m.insert("inputs".into(), Value::Object(ctx.inputs.clone()));
    m.insert("identity".into(), identity(&ctx.actor));
    m.insert("related".into(), Value::Object(ctx.related.clone()));
    m
}

/// The context a later message carries, from the run record alone.
fn run_context(run: &Run) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("action".into(), run.action.clone().into());
    m.insert("action_version".into(), run.action_version.clone().into());
    m.insert("entity".into(), json!({"id": run.entity}));
    m.insert("inputs".into(), Value::Object(run.inputs.clone()));
    m.insert("identity".into(), identity(&run.requested_by));
    m
}

/// The protocol `run` message.
pub(crate) fn run_message(ctx: &Context, confirmed: &[Value]) -> Value {
    let run_id = ctx.run_id.clone().unwrap_or_default();
    let mut msg = context(ctx);
    msg.insert("type".into(), "run".into());
    msg.insert("run_id".into(), run_id.clone().into());
    msg.insert(
        "idempotency_key".into(),
        format!(
            "{}:{}:{run_id}",
            ctx.action,
            ctx.entity["id"].as_str().unwrap_or("")
        )
        .into(),
    );
    msg.insert("confirmed_effects".into(), Value::Array(confirmed.to_vec()));
    Value::Object(msg)
}

/// `status` or `cancel` for a run the executor carries.
fn status_message(run: &Run, ty: &str) -> Value {
    let mut msg = json!({"type": ty, "run_id": run.id});
    if let Some(id) = run.outputs["executor_run_id"].as_str() {
        msg["executor_run_id"] = json!(id);
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci::fake;
    use std::path::PathBuf;

    fn ctx() -> Context {
        Context {
            run_id: Some("run_7".into()),
            portal: "org".into(),
            action: "rotate".into(),
            action_version: "v1".into(),
            entity: json!({"id": "service:pay"}),
            inputs: serde_json::from_value(json!({"key": "db"})).unwrap(),
            actor: "user:ann".into(),
            approved_by: vec![],
            related: Map::new(),
            grant: None,
            cwd: std::env::temp_dir(),
        }
    }

    fn as_run(out: &Outcome) -> Run {
        serde_json::from_value(json!({
            "id": "run_7", "action": "rotate", "action_version": "v1", "entity": "service:pay",
            "inputs": {"key": "db"}, "requested_by": "user:ann", "via": "cli", "portal": "org",
            "executor": "in-image", "snapshot": "s", "effects": [],
            "approval": {"required": false, "approvers": []}, "state": out.status,
            "events": [], "outputs": out.outputs,
        }))
        .unwrap()
    }

    /// An executor script: `sync` answers `run` with a result, `detached` with `accepted` only.
    /// Every message it receives is appended to `log`.
    const SCRIPT: &str = r#"read -r line
printf '%s\n' "$line" >> "$2"
case "$line" in
*'"type":"preview"'*) echo '{"type":"preview_result","request_id":"req_preview","effects":[{"resource":"secret:db","operation":"update"}]}' ;;
*'"type":"run"'*)
  echo '{"type":"accepted","run_id":"run_7","executor_run_id":"ex_1"}'
  echo '{"type":"progress","run_id":"run_7","message":"started"}'
  [ "$1" = sync ] && echo '{"type":"result","run_id":"run_7","status":"succeeded","outputs":{"version":2}}' ;;
*'ex_1'*'"type":"status"'*)
  echo '{"type":"progress","run_id":"run_7","message":"rotated"}'
  echo '{"type":"result","run_id":"run_7","status":"succeeded","outputs":{"version":3}}' ;;
*'"type":"cancel"'*) echo '{"type":"result","run_id":"run_7","status":"failed","error":{"code":"cancelled","message":"stopped","retryable":false}}' ;;
*'"version":3'*'"type":"verify"'*) echo '{"type":"verify_result","run_id":"run_7","ok":true,"observed":{"version":3}}' ;;
esac
"#;

    fn script(name: &str, mode: &str) -> (Protocol, PathBuf) {
        let dir = std::env::temp_dir().join(format!("seikika-proto-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("exec.sh"), SCRIPT).unwrap();
        let log = dir.join("received.ndjson");
        let _ = std::fs::remove_file(&log);
        let command = ["sh", "exec.sh", mode, log.to_str().unwrap()];
        let p = Protocol {
            transport: Transport::Stdio {
                command: command.iter().map(|s| s.to_string()).collect(),
                env: vec![],
                package: None,
            },
            verify: true,
        };
        (p, dir)
    }

    fn sent(dir: &Path) -> Vec<Value> {
        std::fs::read_to_string(dir.join("received.ndjson"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn assert_valid(msgs: &[Value]) {
        let v = seikika_model::schema::executor_messages();
        for m in msgs {
            let errors: Vec<String> = v.iter_errors(m).map(|e| e.to_string()).collect();
            assert!(errors.is_empty(), "{m}: {errors:?}");
        }
    }

    #[test]
    fn a_synchronous_run_ends_with_its_result() {
        let (p, dir) = script("sync", "sync");
        let c = Context {
            cwd: dir.clone(),
            ..ctx()
        };
        let pre = p.preview(&c).unwrap();
        assert_eq!(pre.effects[0]["resource"], "secret:db");
        let out = p.run(&c, &pre.effects);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        assert_eq!(out.outputs, json!({"version": 2}));
        assert_eq!(out.progress.len(), 1);
        let msgs = sent(&dir);
        assert_eq!(msgs[1]["idempotency_key"], "rotate:service:pay:run_7");
        assert_valid(&msgs);
    }

    #[test]
    fn an_accepted_run_is_dispatched_then_asked_for_status_and_read_back() {
        let (p, dir) = script("detached", "detached");
        let c = Context {
            cwd: dir.clone(),
            ..ctx()
        };
        let out = p.run(&c, &[]);
        assert_eq!(out.status, "dispatched");
        assert_eq!(out.outputs, json!({"executor_run_id": "ex_1"}));
        let mut run = as_run(&out);
        assert!(p.cancel(&run, &dir).unwrap().unwrap().contains("failed"));
        assert!(matches!(
            p.recheck(&mut run, false, &dir),
            Rechecked::Verify
        ));
        assert_eq!(run.state, "executor_succeeded");
        assert_eq!(
            run.outputs,
            json!({"executor_run_id": "ex_1", "version": 3})
        );
        assert_eq!(run.progress.len(), 1);
        assert!(
            p.cancel(&run, &dir).is_none(),
            "only a dispatched run is cancelled"
        );
        assert_eq!(
            p.verify(&run, &dir),
            Some(Ok((true, json!({"version": 3}))))
        );
        let msgs = sent(&dir);
        let types: Vec<&str> = msgs.iter().map(|m| m["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["run", "cancel", "status", "verify"]);
        assert_eq!(msgs[2]["executor_run_id"], "ex_1");
        assert_valid(&msgs);
    }

    #[test]
    fn a_detached_run_without_an_id_fails_and_a_missing_program_previews_with_a_warning() {
        let (p, dir) = script("noid", "detached");
        std::fs::write(dir.join("exec.sh"), "read -r line\n").unwrap();
        let out = p.run(
            &Context {
                cwd: dir.clone(),
                ..ctx()
            },
            &[],
        );
        assert_eq!(out.status, "failed");
        let missing = Protocol {
            transport: Transport::Stdio {
                command: vec!["/nonexistent/exec".into()],
                env: vec![],
                package: Some("github:acme/exec@1.0.0".into()),
            },
            verify: false,
        };
        let pre = missing.preview(&ctx()).unwrap();
        assert!(!pre.executed);
        assert!(pre.warnings[0].contains("run seikika install"));
        assert!(missing.verify(&as_run(&out), &dir).is_none());
    }

    #[test]
    fn http_posts_the_same_messages_with_a_bearer_token() {
        let (url, seen) = fake::serve(
            "",
            vec![(
                "POST /exec",
                200,
                json!([{"type": "accepted", "run_id": "run_7", "executor_run_id": "ex_9"}]),
            )],
        );
        let p = parse_http(
            &json!({"url": format!("{url}/exec"), "token": "PATH"}),
            &Hosts::default(),
        )
        .unwrap();
        assert_eq!(p.kind(), "http");
        let out = p.run(&ctx(), &[]);
        assert_eq!(out.status, "dispatched", "{:?}", out.error);
        let (line, token, body) = seen.lock().unwrap()[0].clone();
        assert_eq!(line, "POST /exec HTTP/1.1");
        assert_eq!(token, std::env::var("PATH").unwrap());
        assert_valid(&[serde_json::from_str(&body).unwrap()]);

        let (url, _) = fake::serve(
            "",
            vec![(
                "POST /exec",
                200,
                json!({"type": "result", "run_id": "run_7", "status": "succeeded", "outputs": {"ok": 1}}),
            )],
        );
        let p = parse_http(&json!({"url": format!("{url}/exec")}), &Hosts::default()).unwrap();
        let mut run = as_run(&out);
        assert!(matches!(
            p.recheck(&mut run, false, Path::new(".")),
            Rechecked::Verify
        ));
        assert_eq!(run.outputs, json!({"executor_run_id": "ex_9", "ok": 1}));
    }

    #[test]
    fn replies_are_one_value_an_array_or_ndjson() {
        assert_eq!(replies(r#"{"type":"result"}"#).len(), 1);
        assert_eq!(replies(r#"[{"type":"a"},{"type":"b"}]"#).len(), 2);
        assert_eq!(replies("{\"type\":\"a\"}\n{\"type\":\"b\"}\n").len(), 2);
    }
}
