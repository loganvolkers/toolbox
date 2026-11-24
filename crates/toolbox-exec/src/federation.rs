//! Federated actions (docs/design/federated-actions.md §5): **actions travel up as data, runs go
//! down as dispatches.** A parent portal names the kind whose entities are its child portals
//! (`federation.portals`); each such entity carries the actions the child declares, read from the
//! child's manifest by the workspace connector's `portal` records. The parent offers them on the
//! child's entity, checks its own permission and approvals, and dispatches the run to the child,
//! which previews, runs and verifies it with its own executor and credentials. The parent never
//! runs a child's code with the parent's credentials.

use crate::executor::Executor;
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};
use toolbox_derive::Derived;
use toolbox_model::Manifest;
use toolbox_model::manifest::Action;

/// Where a federated action really lives.
#[derive(Debug, Clone)]
pub struct Child {
    /// The child portal's name.
    pub portal: String,
    /// The child portal's directory (from the record's `path`).
    pub path: PathBuf,
    /// The action's name in the child.
    pub action: String,
    /// How the parent reaches the child (`{type: local}` or `{type: github-workflow, …}`).
    pub runner: Value,
    /// The child's approval rule, evaluated by the child when it previews.
    pub approval: Option<String>,
    /// The child's executor type (`change` runs are approved by review in the child's repo).
    pub executor: String,
}

/// An action as this portal offers it.
#[derive(Debug, Clone)]
pub struct Offered {
    pub action: Action,
    /// For a child's action: the child portal entity it is offered on.
    pub entity: Option<String>,
    pub child: Option<Child>,
}

impl Offered {
    /// The executor that carries out a run of this action. `program` is the `toolbox` binary a
    /// local child is run with.
    pub fn executor(&self, program: &Path) -> Result<Executor, String> {
        let Some(c) = &self.child else {
            return Executor::parse(&self.action.executor);
        };
        match c
            .runner
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("local")
        {
            "local" => Ok(Executor::Toolbox {
                portal: c.path.clone(),
                action: c.action.clone(),
                entity: String::new(),
                program: program.to_path_buf(),
            }),
            "github-workflow" => {
                let mut w = crate::github::Workflow::parse(&c.runner)?;
                w.federated = Some(c.action.clone());
                Ok(Executor::GithubWorkflow(w))
            }
            other => Err(format!(
                "child portal {}: unknown runner {other} (local, github-workflow)",
                c.portal
            )),
        }
    }
}

/// A child action's definition, as the parent offers it: on the child portal's kind, with the
/// parent's permission, and without `enumFrom` (its options are the child's to compute; the
/// child validates them when it previews).
fn as_parent_action(def: &Value, kind: &str, permission: &str, portal: &str) -> Option<Action> {
    let name = def.get("name")?.as_str()?;
    let mut inputs = def
        .get("inputs")
        .cloned()
        .unwrap_or_else(|| json!({"type": "object"}));
    if let Some(props) = inputs.get_mut("properties").and_then(Value::as_object_mut) {
        for spec in props.values_mut() {
            if let Some(o) = spec.as_object_mut() {
                o.remove("enumFrom");
            }
        }
    }
    let desc = def.get("description").and_then(Value::as_str).unwrap_or("");
    serde_json::from_value(json!({
        "name": name,
        "kind": kind,
        "scope": def.get("scope").cloned().unwrap_or(json!("day2")),
        "description": format!("{desc} (from portal {portal})"),
        "inputs": inputs,
        "permission": permission,
        "executor": {"type": "toolbox"},
        "unverified": true,
        "annotations": def.get("annotations").cloned(),
    }))
    .ok()
}

/// Every action this portal offers: its own, then each child portal's.
pub fn offered(m: &Manifest, d: &Derived) -> Vec<Offered> {
    let mut out: Vec<Offered> = m
        .actions
        .iter()
        .map(|a| Offered {
            action: a.clone(),
            entity: None,
            child: None,
        })
        .collect();
    let Some(fed) = &m.federation else {
        return out;
    };
    let Some(kind) = &fed.portals else {
        return out;
    };
    let permission = fed.permission.as_deref().unwrap_or("can_run_actions");
    for (id, e) in d.entities.iter().filter(|(_, e)| &e.kind == kind) {
        let portal = e
            .fields
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(id)
            .to_owned();
        // `dir` is where the connector found the child on this machine; `path` is relative to the
        // connector's root.
        let path = PathBuf::from(
            e.fields
                .get("dir")
                .and_then(Value::as_str)
                .filter(|d| !d.is_empty())
                .or_else(|| e.fields.get("path").and_then(Value::as_str))
                .unwrap_or("."),
        );
        let runner = e
            .fields
            .get("runner")
            .cloned()
            .filter(|r| !r.is_null())
            .unwrap_or_else(|| json!({"type": "local"}));
        for def in e
            .fields
            .get("actions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(a) = as_parent_action(def, kind, permission, &portal) {
                out.push(Offered {
                    child: Some(Child {
                        portal: portal.clone(),
                        path: path.clone(),
                        action: a.name.clone(),
                        runner: runner.clone(),
                        approval: def
                            .get("approval")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        executor: def
                            .get("executor")
                            .and_then(Value::as_str)
                            .unwrap_or("command")
                            .to_owned(),
                    }),
                    action: a,
                    entity: Some(id.clone()),
                });
            }
        }
    }
    out
}

/// The actions offered on one entity (or every action when `entity` is `None`).
pub fn on_entity<'a>(all: &'a [Offered], d: &Derived, entity: Option<&str>) -> Vec<&'a Offered> {
    all.iter()
        .filter(|o| match entity {
            None => true,
            Some(id) => match &o.entity {
                Some(e) => e == id,
                None => d.entities.get(id).is_some_and(|e| e.kind == o.action.kind),
            },
        })
        .collect()
}

/// Pick the action `name` for `entity`, or the only entity it can apply to.
pub fn resolve<'a>(
    all: &'a [Offered],
    d: &Derived,
    name: &str,
    entity: Option<&str>,
) -> Result<(&'a Offered, String), String> {
    let named: Vec<&Offered> = all.iter().filter(|o| o.action.name == name).collect();
    if named.is_empty() {
        let mut names: Vec<&str> = all.iter().map(|o| o.action.name.as_str()).collect();
        names.dedup();
        return Err(format!("no action {name} (actions: {})", names.join(", ")));
    }
    let targets = |o: &Offered| -> Vec<String> {
        match &o.entity {
            Some(e) => vec![e.clone()],
            None => d
                .entities
                .iter()
                .filter(|(_, e)| e.kind == o.action.kind)
                .map(|(id, _)| id.clone())
                .collect(),
        }
    };
    if let Some(id) = entity {
        return named
            .iter()
            .find(|o| targets(o).iter().any(|t| t == id))
            .map(|o| (*o, id.to_owned()))
            .ok_or_else(|| {
                let all: Vec<String> = named.iter().flat_map(|o| targets(o)).collect();
                format!(
                    "{name} is not offered on {id} (it is on: {})",
                    all.join(", ")
                )
            });
    }
    let all_targets: Vec<(&Offered, String)> = named
        .iter()
        .flat_map(|o| targets(o).into_iter().map(move |t| (*o, t)))
        .collect();
    match all_targets.as_slice() {
        [(o, t)] => Ok((o, t.clone())),
        [] => Err(format!(
            "{name} applies to {}, and there is none",
            named[0].action.kind
        )),
        many => Err(format!(
            "{name} applies to {} entities; pass --entity (one of: {})",
            many.len(),
            many.iter()
                .map(|(_, t)| t.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// A one-line summary of an action's inputs: `bump (patch|minor|major, required)`.
pub fn inputs_line(a: &Action) -> String {
    let req: Vec<&str> = a
        .inputs
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let props = a
        .inputs
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if props.is_empty() {
        return "none".into();
    }
    props
        .iter()
        .map(|(k, spec)| {
            let mut bits: Vec<String> = Vec::new();
            if let Some(e) = spec.get("enum").and_then(Value::as_array) {
                bits.push(
                    e.iter()
                        .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
                        .collect::<Vec<_>>()
                        .join("|"),
                );
            } else {
                bits.push(
                    spec.get("type")
                        .and_then(Value::as_str)
                        .unwrap_or("string")
                        .to_owned(),
                );
            }
            if req.contains(&k.as_str()) {
                bits.push("required".into());
            }
            if let Some(d) = spec.get("default") {
                bits.push(format!("default {d}"));
            }
            format!("{k} ({})", bits.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The JSON listing of one offered action, for `toolbox run` with no action and `list_actions`.
pub fn describe(o: &Offered, d: &Derived) -> Value {
    let a = &o.action;
    let entities: Vec<String> = match &o.entity {
        Some(e) => vec![e.clone()],
        None => d
            .entities
            .iter()
            .filter(|(_, e)| e.kind == a.kind)
            .map(|(id, _)| id.clone())
            .collect(),
    };
    let mut v = Map::new();
    v.insert("name".into(), a.name.clone().into());
    v.insert("kind".into(), a.kind.clone().into());
    v.insert("description".into(), a.description.clone().into());
    v.insert("entities".into(), json!(entities));
    v.insert("inputs".into(), a.inputs.clone());
    v.insert(
        "executor".into(),
        match &o.child {
            Some(_) => json!("toolbox"),
            None => a.executor.get("type").cloned().unwrap_or(json!("command")),
        },
    );
    v.insert(
        "verified".into(),
        json!(a.expect.is_some() || o.child.is_some()),
    );
    v.insert("approval".into(), json!(a.approval));
    if let Some(c) = &o.child {
        v.insert("approval".into(), json!(c.approval));
        v.insert(
            "portal".into(),
            json!({"name": c.portal, "path": c.path, "runner": c.runner, "executor": c.executor}),
        );
    }
    Value::Object(v)
}
