//! `command`: the action's command, from the directory toolbox started in. With `preview: true`
//! it is also its own preview, run with `TOOLBOX_PHASE=preview`.

use super::{Context, Executor, Outcome, Preview, argv, names, outputs_from, spawn};
use serde_json::{Value, json};
use std::sync::Arc;
use toolbox_model::manifest::Forge;

/// A `command` executor: runs one command in the checkout.
#[derive(Debug, Clone)]
pub struct Command {
    pub run: Vec<String>,
    pub preview: bool,
    pub env: Vec<String>,
}

pub(crate) fn parse(v: &Value, _: &[Forge]) -> Result<Arc<dyn Executor>, String> {
    Ok(Arc::new(Command {
        run: argv(v.get("run"), "executor.run")?,
        preview: v.get("preview").and_then(Value::as_bool).unwrap_or(false),
        env: names(v, "secrets"),
    }))
}

impl Executor for Command {
    fn kind(&self) -> &'static str {
        "command"
    }

    fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        if !self.preview {
            return Ok(Preview {
                effects: vec![
                    json!({"operation": "run", "resource": format!("command:{}", self.run.join(" ")),
                    "description": "runs the command; it has no preview of its own"}),
                ],
                ..Preview::default()
            });
        }
        let out = spawn(&self.run, ctx, "preview", &ctx.cwd, &self.env)?;
        if !out.ok {
            return Err(format!("preview failed: {}", out.tail()));
        }
        Ok(Preview {
            effects: super::effects_from(&out.stdout),
            executed: true,
            ..Preview::default()
        })
    }

    fn run(&self, ctx: &Context, _: &[Value]) -> Outcome {
        match spawn(&self.run, ctx, "run", &ctx.cwd, &self.env) {
            Err(e) => Outcome::failed(e),
            Ok(out) if !out.ok => Outcome::failed(format!(
                "exit status {}: {}",
                out.code.map_or("signal".into(), |c| c.to_string()),
                out.tail()
            )),
            Ok(out) => Outcome {
                status: "succeeded".into(),
                outputs: outputs_from(&out.stdout),
                error: None,
                progress: vec![],
                child: None,
            },
        }
    }
}
