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
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use toolbox_exec::federation;
use toolbox_exec::prepare::{Prepared, Refused, Request, prepare};
use toolbox_exec::run::{self as lifecycle, Observer, Run};
use toolbox_model::manifest::Expect;

/// Runs this instance holds, by id. Every change that depends on a run's current state
/// (approve, reject, cancel) happens under one lock, and a run that has ended is never
/// overwritten. Once [`restore`] has run, each change is also written out, in order, to Postgres
/// or (without a database) to files under the state directory, and read back at the next start.
/// One instance owns its runs: a second instance on the same database does not see them live.
#[derive(Debug, Default)]
pub struct Runs {
    runs: Mutex<BTreeMap<String, Run>>,
    /// Confirm tokens already spent, with when they were issued.
    spent: Mutex<HashMap<String, u64>>,
    out: std::sync::OnceLock<tokio::sync::mpsc::UnboundedSender<Persist>>,
}

#[derive(Debug)]
enum Persist {
    Run(Box<Run>),
    Spent(String, u64),
}

impl Runs {
    fn persist(&self, p: Persist) {
        if let Some(tx) = self.out.get() {
            let _ = tx.send(p);
        }
    }
    /// Store `r`, unless the stored run has already ended (a cancel wins over a late result).
    fn save(&self, r: &Run) {
        let mut m = self.runs.lock().unwrap();
        if m.get(&r.id).is_some_and(Run::is_terminal) {
            return;
        }
        m.insert(r.id.clone(), r.clone());
        self.persist(Persist::Run(Box::new(r.clone())));
    }
    fn get(&self, id: &str) -> Option<Run> {
        self.runs.lock().unwrap().get(id).cloned()
    }
    fn all(&self) -> Vec<Run> {
        self.runs.lock().unwrap().values().cloned().collect()
    }
    /// Change the stored run in place, atomically: `f` sees the current record.
    fn update<T>(&self, id: &str, f: impl FnOnce(&mut Run) -> T) -> Option<(T, Run)> {
        let mut m = self.runs.lock().unwrap();
        let r = m.get_mut(id)?;
        let out = f(r);
        self.persist(Persist::Run(Box::new(r.clone())));
        Some((out, r.clone()))
    }
    /// Spend a confirm token: `false` when it was spent before.
    fn spend(&self, token: &str) -> bool {
        let mut m = self.spent.lock().unwrap();
        let now = lifecycle::now_secs();
        m.retain(|_, issued| now.saturating_sub(*issued) <= lifecycle::TOKEN_TTL_SECS + 60);
        let issued = lifecycle::token_issued(token).unwrap_or(now);
        let fresh = m.insert(token.to_owned(), issued).is_none();
        if fresh {
            self.persist(Persist::Spent(token.to_owned(), issued));
        }
        fresh
    }
}

/// Where a server without a database keeps its runs: `<state dir>/served/<portal>/runs`
/// ([`toolbox_model::federation::state_dir`]).
fn runs_dir(portal: &str) -> std::path::PathBuf {
    served_dir(portal).join("runs")
}

/// `<state dir>/served/<portal>`: what a server keeps on this host for one portal.
pub fn served_dir(portal: &str) -> std::path::PathBuf {
    let base = toolbox_model::federation::state_dir();
    let name: String = portal
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    base.join("served").join(name)
}

/// A run cut off by a restart while its executor was working: what it did is unknown, so it is
/// failed with that said, rather than left `running` forever or started again.
fn interrupted(r: &mut Run) -> bool {
    if !["previewed", "approved", "dispatched", "running"].contains(&r.state.as_str())
        || (r.state == "dispatched" && r.executor == "github-workflow")
    {
        return false;
    }
    r.fail(format!(
        "the server stopped while the run was {}; its effect is unknown, so check {} before running it again",
        r.state, r.entity
    ));
    true
}

/// Load the runs and spent tokens a previous start wrote, then write every later change out.
/// Returns how many runs were loaded.
pub async fn restore(s: &AppState) -> Result<usize, String> {
    let dir = runs_dir(&s.manifest.portal.name);
    let (docs, spent) = stored(s, &dir).await?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Persist>();
    let mut loaded = 0;
    {
        let mut runs = s.runs.runs.lock().unwrap();
        for d in docs {
            let Ok(mut r) = serde_json::from_str::<Run>(&d) else {
                tracing::warn!("skipping a stored run that is not a run record");
                continue;
            };
            if interrupted(&mut r) {
                let _ = tx.send(Persist::Run(Box::new(r.clone())));
            }
            runs.insert(r.id.clone(), r);
            loaded += 1;
        }
        s.runs.spent.lock().unwrap().extend(spent);
    }
    if s.pg.is_none() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    tokio::spawn(write_out(s.pg.clone(), dir, rx));
    let _ = s.runs.out.set(tx);
    Ok(loaded)
}

type Stored = (Vec<String>, Vec<(String, u64)>);

/// The stored run records and the tokens spent within a token's lifetime.
async fn stored(s: &AppState, dir: &std::path::Path) -> Result<Stored, String> {
    let since = lifecycle::now_secs().saturating_sub(lifecycle::TOKEN_TTL_SECS + 60);
    if let Some(pg) = &s.pg {
        let docs = pg.runs().await.map_err(|e| e.to_string())?;
        let spent = pg.spent(since as i64).await.map_err(|e| e.to_string())?;
        let spent = spent.into_iter().map(|(t, i)| (t, i.max(0) as u64));
        return Ok((docs, spent.collect()));
    }
    let docs = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .collect();
    let spent = std::fs::read_to_string(dir.join("spent-tokens"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (t, i) = l.split_once(' ')?;
            Some((t.to_owned(), i.parse().ok()?))
        })
        .filter(|(_, i)| *i >= since)
        .collect();
    Ok((docs, spent))
}

/// The single writer: every run change and spent token, in order, to Postgres or the directory.
async fn write_out(
    pg: Option<toolbox_store::pg::PgStore>,
    dir: std::path::PathBuf,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Persist>,
) {
    while let Some(p) = rx.recv().await {
        let res = match (&pg, p) {
            (Some(pg), Persist::Run(r)) => pg
                .put_run(&r.id, &serde_json::to_string(&r).unwrap_or_default())
                .await
                .map_err(|e| e.to_string()),
            (Some(pg), Persist::Spent(t, i)) => {
                pg.put_spent(&t, i as i64).await.map_err(|e| e.to_string())
            }
            (None, Persist::Run(r)) => {
                let file = dir.join(format!("{}.json", r.id));
                let tmp = dir.join(format!(".{}.tmp", r.id));
                std::fs::write(&tmp, serde_json::to_string_pretty(&r).unwrap_or_default())
                    .and_then(|()| std::fs::rename(&tmp, &file))
                    .map_err(|e| format!("{}: {e}", file.display()))
            }
            (None, Persist::Spent(t, i)) => {
                use std::io::Write as _;
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(dir.join("spent-tokens"))
                    .and_then(|mut f| writeln!(f, "{t} {i}"))
                    .map_err(|e| e.to_string())
            }
        };
        if let Err(e) = res {
            tracing::error!(error = %e, "could not store a run");
        }
    }
}

/// Set on requests the MCP endpoint makes on a caller's behalf, so a run records `via: mcp`. A
/// request extension, not a header or body field, so no network caller can claim it.
#[derive(Clone, Copy, Debug)]
pub struct Via(pub &'static str);

fn program() -> std::path::PathBuf {
    std::env::current_exe().unwrap_or_else(|_| "toolbox".into())
}

fn cwd() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| ".".into())
}

/// The approval signing seed: from the environment variable `federation.signingKey` names; else,
/// for a portal that federates children, one this host generates once and keeps in
/// `<state dir>/served/<portal>/signing-key`. Servers that share runs need the variable: a kept
/// seed is one host's, and children pin one key.
fn signing_seed(m: &toolbox_model::Manifest) -> Option<String> {
    let fed = m.federation.as_ref()?;
    if let Some(var) = &fed.signing_key {
        return std::env::var(var).ok().filter(|v| !v.trim().is_empty());
    }
    fed.portals.as_ref()?;
    kept_seed(&served_dir(&m.portal.name).join("signing-key"))
        .map_err(|e| tracing::error!(error = %e, "cannot keep an approval signing key"))
        .ok()
}

fn kept_seed(file: &std::path::Path) -> Result<String, String> {
    use std::io::Write;
    if let Ok(seed) = std::fs::read_to_string(file) {
        return Ok(seed.trim().to_owned());
    }
    let dir = file.parent().ok_or("no directory")?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let seed = toolbox_exec::grant::new_seed();
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    match opts.open(file) {
        Ok(mut f) => f
            .write_all(seed.as_bytes())
            .map(|()| seed)
            .map_err(|e| format!("{}: {e}", file.display())),
        // Another request wrote it first.
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => std::fs::read_to_string(file)
            .map(|s| s.trim().to_owned())
            .map_err(|e| format!("{}: {e}", file.display())),
        Err(e) => Err(format!("{}: {e}", file.display())),
    }
}

/// `ed25519:<hex>`: the key children pin to accept this portal's approvals.
pub fn approval_key(s: &AppState) -> Option<String> {
    approval_key_for(&s.manifest)
}

/// [`approval_key`] for a portal that is not served by this process (`toolbox serve --detach`).
pub fn approval_key_for(m: &toolbox_model::Manifest) -> Option<String> {
    toolbox_exec::grant::public_key(&signing_seed(m)?).ok()
}

/// A portal with git children (`federation.children`) re-reads their branches before it says
/// what they offer or prepares a run on one, so a definition pushed a moment ago counts.
async fn fresh_children(s: &AppState, user: &str) {
    let connector = toolbox_model::federation::CONNECTOR;
    let has_children = s
        .manifest
        .federation
        .as_ref()
        .is_some_and(|f| !f.children.is_empty());
    if !has_children {
        return;
    }
    if s.hub.first_sync(connector, "portal").is_none() {
        if !crate::connectors::reads_children(s) {
            return;
        }
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
        while s.hub.first_sync(connector, "portal").is_none()
            && tokio::time::Instant::now() < deadline
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        if let Some(v) = s.hub.first_sync(connector, "portal") {
            let _ = s.wait_for(v, Duration::from_secs(10)).await;
        }
        return;
    }
    if !s.hub.can_refresh(connector, "portal", "external_ids") {
        return;
    }
    let Some(p) = s.snapshot() else {
        return;
    };
    let ids: Vec<String> = p
        .derived
        .entities
        .values()
        .flat_map(|e| &e.records)
        .filter(|(c, k, _)| c == connector && k == "portal")
        .map(|(_, _, x)| x.clone())
        .collect();
    if ids.is_empty() {
        return;
    }
    let id = s.hub.enqueue(toolbox_connect::Request {
        connector: connector.into(),
        command: "refresh",
        kind: Some("portal".into()),
        kinds: vec![],
        external_ids: ids,
        reason: "read the children's branches".into(),
        priority: "interactive",
        timeout: Duration::from_secs(30),
        requested_by: user.to_owned(),
    });
    let _ = s.hub.wait(&id, Duration::from_secs(30)).await;
    let version = s.store.input_version().await.unwrap_or(0);
    s.changed.notify_one();
    let _ = s.wait_for(version, Duration::from_secs(10)).await;
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
    fresh_children(&s, &caller.user).await;
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
    Json(
        json!({"snapshot": p.snapshot_id, "actions": out, "approval_key": approval_key(&s),
        "unreadable": federation::unreadable(&s.manifest, &p.derived)}),
    )
    .into_response()
}

/// Body of a preview or a run.
#[derive(Deserialize)]
pub struct RunBody {
    entity: Option<String>,
    #[serde(default)]
    inputs: Map<String, Value>,
    confirm_token: Option<String>,
}

/// Authorize and prepare: the shared step of a preview and a run.
async fn prepared(
    s: &Arc<AppState>,
    user: &str,
    caller: &crate::access::Caller,
    name: &str,
    body: &RunBody,
) -> Result<(Prepared, String), Response> {
    fresh_children(s, user).await;
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
    // A server previews and runs child code only where the child said it may and this portal's
    // runners admit it: a preview runs the child's own commands.
    if let Some(c) = &off.child
        && let Err((code, e)) = c.served()
    {
        return Err(problem(StatusCode::CONFLICT, code, e));
    }
    authorize(s, caller, user, &off.action.permission, &entity).await?;
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
    let mut x = result.map_err(refusal)?;
    if let Some(b) = refuses_approvals(s, &x) {
        x.blockers.push(b);
    }
    Ok((x, p.snapshot_id.clone()))
}

async fn authorize(
    s: &AppState,
    caller: &crate::access::Caller,
    user: &str,
    permission: &str,
    entity: &str,
) -> Result<(), Response> {
    if s.allowed(caller, permission, entity).await? {
        return Ok(());
    }
    let mut r = problem(
        StatusCode::FORBIDDEN,
        "forbidden",
        format!("{user} lacks `{permission}` on {entity}"),
    );
    if let Ok(h) = permission.parse() {
        r.headers_mut().insert("x-toolbox-missing-relation", h);
    }
    Err(r)
}

fn refusal(e: Refused) -> Response {
    match e {
        Refused::Invalid(e) => problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid-inputs",
            e.join("; "),
        ),
        Refused::Unknown(e) => problem(StatusCode::NOT_FOUND, "not-found", e),
        Refused::Executor(e) => problem(StatusCode::BAD_GATEWAY, "executor", e),
    }
}

/// A child whose approval rule this portal must satisfy, but that would not accept this portal's
/// signed approvals: approving here could never start the run, so the preview says so first.
fn refuses_approvals(s: &AppState, p: &Prepared) -> Option<String> {
    let child = p.preview.child.as_ref()?;
    let c = p.offered.child.as_ref()?;
    let needs = child["approval"]["required"] == true && child["approval"]["by_review"] != true;
    if !needs {
        return None;
    }
    let ours = approval_key(s);
    let name = &s.manifest.portal.name;
    let pinned = child["accepts_approvals_from"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|t| t["portal"] == name.as_str() && ours.as_deref().is_some_and(|k| t["key"] == k));
    (!pinned).then(|| match ours {
        None => format!(
            "child-refuses-approvals: portal {} requires approval, and this portal has no signing key (federation.signingKey) to pass approvals with",
            c.portal
        ),
        Some(k) => format!(
            "child-refuses-approvals: portal {} requires approval and does not pin this portal's key; add {{portal: {name}, key: {k}}} to its federation.acceptApprovalsFrom",
            c.portal
        ),
    })
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

/// Execute a confirmed (and approved) run in the background, storing each outcome. `grant`
/// carries the run's approvals to a child portal.
fn start(s: &Arc<AppState>, p: Prepared, mut run: Run, grant: Option<String>) {
    let s = s.clone();
    tokio::task::spawn_blocking(move || {
        let mut ctx = p.context(&run);
        ctx.grant = grant;
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
        s.runs.save(&run);
    });
}

/// `POST /v1/actions/{name}/runs` — run a previewed action. The confirm token must come from a
/// preview of the same inputs, effects and facts, less than ten minutes old.
pub async fn create_run(
    State(s): State<Arc<AppState>>,
    via: Option<axum::Extension<Via>>,
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
    if !s.runs.spend(&token) {
        return problem(
            StatusCode::CONFLICT,
            "token-used",
            "this confirm token already started a run; preview again to run the action again",
        );
    }
    let via = via.map_or("api", |v| v.0.0).to_owned();
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
        s.runs.save(&run);
        return (StatusCode::ACCEPTED, Json(run_doc(&run))).into_response();
    }
    s.runs.save(&run);
    start(&s, p, run.clone(), None);
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
/// be one of the pending approvers (a user, or a member of a group), not the requester, and signed
/// in interactively (D57): a personal access token or an MCP session can't approve. The last
/// approval starts the run.
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
    if !interactive(&s, &caller) {
        return problem(
            StatusCode::FORBIDDEN,
            "approval-needs-a-person",
            format!(
                "approvals need an interactive sign-in (proxy or OIDC), not a {} credential: approve in the portal's UI",
                caller.method
            ),
        );
    }
    let run = match visible_run(&s, &caller, &id).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    let member = match memberships(&s, &caller, &run).await {
        Ok(m) => m,
        Err(r) => return r,
    };
    let reject = body.decision.as_deref() == Some("reject");
    let note = body.note.as_ref();
    let decided = s
        .runs
        .update(&id, |run| decide(run, reject, &caller.user, &member, note));
    match decided {
        None => problem(StatusCode::NOT_FOUND, "not-found", format!("no run {id}")),
        Some((Err(e), _)) => problem(StatusCode::FORBIDDEN, "not-an-approver", e),
        Some((Ok(false), run)) => Json(run_doc(&run)).into_response(),
        Some((Ok(true), run)) => start_approved(&s, &caller, run).await,
    }
}

/// A person signed in through a proxy or OIDC, or on a dev server; never a token or MCP (D57).
fn interactive(s: &AppState, caller: &crate::access::Caller) -> bool {
    match caller.method {
        "iap" | "oidc" => true,
        "dev" => s.config.auth_mode == "dev",
        _ => false,
    }
}

/// Records one approval or rejection on `run`, under the runs lock. `Ok(true)` when it was the
/// last approval the rule needed: the run moves out of `awaiting_approval` here, so a second
/// approval can't start it twice.
fn decide(
    run: &mut Run,
    reject: bool,
    user: &str,
    member: &[String],
    note: Option<&String>,
) -> Result<bool, String> {
    if reject {
        if run.state != "awaiting_approval" || member.is_empty() {
            return Err(format!("{user} cannot reject run {}", run.id));
        }
        run.approval.rejected_by = Some(user.to_owned());
        run.set("rejected", Some(user), note.cloned());
        return Ok(false);
    }
    let done = run.approve(user, |g| member.iter().any(|m| m == g))?;
    if let (Some(n), Some(e)) = (note, run.events.last_mut()) {
        e.note = Some(match e.note.take() {
            Some(slot) => format!("as {slot}: {n}"),
            None => n.clone(),
        });
    }
    if done {
        run.state = "approved".into();
    }
    Ok(done)
}

/// The pending approvers (users, or groups) the caller counts as. Without an authorization model
/// nobody is known to be in a group, so a group approver can't be satisfied: an approval must not
/// mean "anyone".
async fn memberships(
    s: &AppState,
    caller: &crate::access::Caller,
    run: &Run,
) -> Result<Vec<String>, Response> {
    let mut member = Vec::new();
    for g in run
        .pending_approvers()
        .iter()
        .flat_map(|a| a.split('|').map(str::to_owned).collect::<Vec<_>>())
    {
        let counts = g.eq_ignore_ascii_case(&caller.user)
            || (s.authz.is_some()
                && g.starts_with("group:")
                && s.allowed(caller, "member", &g).await?);
        if counts {
            member.push(g);
        }
    }
    Ok(member)
}

/// Start a run its last approval released, if it is still the run that was approved.
async fn start_approved(
    s: &Arc<AppState>,
    caller: &crate::access::Caller,
    mut run: Run,
) -> Response {
    // Re-prepare against today's facts: an approval does not freeze a stale preview.
    let rb = RunBody {
        entity: Some(run.entity.clone()),
        inputs: run.inputs.clone(),
        confirm_token: None,
    };
    let requester = crate::access::Caller {
        user: run.requested_by.clone(),
        ..caller.clone()
    };
    let p = match prepared(s, &run.requested_by.clone(), &requester, &run.action, &rb).await {
        Ok((p, _)) => p,
        Err(r) => {
            run.fail("could not prepare the approved run again");
            s.runs.save(&run);
            return r;
        }
    };
    if !p.blockers.is_empty() {
        run.fail(format!("blocked after approval: {}", p.blockers.join("; ")));
        s.runs.save(&run);
        return Json(run_doc(&run)).into_response();
    }
    // The approvers approved a definition and its effects; if either changed since, their
    // approval is not for this run.
    if p.version != run.action_version || p.preview.effects != run.effects {
        run.fail(
            "preview_stale: the action or its effects changed after it was approved; preview and request it again",
        );
        s.runs.save(&run);
        return Json(run_doc(&run)).into_response();
    }
    let grant = match signing_seed(&s.manifest).map(|seed| p.grant(&run, &seed)) {
        Some(Ok(g)) => g,
        Some(Err(e)) => {
            run.fail(format!(
                "could not sign the approvals for the child portal: {e}"
            ));
            s.runs.save(&run);
            return Json(run_doc(&run)).into_response();
        }
        None => None,
    };
    s.runs.save(&run);
    start(s, p, run.clone(), grant);
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
    let run = match visible_run(&s, &caller, &id).await {
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
    // Cancelling stops Toolbox from carrying the run further; a pull request or workflow it
    // already started is closed where it lives.
    let cancelled = s.runs.update(&id, |run| {
        let ok = ["awaiting_approval", "proposed", "dispatched"].contains(&run.state.as_str());
        if ok {
            run.set("cancelled", Some(&caller.user), None);
        }
        ok
    });
    match cancelled {
        Some((true, run)) => Json(run_doc(&run)).into_response(),
        Some((false, run)) => problem(
            StatusCode::CONFLICT,
            "not-cancellable",
            format!("run {id} is {}", run.state),
        ),
        None => problem(StatusCode::NOT_FOUND, "not-found", format!("no run {id}")),
    }
}

/// States in which a run waits on the world and is looked at again.
const WAITING: [&str; 4] = ["proposed", "dispatched", "executor_succeeded", "verifying"];

/// Re-check one waiting run now; `true` when it changed.
fn recheck_one(s: &Arc<AppState>, run: &mut Run) -> bool {
    // Any other state belongs to the thread still executing the run.
    if !WAITING.contains(&run.state.as_str()) {
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
        &cwd(),
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
            s2.runs.save(&run);
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
            .filter(|r| WAITING.contains(&r.state.as_str()))
            .collect();
        if waiting.is_empty() {
            continue;
        }
        let s2 = s.clone();
        let _ = tokio::task::spawn_blocking(move || {
            for mut r in waiting {
                if recheck_one(&s2, &mut r) {
                    s2.runs.save(&r);
                }
            }
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_restart_fails_only_runs_whose_executor_it_cut_off() {
        let run = |state: &str, executor: &str| -> Run {
            serde_json::from_value(json!({"id": "run_1", "action": "a", "action_version": "v",
                "entity": "repository:x", "inputs": {}, "requested_by": "user:ann", "via": "api",
                "portal": "org", "executor": executor, "snapshot": "", "effects": [],
                "approval": {"required": false, "approvers": []}, "state": state, "events": [],
                "outputs": {}}))
            .unwrap()
        };
        for st in ["previewed", "approved", "running"] {
            let mut r = run(st, "command");
            assert!(interrupted(&mut r));
            assert_eq!(r.state, "failed");
            assert!(r.error.unwrap().contains("effect is unknown"));
        }
        for (st, ex) in [
            ("awaiting_approval", "command"),
            ("proposed", "change"),
            ("dispatched", "github-workflow"),
            ("verifying", "command"),
            ("succeeded", "command"),
        ] {
            let mut r = run(st, ex);
            assert!(!interrupted(&mut r), "{st} {ex}");
            assert_eq!(r.state, st);
        }
    }
}
