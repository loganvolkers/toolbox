//! Executors (docs/design/action-runs.md §5, docs/design/federated-actions.md §5): what does the
//! work once a run is confirmed. Every type answers the same two questions — what would you
//! change (preview) and do it (run) — so the lifecycle around them is one piece of code.
//!
//! | `type` | Preview | Run |
//! |---|---|---|
//! | `command` | the same command with `TOOLBOX_PHASE=preview`, when `preview: true` | the command; its last JSON line is the outputs |
//! | `in-image` | executor protocol v1 `preview` over stdio | protocol `run`: `accepted`, `progress`, `result` |
//! | `change` | the command in a throwaway git worktree; the effects are the files it changed, with their diffs | the same, committed and pushed to a new branch: the run is `proposed` until its commit is on the base. With `land: push`, pushed to the base once approved (never forced) and done when the base is seen to have it |
//! | `github-workflow` | the Toolbox layer only | `workflow_dispatch`, then the run is tracked by its `run-name` |
//! | `toolbox` | the child portal's own preview | the child portal's own run (federated dispatch) |
//!
//! A command gets the run's context as JSON on stdin and in the environment: `TOOLBOX_PHASE`,
//! `TOOLBOX_ACTION`, `TOOLBOX_RUN_ID`, `TOOLBOX_ENTITY`, `TOOLBOX_ACTOR`, `TOOLBOX_INPUTS` (JSON)
//! and one `TOOLBOX_INPUT_<NAME>` per input (strings as themselves, other values as JSON).
//!
//! Every process an executor starts gets a cleared environment: the [`BASE_ENV`] set plus the
//! names the action lists in `secrets` (a child portal: the parent's `runners.local.env`). A
//! server's own credentials reach an action only when its manifest names them.

use serde_json::{Map, Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    fn failed(error: impl Into<String>) -> Self {
        Self {
            status: "failed".into(),
            outputs: json!({}),
            error: Some(error.into()),
            progress: vec![],
            child: None,
        }
    }
}

/// A parsed `executor:` block.
#[derive(Debug, Clone)]
pub enum Executor {
    Command {
        run: Vec<String>,
        preview: bool,
        env: Vec<String>,
    },
    InImage {
        command: Vec<String>,
        env: Vec<String>,
    },
    Change {
        run: Vec<String>,
        base: Option<String>,
        remote: String,
        branch: Option<String>,
        message: Option<String>,
        env: Vec<String>,
        /// `land: push`: push the commit to the base once approved, instead of proposing a
        /// branch for review. The action's approval rule is then the only review.
        push: bool,
    },
    GithubWorkflow(crate::github::Workflow),
    /// Dispatch to a child portal (federation): the child previews, runs and verifies.
    Toolbox {
        portal: PathBuf,
        action: String,
        entity: String,
        program: PathBuf,
        env: Vec<String>,
        /// A child cloned from git (`federation.children`): the branch its clone is brought to
        /// before each preview and run, so the definition that runs is the one committed there.
        branch: Option<String>,
    },
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

fn names(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

/// `run: [argv…]` or `run: "shell line"` (run by `sh -c`).
fn argv(v: Option<&Value>, what: &str) -> Result<Vec<String>, String> {
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

impl Executor {
    /// Parse an action's `executor:` value.
    pub fn parse(v: &Value) -> Result<Self, String> {
        let ty = v.get("type").and_then(Value::as_str).unwrap_or("command");
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
        Ok(match ty {
            "command" => Executor::Command {
                run: argv(v.get("run"), "executor.run")?,
                preview: v.get("preview").and_then(Value::as_bool).unwrap_or(false),
                env: names(v, "secrets"),
            },
            "in-image" => Executor::InImage {
                command: argv(v.get("command"), "executor.command")?,
                env: names(v, "secrets"),
            },
            "change" => Executor::Change {
                run: argv(v.get("run"), "executor.run")?,
                base: s("base"),
                remote: s("remote").unwrap_or_else(|| "origin".into()),
                branch: s("branch"),
                message: s("message"),
                env: names(v, "secrets"),
                push: match s("land").as_deref() {
                    None | Some("propose") => false,
                    Some("push") => true,
                    Some(other) => {
                        return Err(format!(
                            "executor.land is propose (a branch for review) or push (straight to the base once approved), not {other}"
                        ));
                    }
                },
            },
            "github-workflow" => Executor::GithubWorkflow(crate::github::Workflow::parse(v)?),
            other => {
                return Err(format!(
                    "unknown executor type {other} (command, in-image, change, github-workflow)"
                ));
            }
        })
    }

    /// The `type` name.
    pub fn kind(&self) -> &'static str {
        match self {
            Executor::Command { .. } => "command",
            Executor::InImage { .. } => "in-image",
            Executor::Change { .. } => "change",
            Executor::GithubWorkflow(_) => "github-workflow",
            Executor::Toolbox { .. } => "toolbox",
        }
    }

    /// Preview: what the run would change. A command, in-image or change executor is the action
    /// author's code and runs here, in a cleared environment; it is asked not to change anything
    /// (`TOOLBOX_PHASE=preview`, a throwaway worktree), which only the author's code can honour.
    pub fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        match self {
            Executor::Command { run, preview, env } => {
                if !preview {
                    return Ok(Preview {
                        effects: vec![
                            json!({"operation": "run", "resource": format!("command:{}", run.join(" ")),
                            "description": "runs the command; it has no preview of its own"}),
                        ],
                        ..Preview::default()
                    });
                }
                let out = spawn(run, ctx, "preview", &ctx.cwd, env)?;
                if !out.ok {
                    return Err(format!("preview failed: {}", out.tail()));
                }
                Ok(Preview {
                    effects: effects_from(&out.stdout),
                    executed: true,
                    ..Preview::default()
                })
            }
            Executor::InImage { command, env } => stdio_preview(command, ctx, env),
            Executor::Change {
                run,
                base,
                remote,
                env,
                push,
                ..
            } => {
                let wt = Worktree::new(&ctx.cwd, base.as_deref(), remote)?;
                let out = spawn(run, ctx, "preview", &wt.dir, env)?;
                if !out.ok {
                    return Err(format!("change command failed: {}", out.tail()));
                }
                let files = wt.changes()?;
                let mut effects: Vec<Value> = files
                    .iter()
                    .map(|(op, path)| {
                        json!({"resource": format!("file:{path}"), "operation": op,
                        "diff": wt.diff(path)})
                    })
                    .collect();
                let mut warnings = Vec::new();
                if effects.is_empty() {
                    warnings.push(format!(
                        "the change command changed no files: {} already has what it writes, so a \
                         run commits nothing and is observed done once it sees the base unchanged",
                        wt.base
                    ));
                } else if *push {
                    effects.push(
                        json!({"resource": format!("branch:{}", base_branch(&wt.base, remote)),
                        "operation": "push", "base": wt.base,
                        "description": "one commit, pushed once approved (never forced)"}),
                    );
                } else {
                    effects.push(
                        json!({"resource": format!("branch:{}", branch_name(self, ctx)),
                        "operation": "propose", "base": wt.base}),
                    );
                }
                Ok(Preview {
                    effects,
                    warnings,
                    executed: true,
                    ..Preview::default()
                })
            }
            Executor::GithubWorkflow(w) => Ok(Preview {
                effects: vec![
                    json!({"resource": format!("github:{}/actions/workflows/{}", w.repo, w.workflow),
                    "operation": "dispatch", "ref": w.git_ref}),
                ],
                ..Preview::default()
            }),
            Executor::Toolbox {
                portal,
                action,
                entity,
                program,
                env,
                branch,
            } => {
                sync_clone(portal, branch.as_deref())?;
                let v = child_run(program, portal, action, entity, ctx, &["--preview"], env)?;
                if let Some(e) = v.get("error").and_then(Value::as_str) {
                    return Err(format!("child portal: {e}"));
                }
                let strings = |k: &str| -> Vec<String> {
                    v.get(k)
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                };
                Ok(Preview {
                    effects: v
                        .get("effects")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                    warnings: strings("warnings"),
                    blockers: strings("blockers"),
                    executed: true,
                    child: Some(v),
                })
            }
        }
    }

    /// Run. `confirmed` are the effects the requester confirmed.
    pub fn run(&self, ctx: &Context, confirmed: &[Value]) -> Outcome {
        match self {
            Executor::Command { run, env, .. } => match spawn(run, ctx, "run", &ctx.cwd, env) {
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
            },
            Executor::InImage { command, env } => stdio_run(command, ctx, confirmed, env),
            Executor::Change {
                run,
                base,
                remote,
                message,
                env,
                push,
                ..
            } => change_run(
                run,
                ctx,
                base.as_deref(),
                remote,
                (!push).then(|| branch_name(self, ctx)).as_deref(),
                message.as_deref(),
                env,
                confirmed,
            )
            .unwrap_or_else(Outcome::failed),
            Executor::GithubWorkflow(w) => w.dispatch(ctx),
            Executor::Toolbox {
                portal,
                action,
                entity,
                program,
                env,
                branch,
            } => match sync_clone(portal, branch.as_deref())
                .and_then(|()| child_run(program, portal, action, entity, ctx, &["--yes"], env))
            {
                Err(e) => Outcome::failed(e),
                Ok(v) => {
                    let state = v.get("state").and_then(Value::as_str).unwrap_or("failed");
                    Outcome {
                        status: match state {
                            "succeeded" | "unverified" => "succeeded".into(),
                            "proposed" | "awaiting_approval" => state.into(),
                            _ => "failed".into(),
                        },
                        outputs: v.get("outputs").cloned().unwrap_or(json!({})),
                        error: v
                            .get("error")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                            .or_else(|| {
                                (state != "succeeded" && state != "unverified")
                                    .then(|| format!("child run ended {state}"))
                            }),
                        progress: vec![],
                        child: Some(v),
                    }
                }
            },
        }
    }
}

fn branch_name(e: &Executor, ctx: &Context) -> String {
    match e {
        Executor::Change {
            branch: Some(b), ..
        } => fill(b, ctx, false),
        _ => format!(
            "toolbox/{}/{}",
            ctx.action,
            ctx.run_id.as_deref().unwrap_or("<run id>")
        ),
    }
}

/// `branch` or `message` with its placeholders filled. Inputs go only into a message: an input
/// can hold characters a branch name cannot.
fn fill(t: &str, ctx: &Context, inputs: bool) -> String {
    let id = ctx.run_id.as_deref().unwrap_or("<run id>");
    let mut out = t
        .replace("{action}", &ctx.action)
        .replace("{run}", id)
        .replace("{run_id}", id);
    if inputs {
        out = out.replace("{entity}", ctx.entity["id"].as_str().unwrap_or(""));
        for (k, v) in &ctx.inputs {
            let v = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
            out = out.replace(&format!("{{inputs.{k}}}"), &v);
        }
    }
    out
}

struct Spawned {
    ok: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Spawned {
    fn tail(&self) -> String {
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

fn spawn(
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

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
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

/// A detached worktree of the repository at the base commit, removed on drop.
struct Worktree {
    repo: PathBuf,
    dir: PathBuf,
    base: String,
}

impl Worktree {
    /// A change starts from the remote's base branch when there is one, so a stale or
    /// differently named local branch does not leak into the proposal.
    fn new(cwd: &Path, base: Option<&str>, remote: &str) -> Result<Self, String> {
        let repo = PathBuf::from(git(cwd, &["rev-parse", "--show-toplevel"])?);
        let base = match base {
            Some(b) => {
                let _ = git(&repo, &["fetch", "-q", remote, b]);
                let tracking = format!("refs/remotes/{remote}/{b}");
                if git(&repo, &["rev-parse", "-q", "--verify", &tracking]).is_ok() {
                    format!("{remote}/{b}")
                } else {
                    b.to_owned()
                }
            }
            None => git(&repo, &["symbolic-ref", "--short", "HEAD"]).unwrap_or("HEAD".into()),
        };
        let dir = std::env::temp_dir().join(format!(
            "toolbox-change-{}-{}",
            std::process::id(),
            crate::run::new_id("wt")
        ));
        git(
            &repo,
            &[
                "worktree",
                "add",
                "--detach",
                dir.to_str().unwrap_or(""),
                &base,
            ],
        )?;
        // A repo portal under a subdirectory runs from the matching subdirectory of the worktree.
        let sub = cwd
            .canonicalize()
            .ok()
            .and_then(|c| {
                repo.canonicalize()
                    .ok()
                    .and_then(|r| c.strip_prefix(r).ok().map(Path::to_path_buf))
            })
            .unwrap_or_default();
        let wt = Self {
            repo,
            dir: dir.join(sub),
            base,
        };
        Ok(wt)
    }

    fn root(&self) -> PathBuf {
        let mut d = self.dir.clone();
        while d.parent().is_some() && !d.join(".git").exists() {
            d.pop();
        }
        d
    }

    /// The staged diff of one changed file, as a preview shows it (cut at 4,000 bytes).
    fn diff(&self, path: &str) -> String {
        let mut d = git(&self.root(), &["diff", "--cached", "--", path]).unwrap_or_default();
        if d.len() > 4000 {
            let mut end = 4000;
            while !d.is_char_boundary(end) {
                end -= 1;
            }
            d.truncate(end);
            d.push_str("\n… (cut)");
        }
        d
    }

    /// `(operation, path)` for every file the command changed.
    fn changes(&self) -> Result<Vec<(String, String)>, String> {
        let root = self.root();
        git(&root, &["add", "-A"])?;
        let out = git(&root, &["diff", "--cached", "--name-status"])?;
        Ok(out
            .lines()
            .filter_map(|l| {
                let (st, path) = l.split_once('\t')?;
                let op = match st.chars().next()? {
                    'A' => "create",
                    'D' => "delete",
                    _ => "update",
                };
                Some((op.to_owned(), path.to_owned()))
            })
            .collect())
    }
}

impl Drop for Worktree {
    fn drop(&mut self) {
        let root = self.root();
        let _ = git(
            &self.repo,
            &["worktree", "remove", "--force", root.to_str().unwrap_or("")],
        );
    }
}

/// The branch a base names: `origin/main` → `main`.
fn base_branch<'a>(base: &'a str, remote: &str) -> &'a str {
    base.strip_prefix(remote)
        .and_then(|b| b.strip_prefix('/'))
        .unwrap_or(base)
}

/// The file effects whose diff differs between a confirmed preview and the run (`[]` when the
/// confirmation carries no file diffs).
fn drifted(confirmed: &[Value], wt: &Worktree, files: &[(String, String)]) -> Vec<String> {
    let shown: std::collections::BTreeMap<&str, &str> = confirmed
        .iter()
        .filter_map(|e| {
            Some((
                e["resource"].as_str()?.strip_prefix("file:")?,
                e["diff"].as_str()?,
            ))
        })
        .collect();
    if shown.is_empty() {
        return vec![];
    }
    let now: std::collections::BTreeMap<&str, String> = files
        .iter()
        .map(|(_, p)| (p.as_str(), wt.diff(p)))
        .collect();
    let mut paths: std::collections::BTreeSet<&str> = shown.keys().copied().collect();
    paths.extend(now.keys().copied());
    paths
        .into_iter()
        .filter(|p| shown.get(p).copied() != now.get(p).map(String::as_str))
        .map(str::to_owned)
        .collect()
}

/// Make the change in a worktree from the base and commit it; then propose it on `branch`, or,
/// without one (`land: push`), push it to the base. A change that differs from the confirmed
/// preview (the base moved) fails before anything is committed.
#[allow(clippy::too_many_arguments)]
fn change_run(
    run: &[String],
    ctx: &Context,
    base: Option<&str>,
    remote: &str,
    branch: Option<&str>,
    message: Option<&str>,
    env: &[String],
    confirmed: &[Value],
) -> Result<Outcome, String> {
    let wt = Worktree::new(&ctx.cwd, base, remote)?;
    let out = spawn(run, ctx, "run", &wt.dir, env)?;
    if !out.ok {
        return Ok(Outcome::failed(format!(
            "change command failed: {}",
            out.tail()
        )));
    }
    let files = wt.changes()?;
    let drift = drifted(confirmed, &wt, &files);
    if !drift.is_empty() {
        return Ok(Outcome::failed(format!(
            "{} on {} no longer match the confirmed preview (the base moved): nothing was \
             committed; preview and request again",
            drift.join(", "),
            wt.base
        )));
    }
    if files.is_empty() {
        let at = git(&wt.root(), &["rev-parse", "HEAD"])?;
        return Ok(Outcome {
            status: "succeeded".into(),
            outputs: json!({"changed": false, "base": wt.base, "at": at}),
            error: None,
            progress: vec![],
            child: None,
        });
    }
    let root = wt.root();
    if let Some(branch) = branch {
        git(&root, &["switch", "-c", branch])?;
    }
    let msg = message.map_or_else(
        || {
            format!(
                "{} on {} (toolbox run {})",
                ctx.action,
                ctx.entity["id"].as_str().unwrap_or(""),
                ctx.run_id.as_deref().unwrap_or("")
            )
        },
        |m| fill(m, ctx, true),
    );
    let ident = [
        "-c",
        "user.name=toolbox",
        "-c",
        "user.email=toolbox@localhost",
    ];
    let mut args: Vec<&str> = Vec::new();
    if git(&root, &["config", "user.email"]).is_err() {
        args.extend(ident);
    }
    let trailer = format!("Toolbox-Run: {}", ctx.run_id.as_deref().unwrap_or(""));
    let mut trailers = format!("{trailer}\nRequested-By: {}", ctx.actor);
    for who in &ctx.approved_by {
        trailers.push_str(&format!("\nApproved-By: {who}"));
    }
    args.extend(["commit", "-q", "-m", &msg, "-m", &trailers]);
    git(&root, &args)?;
    let commit = git(&root, &["rev-parse", "HEAD"])?;
    // From the main checkout: a relative remote URL resolves against it, not the worktree.
    let has_remote = git(&wt.repo, &["remote", "get-url", "--", remote]).ok();
    let files: Vec<Value> = files
        .iter()
        .map(|(op, p)| json!({"operation": op, "path": p}))
        .collect();
    let Some(branch) = branch else {
        let target = base_branch(&wt.base, remote);
        if has_remote.is_none() {
            return Ok(Outcome::failed(format!(
                "land: push needs the remote {remote}, and this repository has none"
            )));
        }
        let refspec = format!("{commit}:refs/heads/{target}");
        if let Err(e) = git(&wt.repo, &["push", "-q", "--", remote, &refspec]) {
            return Ok(Outcome::failed(format!(
                "the push to {remote} {target} was refused, so nothing landed: {e}"
            )));
        }
        return Ok(Outcome {
            status: "succeeded".into(),
            outputs: json!({"landed": true, "base": wt.base, "commit": commit,
                "remote": has_remote, "files": files}),
            error: None,
            progress: vec![],
            child: None,
        });
    };
    let pushed = match &has_remote {
        Some(_) => {
            git(&wt.repo, &["push", "-q", remote, branch])?;
            true
        }
        None => false,
    };
    Ok(Outcome {
        status: "proposed".into(),
        outputs: json!({"branch": branch, "base": wt.base, "commit": commit, "pushed": pushed,
            "remote": has_remote, "files": files}),
        error: None,
        progress: vec![],
        child: None,
    })
}

/// Whether a proposed change run's commit is on its base branch: merged (an ancestor of the base)
/// or squashed or rebased with its `Toolbox-Run` trailer. The base is fetched first when it is a
/// remote branch.
pub fn merged(cwd: &Path, run: &crate::run::Run) -> Result<bool, String> {
    let base = run.outputs["base"].as_str().unwrap_or("");
    let commit = run.outputs["commit"].as_str().unwrap_or("");
    let safe = |s: &str| {
        !s.is_empty()
            && !s.starts_with(['-', '/'])
            && !s.contains("..")
            && s.chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c))
    };
    if !safe(base) || !commit.chars().all(|c| c.is_ascii_hexdigit()) || commit.is_empty() {
        return Err("the run records no base branch and commit to look for".into());
    }
    let repo = PathBuf::from(git(cwd, &["rev-parse", "--show-toplevel"])?);
    if let Some((remote, branch)) = base.split_once('/')
        && git(&repo, &["remote", "get-url", "--", remote]).is_ok()
    {
        // A push updates the tracking ref itself, so an unfetched base would show the commit.
        git(&repo, &["fetch", "-q", "--", remote, branch])
            .map_err(|e| format!("cannot fetch {base} to check it: {e}"))?;
    }
    if git(&repo, &["merge-base", "--is-ancestor", commit, base]).is_ok() {
        return Ok(true);
    }
    let trailer = format!("Toolbox-Run: {}", run.id);
    let found = git(
        &repo,
        &[
            "log",
            "-n",
            "1",
            "--format=%H",
            "-F",
            "--grep",
            &trailer,
            base,
            "--",
        ],
    )?;
    Ok(!found.is_empty())
}

/// Whether a change that committed nothing still describes its base: the base, fetched again, is
/// the commit the command ran on, so the base already has everything the command writes.
pub fn unchanged(cwd: &Path, run: &crate::run::Run) -> Result<(), String> {
    let base = run.outputs["base"].as_str().unwrap_or("");
    let at = run.outputs["at"].as_str().unwrap_or("");
    if base.is_empty() || base.starts_with('-') || !at.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("the run records no base and commit to compare".into());
    }
    let repo = PathBuf::from(git(cwd, &["rev-parse", "--show-toplevel"])?);
    if let Some((remote, branch)) = base.split_once('/')
        && git(&repo, &["remote", "get-url", "--", remote]).is_ok()
    {
        git(&repo, &["fetch", "-q", "--", remote, branch])
            .map_err(|e| format!("cannot fetch {base} to check it: {e}"))?;
    }
    let now = git(&repo, &["rev-parse", "-q", "--verify", base])?;
    if now != at {
        return Err(format!(
            "{base} moved from {at} to {now} while the run ran: preview and request again"
        ));
    }
    Ok(())
}

/// Whether a landed change is on its base as it was pushed: the commit is there (fetched again,
/// so a hook that took it back is seen) and each changed file has the content the commit wrote.
/// Returns each changed file with the blob the commit wrote and whether the base has it.
pub fn landed(cwd: &Path, run: &crate::run::Run) -> Result<Vec<Value>, String> {
    if !merged(cwd, run)? {
        return Err(format!(
            "the commit is not on {}: the push did not land",
            run.outputs["base"].as_str().unwrap_or("the base")
        ));
    }
    let repo = PathBuf::from(git(cwd, &["rev-parse", "--show-toplevel"])?);
    let base = run.outputs["base"].as_str().unwrap_or("");
    let commit = run.outputs["commit"].as_str().unwrap_or("");
    let blob = |rev: &str, path: &str| {
        git(
            &repo,
            &["rev-parse", "-q", "--verify", &format!("{rev}:{path}")],
        )
        .ok()
    };
    Ok(run.outputs["files"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|f| Some((f["path"].as_str()?, &f["operation"])))
        .filter(|(p, _)| !p.starts_with('-') && !p.contains(".."))
        .map(|(p, op)| {
            let wrote = blob(commit, p);
            json!({"path": p, "operation": op, "blob": wrote, "matches": wrote == blob(base, p)})
        })
        .collect())
}

/// Whether `id` is a run id as `crate::run::new_id` makes them.
pub fn is_run_id(id: &str) -> bool {
    id.strip_prefix("run_").is_some_and(|r| {
        !r.is_empty()
            && r.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    })
}

/// Ask a child portal to re-check one of its runs. The child reads its own record by id; the
/// parent's copy of it is never trusted.
pub fn verify_child(
    program: &Path,
    portal: &Path,
    record: &Value,
    wait: bool,
    env: &[String],
) -> Result<Value, String> {
    let id = record
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| is_run_id(id))
        .ok_or("the child's run record has no valid run id")?;
    let mut cmd = Command::new(program);
    clean_env(&mut cmd, env);
    cmd.arg("run")
        .arg("--verify")
        .arg(id)
        .arg("--portal")
        .arg(
            portal
                .canonicalize()
                .unwrap_or_else(|_| portal.to_path_buf()),
        )
        .args(["-o", "json"])
        .current_dir(portal_repo_dir(portal));
    if wait {
        cmd.arg("--wait");
    }
    let out = cmd
        .output()
        .map_err(|e| format!("cannot start {}: {e}", program.display()))?;
    serde_json::from_slice(&out.stdout).map_err(|_| {
        format!(
            "child portal {} answered no run record: {}",
            portal.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )
    })
}

fn child_run(
    program: &Path,
    portal: &Path,
    action: &str,
    entity: &str,
    ctx: &Context,
    extra: &[&str],
    env: &[String],
) -> Result<Value, String> {
    let dir = portal_repo_dir(portal);
    let mut cmd = Command::new(program);
    clean_env(&mut cmd, env);
    cmd.arg("run")
        .arg("--portal")
        .arg(
            portal
                .canonicalize()
                .unwrap_or_else(|_| portal.to_path_buf()),
        )
        .args(["--inputs", &Value::Object(ctx.inputs.clone()).to_string()])
        .args(["--as", &ctx.actor, "-o", "json"])
        .args(extra)
        .current_dir(&dir);
    if !entity.is_empty() {
        cmd.args(["--entity", entity]);
    }
    if let Some(id) = &ctx.run_id {
        cmd.args(["--parent-run", &format!("{}/{id}", ctx.portal)]);
    }
    if let Some(g) = &ctx.grant {
        cmd.args(["--grant", g]);
    }
    cmd.arg("--").arg(action);
    let out = cmd
        .output()
        .map_err(|e| format!("cannot start {}: {e}", program.display()))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str::<Value>(stdout.trim()).map_err(|_| {
        format!(
            "child portal {} answered no run record: {}",
            portal.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )
    })
}

/// Bring a cloned child to the tip of `branch` on its origin. A child with no branch is a
/// checkout someone else keeps, and is left as it is.
fn sync_clone(portal: &Path, branch: Option<&str>) -> Result<(), String> {
    let Some(b) = branch else {
        return Ok(());
    };
    let dir = portal_repo_dir(portal);
    let step = |args: &[&str]| {
        git(&dir, args).map_err(|e| format!("cannot bring the child's clone to origin/{b}: {e}"))
    };
    step(&["fetch", "-q", "origin", "--", b])?;
    step(&["checkout", "-q", "-B", b, "FETCH_HEAD"])?;
    step(&["reset", "-q", "--hard", "FETCH_HEAD"])?;
    step(&["clean", "-qfdx"]).map(drop)
}

/// A child portal's commands run from its repository root (where its connectors' `root: .`
/// points), not from the parent's directory.
pub fn portal_repo_dir(portal: &Path) -> PathBuf {
    git(portal, &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .unwrap_or_else(|_| portal.to_path_buf())
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

    fn sh(dir: &Path, script: &str) {
        let ok = Command::new("sh")
            .args(["-c", script])
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .status()
            .unwrap()
            .success();
        assert!(ok, "{script}");
    }

    /// A clone of a bare origin with one commit on main; `hook` becomes the origin's
    /// `post-receive` (or `pre-receive` when it starts with `pre:`).
    fn clone_with(hook: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(crate::run::new_id("landtest"));
        std::fs::create_dir_all(&root).unwrap();
        sh(
            &root,
            "git init -q --bare -b main origin.git && git init -q -b main seed && cd seed && \
             echo hi > README.md && git add . && git commit -qm init && \
             git push -q ../origin.git main && cd .. && git clone -q origin.git work",
        );
        if let Some((kind, body)) = hook.split_once(':') {
            let path = root.join(format!("origin.git/hooks/{kind}-receive"));
            std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
            sh(&root, &format!("chmod +x {}", path.display()));
        }
        (root.clone(), root.join("work"))
    }

    fn landing(cwd: &Path) -> (Executor, Context) {
        let exec = Executor::parse(&json!({"type": "change", "base": "main", "land": "push",
            "message": "Deprecate {entity}: {inputs.reason}",
            "run": "printf 'Deprecated: %s\\n' \"$TOOLBOX_INPUT_REASON\" > DEPRECATED.md"}))
        .unwrap();
        let mut inputs = Map::new();
        inputs.insert("reason".into(), json!("use v2"));
        let ctx = Context {
            run_id: Some("run_land1".into()),
            portal: "repo".into(),
            action: "mark-deprecated".into(),
            action_version: "sha256:x".into(),
            entity: json!({"id": "repository:work"}),
            inputs,
            actor: "user:alice".into(),
            approved_by: vec!["user:bob".into()],
            related: Map::new(),
            grant: None,
            cwd: cwd.to_path_buf(),
        };
        (exec, ctx)
    }

    fn as_run(ctx: &Context, out: &Outcome) -> crate::run::Run {
        serde_json::from_value(json!({"id": ctx.run_id, "action": ctx.action,
            "action_version": ctx.action_version, "entity": "repository:work", "inputs": {},
            "requested_by": ctx.actor, "via": "cli", "portal": "repo", "executor": "change",
            "state": "running", "outputs": out.outputs, "effects": [], "events": [],
            "snapshot": "", "approval": {"required": false, "approvers": []}}))
        .unwrap()
    }

    #[test]
    fn a_landing_change_pushes_to_its_base_and_is_seen_there() {
        let (root, work) = clone_with("");
        let (exec, ctx) = landing(&work);
        assert!(matches!(exec, Executor::Change { push: true, .. }));
        let p = exec.preview(&ctx).unwrap();
        assert_eq!(p.effects[0]["resource"], "file:DEPRECATED.md");
        assert!(
            p.effects[0]["diff"]
                .as_str()
                .unwrap()
                .contains("+Deprecated: use v2")
        );
        assert_eq!(p.effects[1]["operation"], "push");
        assert_eq!(p.effects[1]["resource"], "branch:main");
        let out = exec.run(&ctx, &p.effects);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        assert_eq!(out.outputs["landed"], true);
        let run = as_run(&ctx, &out);
        let files = landed(&work, &run).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["path"], "DEPRECATED.md");
        assert_eq!(files[0]["matches"], true);
        let body = git(&work, &["log", "-1", "--format=%B", "origin/main"]).unwrap();
        assert!(
            body.starts_with("Deprecate repository:work: use v2\n"),
            "{body}"
        );
        assert!(
            body.contains("Requested-By: user:alice\nApproved-By: user:bob"),
            "{body}"
        );
        let shown = git(&root.join("origin.git"), &["show", "main:DEPRECATED.md"]).unwrap();
        assert_eq!(shown, "Deprecated: use v2");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_change_that_no_longer_matches_its_preview_lands_nothing() {
        let (root, work) = clone_with("");
        let (exec, ctx) = landing(&work);
        let p = exec.preview(&ctx).unwrap();
        sh(
            &work,
            "echo theirs > DEPRECATED.md && git add . && git commit -qm moved && \
             git push -q origin HEAD:main",
        );
        let before = git(&root.join("origin.git"), &["rev-parse", "main"]).unwrap();
        let out = exec.run(&ctx, &p.effects);
        assert_eq!(out.status, "failed");
        let err = out.error.unwrap();
        assert!(
            err.contains("DEPRECATED.md") && err.contains("preview"),
            "{err}"
        );
        let after = git(&root.join("origin.git"), &["rev-parse", "main"]).unwrap();
        assert_eq!(before, after);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_refused_or_undone_push_is_not_a_landed_change() {
        let (root, work) = clone_with("pre:echo protected >&2; exit 1");
        let (exec, ctx) = landing(&work);
        let out = exec.run(&ctx, &[]);
        assert_eq!(out.status, "failed");
        assert!(out.error.unwrap().contains("refused"));
        let _ = std::fs::remove_dir_all(root);

        let (root, work) =
            clone_with("post:while read old new ref; do git update-ref \"$ref\" \"$old\"; done");
        let (exec, ctx) = landing(&work);
        let out = exec.run(&ctx, &[]);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        let err = landed(&work, &as_run(&ctx, &out)).unwrap_err();
        assert!(err.contains("did not land"), "{err}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_change_its_base_already_has_commits_nothing_and_is_checked_against_the_base() {
        let (root, work) = clone_with("");
        let (exec, ctx) = landing(&work);
        assert_eq!(exec.run(&ctx, &[]).status, "succeeded");
        let before = git(&root.join("origin.git"), &["rev-parse", "main"]).unwrap();
        let p = exec.preview(&ctx).unwrap();
        assert!(p.effects.is_empty());
        assert!(p.warnings[0].contains("already has"), "{:?}", p.warnings);
        let out = exec.run(&ctx, &p.effects);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        assert_eq!(out.outputs["changed"], false);
        assert_eq!(out.outputs["at"], before.as_str());
        let run = as_run(&ctx, &out);
        unchanged(&work, &run).unwrap();
        assert_eq!(
            git(&root.join("origin.git"), &["rev-parse", "main"]).unwrap(),
            before
        );
        sh(
            &work,
            "git fetch -q origin && git checkout -q origin/main && echo theirs > DEPRECATED.md && \
             git commit -qam moved && git push -q origin HEAD:main",
        );
        let err = unchanged(&work, &run).unwrap_err();
        assert!(err.contains("moved"), "{err}");
        std::fs::rename(root.join("origin.git"), root.join("gone.git")).unwrap();
        let err = unchanged(&work, &run).unwrap_err();
        assert!(err.contains("cannot fetch"), "{err}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_base_that_cannot_be_fetched_is_not_a_landed_change() {
        let (root, work) = clone_with("");
        let (exec, ctx) = landing(&work);
        let out = exec.run(&ctx, &[]);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        std::fs::rename(root.join("origin.git"), root.join("gone.git")).unwrap();
        let err = landed(&work, &as_run(&ctx, &out)).unwrap_err();
        assert!(err.contains("cannot fetch"), "{err}");
        let _ = std::fs::remove_dir_all(root);
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
            Executor::parse(&json!({"run": "make release"}))
                .unwrap()
                .kind(),
            "command"
        );
        assert!(
            Executor::parse(&json!({"type": "change"}))
                .unwrap_err()
                .contains("executor.run is required")
        );
        assert!(
            Executor::parse(&json!({"type": "lambda"}))
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
}
