//! `seikika`: a federated run, dispatched to a child portal's own `seikika run`, which previews,
//! runs and verifies it with its own executor (docs/design/federated-actions.md §5). The parent
//! never runs the child's code with its own credentials.

use super::{Context, Executor, Outcome, Preview, Rechecked, clean_env, git, is_run_id};
use crate::run::{Run, now_secs};
use seikika_connect::hub::rfc3339;
use seikika_model::forge;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Dispatch to a child portal (federation): the child previews, runs and verifies.
#[derive(Debug, Clone)]
pub struct Federated {
    pub portal: PathBuf,
    pub action: String,
    pub entity: String,
    pub program: PathBuf,
    pub env: Vec<String>,
    /// A child cloned from git (`federation.children`): the branch its clone is brought to
    /// before each preview and run, so the definition that runs is the one committed there.
    pub branch: Option<String>,
    /// The forge the child was cloned from, as this portal resolves it: a branch the child
    /// proposes without opening a review gets one here, with this portal's token.
    pub review: Option<forge::Resolved>,
}

impl Executor for Federated {
    fn kind(&self) -> &'static str {
        "seikika"
    }

    fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        sync_clone(&self.portal, self.branch.as_deref())?;
        let v = child_run(
            &self.program,
            &self.portal,
            &self.action,
            &self.entity,
            ctx,
            &["--preview"],
            &self.env,
        )?;
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

    fn run(&self, ctx: &Context, _: &[Value]) -> Outcome {
        match sync_clone(&self.portal, self.branch.as_deref()).and_then(|()| {
            child_run(
                &self.program,
                &self.portal,
                &self.action,
                &self.entity,
                ctx,
                &["--yes"],
                &self.env,
            )
        }) {
            Err(e) => Outcome::failed(e),
            Ok(v) => child_outcome(v, self.review.as_ref(), &self.action, ctx),
        }
    }

    /// The child is asked about its own run; the parent's copy of it is never trusted.
    fn recheck(&self, run: &mut Run, _: bool, _: &Path) -> Rechecked {
        let Some(child) = run.child.clone() else {
            return Rechecked::Done;
        };
        match verify_child(&self.program, &self.portal, &child, false, &self.env) {
            Ok(v) => {
                let state = v["state"].as_str().unwrap_or("failed").to_owned();
                match state.as_str() {
                    "succeeded" | "unverified" => {
                        run.verification = Some(json!({"by": "child portal",
                            "child_run": v["id"], "child": v["verification"]}));
                        run.set(&state, None, None);
                    }
                    "failed" | "timed_out" | "cancelled" | "rejected" => run.fail(format!(
                        "child run {state}: {}",
                        v["error"].as_str().unwrap_or("")
                    )),
                    _ => {}
                }
                run.child = Some(v);
            }
            Err(e) => run
                .progress
                .push(json!({"at": rfc3339(now_secs()), "note": e})),
        }
        Rechecked::Done
    }
}

/// A federated child's run, as this run's outcome; a branch it pushed gets a review on `review`.
fn child_outcome(
    v: Value,
    review: Option<&forge::Resolved>,
    action: &str,
    ctx: &Context,
) -> Outcome {
    let state = v
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or("failed")
        .to_owned();
    let mut outputs = v.get("outputs").cloned().unwrap_or(json!({}));
    if let Some(f) = review
        && state == "proposed"
    {
        review_child(f, &mut outputs, &v, action, ctx);
    }
    Outcome {
        status: match state.as_str() {
            "succeeded" | "unverified" => "succeeded".into(),
            "proposed" | "awaiting_approval" => state.clone(),
            _ => "failed".into(),
        },
        outputs,
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

/// A child that pushed a branch without opening a review gets one on `f`, with this portal's
/// token.
fn review_child(
    f: &forge::Resolved,
    outputs: &mut Value,
    child: &Value,
    action: &str,
    ctx: &Context,
) {
    if outputs["pushed"] != true || outputs.get("review").is_some() {
        return;
    }
    let Some(head) = outputs["branch"].as_str() else {
        return;
    };
    let base = outputs["base"].as_str().unwrap_or("main");
    let base = base.strip_prefix("origin/").unwrap_or(base);
    let title = outputs["title"].as_str().unwrap_or(action);
    let body = format!(
        "Seikika-Run: {}\n\nOpened by Seikika for {action}, run {} of portal {}.",
        child["id"].as_str().unwrap_or(""),
        ctx.run_id.as_deref().unwrap_or(""),
        ctx.portal
    );
    outputs["review"] = crate::forge::open_review(f, head, base, title, &body);
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
