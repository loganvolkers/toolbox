//! `toolbox run`: self-service actions without a server (docs/design/federated-actions.md §6). In
//! a repo portal the action is previewed, confirmed, executed and verified from the checkout,
//! with the same definition and lifecycle the server uses. With no action it lists what the portal
//! offers, including its child portals' actions (federation).
//!
//! A repo portal has no authorization (docs/design/repo-portals.md §2): whoever has the checkout
//! and its credentials runs. An action whose approval rule names approvers therefore runs here
//! only as a `change` (the review is the approval), or dispatched by a parent portal that
//! recorded the approval and signed it with the key this portal pinned
//! (`federation.acceptApprovalsFrom`, `toolbox_exec::grant`).

use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;
use toolbox_exec::federation::{self, Offered};
use toolbox_exec::prepare::{Refused, Request, prepare};
use toolbox_exec::run::{self as lifecycle, Observer, Run};
use toolbox_model::manifest::Expect;

/// `toolbox run`'s flags.
#[derive(clap::Args)]
pub struct Options {
    /// The action to preview or run. Without it, list the actions this portal offers.
    pub action: Option<String>,
    /// The portal directory or manifest (default: `.`, or `.toolbox` when `.` has no toolbox.yaml).
    #[arg(long, short = 'p')]
    pub portal: Option<PathBuf>,
    /// The target entity (default: the only entity the action applies to).
    #[arg(long)]
    pub entity: Option<String>,
    /// An input, `name=value` (repeatable). Non-string inputs take JSON: `count=3`.
    #[arg(long = "input", short = 'i')]
    pub input: Vec<String>,
    /// All inputs as one JSON object (merged under --input).
    #[arg(long)]
    pub inputs: Option<String>,
    /// Preview only (the default without --confirm or --yes): print the effects and a confirm
    /// token. Changes nothing.
    #[arg(long)]
    pub preview: bool,
    /// Run, if the preview is still the one this token came from (valid 10 minutes).
    #[arg(long, conflicts_with = "preview")]
    pub confirm: Option<String>,
    /// Preview and run in one step (scripts and dispatched runs; the preview is still recorded).
    #[arg(long, conflicts_with_all = ["preview", "confirm"])]
    pub yes: bool,
    /// Re-check a run by its id and print its record: who asked, approved and ran it, and what
    /// was observed (saved under `$XDG_STATE_HOME/toolbox/runs/`, or the server's with --url).
    #[arg(long, value_name = "RUN_ID", conflicts_with_all = ["preview", "confirm", "yes"])]
    pub verify: Option<String>,
    /// Keep checking until the run settles: it succeeds, fails, its deadline passes, or it waits
    /// for a person (approval). With --verify, --confirm or --yes.
    #[arg(long)]
    pub wait: bool,
    /// List the runs: saved ones from a checkout, or the server's with --url.
    #[arg(long, conflicts_with_all = ["preview", "confirm", "yes", "verify"])]
    pub runs: bool,
    /// Run through a served portal (`toolbox serve`) at this URL instead of a checkout: the same
    /// preview, confirm, run and verify, with the server's authorization, approvals and runners.
    #[arg(long, value_name = "URL", env = "TOOLBOX_URL")]
    pub url: Option<String>,
    /// With --url: a personal access token.
    #[arg(long, env = "TOOLBOX_TOKEN", requires = "url", hide_env_values = true)]
    pub token: Option<String>,
    /// With --url: approve a run that waits for you. Approvals need an interactive sign-in; a
    /// server with dev sign-in (the default on a loopback address) takes --as.
    #[arg(long, value_name = "RUN_ID", requires = "url", conflicts_with_all = ["preview", "confirm", "yes", "verify", "runs", "reject"])]
    pub approve: Option<String>,
    /// With --url: reject a run that waits for you.
    #[arg(long, value_name = "RUN_ID", requires = "url", conflicts_with_all = ["preview", "confirm", "yes", "verify", "runs"])]
    pub reject: Option<String>,
    /// A note recorded with --approve or --reject.
    #[arg(long)]
    pub note: Option<String>,
    /// Also write the run record to this file.
    #[arg(long)]
    pub record: Option<PathBuf>,
    /// Who is asking, recorded on the run (default: `user:$USER`).
    #[arg(long = "as", env = "TOOLBOX_AS")]
    pub as_user: Option<String>,
    /// Connectors not to run when reading facts (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub skip: Vec<String>,
    /// Use the portal's fixtures instead of running its connectors.
    #[arg(long)]
    pub fixtures: bool,
    /// Output: md (default) or json.
    #[arg(short = 'o', long, default_value = "md")]
    pub output: String,
    /// Set by a parent portal dispatching this run: `<parent portal>/<parent run id>`.
    #[arg(long, hide = true)]
    pub parent_run: Option<String>,
    /// Set by a parent portal: the approvals it recorded, signed (`toolbox_exec::grant`).
    #[arg(long, hide = true)]
    pub grant: Option<String>,
}

fn default_portal(p: Option<&Path>) -> PathBuf {
    match p {
        Some(p) => p.to_path_buf(),
        None if !Path::new("toolbox.yaml").exists()
            && Path::new(".toolbox/toolbox.yaml").exists() =>
        {
            PathBuf::from(".toolbox")
        }
        None => PathBuf::from("."),
    }
}

fn portal_dir(p: &Path) -> PathBuf {
    let p = if p.is_file() {
        p.parent().unwrap_or(Path::new("."))
    } else {
        p
    };
    if p.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        p.to_path_buf()
    }
}

fn actor(o: &Options) -> String {
    o.as_user.clone().unwrap_or_else(|| {
        format!(
            "user:{}",
            std::env::var("USER").unwrap_or_else(|_| "local".into())
        )
    })
}

fn emit(o: &Options, doc: &Value, md: impl FnOnce() -> String) {
    if o.output == "json" {
        println!("{}", serde_json::to_string_pretty(doc).unwrap_or_default());
    } else {
        print!("{}", md());
    }
}

fn error(o: &Options, code: u8, msg: String) -> ExitCode {
    if o.output == "json" {
        println!("{}", json!({"error": msg}));
    } else {
        eprintln!("error: {msg}");
    }
    ExitCode::from(code)
}

fn parse_inputs(o: &Options, schema: &Value) -> Result<Map<String, Value>, String> {
    let mut m = match &o.inputs {
        Some(s) => match serde_json::from_str::<Value>(s) {
            Ok(Value::Object(m)) => m,
            _ => return Err("--inputs must be a JSON object".into()),
        },
        None => Map::new(),
    };
    for kv in &o.input {
        let (k, v) = kv
            .split_once('=')
            .ok_or_else(|| format!("--input {kv}: expected name=value"))?;
        let ty = schema["properties"][k]["type"].as_str().unwrap_or("string");
        let val = if ty == "string" {
            Value::String(v.into())
        } else {
            serde_json::from_str(v).unwrap_or_else(|_| Value::String(v.into()))
        };
        m.insert(k.into(), val);
    }
    Ok(m)
}

fn effect_line(e: &Value) -> String {
    match (
        e.get("operation").and_then(Value::as_str),
        e.get("resource").and_then(Value::as_str),
    ) {
        (Some(op), Some(r)) => {
            let d = e
                .get("description")
                .and_then(Value::as_str)
                .map(|d| format!(" — {d}"))
                .unwrap_or_default();
            format!("{op} {r}{d}")
        }
        _ => e
            .get("description")
            .and_then(Value::as_str)
            .map_or_else(|| e.to_string(), str::to_owned),
    }
}

/// Re-reads the portal's facts (all its in-image connectors) for verification.
struct Local {
    portal: PathBuf,
    fixtures: bool,
    skip: Vec<String>,
}

impl Observer for Local {
    fn observe(&mut self, run: &Run, _: &Expect) -> Result<(Value, String), String> {
        let d = match run.outputs.get("base").and_then(Value::as_str) {
            Some(base) if run.executor == "change" => {
                if !is_ref(base) {
                    return Err(format!("{base:?} is not a branch name"));
                }
                if let Some((remote, branch)) = base.split_once('/') {
                    let dir = toolbox_exec::executor::portal_repo_dir(&portal_dir(&self.portal));
                    let _ = std::process::Command::new("git")
                        .args(["fetch", "-q", "--", remote, branch])
                        .current_dir(dir)
                        .status();
                }
                crate::report::live_at(&self.portal, self.fixtures, &self.skip, base)?.1
            }
            _ => crate::report::live(&self.portal, self.fixtures, &self.skip)?.2,
        };
        let e = d
            .entity_json(&run.entity)
            .ok_or_else(|| format!("{} is gone", run.entity))?;
        Ok((e, d.snapshot.clone()))
    }
}

/// A branch or `remote/branch` name, and nothing git would read as an option or a revision range.
fn is_ref(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with(['-', '/'])
        && !s.contains("..")
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c))
}

fn list(
    o: &Options,
    portal: &Path,
    m: &Manifest,
    all: &[Offered],
    d: &toolbox_derive::Derived,
) -> ExitCode {
    let shown = federation::on_entity(all, d, o.entity.as_deref());
    let actions: Vec<Value> = shown.iter().map(|x| federation::describe(x, d)).collect();
    let doc = json!({"portal": portal, "actions": actions,
        "unreadable": federation::unreadable(m, d)});
    emit(o, &doc, || {
        list_md(&portal.display().to_string(), &actions, "") + &unreadable_md(&doc)
    });
    ExitCode::SUCCESS
}

/// The child portals that offer nothing, and why.
fn unreadable_md(doc: &Value) -> String {
    doc["unreadable"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|u| {
            format!(
                "\n{} offers no actions: {}\n",
                u["entity"].as_str().unwrap_or(""),
                u["error"].as_str().unwrap_or("")
            )
        })
        .collect()
}

/// The actions as `federation::describe` (or `GET /v1/actions`) gives them. `via` is appended to
/// the suggested command (` --url …` for a served portal).
fn list_md(portal: &str, actions: &[Value], via: &str) -> String {
    let mut s = format!("{} action(s) in {portal}\n", actions.len());
    for v in actions {
        let ents: Vec<&str> = v["entities"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        s.push_str(&format!(
            "\n- {} ({} on {}): {}\n  inputs: {}\n  executor: {}; verified: {}{}{}\n",
            v["name"].as_str().unwrap_or(""),
            v["kind"].as_str().unwrap_or(""),
            if ents.is_empty() {
                "no entity yet".into()
            } else {
                ents.join(", ")
            },
            v["description"].as_str().unwrap_or(""),
            federation::schema_line(&v["inputs"]),
            v["executor"].as_str().unwrap_or(""),
            if v["verified"] == true { "yes" } else { "no" },
            v["approval"]
                .as_str()
                .map(|a| format!("; approval: {a}"))
                .unwrap_or_default(),
            if v["allowed"] == false {
                "; you may not run it"
            } else {
                ""
            },
        ));
    }
    s.push_str(&format!(
        "\nPreview one: toolbox run <action>{via} [--entity ID] --input name=value\n"
    ));
    s
}

/// Lists, previews, runs or verifies an action: in the checkout, or with `--url` on a served
/// portal. The exit code follows the run's state (`exit_code`).
pub fn run(o: &Options) -> ExitCode {
    if let Some(url) = &o.url {
        return remote::run(o, url.trim_end_matches('/'));
    }
    if o.approve.is_some() || o.reject.is_some() {
        return error(
            o,
            2,
            "a checkout holds no approvals: approve through the portal that recorded the run (--url)".into(),
        );
    }
    if o.runs {
        return saved_runs(o);
    }
    let portal = default_portal(o.portal.as_deref());
    if let Some(id) = &o.verify {
        if !toolbox_exec::executor::is_run_id(id) {
            return error(o, 2, format!("--verify takes a run id (run_…), not {id:?}"));
        }
        return verify_file(o, &portal, &runs_dir().join(format!("{id}.json")));
    }
    let (_, m, d) = match crate::report::live(&portal, o.fixtures, &o.skip) {
        Ok(x) => x,
        Err(e) => return error(o, 2, e),
    };
    let all = federation::offered(&m, &d);
    match &o.action {
        None => list(o, &portal, &m, &all, &d),
        Some(name) => act(o, &portal, &m, &d, &all, name),
    }
}

type Manifest = toolbox_model::Manifest;
type Prepared = toolbox_exec::prepare::Prepared;

fn act(
    o: &Options,
    portal: &Path,
    m: &Manifest,
    d: &toolbox_derive::Derived,
    all: &[Offered],
    name: &str,
) -> ExitCode {
    let schema = federation::resolve(all, d, name, o.entity.as_deref())
        .map(|(x, _)| x.action.inputs.clone())
        .unwrap_or(Value::Null);
    let inputs = match parse_inputs(o, &schema) {
        Ok(i) => i,
        Err(e) => return error(o, 2, e),
    };
    let program = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("toolbox"));
    let requested_by = actor(o);
    let req = Request {
        action: name,
        entity: o.entity.as_deref(),
        inputs,
        actor: &requested_by,
        cwd: toolbox_exec::executor::portal_repo_dir(&portal_dir(portal)),
        program: &program,
    };
    let p = match prepare(m, d, req) {
        Ok(p) => p,
        Err(e) => return refused(o, all, d, name, &schema, e),
    };
    if o.confirm.is_none() && !o.yes {
        let (doc, next) = preview(portal, m, &p, name);
        emit(o, &doc, || {
            preview_md(name, &p.entity, &m.portal.name, &doc, &next)
        });
        return ExitCode::SUCCESS;
    }
    if let Some(t) = &o.confirm
        && let Err(e) = p.confirm(t)
    {
        return error(o, 1, e.to_string());
    }
    if !p.blockers.is_empty() {
        return error(
            o,
            1,
            format!("{name} is blocked: {}", p.blockers.join("; ")),
        );
    }
    let via = o
        .parent_run
        .as_ref()
        .map_or_else(|| "cli".into(), |pr| format!("parent:{pr}"));
    let mut run = p.new_run(
        lifecycle::new_id("run"),
        &requested_by,
        via,
        &m.portal.name,
        &d.snapshot,
    );
    if !p.approvers.is_empty() && !approved(o, m, &p, &requested_by, &mut run) {
        return finish(o, &run, 3);
    }
    execute(o, portal, m, &p, run)
}

fn refused(
    o: &Options,
    all: &[Offered],
    d: &toolbox_derive::Derived,
    name: &str,
    schema: &Value,
    e: Refused,
) -> ExitCode {
    let errors = match e {
        Refused::Invalid(errors) => errors,
        e @ Refused::Executor(_) => return error(o, 1, e.to_string()),
        e => return error(o, 2, e.to_string()),
    };
    let line = federation::resolve(all, d, name, o.entity.as_deref())
        .map(|(x, _)| federation::inputs_line(&x.action))
        .unwrap_or_default();
    let doc = json!({"action": name, "errors": errors, "inputs": schema});
    emit(o, &doc, || {
        format!(
            "Invalid inputs for {name}:\n{}Inputs: {line}\n",
            errors
                .iter()
                .map(|e| format!("  - {e}\n"))
                .collect::<String>()
        )
    });
    ExitCode::from(2)
}

/// The preview document and the command that confirms it.
fn preview(portal: &Path, m: &Manifest, p: &Prepared, name: &str) -> (Value, String) {
    let next = format!(
        "toolbox run {name} --portal {} --entity {}{} --confirm {}",
        portal.display(),
        p.entity,
        input_args(&p.inputs),
        p.token
    );
    let mut doc = p.doc(&m.portal.name);
    doc["accepts_approvals_from"] = json!(
        m.federation
            .as_ref()
            .map(|f| f.accept_approvals_from.clone())
            .unwrap_or_default()
    );
    doc["next"] = if p.blockers.is_empty() {
        Value::String(next.clone())
    } else {
        Value::Null
    };
    (doc, next)
}

/// Applies a parent's signed grant, if one came, and reports whether the run may go ahead. If
/// approvers are still pending (and review is not the approval), the run waits for them.
fn approved(o: &Options, m: &Manifest, p: &Prepared, requested_by: &str, run: &mut Run) -> bool {
    let mut refused = None;
    if let Some(g) = o.grant.as_deref().filter(|g| !g.is_empty()) {
        match granted(o, m, p, requested_by, g) {
            Ok(approvals) => {
                run.state = "awaiting_approval".into();
                for a in &approvals {
                    let (which, who) = a.split_once('=').unwrap_or((a, a));
                    let _ = run.approve(who, |g| g == which);
                }
            }
            Err(e) => refused = Some(e),
        }
    }
    let pending = run.pending_approvers();
    if pending.is_empty() || p.reviewed {
        return true;
    }
    let why = refused.map_or_else(
        || "a repo portal holds no approvals, so run it through a portal that does (a parent with `toolbox serve`), or make it a `change` executor that proposes a branch so review is the approval".to_owned(),
        |e| format!("the parent's approvals were refused: {e}"),
    );
    run.set(
        "awaiting_approval",
        None,
        Some(format!("needs approval by {}: {why}", pending.join(", "))),
    );
    false
}

fn execute(o: &Options, portal: &Path, m: &Manifest, p: &Prepared, mut run: Run) -> ExitCode {
    let ctx = p.context(&run);
    let mut obs = Local {
        portal: portal.to_path_buf(),
        fixtures: o.fixtures,
        skip: o.skip.clone(),
    };
    lifecycle::execute(
        &mut run,
        &p.exec,
        &ctx,
        p.offered.action.expect.as_ref(),
        &m.policies,
        &mut obs,
        Duration::from_secs(2),
    );
    if o.wait && !run.is_terminal() && run.state != "awaiting_approval" {
        let file = runs_dir().join(format!("{}.json", run.id));
        let _ = std::fs::create_dir_all(runs_dir());
        let _ = std::fs::write(&file, serde_json::to_string(&run).unwrap_or_default());
        return verify_file(o, portal, &file);
    }
    finish(o, &run, exit_code(&run.state))
}

/// 0 done (or waiting on the world), 1 failed, 3 waiting for a person's approval.
fn exit_code(state: &str) -> u8 {
    match state {
        "failed" | "timed_out" | "rejected" | "cancelled" => 1,
        "awaiting_approval" => 3,
        _ => 0,
    }
}

/// The runs saved under [`runs_dir`], newest first.
fn saved_runs(o: &Options) -> ExitCode {
    let mut runs: Vec<Run> = std::fs::read_dir(runs_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| std::fs::read_to_string(e.path()).ok())
        .filter_map(|t| serde_json::from_str(&t).ok())
        .filter(|r: &Run| o.entity.as_ref().is_none_or(|e| &r.entity == e))
        .filter(|r: &Run| o.action.as_ref().is_none_or(|a| &r.action == a))
        .collect();
    runs.sort_by(|a, b| b.id.cmp(&a.id));
    let docs: Vec<Value> = runs.iter().map(|r| json!(r)).collect();
    emit(o, &json!({"runs": docs}), || runs_md(&docs));
    ExitCode::SUCCESS
}

fn runs_md(runs: &[Value]) -> String {
    let mut s = format!("{} run(s)\n", runs.len());
    for r in runs {
        s.push_str(&format!(
            "- {} {} on {}: {}{}\n",
            r["id"].as_str().unwrap_or(""),
            r["action"].as_str().unwrap_or(""),
            r["entity"].as_str().unwrap_or(""),
            r["state"].as_str().unwrap_or(""),
            r["error"]
                .as_str()
                .map(|e| format!(" ({e})"))
                .unwrap_or_default()
        ));
    }
    s
}

/// The approvals in a parent's grant, if this portal pinned the parent's key and the grant is for
/// exactly this run: the action and its version, entity, inputs, effects and requester.
fn granted(
    o: &Options,
    m: &toolbox_model::Manifest,
    p: &toolbox_exec::prepare::Prepared,
    requested_by: &str,
    token: &str,
) -> Result<Vec<String>, String> {
    let (parent, parent_run) = o
        .parent_run
        .as_deref()
        .and_then(|pr| pr.split_once('/'))
        .ok_or("a grant comes with --parent-run <portal>/<run>")?;
    let pin = m
        .federation
        .as_ref()
        .and_then(|f| f.accept_approvals_from.iter().find(|t| t.portal == parent))
        .ok_or_else(|| {
            format!("this portal does not accept approvals from {parent} (federation.acceptApprovalsFrom)")
        })?;
    let g = toolbox_exec::grant::open(token, &pin.key, lifecycle::now_secs())?;
    let checks = [
        ("parent", g.parent == parent),
        ("parent run", g.parent_run == parent_run),
        ("action", g.action == p.offered.action.name),
        ("action version", g.action_version == p.version),
        ("entity", g.entity == p.entity),
        ("inputs", g.inputs == p.inputs),
        (
            "effects",
            g.effects == toolbox_exec::grant::effects_hash(&p.preview.effects),
        ),
        ("requester", g.requested_by == requested_by),
    ];
    let wrong: Vec<&str> = checks
        .iter()
        .filter(|(_, ok)| !ok)
        .map(|(k, _)| *k)
        .collect();
    if !wrong.is_empty() {
        return Err(format!(
            "the grant is for a different run ({} differ); approve the run through the parent again",
            wrong.join(", ")
        ));
    }
    Ok(g.approvals)
}

/// `--input k=v` for each input, quoted for a shell.
fn input_args(inputs: &Map<String, Value>) -> String {
    inputs
        .iter()
        .map(|(k, v)| {
            let s = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
            format!(" --input {k}={}", shell_quote(&s))
        })
        .collect()
}

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./:@=,".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

pub(crate) fn preview_md(name: &str, entity: &str, portal: &str, p: &Value, next: &str) -> String {
    let mut s = format!("Preview: {name} on {entity} (portal {portal})\n");
    let fx = p["effects"].as_array().cloned().unwrap_or_default();
    s.push_str(&format!("Effects ({}):\n", fx.len()));
    for e in &fx {
        s.push_str(&format!("  - {}\n", effect_line(e)));
        if let Some(d) = e
            .get("diff")
            .and_then(Value::as_str)
            .filter(|d| !d.is_empty())
        {
            for l in d.lines() {
                s.push_str(&format!("      {l}\n"));
            }
        }
    }
    for w in p["warnings"].as_array().into_iter().flatten() {
        s.push_str(&format!("Warning: {}\n", w.as_str().unwrap_or("")));
    }
    if p["approval"]["by_review"] == true {
        s.push_str(&format!(
            "Approval: {} review the proposed change (merging it is the approval)\n",
            p["approval"]["approvers"]
        ));
    } else if p["approval"]["required"] == true {
        s.push_str(&format!(
            "Approval: required from {}\n",
            p["approval"]["approvers"]
        ));
    }
    match p["postcondition"].as_object() {
        Some(pc) => s.push_str(&format!(
            "Done when: {} (checked within {})\n",
            pc["check"].as_str().unwrap_or(""),
            pc["within"].as_str().unwrap_or("")
        )),
        None if p["executor"] == "toolbox" => {
            s.push_str("Done when: the child portal observes its postcondition\n")
        }
        None if p["executor"] == "change" => {
            s.push_str("Done when: its commit is seen on the base branch\n")
        }
        None => s.push_str("Done when: not verified (the action declares no expect)\n"),
    }
    let blockers = p["blockers"].as_array().cloned().unwrap_or_default();
    if blockers.is_empty() {
        s.push_str(&format!("Confirm within 10 minutes:\n  {next}\n"));
    } else {
        for b in blockers {
            s.push_str(&format!("Blocked: {}\n", b.as_str().unwrap_or("")));
        }
    }
    s
}

/// Where every run's record is kept, so `--verify <id>` finds it: `TOOLBOX_STATE_DIR/runs`, else
/// `$XDG_STATE_HOME/toolbox/runs`, else `~/.local/state/toolbox/runs`.
fn runs_dir() -> PathBuf {
    let base = std::env::var_os("TOOLBOX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_STATE_HOME").map(|d| PathBuf::from(d).join("toolbox")))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state/toolbox")))
        .unwrap_or_else(|| std::env::temp_dir().join("toolbox"));
    base.join("runs")
}

fn finish(o: &Options, run: &Run, code: u8) -> ExitCode {
    let doc = serde_json::to_value(run).unwrap_or_default();
    let text = serde_json::to_string_pretty(&doc).unwrap_or_default();
    let dir = runs_dir();
    if std::fs::create_dir_all(&dir).is_ok() {
        let _ = std::fs::write(dir.join(format!("{}.json", run.id)), &text);
    }
    if let Some(p) = &o.record
        && let Err(e) = std::fs::write(p, &text)
    {
        eprintln!("error: {}: {e}", p.display());
    }
    emit(o, &doc, || run_md(run));
    ExitCode::from(code)
}

fn run_md(run: &Run) -> String {
    let mut s = format!(
        "Run {} — {} on {}: {}\n",
        run.id, run.action, run.entity, run.state
    );
    if let Some(e) = &run.error {
        s.push_str(&format!("Error: {e}\n"));
    }
    if run.outputs.as_object().is_some_and(|o| !o.is_empty()) {
        s.push_str(&format!("Outputs: {}\n", run.outputs));
    }
    if let Some(v) = &run.verification {
        s.push_str(&format!("Verification: {v}\n"));
    }
    match run.state.as_str() {
        "proposed" => s.push_str(&format!(
            "The change is on branch {}; it is done when merged and observed. Check with: toolbox run --verify {}\n",
            run.outputs["branch"].as_str().unwrap_or("?"),
            run.id
        )),
        "dispatched" => s.push_str(&format!(
            "Dispatched; it is done when observed. Check with: toolbox run --verify {}\n",
            run.id
        )),
        "awaiting_approval" => {
            if let Some(n) = run.events.last().and_then(|e| e.note.as_ref()) {
                s.push_str(&format!("{n}\n"));
            }
        }
        _ => {}
    }
    s
}

fn verify_file(o: &Options, portal: &Path, file: &Path) -> ExitCode {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) => return error(o, 2, format!("{}: {e}", file.display())),
    };
    let mut run: Run = match serde_json::from_str(&text) {
        Ok(r) => r,
        Err(e) => return error(o, 2, format!("{}: not a run record: {e}", file.display())),
    };
    if run.is_terminal() {
        return finish(o, &run, exit_code(&run.state));
    }
    let (_, m, d) = match crate::report::live(portal, o.fixtures, &o.skip) {
        Ok(x) => x,
        Err(e) => return error(o, 2, e),
    };
    let all = federation::offered(&m, &d);
    let Some(off) = all.iter().find(|x| {
        x.action.name == run.action && x.entity.as_ref().is_none_or(|e| e == &run.entity)
    }) else {
        return error(
            o,
            2,
            format!("{} no longer has action {}", m.portal.name, run.action),
        );
    };
    let program = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("toolbox"));
    let exec = match off.executor(&program) {
        Ok(x) => x,
        Err(e) => return error(o, 2, e),
    };
    let mut obs = Local {
        portal: portal.to_path_buf(),
        fixtures: o.fixtures,
        skip: o.skip.clone(),
    };
    loop {
        lifecycle::recheck(
            &mut run,
            &exec,
            off.action.expect.as_ref(),
            &m.policies,
            &mut obs,
            portal,
        );
        if !o.wait || run.is_terminal() {
            break;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    finish(o, &run, exit_code(&run.state))
}

/// `toolbox run --url`: the same commands against a served portal's API (`/v1/actions`,
/// `/v1/runs`). The server previews, authorizes, holds approvals and runs; this only asks.
mod remote {
    use super::*;
    use crate::client::Target;

    struct Api<'a> {
        o: &'a Options,
        target: Target,
        rt: tokio::runtime::Runtime,
        client: reqwest::Client,
    }

    impl Api<'_> {
        /// The response body, or a problem's `code: detail` and an exit code (2 for requests that
        /// can't be right as asked, 1 otherwise).
        fn call(
            &self,
            method: &str,
            path: &str,
            body: Option<Value>,
        ) -> Result<Value, (u8, String)> {
            let url = format!("{}{path}", self.target.url);
            self.rt.block_on(async {
                let rb = match method {
                    "GET" => self.client.get(&url),
                    _ => self.client.post(&url).json(&body.unwrap_or(json!({}))),
                };
                let resp = self
                    .target
                    .request(rb)
                    .send()
                    .await
                    .map_err(|e| (2, format!("{}: {e}", self.target.url)))?;
                let status = resp.status();
                let v: Value = resp.json().await.unwrap_or_default();
                if status.is_success() {
                    return Ok(v);
                }
                let code = match status.as_u16() {
                    401 | 403 | 404 | 422 => 2,
                    _ => 1,
                };
                Err((
                    code,
                    format!(
                        "{} {}: {}",
                        status.as_u16(),
                        v["code"].as_str().unwrap_or(""),
                        v["detail"].as_str().unwrap_or("")
                    ),
                ))
            })
        }

        fn fail(&self, (code, msg): (u8, String)) -> ExitCode {
            error(self.o, code, msg)
        }

        /// Print a run as the checkout does, after following it with --wait.
        fn show(&self, mut run: Value) -> ExitCode {
            let settled = |r: &Value| {
                let st = r["state"].as_str().unwrap_or("");
                TERMINAL.contains(&st) || st == "awaiting_approval"
            };
            while self.o.wait && !settled(&run) {
                std::thread::sleep(Duration::from_secs(2));
                let id = run["id"].as_str().unwrap_or("").to_owned();
                let st = run["state"].as_str().unwrap_or("");
                let next = if ["proposed", "dispatched", "verifying", "executor_succeeded"]
                    .contains(&st)
                {
                    self.call("POST", &format!("/v1/runs/{id}/verify"), None)
                } else {
                    self.call("GET", &format!("/v1/runs/{id}"), None)
                };
                match next {
                    Ok(r) => run = r,
                    Err(e) => return self.fail(e),
                }
            }
            let code = exit_code(run["state"].as_str().unwrap_or(""));
            if let Some(p) = &self.o.record
                && let Err(e) =
                    std::fs::write(p, serde_json::to_string_pretty(&run).unwrap_or_default())
            {
                eprintln!("error: {}: {e}", p.display());
            }
            let url = self.target.url.clone();
            emit(self.o, &run, || {
                match serde_json::from_value::<Run>(run.clone()) {
                    Ok(r) => remote_md(&r, &run, &url),
                    Err(_) => format!("{run}\n"),
                }
            });
            ExitCode::from(code)
        }
    }

    use toolbox_exec::run::TERMINAL;

    fn remote_md(r: &Run, doc: &Value, url: &str) -> String {
        let mut s = run_md(r).replace(
            &format!("toolbox run --verify {}", r.id),
            &format!("toolbox run --url {url} --verify {} --wait", r.id),
        );
        if r.state == "awaiting_approval" {
            let who: Vec<&str> = doc["pending_approvers"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            s.push_str(&format!(
                "Waiting for {}. An approver signs in and runs: toolbox run --url {url} --approve {}\n",
                who.join(", "),
                r.id
            ));
        }
        s
    }

    /// `toolbox run --url`: the same requests as in a checkout, made to a served portal's API.
    pub fn run(o: &Options, url: &str) -> ExitCode {
        let api = Api {
            o,
            target: Target {
                url: url.to_owned(),
                token: o.token.clone(),
                as_user: o.as_user.clone(),
            },
            rt: tokio::runtime::Runtime::new().expect("tokio runtime"),
            client: reqwest::Client::new(),
        };
        let decided = o
            .approve
            .as_ref()
            .map(|id| (id, "approve"))
            .or(o.reject.as_ref().map(|id| (id, "reject")));
        let done = if o.runs {
            runs(&api)
        } else if let Some((id, decision)) = decided {
            let body = json!({"decision": decision, "note": o.note});
            api.call("POST", &format!("/v1/runs/{id}/approvals"), Some(body))
                .map(|run| api.show(run))
        } else if let Some(id) = &o.verify {
            api.call("POST", &format!("/v1/runs/{id}/verify"), None)
                .map(|run| api.show(run))
        } else if let Some(name) = &o.action {
            act(&api, url, name)
        } else {
            list(&api, url)
        };
        done.unwrap_or_else(|e| api.fail(e))
    }

    fn query(e: &str) -> String {
        e.replace(':', "%3A").replace('/', "%2F")
    }

    fn runs(api: &Api) -> Result<ExitCode, (u8, String)> {
        let o = api.o;
        let mut path = "/v1/runs".to_owned();
        if let Some(e) = &o.entity {
            path.push_str(&format!("?entity={}", query(e)));
        }
        let v = api.call("GET", &path, None)?;
        let runs: Vec<Value> = v["runs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| o.action.as_ref().is_none_or(|a| r["action"] == a.as_str()))
            .cloned()
            .collect();
        emit(o, &json!({"runs": runs}), || runs_md(&runs));
        Ok(ExitCode::SUCCESS)
    }

    fn list(api: &Api, url: &str) -> Result<ExitCode, (u8, String)> {
        let path = api.o.entity.as_ref().map_or("/v1/actions".to_owned(), |e| {
            format!("/v1/actions?entity={}", query(e))
        });
        let v = api.call("GET", &path, None)?;
        let actions = v["actions"].as_array().cloned().unwrap_or_default();
        emit(api.o, &v, || {
            let mut md = list_md(url, &actions, &format!(" --url {url}")) + &unreadable_md(&v);
            if let Some(k) = v["approval_key"].as_str() {
                md.push_str(&format!("\nChildren that require approval accept this portal's by pinning `{{portal: <name>, key: {k}}}` in `federation.acceptApprovalsFrom`.\n"));
            }
            md
        });
        Ok(ExitCode::SUCCESS)
    }

    /// Preview, or run with --confirm or --yes (which previews first, as the checkout does).
    fn act(api: &Api, url: &str, name: &str) -> Result<ExitCode, (u8, String)> {
        let o = api.o;
        let schema = api
            .call("GET", "/v1/actions", None)
            .ok()
            .and_then(|v| {
                v["actions"]
                    .as_array()?
                    .iter()
                    .find(|a| a["name"] == name)
                    .map(|a| a["inputs"].clone())
            })
            .unwrap_or(Value::Null);
        let inputs = parse_inputs(o, &schema).map_err(|e| (2, e))?;
        let body = |token: Option<&str>| json!({"entity": o.entity, "inputs": inputs, "confirm_token": token});
        let action = format!("/v1/actions/{name}");
        let token = match &o.confirm {
            Some(t) => t.clone(),
            None => {
                let mut p = api.call("POST", &format!("{action}/previews"), Some(body(None)))?;
                let entity = p["entity"].as_str().unwrap_or("").to_owned();
                let token = p["confirm_token"].as_str().unwrap_or("").to_owned();
                let next = format!(
                    "toolbox run {name} --url {url} --entity {entity}{} --confirm {token}",
                    input_args(&inputs)
                );
                let blocked = p["blockers"].as_array().is_some_and(|b| !b.is_empty());
                p["next"] = if blocked { Value::Null } else { json!(next) };
                if !o.yes || blocked {
                    let portal = p["portal"].as_str().unwrap_or(url).to_owned();
                    emit(o, &p, || preview_md(name, &entity, &portal, &p, &next));
                    return Ok(if blocked && o.yes {
                        ExitCode::from(2)
                    } else {
                        ExitCode::SUCCESS
                    });
                }
                token
            }
        };
        let run = api.call("POST", &format!("{action}/runs"), Some(body(Some(&token))))?;
        Ok(api.show(run))
    }
}
