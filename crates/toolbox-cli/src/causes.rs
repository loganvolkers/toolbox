//! Root causes for the ratchet (docs/design/root-causes.md): every regression says what caused
//! it, down to something a person or an agent can act on.
//!
//! - A rule's regression carries its evidence (what the rule found now) and how to see it with
//!   the tool alone: the entity's `command` field when its source records have one (lint records
//!   do, take-it-out), else the record it came from.
//! - An aggregate's regression (a scorecard that reads `cards`, such as a repository level) names
//!   each constituent scorecard whose level fell, and under it the entity changes that are now at
//!   that lower level: a new doc below the floor, a crate whose tests started failing. Those carry
//!   their own evidence, so the chain ends at a file, a line and a command.

use crate::report::Change;
use serde_json::{Map, Value};
use toolbox_derive::Derived;
use toolbox_model::Manifest;
use toolbox_model::manifest::Scorecard;

/// How many evidence items a cause shows per rule.
const SHOWN: usize = 3;
/// How many it shows when the evidence grew: enough that the new item is usually among them.
const WHOLE: usize = 10;

/// `none` is 0, then the scorecard's levels in order.
fn rank(sc: &Scorecard, level: &str) -> Option<usize> {
    if level == "none" {
        return Some(0);
    }
    sc.levels.iter().position(|l| l == level).map(|i| i + 1)
}

/// A scorecard's level as the aggregate sees it: its lowest counted entity (fixture samples
/// under `/demo/` and skipped entities don't count). An empty card of a compact baseline is at
/// its top level: every entity it omits passes.
fn rollup(sc: &Scorecard, rows: &Value) -> Option<(usize, String)> {
    let rows = rows.as_object()?;
    let lowest = rows
        .iter()
        .filter(|(id, row)| !id.contains("/demo/") && row["level"] != "skipped")
        .filter_map(|(_, row)| {
            let level = row["level"].as_str()?;
            Some((rank(sc, level)?, level.to_owned()))
        })
        .min_by_key(|(r, _)| *r);
    lowest.or_else(|| {
        let top = sc.levels.last()?;
        Some((sc.levels.len(), top.clone()))
    })
}

/// What a rule found on an entity now, and how to see it without the box.
pub(crate) fn detail(d: &Derived, c: &Change) -> Vec<String> {
    let (card, entity, rules) = (c.scorecard.as_str(), c.entity.as_str(), c.rules.as_slice());
    let shown = if c.whole_evidence { WHOLE } else { SHOWN };
    let Some(s) = d.scores.get(card).and_then(|rows| rows.get(entity)) else {
        return Vec::new();
    };
    let failing: Vec<String> = match rules {
        [] => s
            .rules
            .iter()
            .flatten()
            .filter(|(_, o)| *o == "fail")
            .map(|(r, _)| r.clone())
            .collect(),
        given => given.to_vec(),
    };
    let mut out = Vec::new();
    for rule in &failing {
        match s.evidence.get(rule) {
            Some(Value::Array(items)) => {
                for item in items.iter().take(shown) {
                    let text = item
                        .as_str()
                        .map_or_else(|| item.to_string(), str::to_owned);
                    out.push(format!("{rule}: {}", text.lines().next().unwrap_or("")));
                }
                if items.len() > shown {
                    out.push(format!("{rule}: … {} more", items.len() - shown));
                }
            }
            Some(other) => out.push(format!("{rule}: {other}")),
            None => {}
        }
    }
    if let Some(e) = d.entities.get(entity) {
        match e.fields.get("command").and_then(Value::as_str) {
            Some(cmd) => out.push(format!("reproduce: {cmd}")),
            None => {
                if let Some((conn, kind, id)) = e.records.first() {
                    out.push(format!("from: connector {conn}, record {kind} {id}"));
                }
            }
        }
    }
    out
}

/// For an aggregate's regression: each constituent scorecard whose level fell between `before`
/// and `now`, with the changes below it that are now at that lower level.
fn aggregate_causes(
    m: &Manifest,
    d: &Derived,
    before: &Map<String, Value>,
    now: &Map<String, Value>,
    candidates: &[Change],
) -> Vec<Change> {
    let mut out = Vec::new();
    for sc in m
        .scorecards
        .iter()
        .filter(|sc| !toolbox_derive::reads_cards(sc))
    {
        let (Some(b), Some(n)) = (
            before.get(&sc.name).and_then(|rows| rollup(sc, rows)),
            now.get(&sc.name).and_then(|rows| rollup(sc, rows)),
        ) else {
            continue;
        };
        if n.0 >= b.0 {
            continue;
        }
        let rows = now.get(&sc.name).and_then(Value::as_object);
        let causes: Vec<Change> = candidates
            .iter()
            .filter(|c| c.scorecard == sc.name)
            .filter(|c| {
                let level = rows
                    .and_then(|r| r.get(&c.entity))
                    .and_then(|r| r["level"].as_str());
                level.and_then(|l| rank(sc, l)).is_some_and(|r| r <= n.0)
            })
            .map(|c| Change {
                detail: detail(d, c),
                ..c.clone()
            })
            .collect();
        let mut fell = Change {
            scorecard: sc.name.clone(),
            entity: "its lowest entity".into(),
            message: format!("fell {} → {}", b.1, n.1),
            causes,
            ..Change::default()
        };
        if fell.causes.is_empty() {
            fell.detail = vec!["no single change below it: an entity at the new level was already there, or one above it was removed".into()];
        }
        out.push(fell);
    }
    out
}

/// Attach root causes to each regression. `candidates` are all the comparison's changes
/// (regressions and other changes), which an aggregate's causes are drawn from.
pub(crate) fn explain(
    regressions: &mut [Change],
    candidates: &[Change],
    before: &Value,
    now: &Value,
    m: &Manifest,
    d: &Derived,
) {
    let empty = Map::new();
    let (b, n) = (
        before.as_object().unwrap_or(&empty),
        now.as_object().unwrap_or(&empty),
    );
    for r in regressions.iter_mut() {
        r.detail = detail(d, r);
        let aggregate = m
            .scorecards
            .iter()
            .find(|sc| sc.name == r.scorecard)
            .is_some_and(toolbox_derive::reads_cards);
        if aggregate {
            r.causes = aggregate_causes(m, d, b, n, candidates);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn card(levels: &[&str]) -> Scorecard {
        serde_json::from_value(json!({
            "name": "docs", "kind": "Doc", "owner": "group:x",
            "levels": levels, "rules": [{"name": "r", "level": levels[0], "check": "true"}]
        }))
        .unwrap()
    }

    #[test]
    fn rollup_is_the_lowest_counted_entity() {
        let sc = card(&["bronze", "silver"]);
        let rows = json!({"doc:a": {"level": "silver"}, "doc:b": {"level": "none"},
                          "doc:demo/x": {"level": "none"}, "doc:c": {"level": "skipped"}});
        // `doc:demo/x` doesn't contain `/demo/`: only paths under a demo directory are samples.
        assert_eq!(rollup(&sc, &rows), Some((0, "none".into())));
        let rows = json!({"doc:a": {"level": "silver"}, "x:/demo/y": {"level": "none"}});
        assert_eq!(rollup(&sc, &rows), Some((2, "silver".into())));
        // A compact card with nothing recorded: everything passes.
        assert_eq!(rollup(&sc, &json!({})), Some((2, "silver".into())));
    }
}
