//! `seikika describe scorecards`: the scorecards and rules of one or more portals as connector
//! records, so a portal can score its own standards (docs/design/scorecards-for-scorecards.md).
//!
//! Each `rule` record carries what the manifest says (level, fix, evidence), what its CEL is made
//! of (size, nesting, loops, fields, functions), whether a test expectation names it, how it came
//! out over the portal's own fixtures, and — when a results file is given — how it comes out on
//! real data. Each `scorecard` record carries the same at the card level.
//!
//! The recursion stops here: scorecards that score scorecards (kinds fed by `scorecard` or `rule`
//! records) are described like any other — so the meta scorecard scores itself — but their *live*
//! outcomes are left out, so describing never depends on its own previous output.

use seikika_model::Manifest;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

type Pinned = BTreeMap<(String, String), BTreeSet<String>>;

fn test_mentions(
    assertions: Vec<seikika_expr::analysis::ScoreAssertion>,
) -> (Pinned, BTreeSet<String>) {
    let mut rules = Pinned::new();
    let mut cards = BTreeSet::new();
    for assertion in assertions {
        if let Some((rule, outcome)) = assertion.rule {
            rules
                .entry((assertion.card.clone(), rule))
                .or_default()
                .insert(outcome);
        }
        cards.insert(assertion.card);
    }
    (rules, cards)
}

/// Kinds that describe portals themselves (fed by `describe portal` records).
fn meta_kinds(m: &Manifest) -> BTreeSet<String> {
    // `connector` is also a common record name (e.g. connector crates), so it counts only from a
    // connector that also sends `scorecard` records: the one running `describe portal`.
    const META: [&str; 6] = ["scorecard", "rule", "kind", "field", "connector", "rubric"];
    let describers: BTreeSet<&str> = m
        .kinds
        .iter()
        .flat_map(|k| &k.map)
        .filter(|r| r.from_parts().1 == "scorecard")
        .map(|r| r.from_parts().0)
        .collect();
    m.kinds
        .iter()
        .filter(|k| {
            k.map.iter().any(|r| {
                let (conn, rec) = r.from_parts();
                (describers.contains(conn) && META.contains(&rec)) || rec == "assessment"
            })
        })
        .map(|k| k.name.clone())
        .collect()
}

/// `[pass, fail, unknown]` (or `[entities, fail, unknown]` for live results) per (scorecard, rule).
type RuleCounts = BTreeMap<(String, String), [u64; 3]>;
/// Entities per level, per scorecard.
type LevelCounts = BTreeMap<String, BTreeMap<String, u64>>;

/// Per rule: outcome counts over the derived scores (`pass`, `fail`, `unknown`), and the levels
/// entities reached per scorecard.
fn fixture_outcomes(d: &seikika_derive::Derived) -> (RuleCounts, LevelCounts) {
    let (mut rules, mut levels): (RuleCounts, LevelCounts) = Default::default();
    for (card, entities) in &d.scores {
        for s in entities.values() {
            *levels
                .entry(card.clone())
                .or_default()
                .entry(s.level.clone())
                .or_default() += 1;
            for (rule, outcome) in s.rules.iter().flatten() {
                let i = match outcome.as_str() {
                    "pass" => 0,
                    "fail" => 1,
                    _ => 2,
                };
                rules.entry((card.clone(), rule.clone())).or_default()[i] += 1;
            }
        }
    }
    (rules, levels)
}

/// Per rule from a `seikika-report/v1` results file: `[entities scored, failing, unknown]`.
fn live_outcomes(results: &Value, skip: &BTreeSet<String>) -> RuleCounts {
    let mut out = RuleCounts::new();
    for (card, rows) in results["results"].as_object().into_iter().flatten() {
        if skip.contains(card) {
            continue;
        }
        for row in rows.as_object().into_iter().flatten().map(|(_, r)| r) {
            if row["level"] == "skipped" {
                continue;
            }
            for (rule, _) in row["failing"].as_object().into_iter().flatten() {
                out.entry((card.clone(), rule.clone())).or_default()[1] += 1;
            }
            for rule in row["unknown"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
            {
                out.entry((card.clone(), rule.to_owned())).or_default()[2] += 1;
            }
            // every rule of the card was evaluated for this entity
            out.entry((card.clone(), String::new())).or_default()[0] += 1;
        }
    }
    out
}

fn cel(expr: &str) -> Value {
    match seikika_expr::analysis::analyse(expr) {
        Ok(a) => json!({"nodes": a.nodes, "depth": a.depth, "loops": a.comprehension_depth,
                        "fields": a.fields, "functions": a.functions, "undeclared": a.undeclared}),
        Err(e) => json!({"error": e}),
    }
}

fn words(s: Option<&str>) -> usize {
    s.map_or(0, |s| s.split_whitespace().count())
}

fn read_live(f: &Path, meta_cards: &BTreeSet<String>) -> Result<RuleCounts, String> {
    let text = std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", f.display()))?;
    Ok(live_outcomes(&v, meta_cards))
}

fn review_rubric_hash(m: &Manifest, name: &str) -> Option<String> {
    m.rubrics
        .iter()
        .find(|rubric| rubric.name == name)
        .map(seikika_api::assess::rubric_hash)
}

fn rules_per_level(sc: &seikika_model::manifest::Scorecard) -> Map<String, Value> {
    sc.levels
        .iter()
        .map(|level| {
            let count = sc.rules.iter().filter(|rule| rule.level == *level).count();
            (level.clone(), json!(count))
        })
        .collect()
}

/// Records for one portal: `(kind, external id, data)`.
fn describe_portal(
    path: &Path,
    results: Option<&Path>,
) -> Result<Vec<(String, String, Value)>, String> {
    let local = crate::portal::load(path)?;
    let m = &local.manifest;
    let portal = m.portal.name.clone();
    let coverage = seikika_derive::testing::verified_coverage(
        &local.root,
        &m.tests,
        std::sync::Arc::new(local.derived.clone()),
    )?;
    let (tested_rules, tested_cards) = test_mentions(coverage.scores);
    let (fixture, fixture_levels) = fixture_outcomes(&local.derived);
    let meta = meta_kinds(m);
    let meta_cards: BTreeSet<String> = m
        .scorecards
        .iter()
        .filter(|s| meta.contains(&s.kind))
        .map(|s| s.name.clone())
        .collect();
    let live = results.map(|f| read_live(f, &meta_cards)).transpose()?;
    let judged = crate::judged::judged_rules(m);
    let rule_rubric_hash = review_rubric_hash(m, "rule-quality");
    let scorecard_rubric_hash = review_rubric_hash(m, "scorecard-quality");
    let mut out = Vec::new();
    for sc in &m.scorecards {
        let card_id = format!("{portal}/{}", sc.name);
        let live_entities = live
            .as_ref()
            .filter(|_| !meta_cards.contains(&sc.name))
            .map(|l| l.get(&(sc.name.clone(), String::new())).map_or(0, |c| c[0]));
        let mut data = json!({
            "portal": portal, "name": sc.name, "kind": sc.kind, "owner": sc.owner,
            "levels": sc.levels, "rulesPerLevel": rules_per_level(sc), "rules": sc.rules.len(),
            "appliesTo": sc.applies_to.is_some(), "tested": tested_cards.contains(&sc.name),
            "meta": meta_cards.contains(&sc.name),
            "described": crate::model::filled(&sc.description),
            "explained": crate::model::filled(&sc.rationale),
            "links": sc.links.len(),
            "fixtureLevels": fixture_levels.get(&sc.name).cloned().unwrap_or_default(),
            "reviewRubricHash": scorecard_rubric_hash,
        });
        if let Some(n) = live_entities {
            data["liveEntities"] = json!(n);
        }
        data["hash"] = json!(crate::model::hash(sc));
        out.push(("scorecard".into(), format!("scorecard:{card_id}"), data));
        for r in &sc.rules {
            let key = (sc.name.clone(), r.name.clone());
            let f = fixture.get(&key).copied().unwrap_or_default();
            let mut data = json!({
                "portal": portal, "scorecard": sc.name, "card": card_id, "name": r.name, "level": r.level,
                "fixWords": words(r.fix.as_deref()), "hasEvidence": r.evidence.is_some(),
                "definition": {"check": r.check, "evidence": r.evidence, "fix": r.fix},
                "check": cel(&r.check),
                "evidence": r.evidence.as_deref().map(cel),
                "tested": tested_rules.contains_key(&key),
                "testedOutcomes": tested_rules.get(&key).cloned().unwrap_or_default(),
                "fixture": {"pass": f[0], "fail": f[1], "unknown": f[2]},
                "explained": crate::model::filled(&r.rationale),
                "reviewRubricHash": rule_rubric_hash,
                "links": r.links.len(),
                "basis": if judged.contains(&key) { "judged" } else { "measured" },
            });
            if let (Some(l), Some(n)) = (&live, live_entities) {
                let c = l.get(&key).copied().unwrap_or_default();
                data["live"] = json!({"entities": n, "fail": c[1], "unknown": c[2],
                                      "pass": n.saturating_sub(c[1] + c[2])});
            }
            data["hash"] = json!(crate::model::hash(r));
            out.push(("rule".into(), format!("rule:{card_id}/{}", r.name), data));
        }
    }
    out.extend(crate::model::records(&local, &coverage.entities));
    Ok(out)
}

/// Print `scorecard` and `rule` records for each portal; `results` pairs a portal path with its
/// results file (`.seikika=.seikika/results.json`).
pub fn describe(portals: &[PathBuf], results: &[String]) -> Result<(), String> {
    let files: BTreeMap<String, PathBuf> = results
        .iter()
        .filter_map(|s| {
            s.split_once('=')
                .map(|(p, f)| (p.trim_end_matches('/').to_owned(), PathBuf::from(f)))
        })
        .collect();
    for p in portals {
        let key = p.to_string_lossy().trim_end_matches('/').to_owned();
        for (kind, id, data) in describe_portal(p, files.get(&key).map(PathBuf::as_path))? {
            println!(
                "{}",
                json!({"type": "record", "kind": kind, "externalId": id, "data": data})
            );
        }
    }
    Ok(())
}
