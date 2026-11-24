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
    /// The child portal's name, as it declares it: a label, not an identity.
    pub portal: String,
    /// Where the workspace connector found the child (`repo`, `team/repo`): its identity, which
    /// the parent's runner allowlists match.
    pub key: String,
    /// The child portal's directory (from the record's `path`).
    pub path: PathBuf,
    /// The action's name in the child.
    pub action: String,
    /// How the child lets a parent reach it (`{type: local}` or `{type: github-workflow, …}`);
    /// null when it declares nothing, which a checkout runs locally and a server refuses.
    pub runner: Value,
    /// The child's approval rule, evaluated by the child when it previews.
    pub approval: Option<String>,
    /// The child's executor type.
    pub executor: String,
    /// Whether the child's runs are approved by review in its repo: a `change` that proposes a
    /// branch. A `change` with `land: push` lands once approved, so the parent's approvals apply.
    pub reviewed: bool,
    /// The parent's `federation.runners`: the credentials and allowlists it dispatches with.
    pub runners: Value,
}

/// `*` matches any run of characters; everything else matches itself.
pub fn glob(pattern: &str, s: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == s,
        Some((head, rest)) => {
            s.starts_with(head)
                && (0..=s.len() - head.len())
                    .any(|i| s.is_char_boundary(head.len() + i) && glob(rest, &s[head.len() + i..]))
        }
    }
}

fn listed(list: &Value, key: &str, s: &str) -> bool {
    list.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .any(|p| glob(p, s))
}

impl Child {
    /// Whether a server may run this child's actions: the child declares how it may be reached
    /// (`federation.runner`), and the parent's `federation.runners` admits it. A checkout runs a
    /// child as the person who ran `toolbox run`, so it needs neither.
    pub fn served(&self) -> Result<(), (&'static str, String)> {
        let reach = format!(
            "its actions run only from a checkout: toolbox run {} --portal {}",
            self.action,
            self.path.display()
        );
        match self.runner.get("type").and_then(Value::as_str) {
            None => Err((
                "no-runner",
                format!(
                    "portal {} declares no federation.runner, so {reach}",
                    self.portal
                ),
            )),
            Some("local") if !listed(&self.runners["local"], "portals", &self.key) => Err((
                "runner-not-allowed",
                format!(
                    "this server runs child portals in its own process only when federation.runners.local.portals lists them, and {} is not listed; {reach}",
                    self.key
                ),
            )),
            _ => Ok(()),
        }
    }
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
                env: c.runners["local"]["env"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect(),
            }),
            "github-workflow" => {
                // The child names its workflow; the parent supplies the endpoint and the token, so
                // a child can't make the parent send its credentials anywhere.
                if c.runner.get("api").is_some() || c.runner.get("token").is_some() {
                    return Err(format!(
                        "child portal {} names an api or token in federation.runner; the parent's federation.runners.github-workflow supplies those",
                        c.portal
                    ));
                }
                let own = &c.runners["github-workflow"];
                if own.is_null() {
                    return Err(format!(
                        "child portal {} runs through a GitHub workflow, and this portal declares no federation.runners.github-workflow",
                        c.portal
                    ));
                }
                let repo = c.runner.get("repo").and_then(Value::as_str).unwrap_or("");
                if !listed(own, "repos", repo) {
                    return Err(format!(
                        "child portal {} dispatches to {repo}, which federation.runners.github-workflow.repos does not list",
                        c.portal
                    ));
                }
                let mut spec = c.runner.clone();
                for k in ["api", "token"] {
                    if let Some(v) = own.get(k) {
                        spec[k] = v.clone();
                    }
                }
                let mut w = crate::github::Workflow::parse(&spec)?;
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

fn is_name(s: &str) -> bool {
    s.starts_with(|c: char| c.is_ascii_lowercase())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Every action this portal offers: its own, then each child portal's. A child whose manifest
/// does not load offers nothing, and neither does an entity that is this portal's own
/// authorization object (`portal:<name>`).
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
    let runners = fed.runners.clone().unwrap_or(Value::Null);
    let own = format!("portal:{}", m.portal.name);
    for (id, e) in d
        .entities
        .iter()
        .filter(|(id, e)| &e.kind == kind && **id != own)
        .filter(|(_, e)| e.fields.get("error").is_none_or(Value::is_null))
    {
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
        let rel = e.fields.get("path").and_then(Value::as_str).unwrap_or(".");
        let key = e
            .fields
            .get("key")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| {
                rel.strip_suffix("/.toolbox")
                    .or_else(|| rel.strip_suffix(".toolbox"))
                    .unwrap_or(rel)
                    .trim_end_matches('/')
                    .to_owned()
            });
        let runner = e.fields.get("runner").cloned().unwrap_or(Value::Null);
        for def in e
            .fields
            .get("actions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if !def.get("name").and_then(Value::as_str).is_some_and(is_name) {
                continue;
            }
            if let Some(a) = as_parent_action(def, kind, permission, &portal) {
                out.push(Offered {
                    child: Some(Child {
                        portal: portal.clone(),
                        key: key.clone(),
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
                        reviewed: def["executor"] == "change" && def["land"] != "push",
                        runners: runners.clone(),
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
    schema_line(&a.inputs)
}

/// [`inputs_line`] from an inputs JSON Schema, as `GET /v1/actions` returns it.
pub fn schema_line(inputs: &Value) -> String {
    let req: Vec<&str> = inputs
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let props = inputs
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
            json!({"name": c.portal, "key": c.key, "path": c.path, "runner": c.runner, "executor": c.executor}),
        );
    }
    Value::Object(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn child(runner: Value, runners: Value) -> Offered {
        Offered {
            action: serde_json::from_value(
                json!({"name": "deploy", "kind": "Portal", "scope": "day2",
                "description": "Deploy.", "inputs": {"type": "object"},
                "permission": "can_run_actions", "executor": {"type": "command", "run": "true"}}),
            )
            .unwrap(),
            entity: Some("portal:team/pay".into()),
            child: Some(Child {
                portal: "pay".into(),
                key: "team/pay".into(),
                path: "/org/team/pay".into(),
                action: "deploy".into(),
                runner,
                approval: None,
                executor: "command".into(),
                reviewed: false,
                runners,
            }),
        }
    }

    #[test]
    fn a_server_runs_a_child_only_through_a_runner_the_parent_allows() {
        let served = |runner, runners| child(runner, runners).child.unwrap().served();
        assert_eq!(served(Value::Null, json!({})).unwrap_err().0, "no-runner");
        let local = json!({"type": "local"});
        assert_eq!(
            served(local.clone(), json!({})).unwrap_err().0,
            "runner-not-allowed"
        );
        assert_eq!(
            served(local.clone(), json!({"local": {"portals": ["other/*"]}}))
                .unwrap_err()
                .0,
            "runner-not-allowed"
        );
        assert!(served(local, json!({"local": {"portals": ["team/*"]}})).is_ok());
    }

    #[test]
    fn a_child_cannot_choose_where_the_parents_token_goes() {
        let bin = Path::new("toolbox");
        let wf = json!({"type": "github-workflow", "repo": "acme/pay", "workflow": "toolbox.yml"});
        let own = json!({"github-workflow": {"repos": ["acme/*"], "token": "ORG_TOKEN",
            "api": "https://ghe.example/api/v3"}});
        let mut evil = wf.clone();
        evil["api"] = "http://attacker.example".into();
        assert!(child(evil, own.clone()).executor(bin).is_err());
        let mut evil = wf.clone();
        evil["token"] = "PARENT_SECRET".into();
        assert!(child(evil, own.clone()).executor(bin).is_err());
        assert!(child(wf.clone(), json!({})).executor(bin).is_err());
        let mut elsewhere = wf.clone();
        elsewhere["repo"] = "victim/prod".into();
        assert!(child(elsewhere, own.clone()).executor(bin).is_err());
        let Ok(Executor::GithubWorkflow(w)) = child(wf, own).executor(bin) else {
            panic!("an allowed child should dispatch");
        };
        assert_eq!(
            (w.api.as_str(), w.token_env.as_str()),
            ("https://ghe.example/api/v3", "ORG_TOKEN")
        );
        assert_eq!(w.federated.as_deref(), Some("deploy"));
    }

    #[test]
    fn globs_match_runs_of_characters() {
        assert!(glob("*", "team/repo"));
        assert!(glob("team/*", "team/repo"));
        assert!(!glob("team/*", "other/repo"));
        assert!(glob("org/*-svc", "org/pay-svc"));
        assert!(!glob("org/*-svc", "org/pay-svc2"));
        assert!(glob("exact", "exact"));
        assert!(!glob("exact", "exactly"));
    }
}
