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
use std::collections::BTreeMap;
use toolbox_derive::Derived;
use toolbox_model::Manifest;
use toolbox_model::manifest::Scorecard;

/// How many evidence items a cause shows per rule.
const SHOWN: usize = 3;
/// How many it shows when the evidence grew: the new items when the baseline has their keys,
/// else enough that the new item is usually among them.
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

/// An evidence item's text: a string as it is, anything else as JSON.
pub(crate) fn evidence_text(item: &Value) -> String {
    item.as_str()
        .map_or_else(|| item.to_string(), str::to_owned)
}

/// An evidence item's identity in the baseline: a hash of its text with every run of digits
/// folded to `#`, so a finding that only moved (a line or column number, a count in its
/// message) keeps its key. FNV-1a, so the key is the same on every platform and Rust version.
pub(crate) fn evidence_key(item: &Value) -> String {
    let mut h: u32 = 0x811c_9dc5;
    let mut digits = false;
    for b in evidence_text(item).bytes() {
        let was = std::mem::replace(&mut digits, b.is_ascii_digit());
        let b = match (digits, was) {
            (true, true) => continue,
            (true, false) => b'#',
            _ => b,
        };
        h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

/// One rule's evidence on a regression: the items to show, how many more there are, and how
/// many the baseline already had.
pub(crate) struct Shown {
    pub(crate) rule: String,
    /// The baseline had this rule's keys, so `items` are only what is new.
    pub(crate) marked: bool,
    pub(crate) items: Vec<String>,
    pub(crate) more: usize,
    pub(crate) old: usize,
}

impl Shown {
    fn lines(&self) -> Vec<String> {
        let (rule, plus) = (&self.rule, if self.marked { "+ " } else { "" });
        let mut out: Vec<String> = self
            .items
            .iter()
            .map(|t| format!("{rule}: {plus}{t}"))
            .collect();
        if self.more > 0 {
            let new = if self.marked { " new" } else { "" };
            out.push(format!("{rule}: … {} more{new}", self.more));
        }
        if self.old > 0 {
            out.push(format!("{rule}: and {} already in the baseline", self.old));
        }
        out
    }
}

/// A grown list of evidence, against the keys the baseline recorded: the items that are new,
/// then how many were there before. A key that is both (one more of a finding that was already
/// there, at another line) is one item for all of its findings.
fn grown(items: &[Value], known: &[String]) -> (Vec<String>, usize) {
    let mut before: BTreeMap<&str, usize> = BTreeMap::new();
    for k in known {
        *before.entry(k.as_str()).or_default() += 1;
    }
    let mut keys: Vec<String> = Vec::new();
    let mut by_key: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for item in items {
        let k = evidence_key(item);
        if !by_key.contains_key(&k) {
            keys.push(k.clone());
        }
        by_key.entry(k).or_default().push(item);
    }
    let first = |v: &Value| evidence_text(v).lines().next().unwrap_or("").to_owned();
    let (mut new, mut old) = (Vec::new(), 0);
    for k in &keys {
        let all = &by_key[k];
        let had = before.get(k.as_str()).copied().unwrap_or(0);
        match all.len().saturating_sub(had) {
            0 => old += all.len(),
            n if n == all.len() => new.extend(all.iter().map(|v| first(v))),
            n => {
                old += all.len() - n;
                new.push(format!(
                    "{n} more like: {} ({} alike in all)",
                    first(all[0]),
                    all.len()
                ));
            }
        }
    }
    (new, old)
}

/// What each rule of a change found on its entity now: at most `cap` items per rule, the new
/// ones when the baseline recorded the rule's keys.
pub(crate) fn shown(d: &Derived, c: &Change, cap: usize) -> Vec<Shown> {
    let (card, entity, rules) = (c.scorecard.as_str(), c.entity.as_str(), c.rules.as_slice());
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
    for rule in failing {
        let (marked, mut items, old) = match s.evidence.get(&rule) {
            Some(Value::Array(items)) if c.known.is_some() => {
                let (new, old) = grown(items, c.known.as_deref().unwrap_or_default());
                (true, new, old)
            }
            Some(Value::Array(items)) => {
                let first = |v: &Value| evidence_text(v).lines().next().unwrap_or("").to_owned();
                (false, items.iter().map(first).collect(), 0)
            }
            Some(other) => (false, vec![other.to_string()], 0),
            None => continue,
        };
        let more = items.len().saturating_sub(cap);
        items.truncate(cap);
        out.push(Shown {
            rule,
            marked,
            items,
            more,
            old,
        });
    }
    out
}

/// How to see an entity's findings without the box: its `command` field when its source records
/// have one (lint records do, take-it-out), else the record it came from.
fn source(d: &Derived, entity: &str) -> Option<String> {
    let e = d.entities.get(entity)?;
    match e.fields.get("command").and_then(Value::as_str) {
        Some(cmd) => Some(format!("reproduce: {cmd}")),
        None => e
            .records
            .first()
            .map(|(conn, kind, id)| format!("from: connector {conn}, record {kind} {id}")),
    }
}

/// What a rule found on an entity now, and how to see it without the box.
pub(crate) fn detail(d: &Derived, c: &Change) -> Vec<String> {
    let cap = if c.whole_evidence { WHOLE } else { SHOWN };
    let mut out: Vec<String> = shown(d, c, cap).iter().flat_map(Shown::lines).collect();
    out.extend(source(d, &c.entity));
    out
}

/// The ratchet's regressions as its summary lists them: each new finding once, under the entity
/// it is on, after every rule that reports it there or on another entity (zizmor and a pin rule
/// both see an unpinned action; a gold rule's evidence repeats a bronze one's). What the baseline
/// already had is one count at the end. A regression without items of its own (an aggregate, a
/// level, a measure) keeps its full lines. The `--verdict` document keeps every rule's detail.
pub(crate) fn gate(d: &Derived, regressions: &[Change]) -> Vec<String> {
    let mut list = Findings::default();
    let mut rest = Vec::new();
    for c in regressions {
        match new_items(d, c) {
            Some(shown) => list.add(d, c, shown),
            None => rest.extend(c.lines("✗ worse  ", 0)),
        }
    }
    let mut out = list.lines();
    out.extend(rest);
    if list.old > 0 {
        out.push(format!(
            "and {} finding(s) already in the baseline",
            list.old
        ));
    }
    out
}

/// A regression's evidence when it is a list of items that are all new: a rule that now fails,
/// or a grown list the baseline recorded keys for. `None` for anything else.
fn new_items(d: &Derived, c: &Change) -> Option<Vec<Shown>> {
    // A grown list without the baseline's keys can't tell its new items from its old ones.
    let unmarked = c.whole_evidence && c.known.is_none();
    if !c.causes.is_empty() || c.rules.is_empty() || unmarked {
        return None;
    }
    let shown = shown(d, c, WHOLE);
    shown
        .iter()
        .any(|s| !s.items.is_empty() || s.more > 0)
        .then_some(shown)
}

/// How many source lines the gate quotes per entity, and how wide each one is.
const QUOTED: usize = 16;
const QUOTE_WIDTH: usize = 120;
/// Lines between two quoted lines that are shown rather than elided.
const QUOTE_GAP: usize = 5;

/// The new findings of an entity, for the gate: each with the rules that report it.
struct Group {
    entity: String,
    cards: Vec<String>,
    items: Vec<(Vec<String>, String)>,
    reproduce: Option<String>,
    /// The entity's `path` field: the file its `at line N` findings point into.
    path: Option<String>,
}

/// Every `at line N` an item names.
fn lines_named(text: &str) -> Vec<usize> {
    text.match_indices("at line ")
        .filter_map(|(i, m)| {
            let rest = &text[i + m.len()..];
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            rest[..end].parse().ok()
        })
        .filter(|n| *n > 0)
        .collect()
}

/// The lines of `path` (relative to the working directory, where the workspace connector read
/// it) that the findings name, with short gaps filled and long ones elided, so the change can be
/// made without opening the file. Nothing when the file isn't there.
fn quote(path: &str, wanted: &[usize]) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut wanted: Vec<usize> = wanted
        .iter()
        .copied()
        .filter(|n| *n <= lines.len())
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    let mut shown: Vec<usize> = Vec::new();
    for n in wanted {
        if let Some(&last) = shown.last()
            && n > last + 1
            && n - last - 1 <= QUOTE_GAP
        {
            shown.extend(last + 1..n);
        }
        shown.push(n);
    }
    if shown.is_empty() {
        return Vec::new();
    }
    let width = shown.last().map_or(1, |n| n.to_string().len());
    let mut out = vec![format!("    {path}:")];
    let mut prev = None;
    for &n in shown.iter().take(QUOTED) {
        if prev.is_some_and(|p| n > p + 1) {
            out.push(format!("      {:>width$} |", "⋮"));
        }
        let line: String = lines[n - 1].chars().take(QUOTE_WIDTH).collect();
        out.push(format!("      {n:>width$} | {line}"));
        prev = Some(n);
    }
    if shown.len() > QUOTED {
        out.push(format!("      … {} more line(s)", shown.len() - QUOTED));
    }
    out
}

/// Every regression's new findings, each text once: where it was first seen, by group and item.
#[derive(Default)]
struct Findings {
    groups: Vec<Group>,
    seen: BTreeMap<String, (usize, usize)>,
    old: usize,
}

impl Findings {
    fn group(&mut self, d: &Derived, c: &Change) -> usize {
        if let Some(g) = self.groups.iter().position(|g| g.entity == c.entity) {
            return g;
        }
        self.groups.push(Group {
            entity: c.entity.clone(),
            cards: Vec::new(),
            items: Vec::new(),
            reproduce: source(d, &c.entity).filter(|s| s.starts_with("reproduce: ")),
            path: d
                .entities
                .get(&c.entity)
                .and_then(|e| e.fields.get("path"))
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
        self.groups.len() - 1
    }

    fn add(&mut self, d: &Derived, c: &Change, shown: Vec<Shown>) {
        let g = self.group(d, c);
        if !self.groups[g].cards.contains(&c.scorecard) {
            self.groups[g].cards.push(c.scorecard.clone());
        }
        for s in shown {
            self.old += s.old;
            for text in s.items {
                self.item(g, &c.entity, &s.rule, text);
            }
            if s.more > 0 {
                let more = format!("… {} more", s.more);
                self.groups[g].items.push((vec![s.rule.clone()], more));
            }
        }
    }

    /// One finding: a new line, or one more rule on the line that already has its text.
    fn item(&mut self, g: usize, entity: &str, rule: &str, text: String) {
        let Some(&(at, i)) = self.seen.get(&text) else {
            self.seen
                .insert(text.clone(), (g, self.groups[g].items.len()));
            self.groups[g].items.push((vec![rule.to_owned()], text));
            return;
        };
        let rule = match at == g {
            true => rule.to_owned(),
            false => format!("{rule} ({entity})"),
        };
        let rules = &mut self.groups[at].items[i].0;
        if !rules.contains(&rule) {
            rules.push(rule);
        }
    }

    fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        for g in self.groups.iter().filter(|g| !g.items.is_empty()) {
            out.push(format!("✗ worse   {} · {}", g.entity, g.cards.join(", ")));
            for (rules, text) in &g.items {
                out.push(format!("    {}: + {text}", rules.join(", ")));
            }
            out.extend(g.reproduce.as_ref().map(|r| format!("    {r}")));
            if let Some(path) = &g.path {
                let wanted: Vec<usize> = g.items.iter().flat_map(|(_, t)| lines_named(t)).collect();
                if !wanted.is_empty() {
                    out.extend(quote(path, &wanted));
                }
            }
        }
        out
    }
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

    #[test]
    fn evidence_keys_survive_moved_lines() {
        let at = |s: &str| evidence_key(&json!(s));
        assert_eq!(
            at("high cache-poisoning at line 195"),
            at("high cache-poisoning at line 201")
        );
        assert_eq!(at("x:12:3 v1"), at("x:4:56 v2"));
        assert_ne!(
            at("high cache-poisoning at line 195"),
            at("high unpinned-uses at line 195")
        );
        assert_eq!(
            evidence_key(&json!({"line": 3})),
            evidence_key(&json!({"line": 40}))
        );
        assert_eq!(at("").len(), 8);
    }

    #[test]
    fn quotes_the_named_lines_with_short_gaps_filled() {
        assert_eq!(lines_named("high x at line 12: y, and at line 3"), [12, 3]);
        assert!(lines_named("actions/checkout@v4").is_empty());
        let dir = std::env::temp_dir().join(format!("toolbox-quote-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("w.yml");
        let text: String = (1..=30).map(|n| format!("line {n}\n")).collect();
        std::fs::write(&file, text).unwrap();
        let path = file.to_str().unwrap();
        assert_eq!(
            quote(path, &[12, 9, 25, 99]),
            [
                format!("    {path}:"),
                "       9 | line 9".into(),
                "      10 | line 10".into(),
                "      11 | line 11".into(),
                "      12 | line 12".into(),
                "       ⋮ |".into(),
                "      25 | line 25".into(),
            ]
        );
        assert!(quote(path, &[99]).is_empty());
        assert!(quote(&format!("{path}.missing"), &[1]).is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn grown_evidence_marks_only_the_new_items() {
        let items: Vec<Value> = [
            "medium artipacked at line 17",
            "high cache-poisoning at line 201",
            "medium artipacked at line 48",
            "high template-injection at line 230",
            "medium artipacked at line 60",
        ]
        .iter()
        .map(|s| json!(s))
        .collect();
        // The baseline had two artipacked findings (at other lines) and the cache poisoning one.
        let known: Vec<String> = [
            "medium artipacked at line 9",
            "medium artipacked at line 30",
            "high cache-poisoning at line 195",
        ]
        .iter()
        .map(|s| evidence_key(&json!(s)))
        .collect();
        let (new, old) = grown(&items, &known);
        assert_eq!(
            new,
            [
                "1 more like: medium artipacked at line 17 (3 alike in all)",
                "high template-injection at line 230",
            ]
        );
        assert_eq!(old, 3);
        let shown = Shown {
            rule: "zizmor".into(),
            marked: true,
            items: new,
            more: 2,
            old,
        };
        assert_eq!(
            shown.lines(),
            [
                "zizmor: + 1 more like: medium artipacked at line 17 (3 alike in all)",
                "zizmor: + high template-injection at line 230",
                "zizmor: … 2 more new",
                "zizmor: and 3 already in the baseline",
            ]
        );
    }
}
