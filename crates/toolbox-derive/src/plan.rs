//! `toolbox plan` (docs/v1-spec.md §6.4): two recomputes over one snapshot, then a diff of
//! entities, fields (with provenance), scorecard levels and rule outcomes, tuples, options, actions
//! and promoted MCP tools.

use crate::Derived;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use toolbox_model::Manifest;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FieldChange {
    pub entity: String,
    pub field: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub source_before: Option<String>,
    pub source_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScoreChange {
    pub scorecard: String,
    pub entity: String,
    pub level_before: Option<String>,
    pub level_after: Option<String>,
    /// rule → (before, after), only rules whose outcome changed
    pub rules: BTreeMap<String, (Option<String>, Option<String>)>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OptionChange {
    pub action: String,
    pub input: String,
    pub entity: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct PlanDiff {
    pub entities_added: Vec<String>,
    pub entities_removed: Vec<String>,
    pub fields: Vec<FieldChange>,
    pub scores: Vec<ScoreChange>,
    pub tuples_added: Vec<Value>,
    pub tuples_removed: Vec<Value>,
    pub options: Vec<OptionChange>,
    pub actions_added: Vec<String>,
    pub actions_removed: Vec<String>,
    pub mcp_tools_added: Vec<String>,
    pub mcp_tools_removed: Vec<String>,
}

impl PlanDiff {
    pub fn is_empty(&self) -> bool {
        *self == PlanDiff::default()
    }

    /// A Markdown summary for PR comments.
    pub fn markdown(&self) -> String {
        let mut s = String::from("## Toolbox plan\n\n");
        if self.is_empty() {
            s.push_str("No changes.\n");
            return s;
        }
        let row = |s: &mut String, label: &str, n: usize| {
            if n > 0 {
                s.push_str(&format!("| {label} | {n} |\n"));
            }
        };
        s.push_str("| Change | Count |\n|---|---|\n");
        row(&mut s, "Entities added", self.entities_added.len());
        row(&mut s, "Entities removed", self.entities_removed.len());
        row(&mut s, "Field changes", self.fields.len());
        row(&mut s, "Scorecard changes", self.scores.len());
        row(
            &mut s,
            "Authorization tuples gained",
            self.tuples_added.len(),
        );
        row(
            &mut s,
            "Authorization tuples lost",
            self.tuples_removed.len(),
        );
        row(&mut s, "Action option changes", self.options.len());
        row(&mut s, "Actions added", self.actions_added.len());
        row(&mut s, "Actions removed", self.actions_removed.len());
        row(&mut s, "MCP tools added", self.mcp_tools_added.len());
        row(&mut s, "MCP tools removed", self.mcp_tools_removed.len());
        let show = |v: &Option<String>| v.clone().unwrap_or_else(|| "—".into());
        if !self.scores.is_empty() {
            s.push_str("\n### Scorecards\n\n| Scorecard | Entity | Level | Rules changed |\n|---|---|---|---|\n");
            for c in &self.scores {
                let rules: Vec<String> = c
                    .rules
                    .iter()
                    .map(|(r, (a, b))| format!("{r}: {} → {}", show(a), show(b)))
                    .collect();
                s.push_str(&format!(
                    "| {} | {} | {} → {} | {} |\n",
                    c.scorecard,
                    c.entity,
                    show(&c.level_before),
                    show(&c.level_after),
                    rules.join("; ")
                ));
            }
        }
        if !self.fields.is_empty() {
            s.push_str("\n### Fields\n\n| Entity | Field | Before | After |\n|---|---|---|---|\n");
            for f in &self.fields {
                let v = |x: &Option<Value>| x.as_ref().map_or("—".into(), |v| v.to_string());
                s.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    f.entity,
                    f.field,
                    v(&f.before),
                    v(&f.after)
                ));
            }
        }
        if !self.tuples_added.is_empty() || !self.tuples_removed.is_empty() {
            s.push_str("\n### Access\n\n");
            for t in &self.tuples_added {
                s.push_str(
                    &format!(
                        "- gained: `{} {} {}`\n",
                        t["user"], t["relation"], t["object"]
                    )
                    .replace('"', ""),
                );
            }
            for t in &self.tuples_removed {
                s.push_str(
                    &format!(
                        "- lost: `{} {} {}`\n",
                        t["user"], t["relation"], t["object"]
                    )
                    .replace('"', ""),
                );
            }
        }
        s
    }
}

fn names<T>(xs: &[T], f: impl Fn(&T) -> Option<String>) -> BTreeSet<String> {
    xs.iter().filter_map(f).collect()
}

/// Diff two derivations (and their manifests) of the same snapshot.
pub fn diff(old_m: &Manifest, old: &Derived, new_m: &Manifest, new: &Derived) -> PlanDiff {
    let mut p = PlanDiff::default();
    let (a, b): (BTreeSet<&String>, BTreeSet<&String>) =
        (old.entities.keys().collect(), new.entities.keys().collect());
    p.entities_added = b.difference(&a).map(|s| s.to_string()).collect();
    p.entities_removed = a.difference(&b).map(|s| s.to_string()).collect();
    for id in a.intersection(&b) {
        let (x, y) = (&old.entities[*id], &new.entities[*id]);
        let fields: BTreeSet<&String> = x.fields.keys().chain(y.fields.keys()).collect();
        for f in fields {
            let (vb, va) = (x.fields.get(f), y.fields.get(f));
            let (sb, sa) = (x.provenance.get(f), y.provenance.get(f));
            if vb != va || sb != sa {
                p.fields.push(FieldChange {
                    entity: id.to_string(),
                    field: f.clone(),
                    before: vb.cloned(),
                    after: va.cloned(),
                    source_before: sb.cloned(),
                    source_after: sa.cloned(),
                });
            }
        }
    }
    let cards: BTreeSet<&String> = old.scores.keys().chain(new.scores.keys()).collect();
    for card in cards {
        let (x, y) = (old.scores.get(card), new.scores.get(card));
        let ids: BTreeSet<&String> = x
            .into_iter()
            .flat_map(|m| m.keys())
            .chain(y.into_iter().flat_map(|m| m.keys()))
            .collect();
        for id in ids {
            let (sb, sa) = (x.and_then(|m| m.get(id)), y.and_then(|m| m.get(id)));
            if sb == sa {
                continue;
            }
            let empty = BTreeMap::new();
            let (rb, ra) = (
                sb.and_then(|s| s.rules.as_ref()).unwrap_or(&empty),
                sa.and_then(|s| s.rules.as_ref()).unwrap_or(&empty),
            );
            let rules = rb
                .keys()
                .chain(ra.keys())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .filter(|r| rb.get(*r) != ra.get(*r))
                .map(|r| (r.clone(), (rb.get(r).cloned(), ra.get(r).cloned())))
                .collect();
            p.scores.push(ScoreChange {
                scorecard: card.clone(),
                entity: id.clone(),
                level_before: sb.map(|s| s.level.clone()),
                level_after: sa.map(|s| s.level.clone()),
                rules,
            });
        }
    }
    let key = |t: &Value| crate::pyjson::compact(t);
    let (ta, tb): (BTreeMap<String, &Value>, BTreeMap<String, &Value>) = (
        old.tuples.iter().map(|t| (key(t), t)).collect(),
        new.tuples.iter().map(|t| (key(t), t)).collect(),
    );
    p.tuples_added = tb
        .iter()
        .filter(|(k, _)| !ta.contains_key(*k))
        .map(|(_, t)| (*t).clone())
        .collect();
    p.tuples_removed = ta
        .iter()
        .filter(|(k, _)| !tb.contains_key(*k))
        .map(|(_, t)| (*t).clone())
        .collect();
    for (action, inputs) in &new.options {
        for (input, by_entity) in inputs {
            for (id, after) in by_entity {
                let before = old
                    .options
                    .get(action)
                    .and_then(|i| i.get(input))
                    .and_then(|e| e.get(id));
                if before != Some(after) {
                    p.options.push(OptionChange {
                        action: action.clone(),
                        input: input.clone(),
                        entity: id.clone(),
                        before: before.cloned().unwrap_or_default(),
                        after: after.clone(),
                    });
                }
            }
        }
    }
    let (aa, ab) = (
        names(&old_m.actions, |a| Some(a.name.clone())),
        names(&new_m.actions, |a| Some(a.name.clone())),
    );
    p.actions_added = ab.difference(&aa).cloned().collect();
    p.actions_removed = aa.difference(&ab).cloned().collect();
    let promoted = |m: &Manifest| {
        names(&m.actions, |a| {
            a.mcp
                .as_ref()
                .and_then(|x| x.get("promote"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
    };
    let (ma, mb) = (promoted(old_m), promoted(new_m));
    p.mcp_tools_added = mb.difference(&ma).cloned().collect();
    p.mcp_tools_removed = ma.difference(&mb).cloned().collect();
    p
}
