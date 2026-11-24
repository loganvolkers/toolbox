//! `toolbox run`: self-service actions without a server (docs/design/federated-actions.md §6). In
//! a repo portal the action is previewed, confirmed, executed and verified from the checkout,
//! with the same definition and lifecycle the server uses. With no action it lists what the portal
//! offers, including its child portals' actions (federation).
//!
//! A repo portal has no authorization (docs/design/repo-portals.md §2): whoever has the checkout
//! and its credentials runs. An action whose approval rule names approvers therefore runs here
//! only as a `change` (the review is the approval), or dispatched by a parent portal that
//! recorded the approval and that this portal trusts (`federation.acceptApprovalsFrom`).

use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;
use toolbox_exec::check::{check, with_defaults};
use toolbox_exec::executor::{Context, Executor};
use toolbox_exec::federation::{self, Offered};
use toolbox_exec::run::{self as lifecycle, Approval, Observer, Run};
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
    /// Re-check a `proposed` or `dispatched` run from its record file (written by --record),
    /// and update the file.
    #[arg(long, conflicts_with_all = ["preview", "confirm", "yes"])]
    pub verify: Option<PathBuf>,
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
    /// Set by a parent portal: an approval it recorded, `<approver>=<user>` (repeatable).
    #[arg(long, hide = true)]
    pub approved_by: Vec<String>,
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
                if let Some((remote, branch)) = base.split_once('/') {
                    let dir = toolbox_exec::executor::portal_repo_dir(&portal_dir(&self.portal));
                    let _ = std::process::Command::new("git")
                        .args(["fetch", "-q", remote, branch])
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
    if let Some(f) = &o.verify {
        return verify_file(o, &portal, f);
    }
    let (_, m, d) = match crate::report::live(&portal, o.fixtures, &o.skip) {
        Ok(x) => x,
        Err(e) => return error(o, 2, e),
    };
    let all = federation::offered(&m, &d);
    let Some(name) = &o.action else {
        return list(o, &portal, &all, &d);
    };
    let (off, entity) = match federation::resolve(&all, &d, name, o.entity.as_deref()) {
        Ok(x) => x,
        Err(e) => return error(o, 2, e),
    };
    let action = &off.action;
    let inputs = match parse_inputs(o, &action.inputs) {
        Ok(i) => with_defaults(action, &i),
        Err(e) => return error(o, 2, e),
    };
    let checked = match check(action, &d, &m.policies, &entity, &inputs) {
        Ok(c) => c,
        Err(e) => return error(o, 2, e),
    };
    if !checked.errors.is_empty() {
        let doc = json!({"action": name, "entity": entity, "errors": checked.errors,
            "inputs": action.inputs});
        emit(o, &doc, || {
            format!(
                "Invalid inputs for {name} on {entity}:\n{}\nInputs: {}\n",
                checked
                    .errors
                    .iter()
                    .map(|e| format!("  - {e}\n"))
                    .collect::<String>(),
                federation::inputs_line(action)
            )
        });
        return ExitCode::from(2);
    }
    let program = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("toolbox"));
    let exec = match off.executor(&program) {
        Ok(x) => x,
        Err(e) => return error(o, 2, format!("{name}: {e}")),
    };
    let requested_by = actor(o);
    let run_id = lifecycle::new_id("run");
    let mut ctx = Context {
        run_id: None,
        portal: m.portal.name.clone(),
        action: name.clone(),
        action_version: lifecycle::action_version(action),
        entity: checked.entity.clone(),
        inputs: inputs.clone(),
        actor: requested_by.clone(),
        related: checked.related.clone(),
        approvals: vec![],
        cwd: toolbox_exec::executor::portal_repo_dir(&portal_dir(&portal)),
    };
    let pv = match exec.preview(&ctx) {
        Ok(p) => p,
        Err(e) => return error(o, 1, format!("{name}: {e}")),
    };
    let mut warnings = pv.warnings.clone();
    if let Some(e) = &action.expect
        && lifecycle::postcondition_holds(e, &checked.entity, &inputs, &m.policies, &json!({}))
            .unwrap_or(false)
    {
        warnings.push(format!(
            "the postcondition already holds ({}): running it may change nothing",
            e.postcondition
        ));
    }
    let mut blockers = checked.blockers.clone();
    blockers.extend(pv.blockers.iter().cloned());
    let mut approvers = checked.approvers.clone();
    if let Some(c) = &pv.child {
        approvers.extend(
            c["approval"]["approvers"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
        );
    }
    let basis = lifecycle::basis(&checked.entity, &checked.related);
    let version = ctx.action_version.clone();
    let token = lifecycle::confirm_token(
        &version,
        &inputs,
        &pv.effects,
        &basis,
        lifecycle::now_secs(),
    );
    let portal_arg = portal.display().to_string();
    let input_args: String = inputs
        .iter()
        .map(|(k, v)| {
            let s = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
            format!(" --input {k}={}", shell_quote(&s))
        })
        .collect();
    let next = format!(
        "toolbox run {name} --portal {portal_arg} --entity {entity}{input_args} --confirm {token}"
    );
    let preview_doc = json!({"action": name, "entity": entity, "portal": m.portal.name, "inputs": inputs,
        "effects": pv.effects, "warnings": warnings, "blockers": blockers,
        "approval": {"required": !approvers.is_empty(), "approvers": approvers},
        "executor": exec.kind(), "executor_preview": pv.executed,
        "postcondition": action.expect.as_ref().map(|e| json!({"check": e.postcondition, "within": e.within})),
        "child": pv.child, "confirm_token": token, "expires_in_secs": lifecycle::TOKEN_TTL_SECS,
        "next": if blockers.is_empty() { Value::String(next.clone()) } else { Value::Null }});
    if o.confirm.is_none() && !o.yes {
        emit(o, &preview_doc, || {
            preview_md(name, &entity, &m.portal.name, &preview_doc, &next)
        });
        return ExitCode::SUCCESS;
    }
    if let Some(t) = &o.confirm
        && let Err(e) = lifecycle::check_token(t, &version, &inputs, &pv.effects, &basis)
    {
        return error(o, 1, e.to_string());
    }
    if !blockers.is_empty() {
        return error(o, 1, format!("{name} is blocked: {}", blockers.join("; ")));
    }
    let mut run = Run {
        id: run_id.clone(),
        action: name.clone(),
        action_version: version,
        entity: entity.clone(),
        inputs: inputs.clone(),
        requested_by: requested_by.clone(),
        via: o
            .parent_run
            .as_ref()
            .map_or_else(|| "cli".into(), |p| format!("parent:{p}")),
        portal: m.portal.name.clone(),
        executor: exec.kind().into(),
        snapshot: d.snapshot.clone(),
        effects: pv.effects.clone(),
        approval: Approval {
            required: !approvers.is_empty(),
            approvers: approvers.clone(),
            ..Approval::default()
        },
        state: "previewed".into(),
        events: vec![],
        outputs: json!({}),
        error: None,
        verification: None,
        child: None,
        progress: vec![],
        deadline: None,
    };
    run.set("previewed", Some(&requested_by), None);
    run.set("confirmed", Some(&requested_by), None);
    if !approvers.is_empty() {
        let trusted = o.parent_run.as_ref().is_some_and(|p| {
            let parent = p.split('/').next().unwrap_or("");
            m.federation
                .as_ref()
                .is_some_and(|f| f.accept_approvals_from.iter().any(|a| a == parent))
        });
        if trusted {
            for a in &o.approved_by {
                let (which, who) = a.split_once('=').unwrap_or((a, a));
                let _ = run.approve(who, |g| g == which);
            }
        }
        let pending = run.pending_approvers();
        let is_change = matches!(exec, Executor::Change { .. })
            || off.child.as_ref().is_some_and(|c| c.executor == "change");
        if !pending.is_empty() && !is_change {
            run.set(
                "awaiting_approval",
                None,
                Some(format!(
                    "needs approval by {}: a repo portal holds no approvals, so run it through a portal that does (a parent with `toolbox serve`), or make it a `change` executor so review is the approval",
                    pending.join(", ")
                )),
            );
            return finish(o, &run, 3);
        }
    }
    ctx.run_id = Some(run_id);
    ctx.approvals = run.approval.approved_by.clone();
    let mut obs = Local {
        portal: portal.clone(),
        fixtures: o.fixtures,
        skip: o.skip.clone(),
    };
    lifecycle::execute(
        &mut run,
        &exec,
        &ctx,
        action.expect.as_ref(),
        &m.policies,
        &mut obs,
        Duration::from_secs(2),
    );
    let code = if run.state == "failed" { 1 } else { 0 };
    finish(o, &run, code)
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
    if p["approval"]["required"] == true {
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

fn finish(o: &Options, run: &Run, code: u8) -> ExitCode {
    let doc = serde_json::to_value(run).unwrap_or_default();
    if let Some(p) = &o.record
        && let Err(e) = std::fs::write(p, serde_json::to_string_pretty(&doc).unwrap_or_default())
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
            "The change is on branch {}; it is done when merged and observed. Check with: toolbox run --verify <record file>\n",
            run.outputs["branch"].as_str().unwrap_or("?")
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
        return finish(o, &run, if run.state == "failed" { 1 } else { 0 });
    }
    let (_, m, d) = match crate::report::live(portal, o.fixtures, &o.skip) {
        Ok(x) => x,
        Err(e) => return error(o, 2, e),
    };
    let all = federation::offered(&m, &d);
    let Some(off) = all.iter().find(|x| x.action.name == run.action) else {
        return error(
            o,
            2,
            format!("{} no longer has action {}", m.portal.name, run.action),
        );
    };
    if let (Some(c), Some(child)) = (&off.child, run.child.clone()) {
        return verify_child(o, file, run, &c.path, child);
    }
    let Some(expect) = &off.action.expect else {
        run.set("unverified", None, None);
        return finish(o, &run, 0);
    };
    let mut obs = Local {
        portal: portal.to_path_buf(),
        fixtures: o.fixtures,
        skip: o.skip.clone(),
    };
    lifecycle::verify(
        &mut run,
        expect,
        &m.policies,
        &mut obs,
        Duration::from_secs(5),
        o.wait,
    );
    if !o.wait && run.state == "verifying" {
        if run.deadline.is_some_and(|dl| lifecycle::now_secs() >= dl) {
            run.fail(format!(
                "postcondition not met within {}: {}",
                expect.within, expect.postcondition
            ));
        } else {
            run.state = "proposed".into();
        }
    }
    let p = o.record.clone().unwrap_or_else(|| file.to_path_buf());
    let o2 = Options {
        record: Some(p),
        ..clone_opts(o)
    };
    let code = if run.state == "failed" { 1 } else { 0 };
    finish(&o2, &run, code)
}

/// A federated run is verified by the child portal, from the child's own run record.
fn verify_child(o: &Options, file: &Path, mut run: Run, dir: &Path, child: Value) -> ExitCode {
    let tmp = std::env::temp_dir().join(format!("toolbox-child-{}.json", run.id));
    if let Err(e) = std::fs::write(&tmp, child.to_string()) {
        return error(o, 1, e.to_string());
    }
    let program = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("toolbox"));
    let mut cmd = std::process::Command::new(program);
    cmd.arg("run")
        .arg("--verify")
        .arg(&tmp)
        .arg("--portal")
        .arg(dir)
        .args(["-o", "json"])
        .current_dir(toolbox_exec::executor::portal_repo_dir(dir));
    if o.wait {
        cmd.arg("--wait");
    }
    let out = cmd.output();
    let _ = std::fs::remove_file(&tmp);
    let v: Value = match out {
        Ok(out) => match serde_json::from_slice(&out.stdout) {
            Ok(v) => v,
            Err(_) => {
                return error(
                    o,
                    1,
                    format!(
                        "child portal answered no run record: {}",
                        String::from_utf8_lossy(&out.stderr).trim()
                    ),
                );
            }
        },
        Err(e) => return error(o, 1, e.to_string()),
    };
    let state = v["state"].as_str().unwrap_or("failed").to_owned();
    match state.as_str() {
        "succeeded" | "unverified" => {
            run.verification = Some(json!({"by": "child portal", "child_run": v["id"],
                "child": v["verification"]}));
            run.set(&state, None, None);
        }
        "failed" => run.fail(format!(
            "child run failed: {}",
            v["error"].as_str().unwrap_or("")
        )),
        _ => {}
    }
    run.child = Some(v);
    let o2 = Options {
        record: Some(o.record.clone().unwrap_or_else(|| file.to_path_buf())),
        ..clone_opts(o)
    };
    finish(&o2, &run, if run.state == "failed" { 1 } else { 0 })
}

fn clone_opts(o: &Options) -> Options {
    Options {
        action: o.action.clone(),
        portal: o.portal.clone(),
        entity: o.entity.clone(),
        input: o.input.clone(),
        inputs: o.inputs.clone(),
        preview: o.preview,
        confirm: o.confirm.clone(),
        yes: o.yes,
        verify: o.verify.clone(),
        wait: o.wait,
        record: o.record.clone(),
        as_user: o.as_user.clone(),
        skip: o.skip.clone(),
        fixtures: o.fixtures,
        output: o.output.clone(),
        parent_run: o.parent_run.clone(),
        approved_by: o.approved_by.clone(),
    }
}
