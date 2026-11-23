//! The portal's model as records: `kind`, `field` and `connector` (docs/design/scorecards-for-scorecards.md
//! §5). With `scorecard` and `rule` (meta.rs) they let a portal score every part of itself.
//!
//! Every record carries a content `hash` of its definition, so an assessment written against one
//! version of a kind, field or rule can be recognised as stale when the definition changes.

use crate::portal::Local;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use toolbox_model::Manifest;

/// The first 12 hex digits of the SHA-256 of `value`'s canonical JSON.
pub fn hash(value: &impl serde::Serialize) -> String {
    let text = serde_json::to_string(value).unwrap_or_default();
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
}

/// Where each kind's fields are read: `(kind, field) → [reader]`. Readers are rules and evidence of
/// scorecards on the kind, its computed fields, its tuple derivations and its CODEOWNERS authority;
/// fields of other kinds read through `inbound` lists count for every kind (an over-approximation:
/// a field is never wrongly called unused).
type Readers = BTreeMap<(String, String), BTreeSet<String>>;

/// Record that `who` reads, in `cel` evaluated over a `kind` entity, the fields it selects.
fn add(out: &mut Readers, m: &Manifest, kind: &str, cel: &str, who: String) {
    let Ok(a) = toolbox_expr::analysis::analyse(cel) else {
        return;
    };
    for f in &a.fields {
        out.entry((kind.to_owned(), f.clone()))
            .or_default()
            .insert(who.clone());
    }
    if cel.contains("inbound") {
        for k in &m.kinds {
            for f in a.selects.iter().filter(|f| k.fields.contains_key(*f)) {
                out.entry((k.name.clone(), f.clone()))
                    .or_default()
                    .insert(format!("{who} (via inbound)"));
            }
        }
    }
}

fn readers(m: &Manifest) -> Readers {
    let mut out = Readers::new();
    for sc in &m.scorecards {
        for r in &sc.rules {
            add(
                &mut out,
                m,
                &sc.kind,
                &r.check,
                format!("rule {}/{}", sc.name, r.name),
            );
            if let Some(e) = &r.evidence {
                add(
                    &mut out,
                    m,
                    &sc.kind,
                    e,
                    format!("evidence {}/{}", sc.name, r.name),
                );
            }
            if let Some(e) = &r.measure {
                add(
                    &mut out,
                    m,
                    &sc.kind,
                    e,
                    format!("measure {}/{}", sc.name, r.name),
                );
            }
        }
        if let Some(a) = &sc.applies_to {
            add(&mut out, m, &sc.kind, a, format!("appliesTo {}", sc.name));
        }
    }
    for k in &m.kinds {
        for (name, f) in &k.fields {
            if let Some(c) = &f.computed {
                add(
                    &mut out,
                    m,
                    &k.name,
                    c,
                    format!("computed {}.{name}", k.name),
                );
            }
            if let Some(a) = &f.authority {
                for read in [&a.claim, &a.rules] {
                    out.entry((k.name.clone(), read.clone()))
                        .or_default()
                        .insert(format!("authority {}.{name}", k.name));
                }
            }
        }
    }
    for t in m.authz.iter().flat_map(|a| &a.tuples) {
        if let toolbox_model::manifest::TupleDerivation::Kind { kind, emit } = t {
            add(&mut out, m, kind, emit, format!("tuples {kind}"));
        }
    }
    out
}

/// Per kind: fixture entity ids, and per field how many of them have a value.
fn population(local: &Local) -> BTreeMap<String, (Vec<String>, BTreeMap<String, u64>)> {
    let mut out: BTreeMap<String, (Vec<String>, BTreeMap<String, u64>)> = BTreeMap::new();
    for (id, e) in &local.derived.entities {
        let slot = out.entry(e.kind.clone()).or_default();
        slot.0.push(id.clone());
        for (f, v) in &e.fields {
            if !v.is_null() {
                *slot.1.entry(f.clone()).or_default() += 1;
            }
        }
    }
    out
}

fn field_record(
    portal: &str,
    kind: &str,
    name: &str,
    f: &toolbox_model::manifest::Field,
    ctx: &Ctx,
) -> Value {
    let read: Vec<&String> = ctx
        .readers
        .get(&(kind.to_owned(), name.to_owned()))
        .into_iter()
        .flatten()
        .collect();
    let k = ctx.m.kinds.iter().find(|k| k.name == kind);
    let sources: Vec<&String> = k
        .map(|k| {
            k.sources
                .fields
                .get(name)
                .unwrap_or(&k.sources.default)
                .iter()
                .collect()
        })
        .unwrap_or_default();
    let (entities, with) = ctx
        .pop
        .get(kind)
        .map(|(ids, f)| (ids.len() as u64, f.get(name).copied().unwrap_or(0)))
        .unwrap_or((0, 0));
    json!({
        "portal": portal, "kind": kind, "name": name, "type": f.ty, "described": filled(&f.description),
        "computed": f.computed.is_some(), "sources": if f.computed.is_some() { vec![] } else { sources },
        "opaque": f.ty == "object" || (f.ty == "list" && f.items.as_deref() == Some("object")),
        "readBy": read, "fixture": {"entities": entities, "withValue": with}, "hash": hash(f),
        // Size and loops of a computed field's CEL, scored like a rule's check (meta-field), so
        // moving logic out of a rule into a computed field doesn't hide it.
        "computedCel": f.computed.as_deref().and_then(|c| toolbox_expr::analysis::analyse(c).ok())
            .map(|a| json!({"nodes": a.nodes, "loops": a.comprehension_depth})),
    })
}

struct Ctx<'a> {
    m: &'a Manifest,
    readers: Readers,
    pop: BTreeMap<String, (Vec<String>, BTreeMap<String, u64>)>,
}

fn kind_record(portal: &str, k: &toolbox_model::manifest::Kind, ctx: &Ctx, expects: &str) -> Value {
    let ids = ctx
        .pop
        .get(&k.name)
        .map(|p| p.0.clone())
        .unwrap_or_default();
    let tested = ids.iter().any(|id| expects.contains(&format!("'{id}'")));
    let scorecards: Vec<&String> = ctx
        .m
        .scorecards
        .iter()
        .filter(|s| s.kind == k.name)
        .map(|s| &s.name)
        .collect();
    let related_in: Vec<String> = ctx
        .m
        .kinds
        .iter()
        .flat_map(|o| {
            o.relations
                .iter()
                .filter(|(_, r)| r.to == k.name)
                .map(move |(n, _)| format!("{}.{n}", o.name))
        })
        .collect();
    json!({
        "portal": portal, "name": k.name, "described": filled(&k.description),
        "fields": k.fields.len(), "fieldsDescribed": k.fields.values().filter(|f| filled(&f.description)).count(),
        "identityFrom": k.identity_from, "scorecards": scorecards, "relationsOut": k.relations.keys().collect::<Vec<_>>(),
        "relationsIn": related_in, "fixtureEntities": ids.len(), "tested": tested, "hash": hash(k),
    })
}

fn connector_record(portal: &str, c: &toolbox_model::manifest::Connector, local: &Local) -> Value {
    let mapped: BTreeSet<&str> = local
        .manifest
        .kinds
        .iter()
        .flat_map(|k| &k.map)
        // A map rule maps a record only if it reads it: `set: "{}"` with no relation from the
        // record keeps the record unused, whatever `from` says.
        .filter(|r| r.set.contains("record") || r.relations.values().any(|v| v.contains("record")))
        .filter_map(|r| {
            r.from
                .split_once('/')
                .filter(|(conn, _)| *conn == c.name)
                .map(|(_, kind)| kind)
        })
        .collect();
    let freshness: BTreeSet<&str> = c.freshness.keys().map(String::as_str).collect();
    let fixtures = std::fs::read_to_string(
        local
            .root
            .join("fixtures")
            .join(format!("{}.ndjson", c.name)),
    )
    .map(|t| t.lines().filter(|l| !l.trim().is_empty()).count())
    .unwrap_or(0);
    json!({
        "portal": portal, "name": c.name, "runtime": c.runtime, "package": c.package, "records": c.records,
        "unmapped": c.records.iter().filter(|r| !mapped.contains(r.as_str())).collect::<Vec<_>>(),
        "withoutFreshness": c.records.iter().filter(|r| !freshness.contains(r.as_str())).collect::<Vec<_>>(),
        "refreshable": c.refresh.keys().collect::<Vec<_>>(),
        "fixtureRecords": fixtures, "hash": hash(c),
    })
}

/// The portal's assessment files (`assessments/**`), parsed; unreadable files are skipped
/// (`toolbox assess check` reports them).
fn assessments(local: &Local) -> Vec<Value> {
    let dir = local.root.join("assessments");
    toolbox_connect::files::walk(&dir)
        .iter()
        .filter(|r| r.ends_with(".yaml") || r.ends_with(".yml") || r.ends_with(".json"))
        .filter_map(|r| std::fs::read_to_string(dir.join(r)).ok())
        .filter_map(|t| serde_norway::from_str::<Value>(&t).ok())
        .collect()
}

/// A rubric with how it has been used: assessments made with it and, once there are three or more,
/// the criteria every one of them passed (a question that never discriminates).
fn rubric_record(portal: &str, r: &toolbox_model::manifest::Rubric, docs: &[Value]) -> Value {
    let used: Vec<&Value> = docs
        .iter()
        .filter(|d| d["rubric"] == r.name.as_str())
        .collect();
    let verdicts_for = |c: &str| -> Vec<&Value> {
        used.iter()
            .flat_map(|d| d["verdicts"].as_array().into_iter().flatten())
            .filter(|v| v["criterion"] == c)
            .collect()
    };
    let always_pass: Vec<&String> = if used.len() >= 3 {
        r.criteria
            .iter()
            .map(|c| &c.name)
            .filter(|c| verdicts_for(c).iter().all(|v| v["verdict"] == "pass"))
            .collect()
    } else {
        vec![]
    };
    json!({
        "portal": portal, "name": r.name, "kind": r.kind, "subject": r.subject(),
        "described": filled(&r.description),
        "criteria": r.criteria.iter().map(|c| &c.name).collect::<Vec<_>>(),
        "unguided": r.criteria.iter().filter(|c| !guides(&c.guidance)).map(|c| &c.name).collect::<Vec<_>>(),
        "assessments": used.len(), "alwaysPass": always_pass, "hash": hash(r),
    })
}

/// A description or guidance counts only when it has text: `description: ""` or `guidance: " "`
/// is the cheapest way to game a `described` or `guided` rule.
pub(crate) fn filled(text: &Option<String>) -> bool {
    text.as_deref().is_some_and(|t| !t.trim().is_empty())
}

/// Guidance that tells assessors when to give a verdict: at least ten words, an outcome as a word
/// (fail, pass or unsure, not "failure"), and a condition (`when`, `if`, `unless`, or "and name"),
/// so a blank line, a restated question or "Fail when the answer is no." doesn't count as guided.
fn guides(text: &Option<String>) -> bool {
    static OUTCOME: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static CONDITION: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let outcome = OUTCOME.get_or_init(|| {
        regex::Regex::new(r"(?i)\b(fail|fails|pass|passes|unsure)\b").expect("valid regex")
    });
    let condition = CONDITION.get_or_init(|| {
        regex::Regex::new(r"(?i)\b(when|if|unless|and name)\b").expect("valid regex")
    });
    text.as_deref().is_some_and(|t| {
        t.split_whitespace().count() >= 10 && outcome.is_match(t) && condition.is_match(t)
    })
}

/// `kind`, `field`, `rubric` and `connector` records for a loaded portal; `expects` is the text of its test
/// expectations (to tell which kinds tests reach).
pub fn records(local: &Local, expects: &str) -> Vec<(String, String, Value)> {
    let m = &local.manifest;
    let portal = m.portal.name.as_str();
    let ctx = Ctx {
        m,
        readers: readers(m),
        pop: population(local),
    };
    let mut out = Vec::new();
    for k in &m.kinds {
        out.push((
            "kind".into(),
            format!("kind:{portal}/{}", k.name),
            kind_record(portal, k, &ctx, expects),
        ));
        for (name, f) in &k.fields {
            out.push((
                "field".into(),
                format!("field:{portal}/{}.{name}", k.name),
                field_record(portal, &k.name, name, f, &ctx),
            ));
        }
    }
    let docs = assessments(local);
    for r in &m.rubrics {
        out.push((
            "rubric".into(),
            format!("rubric:{portal}/{}", r.name),
            rubric_record(portal, r, &docs),
        ));
    }
    for c in &m.connectors {
        out.push((
            "connector".into(),
            format!("connector:{portal}/{}", c.name),
            connector_record(portal, c, local),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn blank_text_is_not_a_description() {
        assert!(super::filled(&Some("What it is.".into())));
        assert!(!super::filled(&Some(String::new())));
        assert!(!super::filled(&Some("  \n".into())));
        assert!(!super::filled(&None));
    }

    #[test]
    fn guidance_must_say_what_fails() {
        assert!(super::guides(&Some(
            "Fail when an empty doc comment, a renamed item or a trivial test would pass it."
                .into()
        )));
        assert!(!super::guides(&Some("Fail when the answer is no.".into())));
        assert!(!super::guides(&Some(
            "Consider each failure mode and compass bearing of the rule in turn.".into()
        )));
        assert!(!super::guides(&Some(String::new())));
        assert!(!super::guides(&Some("Is the rule good?".into())));
        assert!(!super::guides(&Some(
            "Consider the rule and its intent carefully.".into()
        )));
    }
}
