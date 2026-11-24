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
use toolbox_model::forge::{self, Kind};
use toolbox_model::manifest::{Action, Forge};

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
    /// How the child lets a parent reach it (`{type: local}`, `{type: ci}` or
    /// `{type: github-workflow, …}`); null when it declares nothing, which a checkout runs
    /// locally and a server refuses.
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
    /// For a child cloned from git (`federation.children`), the branch whose definitions count.
    pub branch: Option<String>,
    /// For a child cloned from git: the URL the parent cloned it from, which decides its forge.
    pub git: Option<String>,
    /// The parent's `forges`.
    pub forges: Vec<Forge>,
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
    /// The child's own `toolbox run`, in this portal's clone or checkout of it.
    fn local(&self, program: &Path) -> Executor {
        Executor::Toolbox {
            portal: self.path.clone(),
            action: self.action.clone(),
            entity: String::new(),
            program: program.to_path_buf(),
            env: self.runners["local"]["env"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect(),
            branch: self.branch.clone(),
            review: self
                .git
                .as_deref()
                .and_then(|g| forge::resolve(&self.forges, g)),
        }
    }

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
    /// The portal's `forges`, which its own executors reach.
    pub forges: Vec<Forge>,
}

impl Offered {
    /// The executor that carries out a run of this action. `program` is the `toolbox` binary a
    /// local child is run with.
    pub fn executor(&self, program: &Path) -> Result<Executor, String> {
        let Some(c) = &self.child else {
            return Executor::parse_in(&self.action.executor, &self.forges);
        };
        match c
            .runner
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("local")
        {
            "local" => Ok(c.local(program)),
            "ci" => c.ci(program),
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
                w.preview = Some(Box::new(c.local(program)));
                Ok(Executor::GithubWorkflow(w))
            }
            other => Err(format!(
                "child portal {}: unknown runner {other} (local, ci, github-workflow)",
                c.portal
            )),
        }
    }
}

impl Child {
    /// `runner: {type: ci}` (docs/design/forges.md §4.3): the child's own CI, on the forge and
    /// project the parent cloned it from, with that forge's API and token. The child names at
    /// most its `workflow` (GitHub) and `ref`. The child previews in this portal's clone, as a
    /// local child does, so its approvers and effects are known and the grant can bind them.
    fn ci(&self, program: &Path) -> Result<Executor, String> {
        for k in ["project", "repo", "api", "url", "token", "forge"] {
            if self.runner.get(k).is_some() {
                return Err(format!(
                    "child portal {} names {k} in federation.runner; a ci runner goes to the project this portal cloned it from, with this portal's forge settings",
                    self.portal
                ));
            }
        }
        let Some(git) = &self.git else {
            return Err(format!(
                "child portal {} is not listed in federation.children by git URL, so this portal can't tell which project's CI to start",
                self.portal
            ));
        };
        let Some(f) = forge::resolve(&self.forges, git) else {
            return Err(format!(
                "child portal {} is on {git}, which is plain git here: list its forge under forges: (url and type) to start its CI, or let it run locally",
                self.portal
            ));
        };
        if !f.admits() {
            return Err(format!(
                "child portal {} is {} on {}, which that forge's projects: does not list",
                self.portal, f.project, f.url
            ));
        }
        let s = |k: &str| {
            self.runner
                .get(k)
                .and_then(Value::as_str)
                .map(str::to_owned)
        };
        let git_ref = s("ref")
            .or_else(|| self.branch.clone())
            .unwrap_or_else(|| "main".into());
        Ok(match f.kind {
            Kind::Github => Executor::GithubWorkflow(crate::github::Workflow {
                repo: f.project,
                workflow: s("workflow").unwrap_or_else(|| "toolbox.yml".into()),
                git_ref,
                api: f.api,
                token_env: f.token,
                federated: Some(self.action.clone()),
                preview: Some(Box::new(self.local(program))),
            }),
            Kind::Gitlab => Executor::GitlabPipeline(crate::gitlab::Pipeline {
                project: f.project,
                git_ref,
                api: f.api,
                token_env: f.token,
                federated: Some(self.action.clone()),
                clone: Some(self.path.clone()),
                preview: Some(Box::new(self.local(program))),
            }),
        })
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

/// The child portals that offer nothing because their manifest does not load: `{entity, error}`.
pub fn unreadable(m: &Manifest, d: &Derived) -> Vec<Value> {
    let Some(kind) = m.federation.as_ref().and_then(|f| f.portals.as_ref()) else {
        return Vec::new();
    };
    d.entities
        .iter()
        .filter(|(_, e)| &e.kind == kind)
        .filter_map(|(id, e)| {
            let err = e.fields.get("error")?.as_str()?;
            Some(json!({"entity": id, "error": err}))
        })
        .collect()
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
            forges: m.forges.clone(),
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
        let branch = e
            .fields
            .get("git")
            .and_then(|g| g.get("branch"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let git = e
            .fields
            .get("git")
            .and_then(|g| g.get("url"))
            .and_then(Value::as_str)
            .map(str::to_owned);
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
                        branch: branch.clone(),
                        git: git.clone(),
                        forges: m.forges.clone(),
                    }),
                    action: a,
                    entity: Some(id.clone()),
                    forges: m.forges.clone(),
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

/// `; <child> offers no actions: <why>` for each child portal (or just `only`) whose manifest
/// does not load, which offers nothing, so a missing action may be its.
fn broken(d: &Derived, only: Option<&str>) -> String {
    d.entities
        .iter()
        .filter(|(id, e)| e.fields.contains_key("actions") && only.is_none_or(|o| o == id.as_str()))
        .filter_map(|(id, e)| {
            Some(format!(
                "; {id} offers no actions: {}",
                e.fields.get("error")?.as_str()?
            ))
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
        names.sort_unstable();
        names.dedup();
        return Err(format!(
            "no action {name} (actions: {}){}",
            names.join(", "),
            broken(d, None)
        ));
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
                    "{name} is not offered on {id} (it is on: {}){}",
                    all.join(", "),
                    broken(d, Some(id))
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
        json!(a.expect.is_some() || o.child.is_some() || a.executor["type"] == "change"),
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
                branch: None,
                git: None,
                forges: vec![],
            }),
            forges: vec![],
        }
    }

    /// A child cloned from `git`, in a parent that declares `forges`.
    fn cloned(runner: Value, git: &str, forges: Vec<Forge>) -> Offered {
        let mut o = child(runner, json!({}));
        let c = o.child.as_mut().unwrap();
        c.git = Some(git.into());
        c.forges = forges;
        c.branch = Some("trunk".into());
        o
    }

    #[test]
    fn a_ci_runner_starts_the_ci_of_the_project_the_parent_cloned() {
        let bin = Path::new("toolbox");
        let ci = json!({"type": "ci"});
        let Ok(Executor::GitlabPipeline(p)) = cloned(
            ci.clone(),
            "https://gitlab.com/acme/platform/pay.git",
            vec![],
        )
        .executor(bin) else {
            panic!("a gitlab.com child should get a pipeline");
        };
        assert_eq!(
            (
                p.project.as_str(),
                p.git_ref.as_str(),
                p.api.as_str(),
                p.token_env.as_str()
            ),
            (
                "acme/platform/pay",
                "trunk",
                "https://gitlab.com/api/v4",
                "GITLAB_TOKEN"
            )
        );
        assert_eq!(p.federated.as_deref(), Some("deploy"));
        assert_eq!(p.clone.as_deref(), Some(Path::new("/org/team/pay")));
        let Ok(Executor::GithubWorkflow(w)) =
            cloned(ci.clone(), "git@github.com:acme/web.git", vec![]).executor(bin)
        else {
            panic!("a github.com child should get a workflow");
        };
        assert_eq!(
            (w.repo.as_str(), w.workflow.as_str()),
            ("acme/web", "toolbox.yml")
        );
        let ghes = Forge {
            url: "https://ghe.acme.dev".into(),
            kind: Some("github".into()),
            api: None,
            token: Some("GHES_TOKEN".into()),
            projects: Some(vec!["platform/*".into()]),
        };
        let Ok(Executor::GithubWorkflow(w)) = cloned(
            json!({"type": "ci", "workflow": "fleet.yml", "ref": "main"}),
            "https://ghe.acme.dev/platform/ledger.git",
            vec![ghes.clone()],
        )
        .executor(bin) else {
            panic!("an Enterprise Server child should get a workflow");
        };
        assert_eq!(
            (
                w.api.as_str(),
                w.token_env.as_str(),
                w.workflow.as_str(),
                w.git_ref.as_str()
            ),
            (
                "https://ghe.acme.dev/api/v3",
                "GHES_TOKEN",
                "fleet.yml",
                "main"
            )
        );
    }

    #[test]
    fn a_ci_runner_refuses_what_the_child_cannot_choose() {
        let bin = Path::new("toolbox");
        let on = |runner: Value, git: &str, forges: Vec<Forge>| {
            cloned(runner, git, forges)
                .executor(bin)
                .map(|_| ())
                .unwrap_err()
        };
        for k in ["project", "repo", "api", "url", "token", "forge"] {
            let mut r = json!({"type": "ci"});
            r[k] = "elsewhere".into();
            assert!(on(r, "https://gitlab.com/acme/pay.git", vec![]).contains(k));
        }
        assert!(on(json!({"type": "ci"}), "/srv/git/tools.git", vec![]).contains("plain git"));
        assert!(
            on(
                json!({"type": "ci"}),
                "https://gitlab.acme.dev/acme/pay.git",
                vec![]
            )
            .contains("forges:")
        );
        let narrow = Forge {
            url: "https://gitlab.acme.dev".into(),
            kind: Some("gitlab".into()),
            api: None,
            token: None,
            projects: Some(vec!["acme/*".into()]),
        };
        assert!(
            on(
                json!({"type": "ci"}),
                "https://gitlab.acme.dev/acme/platform/pay.git",
                vec![narrow.clone()]
            )
            .contains("projects:")
        );
        assert!(
            cloned(
                json!({"type": "ci"}),
                "https://gitlab.acme.dev/acme/pay.git",
                vec![narrow]
            )
            .executor(bin)
            .is_ok()
        );
        let mut c = child(json!({"type": "ci"}), json!({}));
        c.child.as_mut().unwrap().git = None;
        assert!(
            c.executor(bin)
                .map(|_| ())
                .unwrap_err()
                .contains("federation.children")
        );
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
