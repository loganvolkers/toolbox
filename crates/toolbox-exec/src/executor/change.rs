//! `change`: the command in a worktree of the base branch, committed. By default the commit is
//! proposed on a new branch, with a review on the remote's forge, and the run is `proposed` until
//! the commit is on the base; with `land: push` it is pushed to the base once approved (never
//! forced) and done when the base is seen to have it.

use super::{Context, Executor, Outcome, Preview, Rechecked, argv, git, names, spawn};
use crate::run::{Run, now_secs};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use toolbox_connect::hub::rfc3339;
use toolbox_model::forge;
use toolbox_model::manifest::Forge;

/// A `change` executor: runs a command in a worktree and lands, or proposes, the commit.
#[derive(Debug, Clone)]
pub struct Change {
    pub run: Vec<String>,
    pub base: Option<String>,
    pub remote: String,
    pub branch: Option<String>,
    pub message: Option<String>,
    pub env: Vec<String>,
    /// `land: push`: push the commit to the base once approved, instead of proposing a
    /// branch for review. The action's approval rule is then the only review.
    pub push: bool,
    /// Open a review for a proposed branch on the remote's forge (`review`, default true).
    pub review: bool,
    /// The portal's forges, which the remote's URL is matched against.
    pub forges: Vec<Forge>,
}

pub(crate) fn parse(v: &Value, forges: &[Forge]) -> Result<Arc<dyn Executor>, String> {
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
    Ok(Arc::new(Change {
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
        review: v.get("review").and_then(Value::as_bool).unwrap_or(true),
        forges: forges.to_vec(),
    }))
}

impl Executor for Change {
    fn kind(&self) -> &'static str {
        "change"
    }

    fn preview(&self, ctx: &Context) -> Result<Preview, String> {
        let wt = Worktree::new(&ctx.cwd, self.base.as_deref(), &self.remote)?;
        let out = spawn(&self.run, ctx, "preview", &wt.dir, &self.env)?;
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
        } else if self.push {
            effects.push(
                json!({"resource": format!("branch:{}", base_branch(&wt.base, &self.remote)),
                "operation": "push", "base": wt.base,
                "description": "one commit, pushed once approved (never forced)"}),
            );
        } else {
            effects.push(
                json!({"resource": format!("branch:{}", self.branch_name(ctx)),
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

    fn run(&self, ctx: &Context, confirmed: &[Value]) -> Outcome {
        change_run(
            &self.run,
            ctx,
            self.base.as_deref(),
            &self.remote,
            (!self.push).then(|| self.branch_name(ctx)).as_deref(),
            self.message.as_deref(),
            &self.env,
            confirmed,
            self.review.then_some(self.forges.as_slice()),
        )
        .unwrap_or_else(Outcome::failed)
    }

    /// A proposed change is done only once its commit is on the base branch: the base
    /// satisfying the postcondition by other means is not this run's effect.
    fn recheck(&self, run: &mut Run, expects: bool, cwd: &Path) -> Rechecked {
        if run.state != "proposed" {
            return Rechecked::Verify;
        }
        match merged(cwd, run) {
            Ok(true) if !expects => {
                run.verification = Some(json!({"by": "base", "merged": true,
                    "base": run.outputs["base"], "commit": run.outputs["commit"]}));
                run.set("succeeded", None, None);
            }
            Ok(true) => return Rechecked::Verify,
            Ok(false) => {
                if run.deadline.is_some_and(|dl| now_secs() >= dl) {
                    run.fail("the proposed change was not merged before the deadline");
                }
            }
            Err(e) => run
                .progress
                .push(json!({"at": rfc3339(now_secs()), "note": e})),
        }
        Rechecked::Done
    }

    fn reviewed(&self) -> bool {
        !self.push
    }
}

impl Change {
    fn branch_name(&self, ctx: &Context) -> String {
        match &self.branch {
            Some(b) => fill(b, ctx, false),
            None => format!(
                "toolbox/{}/{}",
                ctx.action,
                ctx.run_id.as_deref().unwrap_or("<run id>")
            ),
        }
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

/// A remote URL as a run record keeps it: without the password a URL may carry.
fn without_password(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return url.to_owned();
    };
    let (auth, tail) = rest.split_once('/').unwrap_or((rest, ""));
    match auth.rsplit_once('@') {
        Some((user, host)) => {
            let user = user.split(':').next().unwrap_or("");
            format!("{scheme}://{user}@{host}/{tail}")
        }
        None => url.to_owned(),
    }
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
    review: Option<&[Forge]>,
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
    let url = git(&wt.repo, &["remote", "get-url", "--", remote]).ok();
    let has_remote = url.as_deref().map(without_password);
    let files: Vec<Value> = files
        .iter()
        .map(|(op, p)| json!({"operation": op, "path": p}))
        .collect();
    let Some(branch) = branch else {
        return Ok(land_push(&wt, remote, &commit, has_remote, files));
    };
    let target = base_branch(&wt.base, remote);
    let on = url
        .as_deref()
        .zip(review)
        .and_then(|(u, fs)| forge::resolve(fs, u));
    let title = msg.lines().next().unwrap_or("");
    let body = format!(
        "{trailers}\n\nOpened by Toolbox for {} on {}.",
        ctx.action,
        ctx.entity["id"].as_str().unwrap_or("")
    );
    let pushed = has_remote.is_some();
    let opened = match &has_remote {
        Some(_) => push_for_review(&wt.repo, remote, branch, target, title, &body, on.as_ref())?,
        None => Value::Null,
    };
    let mut outputs = json!({"branch": branch, "base": wt.base, "commit": commit, "pushed": pushed,
        "remote": has_remote, "files": files, "title": title});
    if !opened.is_null() {
        outputs["review"] = opened;
    }
    Ok(Outcome {
        status: "proposed".into(),
        outputs,
        error: None,
        progress: vec![],
        child: None,
    })
}

/// `land: push`: the commit pushed straight to the base, never forced.
fn land_push(
    wt: &Worktree,
    remote: &str,
    commit: &str,
    has_remote: Option<String>,
    files: Vec<Value>,
) -> Outcome {
    let target = base_branch(&wt.base, remote);
    if has_remote.is_none() {
        return Outcome::failed(format!(
            "land: push needs the remote {remote}, and this repository has none"
        ));
    }
    let refspec = format!("{commit}:refs/heads/{target}");
    if let Err(e) = git(&wt.repo, &["push", "-q", "--", remote, &refspec]) {
        return Outcome::failed(format!(
            "the push to {remote} {target} was refused, so nothing landed: {e}"
        ));
    }
    Outcome {
        status: "succeeded".into(),
        outputs: json!({"landed": true, "base": wt.base, "commit": commit,
            "remote": has_remote, "files": files}),
        error: None,
        progress: vec![],
        child: None,
    }
}

/// Push a proposed branch and open its review on the remote's forge, if it has one: the review
/// (`Null` when none was asked for).
fn push_for_review(
    repo: &Path,
    remote: &str,
    branch: &str,
    target: &str,
    title: &str,
    body: &str,
    on: Option<&forge::Resolved>,
) -> Result<Value, String> {
    match on {
        Some(f) => crate::forge::driver(f.kind).propose(
            f,
            &crate::forge::Proposal {
                repo,
                remote,
                branch,
                target,
                title,
                body,
            },
        ),
        None => git(repo, &["push", "-q", remote, branch]).map(|_| Value::Null),
    }
}

/// Whether a proposed change run's commit is on its base branch: merged (an ancestor of the base)
/// or squashed or rebased with its `Toolbox-Run` trailer, or with the `(toolbox run <id>)` of
/// its default subject, which a squash that keeps only the title (GitLab's default) still has.
/// The base is fetched first when it is a remote branch.
pub fn merged(cwd: &Path, run: &Run) -> Result<bool, String> {
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
    let subject = format!("(toolbox run {})", run.id);
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
            "--grep",
            &subject,
            base,
            "--",
        ],
    )?;
    Ok(!found.is_empty())
}

/// Whether a change that committed nothing still describes its base: the base, fetched again, is
/// the commit the command ran on, so the base already has everything the command writes.
pub fn unchanged(cwd: &Path, run: &Run) -> Result<(), String> {
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
pub fn landed(cwd: &Path, run: &Run) -> Result<Vec<Value>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;
    use std::process::Command;

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

    fn landing(cwd: &Path) -> (Arc<dyn Executor>, Context) {
        let exec = super::super::parse(&json!({"type": "change", "base": "main", "land": "push",
            "message": "Publish support for {entity}: {inputs.channel}",
            "run": "printf 'Support: %s\\n' \"$TOOLBOX_INPUT_CHANNEL\" > SUPPORT.md"}))
        .unwrap();
        let mut inputs = Map::new();
        inputs.insert("channel".into(), json!("#help-work"));
        let ctx = Context {
            run_id: Some("run_land1".into()),
            portal: "repo".into(),
            action: "publish-support".into(),
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

    fn as_run(ctx: &Context, out: &Outcome) -> Run {
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
        assert!(exec.downcast::<Change>().is_some_and(|c| c.push));
        let p = exec.preview(&ctx).unwrap();
        assert_eq!(p.effects[0]["resource"], "file:SUPPORT.md");
        assert!(
            p.effects[0]["diff"]
                .as_str()
                .unwrap()
                .contains("+Support: #help-work")
        );
        assert_eq!(p.effects[1]["operation"], "push");
        assert_eq!(p.effects[1]["resource"], "branch:main");
        let out = exec.run(&ctx, &p.effects);
        assert_eq!(out.status, "succeeded", "{:?}", out.error);
        assert_eq!(out.outputs["landed"], true);
        let run = as_run(&ctx, &out);
        let files = landed(&work, &run).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["path"], "SUPPORT.md");
        assert_eq!(files[0]["matches"], true);
        let body = git(&work, &["log", "-1", "--format=%B", "origin/main"]).unwrap();
        assert!(
            body.starts_with("Publish support for repository:work: #help-work\n"),
            "{body}"
        );
        assert!(
            body.contains("Requested-By: user:alice\nApproved-By: user:bob"),
            "{body}"
        );
        let shown = git(&root.join("origin.git"), &["show", "main:SUPPORT.md"]).unwrap();
        assert_eq!(shown, "Support: #help-work");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_change_that_no_longer_matches_its_preview_lands_nothing() {
        let (root, work) = clone_with("");
        let (exec, ctx) = landing(&work);
        let p = exec.preview(&ctx).unwrap();
        sh(
            &work,
            "echo theirs > SUPPORT.md && git add . && git commit -qm moved && \
             git push -q origin HEAD:main",
        );
        let before = git(&root.join("origin.git"), &["rev-parse", "main"]).unwrap();
        let out = exec.run(&ctx, &p.effects);
        assert_eq!(out.status, "failed");
        let err = out.error.unwrap();
        assert!(
            err.contains("SUPPORT.md") && err.contains("preview"),
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
            "git fetch -q origin && git checkout -q origin/main && echo theirs > SUPPORT.md && \
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
    fn a_proposal_squashed_to_its_title_is_still_seen_merged() {
        let (root, work) = clone_with("");
        let exec = super::super::parse(&json!({"type": "change", "base": "main",
            "run": "echo MIT > LICENSE"}))
        .unwrap();
        let (_, ctx) = landing(&work);
        let out = exec.run(&ctx, &[]);
        assert_eq!(out.status, "proposed", "{:?}", out.error);
        let title = out.outputs["title"].as_str().unwrap().to_owned();
        assert!(title.ends_with("(toolbox run run_land1)"), "{title}");
        let run = as_run(&ctx, &out);
        assert!(!merged(&work, &run).unwrap());
        sh(
            &work,
            &format!(
                "git checkout -q -B squash origin/main && git checkout -q origin/{} -- LICENSE && \
                 git commit -qm '{title}' && git push -q origin squash:main",
                out.outputs["branch"].as_str().unwrap()
            ),
        );
        assert!(merged(&work, &run).unwrap());
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
}
