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
    /// Re-check a `proposed` or `dispatched` run by its id (every run is saved under
    /// `$XDG_STATE_HOME/toolbox/runs/`).
    #[arg(long, value_name = "RUN_ID", conflicts_with_all = ["preview", "confirm", "yes"])]
    pub verify: Option<String>,
    /// With --verify: keep checking until the postcondition holds or the run's deadline passes.
    #[arg(long, requires = "verify")]
    pub wait: bool,
    /// Also write the run record to this file.
    #[arg(long)]
    pub record: Option<PathBuf>,
    /// Who is asking, recorded on the run (default: `user:$USER`).
    #[arg(long = "as")]
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

fn list(o: &Options, portal: &Path, all: &[Offered], d: &toolbox_derive::Derived) -> ExitCode {
    let shown = federation::on_entity(all, d, o.entity.as_deref());
    let doc = json!({"portal": portal, "actions": shown.iter().map(|x| federation::describe(x, d)).collect::<Vec<_>>()});
    emit(o, &doc, || {
        let mut s = format!("{} action(s) in {}\n", shown.len(), portal.display());
        for x in &shown {
            let v = federation::describe(x, d);
            let ents: Vec<&str> = v["entities"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            s.push_str(&format!(
                "\n- {} ({} on {}): {}\n  inputs: {}\n  executor: {}; verified: {}{}\n",
                x.action.name,
                x.action.kind,
                if ents.is_empty() {
                    "no entity yet".into()
                } else {
                    ents.join(", ")
                },
                x.action.description,
                federation::inputs_line(&x.action),
                v["executor"].as_str().unwrap_or(""),
                if v["verified"] == true { "yes" } else { "no" },
                v["approval"]
                    .as_str()
                    .map(|a| format!("; approval: {a}"))
                    .unwrap_or_default(),
            ));
        }
        s.push_str("\nPreview one: toolbox run <action> [--entity ID] --input name=value\n");
        s
    });
    ExitCode::SUCCESS
}

pub fn run(o: &Options) -> ExitCode {
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
    let Some(name) = &o.action else {
        let all = federation::offered(&m, &d);
        return list(o, &portal, &all, &d);
    };
    let all = federation::offered(&m, &d);
    let schema = federation::resolve(&all, &d, name, o.entity.as_deref())
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
        cwd: toolbox_exec::executor::portal_repo_dir(&portal_dir(&portal)),
        program: &program,
    };
    let p = match prepare(&m, &d, req) {
        Ok(p) => p,
        Err(Refused::Invalid(errors)) => {
            let action = federation::resolve(&all, &d, name, o.entity.as_deref())
                .ok()
                .map(|(x, _)| x.action.clone());
            let line = action
                .as_ref()
                .map(federation::inputs_line)
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
            return ExitCode::from(2);
        }
        Err(e @ Refused::Executor(_)) => return error(o, 1, e.to_string()),
        Err(e) => return error(o, 2, e.to_string()),
    };
    let entity = p.entity.clone();
    let portal_arg = portal.display().to_string();
    let input_args: String = p
        .inputs
        .iter()
        .map(|(k, v)| {
            let s = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
            format!(" --input {k}={}", shell_quote(&s))
        })
        .collect();
    let next = format!(
        "toolbox run {name} --portal {portal_arg} --entity {entity}{input_args} --confirm {}",
        p.token
    );
    let mut preview_doc = p.doc(&m.portal.name);
    preview_doc["accepts_approvals_from"] = json!(
        m.federation
            .as_ref()
            .map(|f| f.accept_approvals_from.clone())
            .unwrap_or_default()
    );
    preview_doc["next"] = if p.blockers.is_empty() {
        Value::String(next.clone())
    } else {
        Value::Null
    };
    if o.confirm.is_none() && !o.yes {
        emit(o, &preview_doc, || {
            preview_md(name, &entity, &m.portal.name, &preview_doc, &next)
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
    if !p.approvers.is_empty() {
        let mut refused = None;
        if let Some(g) = o.grant.as_deref().filter(|g| !g.is_empty()) {
            match granted(o, &m, &p, &requested_by, g) {
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
        if !pending.is_empty() && !p.reviewed {
            let why = refused.map_or_else(
                || "a repo portal holds no approvals, so run it through a portal that does (a parent with `toolbox serve`), or make it a `change` executor so review is the approval".to_owned(),
                |e| format!("the parent's approvals were refused: {e}"),
            );
            run.set(
                "awaiting_approval",
                None,
                Some(format!("needs approval by {}: {why}", pending.join(", "))),
            );
            return finish(o, &run, 3);
        }
    }
    let ctx = p.context(&run);
    let mut obs = Local {
        portal: portal.clone(),
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
    let code = if matches!(run.state.as_str(), "failed" | "timed_out") {
        1
    } else {
        0
    };
    finish(o, &run, code)
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

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./:@=,".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

fn preview_md(name: &str, entity: &str, portal: &str, p: &Value, next: &str) -> String {
    let mut s = format!("Preview: {name} on {entity} (portal {portal})\n");
    let fx = p["effects"].as_array().cloned().unwrap_or_default();
    s.push_str(&format!("Effects ({}):\n", fx.len()));
    for e in &fx {
        s.push_str(&format!("  - {}\n", effect_line(e)));
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
        return finish(
            o,
            &run,
            if matches!(run.state.as_str(), "failed" | "timed_out") {
                1
            } else {
                0
            },
        );
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
    let code = if matches!(run.state.as_str(), "failed" | "timed_out") {
        1
    } else {
        0
    };
    finish(o, &run, code)
}
