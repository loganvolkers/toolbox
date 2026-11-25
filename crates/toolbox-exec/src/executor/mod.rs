//! Executors (docs/design/action-runs.md §5, docs/design/federated-actions.md §5): what does the
//! work once a run is confirmed. Every type answers the same questions — what would you change
//! (preview), do it (run), and, for a run that waits on the world, has it happened yet (recheck)
//! — so the lifecycle around them is one piece of code. Each type is one [`Executor`] in its own
//! module, and [`parse_in`] is the one table of them, held equal to the manifest schema's
//! `executor.type` (`toolbox_model::executors`).
//!
//! | `type` | Preview | Run |
//! |---|---|---|
//! | `command` | the same command with `TOOLBOX_PHASE=preview`, when `preview: true` | the command; its last JSON line is the outputs |
//! | `in-image` | executor protocol v1 `preview` over stdio | protocol `run`: `accepted`, `progress`, `result` |
//! | `change` | the command in a throwaway git worktree; the effects are the files it changed, with their diffs | the same, committed and pushed to a new branch: the run is `proposed` until its commit is on the base. With `land: push`, pushed to the base once approved (never forced) and done when the base is seen to have it |
//! | `github-workflow` | the Toolbox layer only | `workflow_dispatch`, then the run is tracked by its `run-name` ([`crate::ci`]) |
//! | `gitlab-pipeline` | the Toolbox layer only | a pipeline with the inputs, tracked by its id ([`crate::ci`]) |
//! | `toolbox` | the child portal's own preview | the child portal's own run (federated dispatch) |
//!
//! A command gets the run's context as JSON on stdin and in the environment: `TOOLBOX_PHASE`,
//! `TOOLBOX_ACTION`, `TOOLBOX_RUN_ID`, `TOOLBOX_ENTITY`, `TOOLBOX_ACTOR`, `TOOLBOX_INPUTS` (JSON)
//! and one `TOOLBOX_INPUT_<NAME>` per input (strings as themselves, other values as JSON).
//!
//! Every process an executor starts gets a cleared environment: the [`BASE_ENV`] set plus the
//! names the action lists in `secrets` (a child portal: the parent's `runners.local.env`). A
//! server's own credentials reach an action only when its manifest names them.

pub mod change;
pub mod command;
pub mod federated;
pub mod in_image;

pub use change::{landed, merged, unchanged};
pub use federated::{portal_repo_dir, verify_child};

use crate::run::Run;
use serde_json::{Map, Value, json};
use std::any::Any;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use toolbox_model::manifest::Forge;

/// What an executor is told about a run.
#[derive(Debug, Clone)]
pub struct Context {
    pub run_id: Option<String>,
    /// The portal the run was requested through; a child portal sees `--parent-run <portal>/<id>`.
    pub portal: String,
    pub action: String,
    pub action_version: String,
    pub entity: Value,
    pub inputs: Map<String, Value>,
    /// `user:<id>` or the portal's service identity.
    pub actor: String,
    /// Who approved the run, once it is approved.
    pub approved_by: Vec<String>,
    pub related: Map<String, Value>,
    /// The parent's signed approval grant (`crate::grant`), passed to a child portal on dispatch.
    pub grant: Option<String>,
    /// Where commands run: the portal's repository root, where its connectors' `root: .` points.
    pub cwd: PathBuf,
}

/// The executor layer of a preview.
#[derive(Debug, Clone, Default)]
pub struct Preview {
    pub effects: Vec<Value>,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    /// Whether the executor itself previewed (false: the Toolbox layer only).
    pub executed: bool,
    /// For a federated action: the child portal's preview, as it returned it.
    pub child: Option<Value>,
}

/// How an executor finished.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// `succeeded`, `failed`, `proposed` (a change awaiting review) or `dispatched` (handed to a
    /// remote runner that reports later).
    pub status: String,
    pub outputs: Value,
    pub error: Option<String>,
    pub progress: Vec<Value>,
    /// For a federated action: the child portal's run record.
    pub child: Option<Value>,
}

impl Outcome {
    pub(crate) fn failed(error: impl Into<String>) -> Self {
        Self {
            status: "failed".into(),
            outputs: json!({}),
            error: Some(error.into()),
            progress: vec![],
            child: None,
        }
    }
}

/// What every executor type does. A run that waits on the world (a `proposed` change, a
/// `dispatched` pipeline, a child's run) is looked at again by [`Executor::recheck`], from a
/// fresh process if need be: everything it needs is in the run record.
pub trait Executor: std::fmt::Debug + Send + Sync + Any {
    /// The `type` a run records.
    fn kind(&self) -> &'static str;

    /// What the run would change. A command, in-image or change executor is the action author's
    /// code and runs here, in a cleared environment; it is asked not to change anything
    /// (`TOOLBOX_PHASE=preview`, a throwaway worktree), which only the author's code can honour.
    fn preview(&self, ctx: &Context) -> Result<Preview, String>;

    /// Run. `confirmed` are the effects the requester confirmed.
    fn run(&self, ctx: &Context, confirmed: &[Value]) -> Outcome;

    /// One look at a run this executor left waiting, before its postcondition. `expects`: the
    /// action declares one.
    fn recheck(&self, _run: &mut Run, _expects: bool, _cwd: &Path) -> Rechecked {
        Rechecked::Verify
    }

    /// Whether review of what the run proposes is its approval.
    fn reviewed(&self) -> bool {
        false
    }
}

impl dyn Executor {
    /// This executor as its own type, if it is one.
    pub fn downcast<T: Executor>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref()
    }
}

/// Where [`Executor::recheck`] left a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rechecked {
    /// The look is over: the run changed state, or there is nothing more to see yet.
    Done,
    /// The executor's part is done: check the postcondition.
    Verify,
}

type Parse = fn(&Value, &[Forge]) -> Result<Arc<dyn Executor>, String>;

/// Every `type` a manifest may name, with its parser.
const PARSERS: &[(&str, Parse)] = &[
    ("command", command::parse),
    ("in-image", in_image::parse),
    ("change", change::parse),
    ("github-workflow", crate::ci::github::parse),
    ("gitlab-pipeline", crate::ci::gitlab::parse),
];

/// Parse an action's `executor:` value, with no forges but the known hosts.
pub fn parse(v: &Value) -> Result<Arc<dyn Executor>, String> {
    parse_in(v, &[])
}

/// Parse an action's `executor:` value in a portal that declares `forges`.
pub fn parse_in(v: &Value, forges: &[Forge]) -> Result<Arc<dyn Executor>, String> {
    let ty = v.get("type").and_then(Value::as_str).unwrap_or("command");
    let Some((_, parse)) = PARSERS.iter().find(|(name, _)| *name == ty) else {
        return Err(format!(
            "unknown executor type {ty} ({})",
            toolbox_model::executors::names()
        ));
    };
    parse(v, forges)
}

/// Variables every executor process keeps: where it runs, who it runs as, locale, proxies, git's
/// identity and SSH agent, and Toolbox's own directories. Nothing here is a credential.
pub const BASE_ENV: &[&str] = &[
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "LANG",
    "LANGUAGE",
    "TERM",
    "TZ",
    "TMPDIR",
    "SSH_AUTH_SOCK",
    "GIT_SSH_COMMAND",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
    "ALL_PROXY",
    "http_proxy",
    "https_proxy",
    "no_proxy",
    "all_proxy",
    "TOOLBOX_STATE_DIR",
    "TOOLBOX_CACHE_DIR",
    "TOOLBOX_CONNECTORS_DIR",
];
const BASE_PREFIXES: &[&str] = &[
    "LC_",
    "XDG_",
    "GIT_CONFIG_",
    "GIT_AUTHOR_",
    "GIT_COMMITTER_",
];

/// Clear `cmd`'s environment down to [`BASE_ENV`] and `extra`.
pub fn clean_env(cmd: &mut Command, extra: &[String]) {
    cmd.env_clear();
    for (k, v) in std::env::vars_os() {
        let Some(name) = k.to_str() else { continue };
        if BASE_ENV.contains(&name)
            || BASE_PREFIXES.iter().any(|p| name.starts_with(p))
            || extra.iter().any(|e| e == name)
        {
            cmd.env(&k, v);
        }
    }
}

pub(crate) fn names(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// `run: [argv…]` or `run: "shell line"` (run by `sh -c`).
pub(crate) fn argv(v: Option<&Value>, what: &str) -> Result<Vec<String>, String> {
    match v {
        Some(Value::String(s)) => Ok(vec!["sh".into(), "-c".into(), s.clone()]),
        Some(Value::Array(a)) if !a.is_empty() => a
            .iter()
            .map(|x| {
                x.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| format!("{what} must be strings"))
            })
            .collect(),
        _ => Err(format!(
            "{what} is required: a command line or a list of arguments"
        )),
    }
}

pub(crate) struct Spawned {
    pub(crate) ok: bool,
    pub(crate) code: Option<i32>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

impl Spawned {
    pub(crate) fn tail(&self) -> String {
        let text = if self.stderr.trim().is_empty() {
            &self.stdout
        } else {
            &self.stderr
        };
        let lines: Vec<&str> = text.lines().collect();
        lines[lines.len().saturating_sub(5)..].join("\n")
    }
}

fn context_json(ctx: &Context, phase: &str) -> Value {
    json!({"phase": phase, "run_id": ctx.run_id, "action": ctx.action, "action_version": ctx.action_version,
        "entity": ctx.entity, "inputs": ctx.inputs, "identity": {"principal": ctx.actor},
        "related": ctx.related})
}

fn env_name(k: &str) -> String {
    k.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn spawn(
    argv: &[String],
    ctx: &Context,
    phase: &str,
    cwd: &Path,
    env: &[String],
) -> Result<Spawned, String> {
    let mut cmd = Command::new(&argv[0]);
    clean_env(&mut cmd, env);
    cmd.args(&argv[1..])
        .current_dir(cwd)
        .env("TOOLBOX_PHASE", phase)
        .env("TOOLBOX_ACTION", &ctx.action)
        .env("TOOLBOX_RUN_ID", ctx.run_id.as_deref().unwrap_or(""))
        .env(
            "TOOLBOX_ENTITY",
            ctx.entity.get("id").and_then(Value::as_str).unwrap_or(""),
        )
        .env("TOOLBOX_ACTOR", &ctx.actor)
        .env(
            "TOOLBOX_INPUTS",
            Value::Object(ctx.inputs.clone()).to_string(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in &ctx.inputs {
        let s = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
        cmd.env(format!("TOOLBOX_INPUT_{}", env_name(k)), s);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("cannot start {}: {e}", argv[0]))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(context_json(ctx, phase).to_string().as_bytes());
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    Ok(Spawned {
        ok: out.status.success(),
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// A preview's stdout: a JSON list of effects, `{"effects": […]}`, one JSON effect per line, or
/// plain lines (each becomes `{"description": line}`).
pub fn effects_from(stdout: &str) -> Vec<Value> {
    match serde_json::from_str::<Value>(stdout.trim()) {
        Ok(Value::Array(a)) => return a,
        Ok(Value::Object(o)) if o.get("effects").is_some_and(Value::is_array) => {
            return o["effects"].as_array().cloned().unwrap_or_default();
        }
        _ => {}
    }
    stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| match serde_json::from_str::<Value>(l) {
            Ok(v @ Value::Object(_)) => v,
            _ => json!({"description": l}),
        })
        .collect()
}

/// A run's stdout: its last line that is a JSON object is the outputs.
pub fn outputs_from(stdout: &str) -> Value {
    stdout
        .lines()
        .rev()
        .find_map(|l| match serde_json::from_str::<Value>(l.trim()) {
            Ok(v @ Value::Object(_)) => Some(v),
            _ => None,
        })
        .unwrap_or_else(|| json!({}))
}

/// `git` that returns what the remote said (stderr) as well as stdout.
pub(crate) fn git_said(dir: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    clean_env(&mut cmd, &[]);
    let out = cmd
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if !out.status.success() {
        return Err(format!("git {}: {}", args.join(" "), said.trim()));
    }
    Ok(said)
}

pub(crate) fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let mut cmd = Command::new("git");
    clean_env(&mut cmd, &[]);
    let out = cmd
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// Whether `id` is a run id as `crate::run::new_id` makes them.
pub fn is_run_id(id: &str) -> bool {
    id.strip_prefix("run_").is_some_and(|r| {
        !r.is_empty()
            && r.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_executor_sees_only_the_base_environment_and_its_secrets() {
        let allowed =
            |n: &str| BASE_ENV.contains(&n) || BASE_PREFIXES.iter().any(|p| n.starts_with(p));
        let Some(name) = std::env::vars().map(|(k, _)| k).find(|k| !allowed(k)) else {
            return;
        };
        let seen = |extra: &[String]| {
            let mut cmd = Command::new("env");
            clean_env(&mut cmd, extra);
            let out = String::from_utf8(cmd.output().unwrap().stdout).unwrap();
            out.lines().any(|l| l.starts_with(&format!("{name}=")))
        };
        assert!(
            !seen(&[]),
            "{name} reached an executor that did not ask for it"
        );
        assert!(seen(std::slice::from_ref(&name)));
    }

    #[test]
    fn effects_take_every_shape() {
        assert_eq!(
            effects_from(r#"[{"resource":"a"}]"#),
            vec![json!({"resource": "a"})]
        );
        assert_eq!(
            effects_from(r#"{"effects":[{"resource":"b"}]}"#),
            vec![json!({"resource": "b"})]
        );
        assert_eq!(
            effects_from("tag v1.2.0\n{\"resource\":\"c\"}\n"),
            vec![
                json!({"description": "tag v1.2.0"}),
                json!({"resource": "c"})
            ]
        );
    }

    #[test]
    fn outputs_are_the_last_json_object_line() {
        assert_eq!(
            outputs_from("working\n{\"a\":1}\n{\"version\":\"1.2.0\"}\ndone\n"),
            json!({"version": "1.2.0"})
        );
        assert_eq!(outputs_from("nothing"), json!({}));
    }

    #[test]
    fn executors_parse_or_say_what_is_missing() {
        assert_eq!(
            parse(&json!({"run": "make release"})).unwrap().kind(),
            "command"
        );
        assert!(
            parse(&json!({"type": "change"}))
                .unwrap_err()
                .contains("executor.run is required")
        );
        assert!(
            parse(&json!({"type": "lambda"}))
                .unwrap_err()
                .contains("unknown executor type")
        );
    }

    #[test]
    fn inputs_become_environment_names() {
        assert_eq!(env_name("bump-part"), "BUMP_PART");
    }

    #[test]
    fn run_ids_are_only_what_new_id_makes() {
        assert!(is_run_id(&crate::run::new_id("run")));
        for bad in [
            "run_",
            "../x",
            "run_../../etc",
            "run_A1",
            "x_run_1",
            "/tmp/r.json",
        ] {
            assert!(!is_run_id(bad), "{bad}");
        }
    }

    #[test]
    fn the_parsers_are_the_types_the_schema_names() {
        let ours: Vec<&str> = PARSERS.iter().map(|(n, _)| *n).collect();
        let model: Vec<&str> = toolbox_model::executors::TYPES
            .iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(ours, model);
    }
}
