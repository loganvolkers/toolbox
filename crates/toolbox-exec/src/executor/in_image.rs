//! `in-image`: a program shipped in the image, spoken to with executor protocol v1 over stdio
//! (`schema/executor-messages.schema.json`).

use super::{Context, Executor, Outcome, Preview, argv, clean_env, names};
use serde_json::{Map, Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::Arc;
use toolbox_model::manifest::Forge;

/// An `in-image` executor: a program in the image, spoken to over the stdio protocol.
#[derive(Debug, Clone)]
pub struct InImage {
    pub command: Vec<String>,
    pub env: Vec<String>,
}

pub(crate) fn parse(v: &Value, _: &[Forge]) -> Result<Arc<dyn Executor>, String> {
    Ok(Arc::new(InImage {
        command: argv(v.get("command"), "executor.command")?,
        env: names(v, "secrets"),
    }))
}

impl Executor for InImage {
    fn kind(&self) -> &'static str {
        "in-image"
    }

    /// A program that is not installed here (a checkout, not the image) previews the Toolbox
    /// layer only, with a warning that says so.
    fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        match stdio_preview(&self.command, ctx, &self.env) {
            Err(e) if e.starts_with("cannot start ") => Ok(Preview {
                warnings: vec![format!(
                    "{e}: the executor is not installed here, so this preview checks inputs, preconditions and approvals only"
                )],
                ..Preview::default()
            }),
            other => other,
        }
    }

    fn run(&self, ctx: &Context, confirmed: &[Value]) -> Outcome {
        stdio_run(&self.command, ctx, confirmed, &self.env)
    }
}

fn stdio_session(
    command: &[String],
    ctx: &Context,
    msg: &Value,
    env: &[String],
) -> Result<Vec<Value>, String> {
    let mut cmd = Command::new(&command[0]);
    clean_env(&mut cmd, env);
    let mut child = cmd
        .args(&command[1..])
        .current_dir(&ctx.cwd)
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
            Some("preview_result" | "result" | "error")
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

fn protocol_context(ctx: &Context) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("action".into(), ctx.action.clone().into());
    m.insert("action_version".into(), ctx.action_version.clone().into());
    m.insert("entity".into(), ctx.entity.clone());
    m.insert("inputs".into(), Value::Object(ctx.inputs.clone()));
    let mode = if ctx.actor.starts_with("user:") {
        "user"
    } else {
        "service"
    };
    m.insert(
        "identity".into(),
        json!({"mode": mode, "principal": ctx.actor}),
    );
    m.insert("related".into(), Value::Object(ctx.related.clone()));
    m
}

fn stdio_preview(command: &[String], ctx: &Context, env: &[String]) -> Result<Preview, String> {
    let mut msg = protocol_context(ctx);
    msg.insert("type".into(), "preview".into());
    msg.insert("request_id".into(), "req_preview".into());
    let msgs = stdio_session(command, ctx, &Value::Object(msg), env)?;
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

fn stdio_run(command: &[String], ctx: &Context, confirmed: &[Value], env: &[String]) -> Outcome {
    let run_id = ctx.run_id.clone().unwrap_or_default();
    let mut msg = protocol_context(ctx);
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
    let msgs = match stdio_session(command, ctx, &Value::Object(msg), env) {
        Ok(m) => m,
        Err(e) => return Outcome::failed(e),
    };
    let progress: Vec<Value> = msgs
        .iter()
        .filter(|m| m["type"] == "progress")
        .cloned()
        .collect();
    match msgs.iter().find(|m| m["type"] == "result") {
        Some(r) => Outcome {
            status: if r["status"] == "succeeded" {
                "succeeded".into()
            } else {
                "failed".into()
            },
            outputs: r.get("outputs").cloned().unwrap_or(json!({})),
            error: r.get("error").map(|e| {
                e.get("message")
                    .and_then(Value::as_str)
                    .map_or_else(|| e.to_string(), str::to_owned)
            }),
            progress,
            child: None,
        },
        None => Outcome {
            progress,
            ..Outcome::failed("the executor ended without a result")
        },
    }
}
