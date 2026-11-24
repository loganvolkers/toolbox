//! Self-service actions on a served portal (docs/design/action-runs.md, federated-actions.md):
//! the list with permissions (own actions and child portals'), previews with effects and a confirm
//! token, runs with approvals, and verification by observation. The preview, executors and the
//! run lifecycle are `toolbox_exec`'s, the same ones `toolbox run` uses in a checkout; this module
//! adds what a repo portal lacks: authorization, approvals by other people, and a run store.

use crate::routes::problem;
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use toolbox_exec::federation;
use toolbox_exec::prepare::{Prepared, Refused, Request, prepare};
use toolbox_exec::run::{self as lifecycle, Observer, Run};
use toolbox_model::manifest::Expect;

/// Runs this instance holds, by id. Runs live in memory: a restart loses runs in flight, and
/// their effects are still observed by the scorecards that read the same facts.
#[derive(Debug, Default)]
pub struct Runs(Mutex<BTreeMap<String, Run>>);

impl Runs {
    fn put(&self, r: &Run) {
        self.0.lock().unwrap().insert(r.id.clone(), r.clone());
    }
    fn get(&self, id: &str) -> Option<Run> {
        self.0.lock().unwrap().get(id).cloned()
    }
    fn all(&self) -> Vec<Run> {
        self.0.lock().unwrap().values().cloned().collect()
    }
}

fn program() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "toolbox".into())
}

fn cwd() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| ".".into())
}

/// Query of `GET /v1/actions`.
#[derive(Deserialize)]
pub struct ListQuery {
    entity: Option<String>,
}

/// `GET /v1/actions?entity=` — the actions offered (on the entity), with whether the caller may
/// run them. A child portal's actions are offered on its portal entity.
pub async fn list(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    if let Some(id) = &q.entity
        && !p.derived.entities.contains_key(id)
    {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    }
    let all = federation::offered(&s.manifest, &p.derived);
    let mut out = Vec::new();
    for o in federation::on_entity(&all, &p.derived, q.entity.as_deref()) {
        let a = &o.action;
        let target = q.entity.clone().or_else(|| o.entity.clone());
        let allowed = match &target {
            Some(id) => match s.allowed(&caller, &a.permission, id).await {
                Ok(b) => Some(b),
                Err(r) => return r,
            },
            None => None,
        };
        let mut v = federation::describe(o, &p.derived);
        v["scope"] = json!(a.scope);
        v["permission"] = json!(a.permission);
        v["allowed"] = json!(allowed);
        v["annotations"] = json!(a.annotations);
        out.push(v);
    }
    Json(json!({"snapshot": p.snapshot_id, "actions": out})).into_response()
}

/// Body of a preview or a run.
#[derive(Deserialize)]
pub struct RunBody {
    entity: Option<String>,
    #[serde(default)]
    inputs: Map<String, Value>,
    confirm_token: Option<String>,
    /// `api` (default) or `mcp`, recorded on the run.
    via: Option<String>,
}

/// Authorize and prepare: the shared step of a preview and a run.
async fn prepared(
    s: &Arc<AppState>,
    user: &str,
    caller: &crate::access::Caller,
    name: &str,
    body: &RunBody,
) -> Result<(Prepared, String), Response> {
    let Some(p) = s.snapshot() else {
        return Err(problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        ));
    };
    let all = federation::offered(&s.manifest, &p.derived);
    let (off, entity) = federation::resolve(&all, &p.derived, name, body.entity.as_deref())
        .map_err(|e| {
            let code = if all.iter().any(|o| o.action.name == name) {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::NOT_FOUND
            };
            problem(code, "not-offered", e)
        })?;
    // A server previews and runs child code only where the child said it may: an undeclared
    // runner would run the child's commands with this server's credentials.
    if let Some(c) = &off.child
        && c.runner["type"].as_str().is_none()
    {
        return Err(problem(
            StatusCode::CONFLICT,
            "no-runner",
            format!(
                "portal {} declares no federation.runner, so its actions run only from a checkout: toolbox run {} --portal {}",
                c.portal,
                c.action,
                c.path.display()
            ),
        ));
    }
    let permission = off.action.permission.clone();
    match s.allowed(caller, &permission, &entity).await? {
        true => {}
        false => {
            let mut r = problem(
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("{user} lacks `{permission}` on {entity}"),
            );
            if let Ok(h) = permission.parse() {
                r.headers_mut().insert("x-toolbox-missing-relation", h);
            }
            return Err(r);
        }
    }
    let manifest = s.manifest.clone();
    let derived = p.derived.clone();
    let (name, entity, inputs, user) = (
        name.to_owned(),
        entity.clone(),
        body.inputs.clone(),
        user.to_owned(),
    );
    let result = tokio::task::spawn_blocking(move || {
        let program = program();
        prepare(
            &manifest,
            &derived,
            Request {
                action: &name,
                entity: Some(&entity),
                inputs,
                actor: &user,
                cwd: cwd(),
                program: &program,
            },
        )
    })
    .await
    .map_err(|e| problem(StatusCode::INTERNAL_SERVER_ERROR, "internal", e.to_string()))?;
    match result {
        Ok(x) => Ok((x, p.snapshot_id.clone())),
        Err(Refused::Invalid(e)) => Err(problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid-inputs",
            e.join("; "),
        )),
        Err(Refused::Unknown(e)) => Err(problem(StatusCode::NOT_FOUND, "not-found", e)),
        Err(Refused::Executor(e)) => Err(problem(StatusCode::BAD_GATEWAY, "executor", e)),
    }
}

/// `POST /v1/actions/{name}/previews` — validation, blockers, approvals, the executor's effects
/// and a confirm token. Changes nothing.
pub async fn preview(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(body): Json<RunBody>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match prepared(&s, &caller.user.clone(), &caller, &name, &body).await {
        Ok((p, snapshot)) => {
            let mut doc = p.doc(&s.manifest.portal.name);
            doc["snapshot"] = json!(snapshot);
            doc["requested_by"] = json!(caller.user);
            Json(doc).into_response()
        }
        Err(r) => r,
    }
}

/// Reads facts for verification from the published snapshots, asking the connectors behind the
/// entity to re-read it first.
struct Served {
    s: Arc<AppState>,
}

impl Observer for Served {
    fn observe(&mut self, run: &Run, _: &Expect) -> Result<(Value, String), String> {
        self.s.runs.put(run);
        let p = self.s.snapshot().ok_or("no snapshot")?;
        if let Some(e) = p.derived.entities.get(&run.entity) {
            for (connector, kind, ext) in &e.records {
                if self.s.hub.can_refresh(connector, kind, "external_ids") {
                    self.s.hub.enqueue(toolbox_connect::Request {
                        connector: connector.clone(),
                        command: "refresh",
                        kind: Some(kind.clone()),
                        kinds: vec![],
                        external_ids: vec![ext.clone()],
                        reason: format!("verify run {}", run.id),
                        priority: "interactive",
                        timeout: Duration::from_secs(60),
                        requested_by: run.requested_by.clone(),
                    });
                }
            }
        }
        let e = p
            .derived
            .entity_json(&run.entity)
            .ok_or_else(|| format!("{} is gone", run.entity))?;
        Ok((e, p.snapshot_id.clone()))
    }
}

/// Execute a confirmed (and approved) run in the background, storing each outcome.
fn start(s: &Arc<AppState>, p: Prepared, mut run: Run) {
    let s = s.clone();
    tokio::task::spawn_blocking(move || {
        let ctx = p.context(&run);
        let mut obs = Served { s: s.clone() };
        lifecycle::execute(
            &mut run,
            &p.exec,
            &ctx,
            p.offered.action.expect.as_ref(),
            &s.manifest.policies,
            &mut obs,
            Duration::from_secs(2),
        );
        s.runs.put(&run);
    });
}

/// `POST /v1/actions/{name}/runs` — run a previewed action. The confirm token must come from a
/// preview of the same inputs, effects and facts, less than ten minutes old.
pub async fn create_run(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(name): Path<String>,
    Json(body): Json<RunBody>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(token) = body.confirm_token.clone() else {
        return problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "confirm-token-required",
            "preview the action first and pass its confirm_token",
        );
    };
    let (p, snapshot) = match prepared(&s, &caller.user.clone(), &caller, &name, &body).await {
        Ok(x) => x,
        Err(r) => return r,
    };
    if let Err(e) = p.confirm(&token) {
        let code = match e {
            lifecycle::TokenError::Stale => "preview-stale",
            lifecycle::TokenError::Expired => "token-expired",
            lifecycle::TokenError::Malformed => "token-malformed",
        };
        return problem(StatusCode::CONFLICT, code, e.to_string());
    }
    if !p.blockers.is_empty() {
        return problem(StatusCode::CONFLICT, "blocked", p.blockers.join("; "));
    }
    let via = body.via.clone().unwrap_or_else(|| "api".into());
    let mut run = p.new_run(
        lifecycle::new_id("run"),
        &caller.user,
        via,
        &s.manifest.portal.name,
        &snapshot,
    );
    if !p.approvers.is_empty() && !p.reviewed {
        run.set(
            "awaiting_approval",
            None,
            Some(format!(
                "needs approval by {}",
                run.pending_approvers().join(", ")
            )),
        );
        s.runs.put(&run);
        return (StatusCode::ACCEPTED, Json(run_doc(&run))).into_response();
    }
    s.runs.put(&run);
    start(&s, p, run.clone());
    (StatusCode::ACCEPTED, Json(run_doc(&run))).into_response()
}

/// Query of `GET /v1/runs`.
#[derive(Deserialize)]
pub struct RunsQuery {
    entity: Option<String>,
    state: Option<String>,
}

/// A run as the API returns it: the record plus who it still waits for.
fn run_doc(r: &Run) -> Value {
    let mut v = json!(r);
    v["pending_approvers"] = json!(r.pending_approvers());
    v
}

/// `GET /v1/runs?entity=&state=` — runs on entities the caller can see.
pub async fn list_runs(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<RunsQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let runs: Vec<Run> = s
        .runs
        .all()
        .into_iter()
        .filter(|r| q.entity.as_ref().is_none_or(|e| &r.entity == e))
        .filter(|r| q.state.as_ref().is_none_or(|st| &r.state == st))
        .collect();
    let ids: Vec<&str> = runs.iter().map(|r| r.entity.as_str()).collect();
    let vis = match s.visibility(&caller, &ids).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let shown: Vec<Value> = runs
        .iter()
        .zip(vis)
        .filter(|(r, v)| v.view || r.requested_by == caller.user)
        .map(|(r, _)| {
            json!({"id": r.id, "action": r.action, "entity": r.entity, "state": r.state,
                "requested_by": r.requested_by, "pending_approvers": r.pending_approvers(),
                "started": r.events.first().map(|e| e.at.clone())})
        })
        .collect();
    Json(json!({"runs": shown})).into_response()
}

async fn visible_run(
    s: &Arc<AppState>,
    caller: &crate::access::Caller,
    id: &str,
) -> Result<Run, Response> {
    let not_found = || problem(StatusCode::NOT_FOUND, "not-found", format!("no run {id}"));
    let run = s.runs.get(id).ok_or_else(not_found)?;
    if run.requested_by == caller.user {
        return Ok(run);
    }
    let v = s.visibility(caller, &[run.entity.as_str()]).await?;
    if v.first().is_some_and(|v| v.view) {
        Ok(run)
    } else {
        Err(not_found())
    }
}

/// `GET /v1/runs/{id}` — the run record: every state, the effects, approvals and verification.
pub async fn get_run(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    match visible_run(&s, &caller, &id).await {
        Ok(r) => Json(run_doc(&r)).into_response(),
        Err(r) => r,
    }
}

/// Body of `POST /v1/runs/{id}/approvals`.
#[derive(Deserialize)]
pub struct ApprovalBody {
    /// `approve` (default) or `reject`.
    decision: Option<String>,
    note: Option<String>,
}

/// `POST /v1/runs/{id}/approvals` — approve or reject a run awaiting approval. The approver must
/// be one of the pending approvers (a user, or a member of a group) and not the requester. The
/// last approval starts the run.
pub async fn approve(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<ApprovalBody>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let mut run = match visible_run(&s, &caller, &id).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    // Groups the caller is in. Without an authorization model nobody is known to be in a group,
    // so a group approver can't be satisfied: an approval must not mean "anyone".
    let mut member = Vec::new();
    for g in run
        .pending_approvers()
        .iter()
        .flat_map(|a| a.split('|').map(str::to_owned).collect::<Vec<_>>())
    {
        if g == caller.user {
            member.push(g);
            continue;
        }
        if s.authz.is_none() || !g.starts_with("group:") {
            continue;
        }
        match s.allowed(&caller, "member", &g).await {
            Ok(true) => member.push(g),
            Ok(false) => {}
            Err(r) => return r,
        }
    }
    if body.decision.as_deref() == Some("reject") {
        if run.state != "awaiting_approval" || member.is_empty() {
            return problem(
                StatusCode::FORBIDDEN,
                "not-an-approver",
                format!("{} cannot reject run {id}", caller.user),
            );
        }
        run.approval.rejected_by = Some(caller.user.clone());
        run.set("rejected", Some(&caller.user), body.note.clone());
        s.runs.put(&run);
        return Json(run_doc(&run)).into_response();
    }
    let done = match run.approve(&caller.user, |g| member.iter().any(|m| m == g)) {
        Ok(d) => d,
        Err(e) => return problem(StatusCode::FORBIDDEN, "not-an-approver", e),
    };
    s.runs.put(&run);
    if !done {
        return Json(run_doc(&run)).into_response();
    }
    // Re-prepare against today's facts: an approval does not freeze a stale preview.
    let rb = RunBody {
        entity: Some(run.entity.clone()),
        inputs: run.inputs.clone(),
        confirm_token: None,
        via: None,
    };
    let requester = crate::access::Caller {
        user: run.requested_by.clone(),
        ..caller.clone()
    };
    let p = match prepared(&s, &run.requested_by.clone(), &requester, &run.action, &rb).await {
        Ok((p, _)) => p,
        Err(r) => {
            run.fail("could not prepare the approved run again");
            s.runs.put(&run);
            return r;
        }
    };
    if !p.blockers.is_empty() {
        run.fail(format!("blocked after approval: {}", p.blockers.join("; ")));
        s.runs.put(&run);
        return Json(run_doc(&run)).into_response();
    }
    start(&s, p, run.clone());
    Json(run_doc(&run)).into_response()
}

/// `POST /v1/runs/{id}/cancel` — cancel a run that has not started executing, or that waits on a
/// review or a workflow. Only the requester (or an admin) may.
pub async fn cancel(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let mut run = match visible_run(&s, &caller, &id).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    if run.requested_by != caller.user && !caller.admin {
        return problem(
            StatusCode::FORBIDDEN,
            "forbidden",
            "only the requester can cancel a run",
        );
    }
    if !["awaiting_approval", "proposed", "dispatched"].contains(&run.state.as_str()) {
        return problem(
            StatusCode::CONFLICT,
            "not-cancellable",
            format!("run {id} is {}", run.state),
        );
    }
    run.set("cancelled", Some(&caller.user), None);
    s.runs.put(&run);
    Json(run_doc(&run)).into_response()
}

/// Re-check one waiting run now; `true` when it changed.
fn recheck_one(s: &Arc<AppState>, run: &mut Run) -> bool {
    // Any other state belongs to the thread still executing the run.
    if !["proposed", "dispatched"].contains(&run.state.as_str()) {
        return false;
    }
    let Some(p) = s.snapshot() else {
        return false;
    };
    let all = federation::offered(&s.manifest, &p.derived);
    let Some(off) = all.iter().find(|o| {
        o.action.name == run.action && o.entity.as_ref().is_none_or(|e| e == &run.entity)
    }) else {
        return false;
    };
    let Ok(exec) = off.executor(&program()) else {
        return false;
    };
    let mut obs = Served { s: s.clone() };
    lifecycle::recheck(
        run,
        &exec,
        off.action.expect.as_ref(),
        &s.manifest.policies,
        &mut obs,
    )
}

/// `POST /v1/runs/{id}/verify` — look again now at a run waiting on a merge, a workflow or a
/// child portal, instead of waiting for the next background check.
pub async fn verify(
    State(s): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let run = match visible_run(&s, &caller, &id).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    let s2 = s.clone();
    let run = tokio::task::spawn_blocking(move || {
        let mut run = run;
        if recheck_one(&s2, &mut run) {
            s2.runs.put(&run);
        }
        run
    })
    .await
    .unwrap_or_else(|_| s.runs.get(&id).unwrap());
    Json(run_doc(&run)).into_response()
}

/// Background: every few seconds, re-check runs waiting on the world.
pub async fn runs_loop(s: Arc<AppState>) {
    loop {
        tokio::time::sleep(Duration::from_secs(10)).await;
        let waiting: Vec<Run> = s
            .runs
            .all()
            .into_iter()
            .filter(|r| ["proposed", "dispatched"].contains(&r.state.as_str()))
            .collect();
        if waiting.is_empty() {
            continue;
        }
        let s2 = s.clone();
        let _ = tokio::task::spawn_blocking(move || {
            for mut r in waiting {
                if recheck_one(&s2, &mut r) {
                    s2.runs.put(&r);
                }
            }
        })
        .await;
    }
}
