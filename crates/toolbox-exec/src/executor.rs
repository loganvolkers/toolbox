//! Executors (docs/design/action-runs.md §5, docs/design/federated-actions.md §5): what does the
//! work once a run is confirmed. Every type answers the same two questions — what would you
//! change (preview) and do it (run) — so the lifecycle around them is one piece of code.
//!
//! | `type` | Preview | Run |
//! |---|---|---|
//! | `command` | the same command with `TOOLBOX_PHASE=preview`, when `preview: true` | the command; its last JSON line is the outputs |
//! | `in-image` | executor protocol v1 `preview` over stdio | protocol `run`: `accepted`, `progress`, `result` |
//! | `change` | the command in a throwaway git worktree; the effects are the files it changed | the same, committed to a new branch and pushed: the run is `proposed` until its postcondition is observed on the base branch |
//! | `github-workflow` | the Toolbox layer only | `workflow_dispatch`, then the run is tracked by its `run-name` |
//! | `toolbox` | the child portal's own preview | the child portal's own run (federated dispatch) |
//!
//! A command gets the run's context as JSON on stdin and in the environment: `TOOLBOX_PHASE`,
//! `TOOLBOX_ACTION`, `TOOLBOX_RUN_ID`, `TOOLBOX_ENTITY`, `TOOLBOX_ACTOR`, `TOOLBOX_INPUTS` (JSON)
//! and one `TOOLBOX_INPUT_<NAME>` per input (strings as themselves, other values as JSON).

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
    pub related: Map<String, Value>,
    /// Approvals the parent recorded (`group:x=user:y`), passed to a child portal on dispatch.
    pub approvals: Vec<String>,
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
    },
    InImage {
        command: Vec<String>,
    },
    Change {
        run: Vec<String>,
        base: Option<String>,
        remote: String,
        branch: Option<String>,
        message: Option<String>,
    },
    GithubWorkflow(crate::github::Workflow),
    /// Dispatch to a child portal (federation): the child previews, runs and verifies.
    Toolbox {
        portal: PathBuf,
        action: String,
        entity: String,
        program: PathBuf,
    },
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
            },
            "in-image" => Executor::InImage {
                command: argv(v.get("command"), "executor.command")?,
            },
            "change" => Executor::Change {
                run: argv(v.get("run"), "executor.run")?,
                base: s("base"),
                remote: s("remote").unwrap_or_else(|| "origin".into()),
                branch: s("branch"),
                message: s("message"),
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

    /// Preview: what the run would change. Never has side effects outside a temporary directory.
    pub fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        match self {
            Executor::Command { run, preview } => {
                if !preview {
                    return Ok(Preview {
                        effects: vec![
                            json!({"operation": "run", "resource": format!("command:{}", run.join(" ")),
                            "description": "runs the command; it has no preview of its own"}),
                        ],
                        ..Preview::default()
                    });
                }
                let out = spawn(run, ctx, "preview", &ctx.cwd)?;
                if !out.ok {
                    return Err(format!("preview failed: {}", out.tail()));
                }
                Ok(Preview {
                    effects: effects_from(&out.stdout),
                    executed: true,
                    ..Preview::default()
                })
            }
            Executor::InImage { command } => stdio_preview(command, ctx),
            Executor::Change {
                run, base, remote, ..
            } => {
                let wt = Worktree::new(&ctx.cwd, base.as_deref(), remote)?;
                let out = spawn(run, ctx, "preview", &wt.dir)?;
                if !out.ok {
                    return Err(format!("change command failed: {}", out.tail()));
                }
                let files = wt.changes()?;
                let mut effects: Vec<Value> = files
                    .iter()
                    .map(|(op, path)| json!({"resource": format!("file:{path}"), "operation": op}))
                    .collect();
                let mut warnings = Vec::new();
                if effects.is_empty() {
                    warnings.push("the change command changed no files".into());
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
            } => {
                let v = child_run(program, portal, action, entity, ctx, &["--preview"])?;
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
            Executor::Command { run, .. } => match spawn(run, ctx, "run", &ctx.cwd) {
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
            Executor::InImage { command } => stdio_run(command, ctx, confirmed),
            Executor::Change {
                run,
                base,
                remote,
                message,
                ..
            } => change_run(
                run,
                ctx,
                base.as_deref(),
                remote,
                &branch_name(self, ctx),
                message.as_deref(),
            )
            .unwrap_or_else(Outcome::failed),
            Executor::GithubWorkflow(w) => w.dispatch(ctx),
            Executor::Toolbox {
                portal,
                action,
                entity,
                program,
            } => match child_run(program, portal, action, entity, ctx, &["--yes"]) {
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
    let id = ctx.run_id.as_deref().unwrap_or("preview");
    match e {
        Executor::Change {
            branch: Some(b), ..
        } => b
            .replace("{action}", &ctx.action)
            .replace("{run}", id)
            .replace("{run_id}", id),
        _ => format!("toolbox/{}/{id}", ctx.action),
    }
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

fn spawn(argv: &[String], ctx: &Context, phase: &str, cwd: &Path) -> Result<Spawned, String> {
    let mut cmd = Command::new(&argv[0]);
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

fn stdio_session(command: &[String], ctx: &Context, msg: &Value) -> Result<Vec<Value>, String> {
    let mut child = Command::new(&command[0])
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

fn stdio_preview(command: &[String], ctx: &Context) -> Result<Preview, String> {
    let mut msg = protocol_context(ctx);
    msg.insert("type".into(), "preview".into());
    msg.insert("request_id".into(), "req_preview".into());
    let msgs = stdio_session(command, ctx, &Value::Object(msg))?;
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

fn stdio_run(command: &[String], ctx: &Context, confirmed: &[Value]) -> Outcome {
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
    let msgs = match stdio_session(command, ctx, &Value::Object(msg)) {
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
    let out = Command::new("git")
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

fn change_run(
    run: &[String],
    ctx: &Context,
    base: Option<&str>,
    remote: &str,
    branch: &str,
    message: Option<&str>,
) -> Result<Outcome, String> {
    let wt = Worktree::new(&ctx.cwd, base, remote)?;
    let out = spawn(run, ctx, "run", &wt.dir)?;
    if !out.ok {
        return Ok(Outcome::failed(format!(
            "change command failed: {}",
            out.tail()
        )));
    }
    let files = wt.changes()?;
    if files.is_empty() {
        return Ok(Outcome {
            status: "succeeded".into(),
            outputs: json!({"changed": false}),
            error: None,
            progress: vec![],
            child: None,
        });
    }
    let root = wt.root();
    git(&root, &["switch", "-c", branch])?;
    let msg = message.map_or_else(
        || {
            format!(
                "{} on {} (toolbox run {})",
                ctx.action,
                ctx.entity["id"].as_str().unwrap_or(""),
                ctx.run_id.as_deref().unwrap_or("")
            )
        },
        str::to_owned,
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
    let requested = format!("Requested-By: {}", ctx.actor);
    args.extend(["commit", "-q", "-m", &msg, "-m", &trailer, "-m", &requested]);
    git(&root, &args)?;
    let commit = git(&root, &["rev-parse", "HEAD"])?;
    // From the main checkout: a relative remote URL resolves against it, not the worktree.
    let has_remote = git(&wt.repo, &["remote", "get-url", remote]).ok();
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
            "remote": has_remote, "files": files.iter().map(|(op, p)| json!({"operation": op, "path": p})).collect::<Vec<_>>()}),
        error: None,
        progress: vec![],
        child: None,
    })
}

fn child_run(
    program: &Path,
    portal: &Path,
    action: &str,
    entity: &str,
    ctx: &Context,
    extra: &[&str],
) -> Result<Value, String> {
    let dir = portal_repo_dir(portal);
    let mut cmd = Command::new(program);
    cmd.arg("run")
        .arg(action)
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
    for a in ctx.approvals.iter() {
        cmd.args(["--approved-by", a]);
    }
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
}
