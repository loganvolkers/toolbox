//! One preview, whoever asks (docs/design/federated-actions.md §6): `seikika run` in a checkout
//! and `seikika serve` behind the API and MCP both prepare a run here, so an action previews,
//! confirms and records the same way at every level. Authorization is the caller's: a repo portal
//! has none, a served portal checks the action's permission before it calls this.

use crate::check::{Checked, check, with_defaults};
use crate::executor::{Context, Executor, Preview};
use crate::federation::{self, Offered};
use crate::run::{self, Approval, Run, TokenError};
use seikika_derive::Derived;
use seikika_model::Manifest;
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Why a run could not be prepared.
#[derive(Debug)]
pub enum Refused {
    /// No such action, or not on that entity: the message lists what there is.
    Unknown(String),
    /// The inputs fail the action's schema or options.
    Invalid(Vec<String>),
    /// The executor could not preview.
    Executor(String),
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refused::Unknown(m) | Refused::Executor(m) => f.write_str(m),
            Refused::Invalid(e) => write!(f, "invalid inputs: {}", e.join("; ")),
        }
    }
}

/// A prepared run: everything a preview shows and a confirmation is checked against.
#[derive(Debug, Clone)]
pub struct Prepared {
    pub offered: Offered,
    pub entity: String,
    pub inputs: Map<String, Value>,
    pub checked: Checked,
    pub exec: Arc<dyn Executor>,
    pub ctx: Context,
    pub preview: Preview,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub approvers: Vec<Value>,
    pub version: String,
    pub basis: String,
    pub token: String,
    /// Review of the proposed change is the approval (the executor, or the child's, is `change`).
    pub reviewed: bool,
    /// The postcondition already held at preview (see `Run::held_before`).
    pub held_before: bool,
}

/// Who asks, and from where.
pub struct Request<'a> {
    pub action: &'a str,
    pub entity: Option<&'a str>,
    pub inputs: Map<String, Value>,
    pub actor: &'a str,
    /// The portal's repository root: where commands run.
    pub cwd: PathBuf,
    /// The `seikika` binary a local child portal is run with.
    pub program: &'a Path,
}

/// Resolve, check, and preview a run.
pub fn prepare(m: &Manifest, d: &Derived, req: Request<'_>) -> Result<Prepared, Refused> {
    let all = federation::offered(m, d);
    let (off, entity) =
        federation::resolve(&all, d, req.action, req.entity).map_err(Refused::Unknown)?;
    let off = off.clone();
    let action = &off.action;
    let inputs = with_defaults(action, &req.inputs);
    let checked = check(action, d, &m.policies, &entity, &inputs).map_err(Refused::Unknown)?;
    if !checked.errors.is_empty() {
        return Err(Refused::Invalid(checked.errors.clone()));
    }
    let exec = off
        .executor(req.program)
        .map_err(|e| Refused::Executor(format!("{}: {e}", action.name)))?;
    let ctx = Context {
        run_id: None,
        portal: m.portal.name.clone(),
        action: action.name.clone(),
        action_version: run::action_version(action),
        entity: checked.entity.clone(),
        inputs: inputs.clone(),
        actor: req.actor.to_owned(),
        approved_by: vec![],
        related: checked.related.clone(),
        grant: None,
        cwd: req.cwd,
    };
    let preview = match exec.preview(&ctx) {
        Ok(p) => p,
        Err(e) => Preview {
            blockers: vec![format!("the executor's preview failed: {e}")],
            ..Preview::default()
        },
    };
    let mut warnings = preview.warnings.clone();
    let held_before = action.expect.as_ref().is_some_and(|e| {
        run::postcondition_holds(e, &checked.entity, &inputs, &m.policies, &json!({}))
            .unwrap_or(false)
    });
    if let Some(e) = action.expect.as_ref().filter(|_| held_before) {
        warnings.push(format!(
            "the postcondition already holds ({}): running it may change nothing, and the run ends unverified",
            e.postcondition
        ));
    }
    let mut blockers = checked.blockers.clone();
    blockers.extend(preview.blockers.iter().cloned());
    let mut approvers = checked.approvers.clone();
    if let Some(c) = &preview.child {
        for a in c["approval"]["approvers"].as_array().into_iter().flatten() {
            if !approvers.contains(a) {
                approvers.push(a.clone());
            }
        }
        blockers.extend(
            c["blockers"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(|b| format!("child portal: {b}")),
        );
    }
    let basis = run::basis(&checked.entity, &checked.related);
    let version = ctx.action_version.clone();
    let token = run::confirm_token(
        &run::Confirmed {
            version: &version,
            inputs: &inputs,
            effects: &preview.effects,
            basis: &basis,
            actor: req.actor,
            entity: &entity,
        },
        run::now_secs(),
    );
    let reviewed = exec.reviewed() || off.child.as_ref().is_some_and(|c| c.reviewed);
    Ok(Prepared {
        offered: off,
        entity,
        inputs,
        checked,
        exec,
        ctx,
        preview,
        warnings,
        blockers,
        approvers,
        version,
        basis,
        token,
        reviewed,
        held_before,
    })
}

impl Prepared {
    /// The preview as the API, MCP and `seikika run -o json` return it.
    pub fn doc(&self, portal: &str) -> Value {
        let a = &self.offered.action;
        json!({
            "action": a.name, "action_version": self.version, "entity": self.entity, "portal": portal, "inputs": self.inputs,
            "effects": self.preview.effects, "warnings": self.warnings, "blockers": self.blockers,
            "approval": {"required": !self.approvers.is_empty(), "approvers": self.approvers,
                "by_review": self.reviewed && !self.approvers.is_empty()},
            "related": self.checked.related,
            "executor": self.exec.kind(), "executor_preview": self.preview.executed,
            "postcondition": a.expect.as_ref().map(|e| json!({"check": e.postcondition, "within": e.within})),
            "child": self.preview.child,
            "confirm_token": if self.blockers.is_empty() { Value::String(self.token.clone()) } else { Value::Null },
            "expires_in_secs": run::TOKEN_TTL_SECS,
        })
    }

    /// Whether `token` confirms this preview.
    pub fn confirm(&self, token: &str) -> Result<(), TokenError> {
        run::check_token(
            token,
            &run::Confirmed {
                version: &self.version,
                inputs: &self.inputs,
                effects: &self.preview.effects,
                basis: &self.basis,
                actor: &self.ctx.actor,
                entity: &self.entity,
            },
        )
    }

    /// A new run, previewed and confirmed by `by`.
    pub fn new_run(&self, id: String, by: &str, via: String, portal: &str, snapshot: &str) -> Run {
        let mut r = Run {
            id,
            action: self.offered.action.name.clone(),
            action_version: self.version.clone(),
            entity: self.entity.clone(),
            inputs: self.inputs.clone(),
            requested_by: by.to_owned(),
            via,
            portal: portal.to_owned(),
            executor: self.exec.kind().into(),
            snapshot: snapshot.to_owned(),
            effects: self.preview.effects.clone(),
            approval: Approval {
                required: !self.approvers.is_empty(),
                approvers: self.approvers.clone(),
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
            held_before: self.held_before,
        };
        r.set("previewed", Some(by), None);
        r.set("confirmed", Some(by), None);
        r
    }

    /// The context an executor runs with, for `run`.
    pub fn context(&self, run: &Run) -> Context {
        let mut ctx = self.ctx.clone();
        ctx.run_id = Some(run.id.clone());
        ctx.approved_by = run.approval.approved_by.clone();
        ctx
    }

    /// For a federated run with recorded approvals: the grant that carries them to the child,
    /// signed with `seed` (hex), naming the child's action, version and effects as its preview
    /// reported them. `None` when there is nothing to carry.
    pub fn grant(&self, run: &Run, seed: &str) -> Result<Option<String>, String> {
        let (Some(c), Some(child)) = (&self.offered.child, &self.preview.child) else {
            return Ok(None);
        };
        if run.approval.approved_by.is_empty() {
            return Ok(None);
        }
        let g = crate::grant::Grant {
            parent: run.portal.clone(),
            parent_run: run.id.clone(),
            action: c.action.clone(),
            action_version: child["action_version"].as_str().unwrap_or("").to_owned(),
            entity: child["entity"].as_str().unwrap_or("").to_owned(),
            inputs: child["inputs"].as_object().cloned().unwrap_or_default(),
            effects: crate::grant::effects_hash(&self.preview.effects),
            requested_by: run.requested_by.clone(),
            approvals: run
                .approval
                .satisfied
                .iter()
                .map(|x| format!("{}={}", x.approver, x.by))
                .collect(),
            expires: run::now_secs() + crate::grant::GRANT_TTL_SECS,
        };
        crate::grant::sign(&g, seed).map(Some)
    }
}
