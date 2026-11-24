//! The run lifecycle (docs/design/action-runs.md §2, docs/v1-spec.md §6.6), shared by a repo
//! portal's `toolbox run` and the server:
//!
//! ```text
//! previewed → [awaiting_approval → approved] → dispatched → running → verifying → succeeded
//!                                         ↘ proposed (a change under review) ↗
//!                                         ↘ failed | unverified | cancelled | rejected
//! ```
//!
//! **Done means observed:** a run with `expect` succeeds only when its postcondition holds over
//! facts re-read after the executor finished (P9). Without `expect` it ends `unverified`.

use crate::executor::{Context, Executor, Outcome};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use toolbox_connect::hub::rfc3339;
use toolbox_model::manifest::{Action, Expect};

/// Confirm tokens are valid for ten minutes (docs/v1-spec.md §6.6).
pub const TOKEN_TTL_SECS: u64 = 600;

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha(parts: &[&str]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p.as_bytes());
        h.update([0]);
    }
    hex(&h.finalize())
}

/// A run, approval or worktree id: `<prefix>_<time><counter>` in base 36.
pub fn new_id(prefix: &str) -> String {
    static N: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    let n = N.fetch_add(1, Ordering::Relaxed) ^ u64::from(std::process::id());
    let mut x = nanos.wrapping_mul(31).wrapping_add(n);
    let mut s = String::new();
    while x > 0 {
        let d = (x % 36) as u8;
        s.push(char::from(if d < 10 { b'0' + d } else { b'a' + d - 10 }));
        x /= 36;
    }
    format!("{prefix}_{}", s.chars().rev().collect::<String>())
}

/// The manifest hash of an action's definition: a run records which version it ran.
pub fn action_version(action: &Action) -> String {
    let s = serde_json::to_string(action).unwrap_or_default();
    format!("sha256:{}", &sha(&[&s])[..16])
}

/// The digest of the facts a preview read: the entity and its context lookups. A resync that
/// leaves them alone does not stale a token.
pub fn basis(entity: &Value, related: &Map<String, Value>) -> String {
    sha(&[
        &entity.to_string(),
        &Value::Object(related.clone()).to_string(),
    ])[..16]
        .to_owned()
}

/// What a confirm token is bound to: the preview a person saw, who saw it, and on what.
#[derive(Clone, Copy)]
pub struct Confirmed<'a> {
    pub version: &'a str,
    pub inputs: &'a Map<String, Value>,
    pub effects: &'a [Value],
    pub basis: &'a str,
    pub actor: &'a str,
    pub entity: &'a str,
}

/// `ct_<issued>_<nonce>_<hash>`: stateless, so a repo portal's preview and run can be two
/// commands. A server also spends each token once, so each preview gets a fresh nonce.
pub fn confirm_token(c: &Confirmed<'_>, issued: u64) -> String {
    let mut nonce = [0u8; 6];
    aws_lc_rs::rand::fill(&mut nonce).expect("system random");
    token_for(c, issued, &hex(&nonce))
}

fn token_for(c: &Confirmed<'_>, issued: u64, nonce: &str) -> String {
    let h = sha(&[
        c.version,
        &Value::Object(c.inputs.clone()).to_string(),
        &Value::Array(c.effects.to_vec()).to_string(),
        c.basis,
        c.actor,
        c.entity,
        &issued.to_string(),
        nonce,
    ]);
    format!("ct_{issued}_{nonce}_{}", &h[..24])
}

/// Why a token does not confirm this preview.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenError {
    Malformed,
    Expired,
    Stale,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TokenError::Malformed => "confirm token is malformed: pass the token a preview returned",
            TokenError::Expired => "confirm token expired (tokens last 10 minutes): preview again",
            TokenError::Stale => {
                "preview_stale: the inputs, effects or facts changed since that preview; preview again and confirm the new token"
            }
        })
    }
}

/// When `token` was issued, if it is shaped like a confirm token.
pub fn token_issued(token: &str) -> Option<u64> {
    token_parts(token).map(|(issued, _)| issued)
}

fn token_parts(token: &str) -> Option<(u64, &str)> {
    let mut parts = token.splitn(4, '_');
    let (Some("ct"), Some(issued), Some(nonce), Some(_)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    Some((issued.parse().ok()?, nonce))
}

/// Check `token` against the preview as it is now (P10), for the same person and entity.
pub fn check_token(token: &str, c: &Confirmed<'_>) -> Result<(), TokenError> {
    let (issued, nonce) = token_parts(token).ok_or(TokenError::Malformed)?;
    let now = now_secs();
    if issued > now + 60 {
        return Err(TokenError::Malformed);
    }
    if now.saturating_sub(issued) > TOKEN_TTL_SECS {
        return Err(TokenError::Expired);
    }
    if token_for(c, issued, nonce) != token {
        return Err(TokenError::Stale);
    }
    Ok(())
}

/// How long a run with no postcondition waits on a review or a workflow before it times out.
pub const UNOBSERVED_WITHIN: &str = "7d";

/// `30s`, `2m`, `1h`, `7d`.
pub fn parse_within(s: &str) -> Duration {
    let s = s.trim();
    let (num, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len()));
    let n: u64 = num.parse().unwrap_or(120);
    Duration::from_secs(match unit {
        "s" | "" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        _ => 120,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub at: String,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Approval {
    pub required: bool,
    pub approvers: Vec<Value>,
    #[serde(default)]
    pub approved_by: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rejected_by: Option<String>,
}

/// One run: its request, every state it passed through, and what was observed at the end.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Run {
    pub id: String,
    pub action: String,
    pub action_version: String,
    pub entity: String,
    pub inputs: Map<String, Value>,
    pub requested_by: String,
    /// `cli`, `api`, `mcp`, or `parent:<run id>` for a federated dispatch.
    pub via: String,
    pub portal: String,
    pub executor: String,
    pub snapshot: String,
    pub effects: Vec<Value>,
    pub approval: Approval,
    pub state: String,
    pub events: Vec<Event>,
    pub outputs: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub child: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub progress: Vec<Value>,
    /// Epoch seconds by which verification must succeed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<u64>,
    /// The postcondition already held when the run was previewed, so observing it later does not
    /// show this run's effect.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub held_before: bool,
}

pub const TERMINAL: &[&str] = &[
    "succeeded",
    "failed",
    "timed_out",
    "unverified",
    "cancelled",
    "rejected",
];

impl Run {
    pub fn is_terminal(&self) -> bool {
        TERMINAL.contains(&self.state.as_str())
    }

    pub fn set(&mut self, state: &str, by: Option<&str>, note: Option<String>) {
        self.state = state.into();
        self.events.push(Event {
            at: rfc3339(now_secs()),
            state: state.into(),
            by: by.map(str::to_owned),
            note,
        });
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        let e = error.into();
        self.error = Some(e.clone());
        self.set("failed", None, Some(e));
    }

    /// Record an approval; `true` once every approver has approved. Approvers are groups, users,
    /// or `a|b|…`, satisfied by any one of them; `member_of` says whether `who` is in a group.
    /// Each person approves once, so two approver entries take two people; principals compare
    /// without regard to case.
    pub fn approve(&mut self, who: &str, member_of: impl Fn(&str) -> bool) -> Result<bool, String> {
        if self.state != "awaiting_approval" {
            return Err(format!(
                "run {} is {}, not awaiting_approval",
                self.id, self.state
            ));
        }
        if !who.starts_with("user:") || who.len() == "user:".len() {
            return Err(format!("{who} is not a person; only a user:<id> approves"));
        }
        let same = |a: &str, b: &str| a.eq_ignore_ascii_case(b);
        if same(who, &self.requested_by) {
            return Err("a requester cannot approve their own run".into());
        }
        if self
            .approval
            .approved_by
            .iter()
            .any(|x| x.rsplit_once('=').is_some_and(|(_, w)| same(w, who)))
        {
            return Err(format!(
                "{who} has already approved run {}; another approver must approve the rest",
                self.id
            ));
        }
        let pending: Vec<String> = self.pending_approvers();
        let Some(which) = pending.iter().find(|a| {
            same(a, who) || member_of(a) || a.split('|').any(|p| same(p, who) || member_of(p))
        }) else {
            return Err(format!(
                "{who} is not one of the pending approvers {pending:?}"
            ));
        };
        let which = which.clone();
        self.approval.approved_by.push(format!("{which}={who}"));
        self.events.push(Event {
            at: rfc3339(now_secs()),
            state: "approved".into(),
            by: Some(who.into()),
            note: Some(which),
        });
        Ok(self.pending_approvers().is_empty())
    }

    pub fn pending_approvers(&self) -> Vec<String> {
        self.approval
            .approvers
            .iter()
            .filter_map(Value::as_str)
            .filter(|a| {
                !self
                    .approval
                    .approved_by
                    .iter()
                    .any(|x| x.split('=').next() == Some(a))
            })
            .map(str::to_owned)
            .collect()
    }
}

/// What verification reads: the entity as it is after a refresh.
pub trait Observer {
    /// Re-read the facts `expect` names and return the entity (fields plus `id`) and the snapshot.
    fn observe(&mut self, run: &Run, expect: &Expect) -> Result<(Value, String), String>;
}

/// Evaluate the postcondition once.
pub fn postcondition_holds(
    expect: &Expect,
    entity: &Value,
    inputs: &Map<String, Value>,
    policies: &Map<String, Value>,
    outputs: &Value,
) -> Result<bool, String> {
    match crate::check::eval(
        &expect.postcondition,
        entity,
        inputs,
        policies,
        Some(outputs),
    )? {
        Value::Bool(b) => Ok(b),
        other => Err(format!("postcondition is not a boolean: {other}")),
    }
}

/// Verify until the postcondition holds or `deadline` passes, polling every `every`. With
/// `wait: false` (a proposed change checked later), observe once.
pub fn verify(
    run: &mut Run,
    expect: &Expect,
    policies: &Map<String, Value>,
    obs: &mut dyn Observer,
    every: Duration,
    wait: bool,
) {
    if run.state != "verifying" {
        run.set("verifying", None, None);
    }
    let deadline = run
        .deadline
        .unwrap_or_else(|| now_secs() + parse_within(&expect.within).as_secs());
    run.deadline = Some(deadline);
    let started = Instant::now();
    loop {
        let (entity, snapshot) = match obs.observe(run, expect) {
            Ok(x) => x,
            Err(e) => {
                run.fail(format!("verification could not read facts: {e}"));
                return;
            }
        };
        match postcondition_holds(expect, &entity, &run.inputs, policies, &run.outputs) {
            Ok(true) => {
                run.verification =
                    Some(json!({"postcondition": expect.postcondition, "held": true,
                    "snapshot": snapshot, "after_secs": started.elapsed().as_secs(),
                    "held_before": run.held_before}));
                if run.held_before {
                    run.set(
                        "unverified",
                        None,
                        Some("the postcondition already held before the run, so observing it does not show the run's effect".into()),
                    );
                } else {
                    run.set("succeeded", None, None);
                }
                return;
            }
            Ok(false) => {
                run.verification =
                    Some(json!({"postcondition": expect.postcondition, "held": false,
                    "snapshot": snapshot}));
            }
            Err(e) => {
                run.fail(format!("postcondition failed to evaluate: {e}"));
                return;
            }
        }
        if !wait {
            return;
        }
        if now_secs() >= deadline {
            run.fail(format!(
                "postcondition not met within {}: {}",
                expect.within, expect.postcondition
            ));
            return;
        }
        std::thread::sleep(every);
    }
}

/// Dispatch a confirmed, approved run and carry it as far as it can go now: to a terminal state,
/// or to `proposed` / `dispatched` when the effect lands later.
#[allow(clippy::too_many_arguments)]
pub fn execute(
    run: &mut Run,
    exec: &Executor,
    ctx: &Context,
    expect: Option<&Expect>,
    policies: &Map<String, Value>,
    obs: &mut dyn Observer,
    every: Duration,
) {
    run.set("dispatched", None, None);
    run.set("running", None, None);
    let Outcome {
        status,
        outputs,
        error,
        progress,
        child,
    } = exec.run(ctx, &run.effects);
    run.outputs = outputs;
    run.progress = progress;
    // A child portal verified its own run: the parent records what the child observed.
    if let Some(c) = &child
        && status == "succeeded"
    {
        let state = c.get("state").and_then(Value::as_str).unwrap_or("");
        run.verification = Some(json!({"by": "child portal", "child_run": c.get("id"),
            "child": c.get("verification")}));
        run.child = child.clone();
        run.set("executor_succeeded", None, None);
        run.set(
            if state == "succeeded" {
                "succeeded"
            } else {
                "unverified"
            },
            None,
            None,
        );
        return;
    }
    run.child = child;
    match status.as_str() {
        "failed" => {
            run.fail(error.unwrap_or_else(|| "the executor failed".into()));
            return;
        }
        "proposed" | "dispatched" | "awaiting_approval" => {
            run.set(&status, None, error);
            let within = expect.map_or(UNOBSERVED_WITHIN, |e| e.within.as_str());
            run.deadline = Some(now_secs() + parse_within(within).as_secs());
            return;
        }
        _ => run.set("executor_succeeded", None, None),
    }
    if run.outputs["landed"] == true && !on_base(run, &ctx.cwd) {
        return;
    }
    done(run, expect, policies, obs, every, true);
}

/// A landed change is observed on its base: the commit is there and every file it changed has
/// the content it wrote. Fails the run otherwise.
fn on_base(run: &mut Run, cwd: &std::path::Path) -> bool {
    let base = run.outputs["base"]
        .as_str()
        .unwrap_or("the base")
        .to_owned();
    let files = match crate::executor::landed(cwd, run) {
        Err(e) => {
            run.fail(e);
            return false;
        }
        Ok(files) => files,
    };
    let differ: Vec<&str> = files
        .iter()
        .filter(|f| f["matches"] != true)
        .filter_map(|f| f["path"].as_str())
        .collect();
    if !differ.is_empty() {
        run.fail(format!(
            "the push landed, but {} on {base} differ from what it wrote",
            differ.join(", ")
        ));
        return false;
    }
    run.verification = Some(json!({"by": "base", "base": base,
        "commit": run.outputs["commit"], "files": files,
        "checked": format!("fetched {base}: the commit is on it, and every file it changed \
            has the content the preview showed")}));
    true
}

/// After the executor: check the postcondition, if the action declares one. A change observed on
/// its base needs none: its effect is the commit, and the base has it.
fn done(
    run: &mut Run,
    expect: Option<&Expect>,
    policies: &Map<String, Value>,
    obs: &mut dyn Observer,
    every: Duration,
    wait: bool,
) {
    let on_base = run.verification.take();
    match expect {
        None if on_base.is_some() => {
            run.verification = on_base;
            run.set("succeeded", None, None);
        }
        None => run.set(
            "unverified",
            None,
            Some("the action declares no postcondition (expect)".into()),
        ),
        Some(e) => {
            verify(run, e, policies, obs, every, wait);
            if let (Some(b), Some(Value::Object(v))) = (on_base, run.verification.as_mut()) {
                v.insert("on_base".into(), b);
            }
        }
    }
}

/// Re-check a run that is waiting on the world: a `proposed` change (merged yet?), a `dispatched`
/// workflow (finished yet?), or a federated run (asked of the child). One look, no waiting; a run
/// past its deadline fails. Returns whether the run changed.
pub fn recheck(
    run: &mut Run,
    exec: &crate::executor::Executor,
    expect: Option<&Expect>,
    policies: &Map<String, Value>,
    obs: &mut dyn Observer,
    cwd: &std::path::Path,
) -> bool {
    use crate::executor::Executor;
    if run.is_terminal() || run.state == "awaiting_approval" {
        return false;
    }
    let before = run.state.clone();
    let events = run.events.len();
    match exec {
        Executor::Toolbox {
            portal,
            program,
            env,
            ..
        } => {
            if let Some(child) = run.child.clone() {
                match crate::executor::verify_child(program, portal, &child, false, env) {
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
                    Err(e) => {
                        run.progress
                            .push(json!({"at": rfc3339(now_secs()), "note": e}));
                    }
                }
            }
            return run.state != before;
        }
        Executor::GithubWorkflow(w) if run.state == "dispatched" => {
            match w.track(&run.id, &run.outputs) {
                Ok(Some(t)) if t.status == "completed" => {
                    if let Some(u) = &t.url
                        && let Some(o) = run.outputs.as_object_mut()
                    {
                        o.insert("workflow_run".into(), json!(u));
                    }
                    if t.conclusion.as_deref() == Some("success") {
                        run.set("executor_succeeded", None, t.url.clone());
                        if expect.is_none() {
                            run.set("unverified", None, None);
                            return true;
                        }
                    } else {
                        run.fail(format!(
                            "workflow concluded {}",
                            t.conclusion.as_deref().unwrap_or("unknown")
                        ));
                        return true;
                    }
                }
                Ok(t) => {
                    if run.deadline.is_some_and(|dl| now_secs() >= dl) {
                        run.set(
                            "timed_out",
                            None,
                            Some(match t {
                                None => "the dispatched workflow run never appeared".into(),
                                Some(t) => format!("the workflow run was still {}", t.status),
                            }),
                        );
                        run.error = Some("the workflow did not finish before the deadline".into());
                    }
                    return run.state != before;
                }
                Err(e) => {
                    run.progress
                        .push(json!({"at": rfc3339(now_secs()), "note": e}));
                    return false;
                }
            }
        }
        // A proposed change is done only once its commit is on the base branch: the base
        // satisfying the postcondition by other means is not this run's effect.
        Executor::Change { .. } if run.state == "proposed" => {
            match crate::executor::merged(cwd, run) {
                Ok(true) if expect.is_none() => {
                    run.verification = Some(json!({"by": "base", "merged": true,
                        "base": run.outputs["base"], "commit": run.outputs["commit"]}));
                    run.set("succeeded", None, None);
                    return true;
                }
                Ok(true) => {}
                Ok(false) => {
                    if run.deadline.is_some_and(|dl| now_secs() >= dl) {
                        run.fail("the proposed change was not merged before the deadline");
                    }
                    return run.state != before;
                }
                Err(e) => {
                    run.progress
                        .push(json!({"at": rfc3339(now_secs()), "note": e}));
                    return false;
                }
            }
        }
        _ => {}
    }
    let Some(expect) = expect else {
        return run.state != before;
    };
    if run.state == "dispatched" {
        return run.state != before;
    }
    verify(run, expect, policies, obs, Duration::from_secs(0), false);
    if run.state == "verifying" {
        if run.deadline.is_some_and(|dl| now_secs() >= dl) {
            let merged = if run.executor == "change" {
                " (the change is merged, but the facts this portal reads never showed its effect)"
            } else {
                ""
            };
            run.fail(format!(
                "postcondition not met within {}{merged}: {}",
                expect.within, expect.postcondition
            ));
        } else if before != "executor_succeeded" {
            // Still waiting: keep the run as it was, without a `verifying` event per look.
            run.state = before.clone();
            run.events.truncate(events);
        }
    }
    run.state != before
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> Run {
        Run {
            id: "run_1".into(),
            action: "release".into(),
            action_version: "sha256:x".into(),
            entity: "repository:a".into(),
            inputs: Map::new(),
            requested_by: "user:bob".into(),
            via: "cli".into(),
            portal: "a".into(),
            executor: "command".into(),
            snapshot: "s".into(),
            effects: vec![],
            approval: Approval {
                required: true,
                approvers: vec![json!("group:maintainers")],
                ..Approval::default()
            },
            state: "awaiting_approval".into(),
            events: vec![],
            outputs: json!({}),
            error: None,
            verification: None,
            child: None,
            progress: vec![],
            deadline: None,
            held_before: false,
        }
    }

    #[test]
    fn tokens_confirm_only_the_preview_they_came_from() {
        let mut inputs = Map::new();
        inputs.insert("bump".into(), "minor".into());
        let fx = vec![json!({"resource": "tag:v1.3.0"})];
        let base = Confirmed {
            version: "v1",
            inputs: &inputs,
            effects: &fx,
            basis: "b",
            actor: "user:ann",
            entity: "repository:a",
        };
        let t = confirm_token(&base, now_secs());
        assert_eq!(check_token(&t, &base), Ok(()));
        let again = confirm_token(&base, token_issued(&t).unwrap());
        assert_ne!(
            again, t,
            "two previews in one second must not share a token"
        );
        assert_eq!(check_token(&again, &base), Ok(()));
        for other in [
            Confirmed {
                effects: &[],
                ..base
            },
            Confirmed {
                basis: "other facts",
                ..base
            },
            Confirmed {
                actor: "user:bob",
                ..base
            },
            Confirmed {
                entity: "repository:b",
                ..base
            },
        ] {
            assert_eq!(check_token(&t, &other), Err(TokenError::Stale));
        }
        let old = confirm_token(&base, now_secs() - TOKEN_TTL_SECS - 1);
        assert_eq!(check_token(&old, &base), Err(TokenError::Expired));
        let future = confirm_token(&base, now_secs() + 3600);
        assert_eq!(check_token(&future, &base), Err(TokenError::Malformed));
        assert_eq!(check_token("nope", &base), Err(TokenError::Malformed));
    }

    #[test]
    fn approvals_need_someone_other_than_the_requester() {
        let mut r = run();
        assert!(r.approve("user:bob", |_| true).is_err());
        assert!(r.approve("user:eve", |_| false).is_err());
        assert_eq!(
            r.approve("user:alice", |g| g == "group:maintainers"),
            Ok(true)
        );
        assert!(r.pending_approvers().is_empty());
    }

    #[test]
    fn any_one_of_a_list_approves_but_never_the_requester() {
        let mut r = run();
        r.approval.approvers = vec![json!("user:bob|user:carol|group:sre"), json!("user:dan")];
        assert!(r.approve("user:bob", |_| true).is_err(), "the requester");
        assert!(r.approve("user:eve", |_| false).is_err(), "not listed");
        assert_eq!(r.approve("user:eve", |g| g == "group:sre"), Ok(false));
        assert_eq!(r.pending_approvers(), vec!["user:dan".to_string()]);
        assert!(
            r.approve("user:carol", |_| false).is_err(),
            "the entry is met"
        );
        assert_eq!(r.approve("user:dan", |_| false), Ok(true));
        assert_eq!(
            r.approval.approved_by,
            vec![
                "user:bob|user:carol|group:sre=user:eve",
                "user:dan=user:dan"
            ]
        );
    }

    #[test]
    fn within_parses_units() {
        assert_eq!(parse_within("30s").as_secs(), 30);
        assert_eq!(parse_within("2m").as_secs(), 120);
        assert_eq!(parse_within("7d").as_secs(), 604_800);
    }

    #[test]
    fn ids_are_unique() {
        assert_ne!(new_id("run"), new_id("run"));
        assert!(new_id("run").starts_with("run_"));
    }
}
