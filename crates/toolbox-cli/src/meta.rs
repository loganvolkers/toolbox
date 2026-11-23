//! `toolbox describe scorecards`: the scorecards and rules of one or more portals as connector
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

use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use toolbox_model::Manifest;

/// `(scorecard, rule)` pairs and scorecards named by the portal's test expectations.
/// The text of every test expectation (to tell which entities tests reach).
fn expectations(root: &Path, m: &Manifest) -> String {
    m.tests
        .iter()
        .filter_map(|t| std::fs::read_to_string(root.join(t)).ok())
        .collect::<Vec<_>>()
        .join("\n")
}

fn test_mentions(root: &Path, m: &Manifest) -> (BTreeSet<(String, String)>, BTreeSet<String>) {
    let score = regex::Regex::new(r"score\('([^']+)'").expect("regex");
    let rule = regex::Regex::new(r"(?:rules|evidence)\['([^']+)'\]").expect("regex");
    let (mut rules, mut cards) = (BTreeSet::new(), BTreeSet::new());
    for t in &m.tests {
        let Ok(text) = std::fs::read_to_string(root.join(t)) else {
            continue;
        };
        let Ok(doc) = serde_norway::from_str::<Value>(&text) else {
            continue;
        };
        for a in doc["assert"].as_array().into_iter().flatten() {
            let expect = a["expect"].as_str().unwrap_or_default();
            for (card, rules_named) in score_segments(&score, &rule, expect) {
                for r in rules_named {
                    rules.insert((card.clone(), r));
                }
                cards.insert(card);
            }
        }
    }
    (rules, cards)
}

/// Each `score('<card>', …)` call in an expectation with the rules named after it, up to the next
/// `score(`: a rule counts as tested only by an expectation on its own scorecard, not by any card
/// that happens to share the expectation.
fn score_segments(
    score: &regex::Regex,
    rule: &regex::Regex,
    expect: &str,
) -> Vec<(String, Vec<String>)> {
    let starts: Vec<_> = score.captures_iter(expect).collect();
    starts
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let from = c.get(0).map_or(0, |m| m.end());
            let to = starts
                .get(i + 1)
                .and_then(|n| n.get(0))
                .map_or(expect.len(), |m| m.start());
            let named = rule
                .captures_iter(&expect[from..to])
                .map(|r| r[1].to_owned())
                .collect();
            (c[1].to_owned(), named)
        })
        .collect()
}

/// Kinds that describe portals themselves (fed by `describe portal` records).
fn meta_kinds(m: &Manifest) -> BTreeSet<String> {
    // `connector` is also a common record name (e.g. connector crates), so it counts only from a
    // connector that also sends `scorecard` records: the one running `describe portal`.
    const META: [&str; 4] = ["scorecard", "rule", "kind", "field"];
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
                META.contains(&rec) || (rec == "connector" && describers.contains(conn))
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
fn fixture_outcomes(d: &toolbox_derive::Derived) -> (RuleCounts, LevelCounts) {
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

/// Per rule from a `toolbox-report/v1` results file: `[entities scored, failing, unknown]`.
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
    match toolbox_expr::analysis::analyse(expr) {
        Ok(a) => json!({"nodes": a.nodes, "depth": a.depth, "loops": a.comprehension_depth,
                        "fields": a.fields, "functions": a.functions, "undeclared": a.undeclared}),
        Err(e) => json!({"error": e}),
    }
}

fn words(s: Option<&str>) -> usize {
    s.map_or(0, |s| s.split_whitespace().count())
}

/// Records for one portal: `(kind, external id, data)`.
fn describe_portal(
    path: &Path,
    results: Option<&Path>,
) -> Result<Vec<(String, String, Value)>, String> {
    let local = crate::portal::load(path)?;
    let m = &local.manifest;
    let portal = m.portal.name.clone();
    let (tested_rules, tested_cards) = test_mentions(&local.root, m);
    let (fixture, fixture_levels) = fixture_outcomes(&local.derived);
    let meta = meta_kinds(m);
    let meta_cards: BTreeSet<String> = m
        .scorecards
        .iter()
        .filter(|s| meta.contains(&s.kind))
        .map(|s| s.name.clone())
        .collect();
    let live = match results {
        Some(f) => {
            let text = std::fs::read_to_string(f).map_err(|e| format!("{}: {e}", f.display()))?;
            let v: Value =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", f.display()))?;
            Some(live_outcomes(&v, &meta_cards))
        }
        None => None,
    };
    let mut out = Vec::new();
    for sc in &m.scorecards {
        let mut per_level = Map::new();
        for l in &sc.levels {
            per_level.insert(
                l.clone(),
                json!(sc.rules.iter().filter(|r| &r.level == l).count()),
            );
        }
        let card_id = format!("{portal}/{}", sc.name);
        let live_entities = live
            .as_ref()
            .filter(|_| !meta_cards.contains(&sc.name))
            .map(|l| l.get(&(sc.name.clone(), String::new())).map_or(0, |c| c[0]));
        let mut data = json!({
            "portal": portal, "name": sc.name, "kind": sc.kind, "owner": sc.owner,
            "levels": sc.levels, "rulesPerLevel": per_level, "rules": sc.rules.len(),
            "appliesTo": sc.applies_to.is_some(), "tested": tested_cards.contains(&sc.name),
            "meta": meta_cards.contains(&sc.name),
            "fixtureLevels": fixture_levels.get(&sc.name).cloned().unwrap_or_default(),
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
                "tested": tested_rules.contains(&key),
                "fixture": {"pass": f[0], "fail": f[1], "unknown": f[2]},
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
    let expects = expectations(&local.root, m);
    out.extend(crate::model::records(&local, &expects));
    Ok(out)
}

/// Print `scorecard` and `rule` records for each portal; `results` pairs a portal path with its
/// results file (`portal=portal/results.json`).
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

#[cfg(test)]
mod tests {
    #[test]
    fn a_rule_is_tested_only_by_its_own_scorecards_expectation() {
        let score = regex::Regex::new(r"score\('([^']+)'").unwrap();
        let rule = regex::Regex::new(r"(?:rules|evidence)\['([^']+)'\]").unwrap();
        let e = "score('a', 'x:1').rules['r1'] == 'pass' && score('b', 'x:1').level == 'gold'";
        let got = super::score_segments(&score, &rule, e);
        assert_eq!(
            got,
            vec![
                ("a".to_owned(), vec!["r1".to_owned()]),
                ("b".to_owned(), vec![])
            ]
        );
    }
}
