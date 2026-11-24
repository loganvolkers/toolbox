//! `toolbox report`: score a repository locally, without a server (docs/design/toolbox-for-toolbox.md
//! F1, F4; docs/design/ratchets.md). Runs the portal's in-image connectors once (a resync
//! each), recomputes, and prints every scorecard with the evidence for failing rules — as Markdown,
//! JSON, one self-contained HTML page, or a directory a host page can slot into.
//!
//! With `--baseline FILE` it is a ratchet in the manner of Betterer's `.betterer.results`: the file
//! records each entity's level and failing rules with their evidence counts. A rule that starts
//! failing, or a failing rule whose evidence grows, is a regression (exit 1). `--update` records
//! improvements; `--ci` also fails when the file is out of date, so the committed file always
//! matches the code.

use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};
use toolbox_derive::{Derived, DerivedState, FullRecompute};

#[derive(clap::Args)]
pub struct Options {
    /// A portal directory (containing toolbox.yaml) or a manifest file.
    #[arg(default_value = ".")]
    pub path: PathBuf,
    /// Use the portal's fixtures instead of running its connectors.
    #[arg(long)]
    pub fixtures: bool,
    /// Connectors not to run (comma-separated), e.g. a slow runner.
    #[arg(long, value_delimiter = ',')]
    pub skip: Vec<String>,
    /// Output: md (default), json (scores, ratchet results and campaigns), failures (a JSON
    /// list of every failing rule: scorecard, rule, entity, path, the entity's scalar fields,
    /// measure, evidence, fix),
    /// html (one self-contained page), or web (a directory: the `<toolbox-report>` element,
    /// its JSON, and that page). A host fills the element's header, nav, and footer slots;
    /// pass --out DIR for web. With --baseline or --against and neither -o nor --out, the
    /// verdict is the output, on stdout: a gate says what changed, not every scorecard.
    #[arg(short = 'o', long)]
    pub output: Option<String>,
    /// Write the output to this file instead of stdout.
    #[arg(long)]
    pub out: Option<PathBuf>,
    /// Also write the one-page HTML report to this file (from the same run).
    #[arg(long)]
    pub html: Option<PathBuf>,
    /// Results file to compare against (the ratchet).
    #[arg(long)]
    pub baseline: Option<PathBuf>,
    /// With --baseline: write the new results when nothing regressed (creates a missing file).
    #[arg(long)]
    pub update: bool,
    /// With --baseline: fail when the results differ from the file in any way.
    #[arg(long)]
    pub ci: bool,
    /// With --baseline: fail (exit 1) when a rule's outcome is unknown although its connector ran
    /// (not excused by --skip), such as a kit whose `optional: true` tool isn't installed. A plain
    /// check passes then, so a gate that must not run without its tools should pass this.
    #[arg(long)]
    pub fail_on_unknown: bool,
    /// With --update: record regressions in these scorecards (comma-separated) instead of
    /// failing, for an intended change to a standard (a tighter threshold, a new pattern).
    /// They are listed; regressions in other scorecards still fail.
    #[arg(long, value_delimiter = ',')]
    pub accept: Vec<String>,
    /// With --accept: keep each campaign's recorded start, so the accepted findings show as the
    /// campaign going backwards (a tool upgrade or a new tool that reports more). Without it,
    /// --accept restarts the campaigns over those scorecards (a changed standard).
    #[arg(long, requires = "accept")]
    pub keep_campaigns: bool,
    /// Write the ratchet's verdict as JSON to this file (result, exit code, regressions,
    /// improvements), for any CI system to read.
    #[arg(long)]
    pub verdict: Option<PathBuf>,
    /// Markdown: list every entity, not only those with a failing or unknown rule.
    #[arg(long)]
    pub all: bool,
    /// Compare with the portal's results at this git commit (e.g. HEAD~1, or a pull request's
    /// base), scored in a temporary worktree, instead of with a baseline file: what this change
    /// made worse or better. Exit 1 on a regression; nothing is written.
    #[arg(long, conflicts_with_all = ["baseline", "update", "ci", "accept"])]
    pub against: Option<String>,
}

const FORMAT: &str = "toolbox-report/v1";

/// Run one in-image connector's resync and return its record and delete messages.
fn resync(
    name: &str,
    program: &str,
    config: &Value,
    kinds: &[String],
) -> Result<Vec<Value>, String> {
    let mut child = Command::new(program)
        .env("TOOLBOX_CONNECTOR_CONFIG", config.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("connector {name}: cannot start {program}: {e}"))?;
    // The kinds the manifest declares: some connectors (github) resync only the kinds named.
    let cmd =
        json!({"type": "command", "command": "resync", "command_id": "report", "kinds": kinds});
    child
        .stdin
        .take()
        .ok_or("no stdin")?
        .write_all(format!("{cmd}\n").as_bytes())
        .map_err(|e| format!("connector {name}: {e}"))?;
    let mut out = Vec::new();
    for line in BufReader::new(child.stdout.take().ok_or("no stdout")?).lines() {
        let line = line.map_err(|e| format!("connector {name}: {e}"))?;
        let Ok(mut m) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match m["type"].as_str() {
            Some("record" | "delete") => {
                if let Some(o) = m.as_object_mut() {
                    o.remove("command_id");
                }
                out.push(m);
            }
            Some("complete") if m["status"] != "succeeded" => {
                let _ = child.kill();
                return Err(format!(
                    "connector {name}: resync {}: {}",
                    m["status"], m["errors"]
                ));
            }
            Some("complete") => break,
            _ => {}
        }
    }
    let _ = child.wait();
    Ok(out)
}

/// Inputs from the connectors (in parallel) or the fixtures.
fn inputs(
    fixtures: bool,
    skip: &[String],
    root: &std::path::Path,
    m: &toolbox_model::Manifest,
) -> Result<toolbox_store::MemStore, String> {
    if fixtures {
        return toolbox_store::load_fixtures(root).map_err(|e| e.to_string());
    }
    let jobs: Vec<_> = m
        .connectors
        .iter()
        .filter(|c| c.runtime == "in-image" && !skip.contains(&c.name))
        .collect();
    for c in m.connectors.iter().filter(|c| c.runtime != "in-image") {
        eprintln!(
            "toolbox report: skipping remote connector {} (it pushes to a server)",
            c.name
        );
    }
    let results: Vec<Result<(String, Vec<Value>), String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = jobs
            .iter()
            .map(|c| {
                scope.spawn(move || {
                    let program = toolbox_api::connectors::binary_for(&c.package);
                    let config = toolbox_api::connectors::resolve_config(
                        &c.name,
                        c.config.as_ref().unwrap_or(&Value::Null),
                    );
                    resync(&c.name, &program, &config, &c.records).map(|r| (c.name.clone(), r))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err("connector thread panicked".into()))
            })
            .collect()
    });
    let mut store = toolbox_store::MemStore::new();
    for r in results {
        let (name, batch) = r?;
        store.ingest(&name, &batch);
    }
    Ok(store)
}

/// The ratchet's view of the results: level and failing rules with evidence counts (null when the
/// evidence isn't a list), per scorecard and entity.
fn results(d: &Derived, advisory: &BTreeSet<String>) -> Value {
    let mut cards = Map::new();
    for (card, entities) in d.scores.iter().filter(|(c, _)| !advisory.contains(*c)) {
        let mut rows = Map::new();
        for (id, s) in entities.iter().filter(|(id, _)| !is_peer(d, card, id)) {
            let failing: Map<String, Value> = s
                .rules
                .iter()
                .flatten()
                .filter(|(_, o)| *o == "fail")
                .map(|(r, _)| {
                    // The rule's measure when it has one; otherwise how many evidence items.
                    let n = match s.measures.get(r) {
                        Some(x) => number(*x),
                        None => json!(s.evidence.get(r).and_then(Value::as_array).map(Vec::len)),
                    };
                    (r.clone(), n)
                })
                .collect();
            // `unknown`: the rule couldn't be evaluated (its data is missing). Never a regression
            // or an improvement. `--ci` refuses it unless the rule's `dataFrom` connector was skipped.
            let unknown: Vec<&String> = s
                .rules
                .iter()
                .flatten()
                .filter(|(_, o)| *o == "unknown")
                .map(|(r, _)| r)
                .collect();
            let mut row = json!({"level": s.level});
            if !failing.is_empty() {
                row["failing"] = Value::Object(failing);
            }
            if !unknown.is_empty() {
                row["unknown"] = json!(unknown);
            }
            let waived: Vec<&String> = s
                .rules
                .iter()
                .flatten()
                .filter(|(_, o)| *o == "waived")
                .map(|(r, _)| r)
                .collect();
            if !waived.is_empty() {
                row["waived"] = json!(waived);
            }
            rows.insert(id.clone(), row);
        }
        cards.insert(card.clone(), Value::Object(rows));
    }
    Value::Object(cards)
}

/// A peer in a comparative scorecard: another product's facts, shown beside the subject's but
/// never this portal's debt, so never in the baseline, the ratchet or the health views.
fn is_peer(d: &Derived, card: &str, id: &str) -> bool {
    d.comparisons
        .get(card)
        .is_some_and(|c| c.peers.iter().any(|p| p == id))
}

/// Scores without the peers of comparative scorecards (see `is_peer`).
fn own_scores(d: &Derived) -> Map<String, Value> {
    d.scores
        .iter()
        .map(|(card, rows)| {
            let rows: Map<String, Value> = rows
                .iter()
                .filter(|(id, _)| !is_peer(d, card, id))
                .map(|(id, s)| (id.clone(), json!(s)))
                .collect();
            (card.clone(), Value::Object(rows))
        })
        .collect()
}

/// Per scorecard, entity and failing rule counted by its evidence: each item's key
/// (crate::causes::evidence_key), sorted. The baseline keeps them beside `results`, so a list
/// that grew can say which items are new. Not compared: a key-only difference is never a
/// regression and never makes the baseline stale.
fn evidence_keys(d: &Derived, advisory: &BTreeSet<String>) -> Value {
    let mut cards = Map::new();
    for (card, entities) in d.scores.iter().filter(|(c, _)| !advisory.contains(*c)) {
        let mut rows = Map::new();
        for (id, s) in entities.iter().filter(|(id, _)| !is_peer(d, card, id)) {
            let rules: Map<String, Value> = s
                .rules
                .iter()
                .flatten()
                .filter(|(r, o)| *o == "fail" && !s.measures.contains_key(*r))
                .filter_map(|(r, _)| {
                    let items = s.evidence.get(r)?.as_array().filter(|a| !a.is_empty())?;
                    let mut keys: Vec<String> =
                        items.iter().map(crate::causes::evidence_key).collect();
                    keys.sort();
                    Some((r.clone(), json!(keys)))
                })
                .collect();
            if !rules.is_empty() {
                rows.insert(id.clone(), Value::Object(rules));
            }
        }
        if !rows.is_empty() {
            cards.insert(card.clone(), Value::Object(rows));
        }
    }
    Value::Object(cards)
}

/// Give each regression whose evidence grew the keys `before` recorded for that rule.
fn mark_known(regressions: &mut [Change], before: &Value) {
    for r in regressions.iter_mut().filter(|r| r.whole_evidence) {
        let [rule] = r.rules.as_slice() else { continue };
        if let Some(keys) = before[&r.scorecard][&r.entity][rule].as_array() {
            r.known = Some(
                keys.iter()
                    .filter_map(|k| k.as_str().map(str::to_owned))
                    .collect(),
            );
        }
    }
}

/// A JSON number, integral when it can be (line counts stay `1070`, not `1070.0`).
fn number(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 9.0e15 {
        json!(x as i64)
    } else {
        json!(x)
    }
}

/// How many rule outcomes in `results` are `unknown`.
fn unknown_count(results: &Value) -> usize {
    results
        .as_object()
        .into_iter()
        .flatten()
        .flat_map(|(_, rows)| rows.as_object().into_iter().flatten())
        .map(|(_, row)| row["unknown"].as_array().map_or(0, Vec::len))
        .sum()
}

#[derive(Default)]
struct Comparison {
    regressions: Vec<Change>,
    improvements: Vec<Change>,
    other: Vec<Change>,
}

/// One difference from the baseline, on one entity of one scorecard. A regression also carries
/// its root cause (crate::causes): the rule's evidence and how to reproduce it, and for an
/// aggregate (a repository level) the constituent scorecards that fell and the entities that
/// made them fall.
#[derive(Debug, Clone, Default)]
pub(crate) struct Change {
    pub(crate) scorecard: String,
    pub(crate) entity: String,
    pub(crate) message: String,
    /// The rules this change is about (empty: every failing rule of the entity).
    pub(crate) rules: Vec<String>,
    /// What the rule found and how to see it with the tool alone.
    pub(crate) detail: Vec<String>,
    /// For an aggregate: the changes below it that caused it.
    pub(crate) causes: Vec<Change>,
    /// Show every evidence item, not the first few: the evidence grew, and the baseline keeps
    /// only counts, so the new item can be anywhere in the list.
    pub(crate) whole_evidence: bool,
    /// The keys of the rule's evidence in the baseline (crate::causes::evidence_key), when it
    /// recorded them: the detail then marks the new items instead of showing the whole list.
    pub(crate) known: Option<Vec<String>>,
}

impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} · {}: {}", self.scorecard, self.entity, self.message)
    }
}

impl Change {
    pub(crate) fn to_json(&self) -> Value {
        let mut v =
            json!({"scorecard": self.scorecard, "entity": self.entity, "message": self.message});
        if !self.detail.is_empty() {
            v["detail"] = json!(self.detail);
        }
        if !self.causes.is_empty() {
            v["causes"] = Value::Array(self.causes.iter().map(Change::to_json).collect());
        }
        v
    }

    /// The change, then its detail and causes, indented under it.
    pub(crate) fn lines(&self, mark: &str, depth: usize) -> Vec<String> {
        let pad = "  ".repeat(depth);
        let head = if depth == 0 {
            format!("{mark} {self}")
        } else {
            format!("{pad}↳ {self}")
        };
        let mut out = vec![head];
        for d in &self.detail {
            out.push(format!("{pad}    {d}"));
        }
        for c in &self.causes {
            out.extend(c.lines(mark, depth + 1));
        }
        out
    }
}

/// Where a change is: a scorecard and an entity.
struct At<'a> {
    card: &'a str,
    id: &'a str,
}

impl At<'_> {
    fn say(&self, message: String) -> Change {
        Change {
            scorecard: self.card.to_owned(),
            entity: self.id.to_owned(),
            message,
            ..Change::default()
        }
    }

    /// A change about one rule.
    fn about(&self, rule: &str, message: String) -> Change {
        Change {
            rules: vec![rule.to_owned()],
            ..self.say(message)
        }
    }
}

/// Per scorecard: its levels, and the level new entities must reach (`ratchet.newEntitiesAtLeast`).
type NewPolicy = BTreeMap<String, (Vec<String>, String)>;

fn new_policy(m: &toolbox_model::Manifest) -> NewPolicy {
    m.scorecards
        .iter()
        .filter_map(|sc| {
            let min = sc.ratchet.as_ref()?.get("newEntitiesAtLeast")?.as_str()?;
            Some((sc.name.clone(), (sc.levels.clone(), min.to_owned())))
        })
        .collect()
}

/// Scorecards whose new-entity floor is their top level: every entity must pass every rule, so
/// the baseline records only those that don't. A file that is clean, new or not, adds nothing to
/// it, so adding clean files never makes the baseline stale.
fn compact_cards(m: &toolbox_model::Manifest) -> BTreeSet<String> {
    m.scorecards
        .iter()
        .filter(|sc| {
            let floor = sc
                .ratchet
                .as_ref()
                .and_then(|r| r["newEntitiesAtLeast"].as_str());
            floor.is_some() && floor == sc.levels.last().map(String::as_str)
        })
        .map(|sc| sc.name.clone())
        .collect()
}

/// The results as the baseline stores them: compact scorecards keep only entities with a
/// failing, unknown or waived rule.
fn compacted(now: &Value, compact: &BTreeSet<String>) -> Value {
    let mut out = now.clone();
    for card in compact {
        if let Some(rows) = out.get_mut(card).and_then(Value::as_object_mut) {
            rows.retain(|_, row| {
                row.get("failing").is_some()
                    || row.get("unknown").is_some()
                    || row.get("waived").is_some()
            });
        }
    }
    out
}

/// Scorecards with `ratchet: {advisory: true}`: reported, never in the baseline or the comparison
/// (judgement-based cards, whose assessments go stale when their subject changes).
fn advisory(m: &toolbox_model::Manifest) -> BTreeSet<String> {
    m.scorecards
        .iter()
        .filter(|sc| sc.ratchet.as_ref().and_then(|r| r["advisory"].as_bool()) == Some(true))
        .map(|sc| sc.name.clone())
        .collect()
}

/// `none` is 0, then the scorecard's levels in order; `skipped` never falls short.
fn rank(levels: &[String], level: &str) -> Option<usize> {
    if level == "none" {
        return Some(0);
    }
    levels.iter().position(|l| l == level).map(|i| i + 1)
}

#[cfg(test)]
fn compare(before: &Value, after: &Value, policy: &NewPolicy) -> Comparison {
    compare_with(before, after, policy, &BTreeSet::new())
}

/// `compare`, where `compact` scorecards' baselines omit passing entities: an entity missing
/// from the baseline that passes now is no change, and one that fails is held to the floor.
fn compare_with(
    before: &Value,
    after: &Value,
    policy: &NewPolicy,
    compact: &BTreeSet<String>,
) -> Comparison {
    let mut c = Comparison::default();
    let empty = Map::new();
    let cards: std::collections::BTreeSet<&String> = before
        .as_object()
        .unwrap_or(&empty)
        .keys()
        .chain(after.as_object().unwrap_or(&empty).keys())
        .collect();
    for card in cards {
        let (b, a) = (obj(&before[card]), obj(&after[card]));
        let is_compact = compact.contains(card.as_str());
        for (id, now) in &a {
            let at = At { card, id };
            let clean = now.get("failing").is_none() && now.get("unknown").is_none();
            if is_compact && !b.contains_key(id) {
                // Compact baselines omit clean entities, so this one was clean or didn't exist.
                if !clean {
                    let rules: Vec<String> = obj(&now["failing"]).keys().cloned().collect();
                    c.regressions.push(at.say(format!(
                        "{} now fails (it was clean, or is new)",
                        rules.join(", ")
                    )));
                }
                continue;
            }
            match b.get(id) {
                // A scorecard new to the baseline baselines everything first (as Betterer's first
                // run does); the floor applies to entities that appear after that.
                None => new_entity(
                    &mut c,
                    &at,
                    now,
                    policy
                        .get(card.as_str())
                        .filter(|_| before.get(card).is_some()),
                ),
                Some(was) => compare_entity(&mut c, &at, was, now),
            }
        }
        for id in b.keys().filter(|id| !a.contains_key(*id)) {
            let at = At { card, id };
            if is_compact {
                // A tool reports only files with findings (SARIF without `artifacts`): gone means
                // its last finding was fixed, or the file was removed.
                c.improvements
                    .push(at.say("no longer reported (fixed, or the file is gone)".into()));
            } else {
                c.other.push(at.say("removed".into()));
            }
        }
    }
    c
}

fn obj(v: &Value) -> Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|x| x.as_str().map(str::to_owned))
        .collect()
}

/// An entity the baseline doesn't have: a regression when it starts below the scorecard's floor.
fn new_entity(c: &mut Comparison, at: &At, now: &Value, floor: Option<&(Vec<String>, String)>) {
    let level = now["level"].as_str().unwrap_or("?");
    let short = floor.and_then(|(levels, min)| {
        let (have, need) = (rank(levels, level)?, rank(levels, min)?);
        (have < need).then_some(min)
    });
    match short {
        Some(min) => c
            .regressions
            .push(at.say(format!("new, at {level}; new entities must reach {min}"))),
        None => c.other.push(at.say(format!("new ({level})"))),
    }
}

/// One entity in both: rules that started failing or whose evidence grew are worse; unknown
/// outcomes on either side are reported but not compared.
fn compare_entity(c: &mut Comparison, at: &At, was: &Value, now: &Value) {
    if now["level"] == "skipped" && was["level"] != "skipped" {
        // Exempted or out of scope: not fixed, so neither better nor worse.
        c.other
            .push(at.say("now skipped (exempt or out of scope)".into()));
        return;
    }
    let (wf, nf) = (obj(&was["failing"]), obj(&now["failing"]));
    let (wu, nu) = (strings(&was["unknown"]), strings(&now["unknown"]));
    for rule in nu.iter().filter(|r| !wu.contains(r)) {
        c.other
            .push(at.say(format!("{rule} unknown now (its data is missing)")));
    }
    for (rule, n) in &nf {
        if wu.contains(rule) {
            c.other
                .push(at.about(rule, format!("{rule} known again: fails")));
        } else if let Some(m) = wf.get(rule) {
            evidence_change(c, at, rule, m, n);
        } else if was["level"] == "skipped" {
            c.other
                .push(at.about(rule, format!("no longer skipped; {rule} fails")));
        } else {
            c.regressions
                .push(at.about(rule, format!("{rule} now fails")));
        }
    }
    for rule in wf
        .keys()
        .filter(|r| !nf.contains_key(*r) && !nu.contains(r))
    {
        c.improvements.push(at.say(format!("{rule} now passes")));
    }
    if was["level"] != now["level"] && wf.keys().eq(nf.keys()) {
        c.other
            .push(at.say(format!("level {} → {}", was["level"], now["level"])));
    }
}

/// A rule failing on both sides: its evidence count may have grown (worse) or shrunk (better).
fn evidence_change(c: &mut Comparison, at: &At, rule: &str, was: &Value, now: &Value) {
    match (was.as_f64(), now.as_f64()) {
        (Some(m), Some(n)) if n > m => c.regressions.push(Change {
            whole_evidence: true,
            ..at.about(rule, format!("{rule} evidence grew {was} → {now}"))
        }),
        (Some(m), Some(n)) if n < m => c
            .improvements
            .push(at.about(rule, format!("{rule} evidence shrank {was} → {now}"))),
        _ => {}
    }
}

/// Each campaign's starting point, as the baseline records it: kept while the campaign's `start`
/// is unchanged, recorded from this run the first time the campaign is active. Changing `start`
/// restarts the campaign from the current state, and so does `--accept` on a scorecard the
/// campaign includes: an intended change to a standard changes what the campaign pays down. A
/// restarted campaign keeps the start it replaced under `restartedFrom`, so its history isn't lost.
fn campaign_starts(
    m: &toolbox_model::Manifest,
    d: &Derived,
    before: Option<&Value>,
    accept: &[String],
) -> Map<String, Value> {
    let mut out = Map::new();
    for c in &m.campaigns {
        let start = json!(c.start);
        let restarted = c.includes.iter().any(|i| accept.contains(&i.scorecard));
        if let Some(was) = before.and_then(|b| b.get(&c.name))
            && was["start"] == start
            && !restarted
        {
            out.insert(c.name.clone(), was.clone());
            continue;
        }
        let Some(p) = d.campaigns.get(&c.name).filter(|p| p.status == "active") else {
            continue;
        };
        let mut rec = json!({
            "start": start,
            "recorded": toolbox_derive::utc_ymd_today(),
            "inScope": p.in_scope,
            "short": p.in_scope - p.meeting
        });
        if let Some(x) = p.measure {
            rec["measure"] = number(x);
        }
        if let Some(was) = before.and_then(|b| b.get(&c.name)) {
            let mut from = was.clone();
            if let Some(o) = from.as_object_mut() {
                o.remove("restartedFrom"); // one level of history is enough to read a restart
            }
            rec["restartedFrom"] = from;
        }
        out.insert(c.name.clone(), rec);
    }
    out
}

/// `from` → `now`, and how far that is towards zero. Negative is backwards.
fn progress_towards_zero(from: f64, now: f64) -> Value {
    let percent = if from > 0.0 {
        ((from - now) / from * 1000.0).round() / 10.0
    } else if now > 0.0 {
        0.0
    } else {
        100.0
    };
    json!({"from": number(from), "now": number(now), "percentDone": percent})
}

/// How far a campaign has come since its recorded start: in entities short of the goal, and in
/// its rules' measure when they have one (occurrences, lines over a limit).
fn paydown(start: &Value, p: &toolbox_derive::CampaignProgress) -> Value {
    let mut out = json!({
        "start": start["start"],
        "recorded": start["recorded"],
        "entitiesShort": progress_towards_zero(
            start["short"].as_f64().unwrap_or(0.0),
            (p.in_scope - p.meeting) as f64
        ),
    });
    if let (Some(a), Some(b)) = (start["measure"].as_f64(), p.measure) {
        out["measure"] = progress_towards_zero(a, b);
    }
    out
}

/// One line per campaign with a recorded start, for the ratchet's summary.
fn paydown_lines(starts: &Map<String, Value>, d: &Derived) -> Vec<String> {
    let part = |label: &str, x: &Value| {
        format!(
            "{label} {} → {} ({}% of the way)",
            x["from"], x["now"], x["percentDone"]
        )
    };
    starts
        .iter()
        .filter_map(|(name, start)| {
            let p = paydown(start, d.campaigns.get(name)?);
            let mut parts = vec![part("entities short", &p["entitiesShort"])];
            if p.get("measure").is_some() {
                parts.push(part("measure", &p["measure"]));
            }
            Some(format!(
                "campaign {name} since {}: {}",
                p["recorded"].as_str().unwrap_or("?"),
                parts.join("; ")
            ))
        })
        .collect()
}

/// Markdown inline code that survives backticks in the text: a fence one longer than the longest
/// run inside, padded with spaces when the text starts or ends with a backtick (CommonMark 6.1).
fn inline_code(text: &str) -> String {
    let longest = text.split(|c| c != '`').map(str::len).max().unwrap_or(0);
    let fence = "`".repeat(longest + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{text}{pad}{fence}")
}

fn markdown(m: &toolbox_model::Manifest, d: &Derived, all: bool) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# {} — scorecards",
        m.portal.title.as_deref().unwrap_or(&m.portal.name)
    );
    let judged = crate::judged::judged_rules(m);
    if !judged.is_empty() {
        let _ = writeln!(s, "\n{}", crate::judged::LEGEND);
    }
    for sc in &m.scorecards {
        let rows = d.scores.get(&sc.name).cloned().unwrap_or_default();
        let mut tally: BTreeMap<&str, usize> = BTreeMap::new();
        for r in rows.values() {
            *tally.entry(r.level.as_str()).or_default() += 1;
        }
        let counts: Vec<String> = tally.iter().map(|(l, n)| format!("{l}: {n}")).collect();
        // A scorecard with no entities (a tool with no findings, whose kit records only files with
        // findings) says so, instead of printing an empty "()".
        let counts = if counts.is_empty() {
            "no entities".to_owned()
        } else {
            counts.join(", ")
        };
        if let Some(c) = d.comparisons.get(&sc.name) {
            comparison_md(&mut s, sc, c, &rows, d);
            continue;
        }
        let n = judged.iter().filter(|(c, _)| *c == sc.name).count();
        let basis = match n {
            0 => String::new(),
            n if n == sc.rules.len() => {
                format!(" · every rule agent-judged {}", crate::judged::MARK)
            }
            n => format!(
                " · {n} of {} rules agent-judged {}",
                sc.rules.len(),
                crate::judged::MARK
            ),
        };
        let _ = writeln!(s, "\n## {}  ({counts}){basis}\n", sc.name);
        let shown = |r: &&toolbox_derive::Score| {
            r.level != "skipped" && (all || r.rules.iter().flatten().any(|(_, o)| o != "pass"))
        };
        let quiet = rows
            .values()
            .filter(|r| r.level != "skipped" && !shown(r))
            .count();
        if quiet > 0 {
            let _ = writeln!(
                s,
                "{quiet} pass every rule (not listed; `--all` lists them).\n"
            );
        }
        let name = |rule: &str| crate::judged::marked(&judged, &sc.name, rule);
        for (id, r) in rows.iter().filter(|(_, r)| shown(r)) {
            entity_md(&mut s, id, r, &name);
        }
    }
    s
}

/// One entity's line in the markdown report: its level, the rules it fails and up to five items
/// of each rule's evidence. `name` is how a rule is named (crate::judged::marked).
fn entity_md(s: &mut String, id: &str, r: &toolbox_derive::Score, name: &dyn Fn(&str) -> String) {
    use std::fmt::Write as _;
    let failing: Vec<String> = r
        .rules
        .iter()
        .flatten()
        .filter(|(_, o)| *o != "pass")
        .map(|(n, _)| name(n))
        .collect();
    let _ = writeln!(
        s,
        "- **{}** `{id}`{}",
        r.level,
        if failing.is_empty() {
            String::new()
        } else {
            format!(": {}", failing.join(", "))
        }
    );
    for (rule, ev) in &r.evidence {
        let rule = name(rule);
        let items: Vec<String> = match ev {
            Value::Array(xs) => xs
                .iter()
                .map(|x| x.as_str().map_or_else(|| x.to_string(), str::to_owned))
                .collect(),
            other => vec![other.to_string()],
        };
        for item in items.iter().take(5) {
            let _ = writeln!(s, "  - {rule}: {}", item.lines().next().unwrap_or(""));
        }
        if items.len() > 5 {
            let _ = writeln!(s, "  - {rule}: … {} more", items.len() - 5);
        }
    }
}

/// Comparisons as the web report reads them: the compared entities' titles, scores and facts,
/// the fields each rule reads, and every field's description, so a reader sees what was
/// measured, and how, without the docs.
fn comparison_defs(m: &toolbox_model::Manifest, d: &Derived) -> Map<String, Value> {
    d.comparisons
        .iter()
        .map(|(name, c)| {
            let mut v = json!(c);
            let ids = || std::iter::once(&c.subject).chain(&c.peers);
            v["titles"] = ids()
                .map(|id| (id.clone(), json!(title_of(d, id))))
                .collect::<Map<String, Value>>()
                .into();
            v["facts"] = ids()
                .filter_map(|id| Some((id.clone(), json!(d.entities.get(id)?.fields))))
                .collect::<Map<String, Value>>()
                .into();
            let sc = m.scorecards.iter().find(|s| &s.name == name);
            let kind = sc.and_then(|sc| m.kinds.iter().find(|k| k.name == sc.kind));
            if let (Some(sc), Some(kind)) = (sc, kind) {
                let reads: Map<String, Value> = sc
                    .rules
                    .iter()
                    .map(|r| {
                        let mut fields: Vec<String> = std::iter::once(r.check.as_str())
                            .chain(r.value.as_deref())
                            .filter_map(|cel| toolbox_expr::analysis::analyse(cel).ok())
                            .flat_map(|a| a.fields)
                            .filter(|f| kind.fields.contains_key(f))
                            .collect();
                        fields.sort();
                        fields.dedup();
                        (r.name.clone(), json!(fields))
                    })
                    .collect();
                v["reads"] = reads.into();
                v["fields"] = kind
                    .fields
                    .iter()
                    .map(|(f, fd)| (f.clone(), json!(fd.description.as_deref().map(str::trim))))
                    .collect::<Map<String, Value>>()
                    .into();
            }
            let scores = d.scores.get(name);
            v["scores"] = ids()
                .filter_map(|id| Some((id.clone(), json!(scores?.get(id)?))))
                .collect::<Map<String, Value>>()
                .into();
            (name.clone(), v)
        })
        .collect()
}

/// An entity's display name: its `title` field, else its id.
fn title_of<'a>(d: &'a Derived, id: &'a str) -> &'a str {
    d.entities
        .get(id)
        .and_then(|e| e.fields.get("title"))
        .and_then(Value::as_str)
        .unwrap_or(id)
}

/// A comparative scorecard as one table: a row per rule, a column per entity (outcome and
/// value), and the subject's standing against each peer.
fn comparison_md(
    s: &mut String,
    sc: &toolbox_model::manifest::Scorecard,
    c: &toolbox_derive::Comparison,
    rows: &BTreeMap<String, toolbox_derive::Score>,
    d: &Derived,
) {
    use std::fmt::Write as _;
    let subject = title_of(d, &c.subject);
    let verdicts: Vec<String> = c
        .peers
        .iter()
        .map(|p| format!("{} against {}", c.verdict[p], title_of(d, p)))
        .collect();
    let _ = writeln!(
        s,
        "\n## {}  ({subject}: {}{})\n",
        sc.name,
        verdicts.join("; "),
        if c.verdict_rule == "majority" {
            ", by the majority of rules"
        } else {
            ""
        }
    );
    if let Some(t) = &sc.title {
        let _ = writeln!(s, "{t}\n");
    }
    let ids: Vec<&String> = std::iter::once(&c.subject).chain(&c.peers).collect();
    let mut head = String::from("| Rule |");
    let mut rule_line = String::from("|---|");
    for id in &ids {
        let _ = write!(head, " {} |", title_of(d, id));
        rule_line.push_str("---|");
    }
    for p in &c.peers {
        let _ = write!(head, " vs {} |", title_of(d, p));
        rule_line.push_str("---|");
    }
    let _ = writeln!(s, "{head}\n{rule_line}");
    for r in &sc.rules {
        let mut line = format!("| {} |", r.name);
        for id in &ids {
            let score = rows.get(*id);
            let outcome = score
                .and_then(|x| x.rules.as_ref()?.get(&r.name))
                .map_or("—", String::as_str);
            let cell = match score.and_then(|x| x.values.get(&r.name)) {
                Some(v) => format!(
                    "{outcome}, {}{}",
                    number(*v),
                    r.unit
                        .as_deref()
                        .map(|u| format!(" {u}"))
                        .unwrap_or_default()
                ),
                None => outcome.to_owned(),
            };
            let _ = write!(line, " {cell} |");
        }
        for p in &c.peers {
            let standing = c
                .rules
                .get(&r.name)
                .and_then(|m| m.get(p))
                .map_or("unknown", String::as_str);
            let _ = write!(line, " {standing} |");
        }
        let _ = writeln!(s, "{line}");
    }
}

/// The `include` entries of the portal's own manifest file (packages and their parameters).
fn includes(path: &std::path::Path) -> Vec<Value> {
    let file = if path.is_dir() {
        path.join("toolbox.yaml")
    } else {
        path.to_path_buf()
    };
    let doc: Value = std::fs::read_to_string(file)
        .ok()
        .and_then(|t| serde_norway::from_str(&t).ok())
        .unwrap_or(Value::Null);
    doc["include"].as_array().cloned().unwrap_or_default()
}

/// One scorecard's section of STANDARD.md: what it is and its rules. No counts: the file is a
/// function of the manifest, so it can be committed and checked like the baseline.
fn standard_scorecard(
    s: &mut String,
    sc: &toolbox_model::manifest::Scorecard,
    judged: &BTreeSet<(String, String)>,
) {
    use std::fmt::Write as _;
    let _ = writeln!(s, "## {}\n", sc.title.as_deref().unwrap_or(&sc.name));
    if let Some(desc) = &sc.description {
        let _ = writeln!(s, "{}\n", desc.trim());
    }
    if let Some(why) = &sc.rationale {
        let _ = writeln!(
            s,
            "**Why:** {}\n",
            why.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }
    if !sc.links.is_empty() {
        let _ = writeln!(s, "See {}.\n", md_links(&sc.links));
    }
    let _ = writeln!(
        s,
        "Scorecard `{}` over `{}` entities, owned by {}. `toolbox report` lists what fails now. \
         *Measured* rules check facts a tool or connector read; *judged* rules pass on an agent's \
         assessment (`toolbox assess`).\n",
        sc.name, sc.kind, sc.owner
    );
    let _ = writeln!(
        s,
        "| Rule | Level | Basis | Passes when | Why | Fix |\n|---|---|---|---|---|---|"
    );
    let cell = |t: &str| {
        t.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace('|', "\\|")
    };
    for r in &sc.rules {
        let basis = if judged.contains(&(sc.name.clone(), r.name.clone())) {
            "judged"
        } else {
            "measured"
        };
        let mut why = cell(r.rationale.as_deref().unwrap_or(""));
        if !r.links.is_empty() {
            let _ = write!(why, " {}", md_links(&r.links));
        }
        let _ = writeln!(
            s,
            "| `{}` | {} | {basis} | `{}` | {} | {} |",
            r.name,
            r.level,
            cell(&r.check),
            why.trim(),
            cell(r.fix.as_deref().unwrap_or(""))
        );
    }
    let _ = writeln!(s);
}

fn md_links(links: &[toolbox_model::manifest::Link]) -> String {
    links
        .iter()
        .map(|l| format!("[{}]({})", l.title.replace(['[', ']', '|'], ""), l.url))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The packages the portal includes, with the parameters it gives them.
fn standard_packages(s: &mut String, incs: &[Value]) {
    use std::fmt::Write as _;
    if incs.is_empty() {
        return;
    }
    let _ = writeln!(s, "## Packages and their settings\n");
    let _ = writeln!(
        s,
        "Change a standard by changing these parameters in the portal's `toolbox.yaml`.\n"
    );
    for i in incs {
        let _ = writeln!(s, "- `{}`", i["package"].as_str().unwrap_or("?"));
        for (k, v) in i["with"].as_object().into_iter().flatten() {
            let _ = writeln!(s, "  - `{k}`: {}", inline_code(&v.to_string()));
        }
    }
    let _ = writeln!(s);
}

/// The campaigns; `toolbox campaign --portal` shows their progress.
fn standard_campaigns(s: &mut String, m: &toolbox_model::Manifest) {
    use std::fmt::Write as _;
    if m.campaigns.is_empty() {
        return;
    }
    let _ = writeln!(s, "## Campaigns\n");
    for c in &m.campaigns {
        let _ = writeln!(
            s,
            "- **{}** (`{}`): {}",
            c.title.as_deref().unwrap_or(&c.name),
            c.name,
            c.description.as_deref().unwrap_or("").trim(),
        );
    }
    let _ = writeln!(s);
}

/// The commands to report, gate, update and change the standard, and a CI workflow.
fn standard_commands(s: &mut String, portal: &str, baseline: &str, accept: &str) {
    use std::fmt::Write as _;
    let rows = [
        (
            "List every failure, with path, fields, measure, evidence and fix (JSON)",
            format!("toolbox report {portal} -o failures"),
        ),
        (
            "Gate: exit 1 when anything got worse than the baseline",
            format!("toolbox report {portal} --baseline {baseline}"),
        ),
        (
            "What this branch changed, against its base",
            format!("toolbox report {portal} --against origin/main"),
        ),
        (
            "Record improvements (commit the file)",
            format!("toolbox report {portal} --baseline {baseline} --update"),
        ),
        (
            "Change a standard on purpose (new pattern, lower limit)",
            format!("toolbox report {portal} --baseline {baseline} --update --accept {accept}"),
        ),
        (
            "Campaign progress since its start",
            format!("toolbox campaign --portal {portal} --baseline {baseline}"),
        ),
        (
            "See exactly what a tool ran, to run it alone",
            format!("toolbox connector records --portal {portal} --name <connector>"),
        ),
    ];
    let _ = writeln!(
        s,
        "## Commands\n\nRun from the repository root.\n\n| To | Run |\n|---|---|"
    );
    for (to, run) in rows {
        let _ = writeln!(s, "| {to} | `{run}` |");
    }
    let _ = writeln!(
        s,
        "\n`--baseline` alone fails only on regressions. `--ci` also fails when the baseline is out of \
         date (an improvement not yet recorded), so the committed file always matches the code.\n"
    );
    let _ = writeln!(
        s,
        "## CI\n\n```yaml\n# .github/workflows/toolbox.yml\nname: toolbox\non:\n  pull_request:\n  push: {{branches: [main]}}\npermissions: {{contents: read}}\njobs:\n  report:\n    runs-on: ubuntu-latest\n    steps:\n      # Pin both to a full commit SHA, with the tag in a comment (zizmor's unpinned-uses).\n      - uses: actions/checkout@<sha> # v6\n        with: {{persist-credentials: false}}\n      # The tools the portal pins must be installed first: `toolbox tools install` or your\n      # installer, from the versions under `tools:`; the action runs `toolbox doctor`.\n      - uses: loganvolkers/toolbox/actions/report@<sha> # <version>\n        with:\n          toolbox-version: \"<version>\"\n          portal: {portal}\n          baseline: {baseline}\n          mode: ci    # or check: fail on regressions only\n```"
    );
}

/// `-o standard`: the portal's STANDARD.md, generated from the manifest so it can't drift: each
/// scorecard with its rules and fixes, the packages and their parameters, the campaigns, the
/// commands, and a CI workflow.
fn standard(o: &Options, m: &toolbox_model::Manifest) -> String {
    use std::fmt::Write as _;
    let portal = o.path.display().to_string();
    let baseline = o.baseline.as_ref().map_or_else(
        || format!("{portal}/results.json"),
        |b| b.display().to_string(),
    );
    let mut s = String::new();
    let title = m.portal.title.as_deref().unwrap_or(&m.portal.name);
    let _ = writeln!(s, "# {title}: standards\n");
    let _ = writeln!(
        s,
        "Generated by `toolbox report {portal} -o standard` from the portal's manifest; regenerate it, \
         don't edit it. Toolbox is the box these standards are carried in; the tools that check them \
         run on their own too (each record's `command`).\n"
    );
    let judged = crate::judged::judged_rules(m);
    for sc in &m.scorecards {
        standard_scorecard(&mut s, sc, &judged);
    }
    let incs: Vec<Value> = includes(&o.path)
        .into_iter()
        .filter(|i| i.get("package").is_some())
        .collect();
    standard_packages(&mut s, &incs);
    standard_campaigns(&mut s, m);
    let accept = m
        .scorecards
        .first()
        .map_or("<scorecard>", |sc| sc.name.as_str());
    standard_commands(&mut s, &portal, &baseline, accept);
    s
}

fn scorecard_defs(m: &toolbox_model::Manifest) -> Map<String, Value> {
    let judged = crate::judged::judged_rules(m);
    m.scorecards
        .iter()
        .map(|sc| {
            let rules: Map<String, Value> = sc
                .rules
                .iter()
                .map(|r| {
                    let basis = if judged.contains(&(sc.name.clone(), r.name.clone())) {
                        "judged"
                    } else {
                        "measured"
                    };
                    let mut def = json!({
                        "level": r.level,
                        "fix": r.fix,
                        "check": r.check,
                        "basis": basis
                    });
                    if let Some(t) = &r.title {
                        def["title"] = json!(t.trim());
                    }
                    if let Some(why) = &r.rationale {
                        def["rationale"] = json!(why.trim());
                    }
                    if !r.links.is_empty() {
                        def["links"] = json!(r.links);
                    }
                    if r.value.is_some() {
                        def["value"] = json!(r.value);
                        def["better"] = json!(r.better);
                        def["unit"] = json!(r.unit);
                    }
                    (r.name.clone(), def)
                })
                .collect();
            let mut def = json!({
                "title": sc.title,
                "description": sc.description.as_deref().map(str::trim),
                "rationale": sc.rationale.as_deref().map(str::trim),
                "links": sc.links,
                "kind": sc.kind,
                "levels": sc.levels,
                "owner": sc.owner,
                "track": sc.track,
                "rules": rules
            });
            if let Some(c) = &sc.compare {
                def["compare"] = json!(c);
            }
            if let Some(a) = &sc.applies_to {
                def["appliesTo"] = json!(a.trim());
            }
            (sc.name.clone(), def)
        })
        .collect()
}

fn campaign_defs(
    m: &toolbox_model::Manifest,
    d: &Derived,
    starts: &Map<String, Value>,
) -> Map<String, Value> {
    m.campaigns
        .iter()
        .map(|c| {
            let progress = d.campaigns.get(&c.name);
            let since = starts
                .get(&c.name)
                .zip(progress)
                .map(|(s, p)| paydown(s, p));
            let includes: Vec<Value> = c
                .includes
                .iter()
                .map(|i| {
                    json!({
                        "scorecard": i.scorecard,
                        "atLeast": i.at_least,
                        "rules": i.rules
                    })
                })
                .collect();
            (
                c.name.clone(),
                json!({
                    "title": c.title,
                    "track": c.track,
                    "owner": c.owner,
                    "description": c.description,
                    "start": c.start,
                    "end": c.end,
                    "targetPercent": c.target_percent(),
                    "draft": c.draft,
                    "includes": includes,
                    "status": progress.map(|p| &p.status),
                    "inScope": progress.map(|p| p.in_scope),
                    "meeting": progress.map(|p| p.meeting),
                    "percent": progress.map(|p| p.percent),
                    "onTarget": progress.map(|p| p.on_target),
                    "measure": progress.and_then(|p| p.measure).map(number),
                    "sinceStart": since,
                    "milestones": progress.map(|p| &p.milestones),
                    "entities": progress.map(|p| &p.entities),
                }),
            )
        })
        .collect()
}

/// Per scorecard with judged rules, the scored entities no assessment points at. Their judged
/// rules were decided with no judgement to read, so a pass there means "nothing found yet".
fn unassessed(m: &toolbox_model::Manifest, d: &Derived) -> Map<String, Value> {
    let kinds = crate::judged::assessment_kinds(m);
    let cards: BTreeSet<String> = crate::judged::judged_rules(m)
        .into_iter()
        .map(|(sc, _)| sc)
        .collect();
    let assessed = |id: &str| {
        d.entities.get(id).is_some_and(|e| {
            e.inbound.values().flatten().any(|src| {
                d.entities
                    .get(src)
                    .is_some_and(|s| kinds.contains(s.kind.as_str()))
            })
        })
    };
    cards
        .iter()
        .filter_map(|card| {
            let ids: Vec<&String> = d
                .scores
                .get(card)?
                .keys()
                .filter(|id| !is_peer(d, card, id) && !assessed(id))
                .collect();
            (!ids.is_empty()).then(|| (card.clone(), json!(ids)))
        })
        .collect()
}

/// The report document the element renders: portal, snapshot, catalog, scorecards, campaigns, scores.
fn report_json(m: &toolbox_model::Manifest, d: &Derived, starts: &Map<String, Value>) -> String {
    let tracks: Map<String, Value> = m
        .tracks
        .iter()
        .map(|t| {
            (
                t.name.clone(),
                json!({
                    "title": t.title,
                    "owner": t.owner,
                    "description": t.description.as_deref().map(str::trim),
                    "links": t.links
                }),
            )
        })
        .collect();
    let mut catalog: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, e) in &d.entities {
        catalog.entry(e.kind.clone()).or_default().push(id.clone());
    }
    let kind_groups: Vec<Value> = m
        .kind_groups
        .iter()
        .map(|g| {
            json!({
                "name": g.name,
                "title": g.title,
                "description": g.description.as_deref().map(str::trim),
                "kinds": g.kinds
            })
        })
        .collect();
    let kinds: Map<String, Value> = m
        .kinds
        .iter()
        .map(|k| {
            (
                k.name.clone(),
                json!({"title": k.title, "description": k.description.as_deref().map(str::trim)}),
            )
        })
        .collect();
    json!({
        "portal": m.portal.title.as_deref().unwrap_or(&m.portal.name),
        "snapshot": d.snapshot,
        "catalog": catalog,
        "kinds": kinds,
        "kindGroups": kind_groups,
        "tracks": tracks,
        "campaigns": campaign_defs(m, d, starts),
        "scorecards": scorecard_defs(m),
        "scores": own_scores(d),
        "unassessed": unassessed(m, d),
        "comparisons": comparison_defs(m, d),
        "repoRollup": m.policies.get("repoRollup").cloned().unwrap_or(Value::Null),
    })
    .to_string()
}

/// One self-contained page: `<toolbox-report>` with the JSON inlined and the element script
/// inlined. Host chrome is not concatenated in; a host fills the element's slots instead.
fn html_page(data_json: &str) -> String {
    // `</` can't appear inside the script element.
    let embedded = data_json.replace("</", "<\\/");
    include_str!("report.html")
        .replace("/*ELEMENT*/", include_str!("toolbox-report.js"))
        .replace("/*DATA*/", &embedded)
}

fn html(m: &toolbox_model::Manifest, d: &Derived, starts: &Map<String, Value>) -> String {
    html_page(&report_json(m, d, starts))
}

/// `toolbox-report.js`, `report.json`, and the one-file `index.html`. A host page loads the
/// first two and projects its own header, nav, and footer into the element's slots.
fn write_web_parts(dir: &std::path::Path, data_json: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let write = |name: &str, body: &str| {
        let path = dir.join(name);
        std::fs::write(&path, body).map_err(|e| format!("{}: {e}", path.display()))
    };
    write("toolbox-report.js", include_str!("toolbox-report.js"))?;
    write("report.json", data_json)?;
    write("index.html", &html_page(data_json))?;
    Ok(())
}

fn write_web(
    dir: &std::path::Path,
    m: &toolbox_model::Manifest,
    d: &Derived,
    starts: &Map<String, Value>,
) -> Result<(), String> {
    write_web_parts(dir, &report_json(m, d, starts))
}

/// `toolbox connector records`: one resync of a portal's connector, its records printed as NDJSON.
/// What a tool reported, before any rule reads it; lint records carry the `command` that
/// reproduces them with the tool alone.
pub fn connector_records(portal: &std::path::Path, name: &str) -> ExitCode {
    let run = || -> Result<Vec<Value>, String> {
        let src = toolbox_model::load::load(portal).map_err(|e| e.to_string())?;
        let m = crate::portal::manifest_from(&src)?;
        let c = m
            .connectors
            .iter()
            .find(|c| c.name == name)
            .ok_or(format!("no connector {name} in {}", portal.display()))?;
        let program = toolbox_api::connectors::binary_for(&c.package);
        let config = toolbox_api::connectors::resolve_config(
            &c.name,
            c.config.as_ref().unwrap_or(&Value::Null),
        );
        resync(&c.name, &program, &config, &c.records)
    };
    match run() {
        Ok(records) => {
            for r in records {
                println!("{r}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

/// `toolbox campaign --portal`: campaign progress from one local run, without a server; with a
/// baseline, the paydown since its recorded start.
pub fn campaign_local(
    portal: &std::path::Path,
    name: Option<&str>,
    baseline: Option<&std::path::Path>,
    skip: &[String],
    output: &str,
) -> ExitCode {
    let (_, m, d) = match live(portal, false, skip) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    };
    let recorded = baseline
        .and_then(|b| std::fs::read_to_string(b).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .map(|b| b["campaigns"].clone());
    let starts = campaign_starts(&m, &d, recorded.as_ref(), &[]);
    let mut defs = campaign_defs(&m, &d, &starts);
    if let Some(n) = name {
        defs.retain(|k, _| k == n);
        if defs.is_empty() {
            eprintln!("error: no campaign {n}");
            return ExitCode::from(2);
        }
    }
    if output == "json" {
        println!("{}", toolbox_derive::pyjson::pretty(&Value::Object(defs)));
        return ExitCode::SUCCESS;
    }
    for (n, c) in &defs {
        campaign_lines(n, c);
    }
    ExitCode::SUCCESS
}

/// One campaign's progress as text: who meets it, and its paydown since the recorded start, in
/// entities and in its measure, as an absolute count and as a share of the way to zero.
fn campaign_lines(n: &str, c: &Value) {
    let title = c["title"].as_str().unwrap_or(n);
    if c["inScope"].as_u64() == Some(0) {
        println!("{title} ({n}): nothing in scope (no entities yet)");
    } else {
        println!(
            "{title} ({n}): {} of {} meet it ({}%)",
            c["meeting"], c["inScope"], c["percent"]
        );
    }
    let since = &c["sinceStart"];
    for (label, key) in [("entities short", "entitiesShort"), ("measure", "measure")] {
        if let Some(x) = since.get(key) {
            let paid = x["from"].as_f64().unwrap_or(0.0) - x["now"].as_f64().unwrap_or(0.0);
            println!(
                "  {label}: {} → {} since {} (paid {}, {}% of the way)",
                x["from"],
                x["now"],
                since["recorded"].as_str().unwrap_or("?"),
                number(paid),
                x["percentDone"]
            );
        }
    }
    if since.is_null() {
        println!("  no recorded start: pass --baseline (written by report --update)");
    }
}

/// A portal's manifest and derived state from one run of its in-image connectors (or its fixtures),
/// with derivation warnings printed to stderr. Returns the portal directory too.
pub fn live(
    path: &std::path::Path,
    fixtures: bool,
    skip: &[String],
) -> Result<(PathBuf, toolbox_model::Manifest, Derived), String> {
    let src = toolbox_model::load::load(path).map_err(|e| e.to_string())?;
    let m = crate::portal::manifest_from(&src)?;
    let store = inputs(fixtures, skip, &src.root, &m)?;
    let d = FullRecompute.recompute(&m, &store.snapshot());
    for diag in &d.diagnostics {
        eprintln!("WARN  {}: {} [{}]", diag.location, diag.message, diag.code);
    }
    Ok((src.root, m, d))
}

/// The ratchet's outcome: for people (`summary`) and for pipelines (`--verdict`, D96).
struct Verdict {
    /// `pass`, `regressed`, `stale`, `missing-baseline`, `unknown` (or `error`, written by `fail`).
    result: &'static str,
    code: u8,
    summary: String,
    written: bool,
    comparison: Comparison,
    /// Regressions `--accept` recorded (an intended change to those scorecards).
    accepted: Vec<Change>,
    unknown: usize,
    /// Each campaign's recorded starting point (from the baseline, or this run's when new).
    starts: Map<String, Value>,
}

/// Rules whose `dataFrom` connector is in `--skip`. Their `unknown` is expected: the connector
/// was not run, so the rule has nothing to read. An unknown whose connector ran is not excused.
fn excused_rules(m: &toolbox_model::Manifest, skip: &[String]) -> BTreeSet<(String, String)> {
    let skip: BTreeSet<&str> = skip.iter().map(String::as_str).collect();
    let mut out = BTreeSet::new();
    for sc in &m.scorecards {
        for r in &sc.rules {
            if r.data_from.as_deref().is_some_and(|src| skip.contains(src)) {
                out.insert((sc.name.clone(), r.name.clone()));
            }
        }
    }
    out
}

/// Unknown outcomes that `--ci` and `--update` still refuse: the rule names no skipped connector.
fn unexpected_unknowns(m: &toolbox_model::Manifest, skip: &[String], now: &Value) -> usize {
    let excused = excused_rules(m, skip);
    let mut n = 0;
    let Some(cards) = now.as_object() else {
        return 0;
    };
    for (card, rows) in cards {
        let Some(rows) = rows.as_object() else {
            continue;
        };
        for row in rows.values() {
            let Some(rules) = row.get("unknown").and_then(Value::as_array) else {
                continue;
            };
            for rule in rules {
                let Some(name) = rule.as_str() else {
                    continue;
                };
                if !excused.contains(&(card.clone(), name.to_string())) {
                    n += 1;
                }
            }
        }
    }
    n
}

/// `--accept` needs `--update` and names scorecards that exist.
fn check_accept(o: &Options, m: &toolbox_model::Manifest) -> Result<(), String> {
    if !o.accept.is_empty() && !o.update {
        return Err("--accept records regressions, so it needs --update".into());
    }
    match o
        .accept
        .iter()
        .find(|a| !m.scorecards.iter().any(|sc| &sc.name == *a))
    {
        Some(card) => Err(format!("--accept {card}: no such scorecard")),
        None => Ok(()),
    }
}

/// One line per change, then one per campaign with a recorded start.
fn change_lines(v: &Verdict, m: &toolbox_model::Manifest, d: &Derived) -> Vec<String> {
    let c = &v.comparison;
    let mut lines = crate::causes::gate(d, &c.regressions, &crate::judged::judged_rules(m));
    for (mark, changes) in [
        ("✓ accept ", &v.accepted),
        ("✓ better ", &c.improvements),
        ("· changed", &c.other),
    ] {
        lines.extend(changes.iter().flat_map(|r| r.lines(mark, 0)));
    }
    lines.extend(paydown_lines(&v.starts, d));
    lines
}

/// The verdict against an existing baseline: sets `v.result` and `v.code`, writes the file when
/// `--update` may, and returns the summary's last line.
fn decide(
    o: &Options,
    v: &mut Verdict,
    path: &std::path::Path,
    differs: bool,
    unexpected: usize,
    write: &dyn Fn() -> Result<(), String>,
) -> Result<String, String> {
    let shown = path.display();
    if !v.comparison.regressions.is_empty() {
        (v.result, v.code) = ("regressed", 1);
        return Ok(format!(
            "{} regression(s) against {shown}",
            v.comparison.regressions.len()
        ));
    }
    if unexpected > 0 && (o.ci || o.update || o.fail_on_unknown) {
        // A connector that ran and still produced no data must not be written into the baseline.
        // Unknowns whose `dataFrom` connector was skipped are expected and are recorded.
        (v.result, v.code) = ("unknown", 1);
        let flag = match (o.ci, o.update) {
            (true, _) => "--ci",
            (_, true) => "--update",
            _ => "--fail-on-unknown",
        };
        return Ok(format!(
            "{unexpected} rule outcome(s) unknown and not excused by --skip; {flag} needs those rules evaluated"
        ));
    }
    Ok(match (differs, o.update) {
        (false, _) if v.unknown > 0 => format!(
            "no regressions; {} rule outcome(s) unknown (excused by --skip, or not gating), not compared",
            v.unknown
        ),
        (false, _) => format!("matches {shown}"),
        (true, true) => {
            write()?;
            v.written = true;
            match v.accepted.len() {
                0 => format!("no regressions; updated {shown}"),
                n => format!(
                    "accepted {n} regression(s) in {}; updated {shown}",
                    o.accept.join(", ")
                ),
            }
        }
        (true, false) => {
            (v.result, v.code) = ("stale", u8::from(o.ci));
            let ci = if o.ci { " (--ci)" } else { "" };
            format!(
                "no regressions, but {shown} is out of date{ci}: run with --update and commit it"
            )
        }
    })
}

/// Compare `now` with the baseline at `path`; write it when `--update` allows.
fn ratchet(
    o: &Options,
    path: &std::path::Path,
    m: &toolbox_model::Manifest,
    d: &Derived,
    now: &Value,
    policy: &NewPolicy,
) -> Result<Verdict, String> {
    check_accept(o, m)?;
    let before = std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let restart: &[String] = if o.keep_campaigns { &[] } else { &o.accept };
    let starts = campaign_starts(m, d, before.as_ref().map(|b| &b["campaigns"]), restart);
    let compact = compact_cards(m);
    let stored = compacted(now, &compact);
    let mut doc = json!({"format": FORMAT, "portal": m.portal.name, "results": stored});
    if !starts.is_empty() {
        doc["campaigns"] = Value::Object(starts.clone());
    }
    doc["evidence"] = evidence_keys(d, &advisory(m));
    let write = || {
        std::fs::write(path, toolbox_derive::pyjson::pretty(&doc))
            .map_err(|e| format!("{}: {e}", path.display()))
    };
    let mut v = Verdict {
        result: "pass",
        code: 0,
        summary: String::new(),
        written: false,
        comparison: Comparison::default(),
        accepted: Vec::new(),
        unknown: unknown_count(now),
        starts,
    };
    let Some(b) = before else {
        if o.update {
            write()?;
            v.written = true;
            v.summary = format!("baseline: created {}\n", path.display());
        } else {
            (v.result, v.code) = ("missing-baseline", u8::from(o.ci));
            v.summary = format!(
                "baseline: {} doesn't exist; create it with --update\n",
                path.display()
            );
        }
        return Ok(v);
    };
    v.comparison = compare_with(&b["results"], now, policy, &compact);
    (v.accepted, v.comparison.regressions) = std::mem::take(&mut v.comparison.regressions)
        .into_iter()
        .partition(|r| o.accept.contains(&r.scorecard));
    mark_known(&mut v.comparison.regressions, &b["evidence"]);
    explain_regressions(&mut v.comparison, &b["results"], now, m, d);
    let recorded = b["campaigns"].as_object().cloned().unwrap_or_default();
    let differs = b["results"] != doc["results"] || recorded != v.starts;
    let unexpected = unexpected_unknowns(m, &o.skip, now);
    let mut text = decide(o, &mut v, path, differs, unexpected, &write)?;
    // Keys explain; they never fail a check. --update keeps them current when nothing else moved.
    if o.update && v.code == 0 && !v.written && b["evidence"] != doc["evidence"] {
        write()?;
        v.written = true;
        text.push_str("; updated its evidence keys");
    }
    v.summary = format!("{}\nbaseline: {text}\n", change_lines(&v, m, d).join("\n"));
    Ok(v)
}

/// Every regression gets its root cause (crate::causes): evidence, how to reproduce it, and for
/// an aggregate the constituent scorecards that fell and the changes that made them fall.
fn explain_regressions(
    c: &mut Comparison,
    before: &Value,
    now: &Value,
    m: &toolbox_model::Manifest,
    d: &Derived,
) {
    let candidates: Vec<Change> = c.regressions.iter().chain(&c.other).cloned().collect();
    crate::causes::explain(&mut c.regressions, &candidates, before, now, m, d);
}

fn git(dir: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

/// The portal's ratchet results at another commit.
fn results_at(o: &Options, rev: &str) -> Result<(Value, Value), String> {
    let (m, d) = live_at(&o.path, o.fixtures, &o.skip, rev)?;
    let advisory = advisory(&m);
    Ok((results(&d, &advisory), evidence_keys(&d, &advisory)))
}

/// The portal's facts at another commit: the repository checked out at `rev` in a temporary
/// worktree, and the portal run there from the same relative directory.
pub fn live_at(
    path: &std::path::Path,
    fixtures: bool,
    skip: &[String],
    rev: &str,
) -> Result<(toolbox_model::Manifest, Derived), String> {
    let here = std::env::current_dir().map_err(|e| e.to_string())?;
    let prefix = git(&here, &["rev-parse", "--show-prefix"])?;
    let top = PathBuf::from(git(&here, &["rev-parse", "--show-toplevel"])?);
    let tree = std::env::temp_dir().join(format!(
        "toolbox-at-{}-{}",
        std::process::id(),
        toolbox_exec::run::new_id("wt")
    ));
    git(
        &top,
        &[
            "worktree",
            "add",
            "--detach",
            "--quiet",
            &tree.to_string_lossy(),
            rev,
        ],
    )?;
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        here.join(path)
    };
    let portal = match abs
        .canonicalize()
        .unwrap_or(abs.clone())
        .strip_prefix(top.canonicalize().unwrap_or(top.clone()))
    {
        Ok(rel) => tree.join(rel),
        Err(_) => path.to_path_buf(),
    };
    let result = std::env::set_current_dir(tree.join(&prefix))
        .map_err(|e| format!("{}: {e}", tree.display()))
        .and_then(|()| live(&portal, fixtures, skip))
        .map(|(_, m, d)| (m, d));
    let _ = std::env::set_current_dir(&here);
    let _ = git(
        &top,
        &["worktree", "remove", "--force", &tree.to_string_lossy()],
    );
    result
}

/// `--against REV`: what changed since that commit. Regressions exit 1; nothing is written.
fn against(
    o: &Options,
    rev: &str,
    m: &toolbox_model::Manifest,
    d: &Derived,
    now: &Value,
) -> Result<Verdict, String> {
    let (before, keys) = results_at(o, rev)?;
    let mut v = Verdict {
        result: "pass",
        code: 0,
        summary: String::new(),
        written: false,
        comparison: compare_with(&before, now, &new_policy(m), &BTreeSet::new()),
        accepted: Vec::new(),
        unknown: unknown_count(now),
        starts: Map::new(),
    };
    mark_known(&mut v.comparison.regressions, &keys);
    explain_regressions(&mut v.comparison, &before, now, m, d);
    let n = v.comparison.regressions.len();
    if n > 0 {
        (v.result, v.code) = ("regressed", 1);
    }
    let text = match n {
        0 => format!("no regressions against {rev}"),
        n => format!("{n} regression(s) against {rev}"),
    };
    v.summary = format!("{}\nagainst: {text}\n", change_lines(&v, m, d).join("\n"));
    Ok(v)
}

/// The `--verdict` document: what a pipeline needs to decide, annotate and comment.
fn verdict_json(o: &Options, v: &Verdict) -> Value {
    let list = |c: &[Change]| c.iter().map(Change::to_json).collect::<Vec<_>>();
    let mode = match (&o.baseline, o.ci, o.update) {
        (None, ..) => "report",
        (_, true, _) => "ci",
        (_, _, true) => "update",
        _ => "check",
    };
    json!({
        "format": "toolbox-verdict/v1",
        "result": v.result,
        "exitCode": v.code,
        "mode": mode,
        "baseline": o.baseline.as_ref().map(|p| p.display().to_string()),
        "written": v.written,
        "unknownRules": v.unknown,
        "regressions": list(&v.comparison.regressions),
        "improvements": list(&v.comparison.improvements),
        "accepted": list(&v.accepted),
        "changes": list(&v.comparison.other),
    })
}

/// An entity's string, number and boolean fields.
fn scalars(e: Option<&toolbox_derive::Entity>) -> Map<String, Value> {
    e.into_iter()
        .flat_map(|e| &e.fields)
        .filter(|(_, v)| v.is_string() || v.is_number() || v.is_boolean())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Every failing rule, flat: what an agent fixes, sorted by scorecard, then largest measure (or
/// most evidence) first. `path` is the entity's `path` field when it has one; `fields` are its
/// scalar fields (strings, numbers, booleans), such as a file's `lines`.
fn failures(m: &toolbox_model::Manifest, d: &Derived) -> Value {
    let mut rows = Vec::new();
    for sc in &m.scorecards {
        for (id, s) in d.scores.get(&sc.name).into_iter().flatten() {
            for (rule, outcome) in s.rules.iter().flatten() {
                if outcome != "fail" {
                    continue;
                }
                let def = sc.rules.iter().find(|r| &r.name == rule);
                let ev = s.evidence.get(rule).cloned().unwrap_or(Value::Null);
                let size = s
                    .measures
                    .get(rule)
                    .copied()
                    .or_else(|| ev.as_array().map(|a| a.len() as f64));
                rows.push((
                    sc.name.clone(),
                    -size.unwrap_or(0.0),
                    json!({
                        "scorecard": sc.name,
                        "rule": rule,
                        "entity": id,
                        "path": d.entities.get(id).and_then(|e| e.fields.get("path")),
                        "fields": scalars(d.entities.get(id)),
                        "sources": d.entities.get(id).map(|e| e.records.iter().map(|(conn, kind, ext)| {
                            json!({"connector": conn, "record": kind, "id": ext})
                        }).collect::<Vec<_>>()),
                        "measure": s.measures.get(rule).map(|x| number(*x)),
                        "evidence": ev,
                        "fix": def.and_then(|r| r.fix.as_deref()),
                    }),
                ));
            }
        }
    }
    rows.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.total_cmp(&b.1))
            .then(a.2["entity"].as_str().cmp(&b.2["entity"].as_str()))
    });
    Value::Array(rows.into_iter().map(|r| r.2).collect())
}

pub fn run(o: &Options) -> ExitCode {
    let save = |doc: &Value| {
        if let Some(p) = &o.verdict
            && let Err(e) = std::fs::write(p, toolbox_derive::pyjson::pretty(doc))
        {
            eprintln!("error: {}: {e}", p.display());
        }
    };
    let fail = |m: String| {
        eprintln!("error: {m}");
        save(
            &json!({"format": "toolbox-verdict/v1", "result": "error", "exitCode": 2, "message": m}),
        );
        ExitCode::from(2)
    };
    let (_, m, d) = match live(&o.path, o.fixtures, &o.skip) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let now = results(&d, &advisory(&m));
    let v = match (&o.baseline, &o.against) {
        (_, Some(rev)) => match against(o, rev, &m, &d, &now) {
            Ok(v) => v,
            Err(e) => return fail(e),
        },
        (Some(path), None) => match ratchet(o, path, &m, &d, &now, &new_policy(&m)) {
            Ok(v) => v,
            Err(e) => return fail(e),
        },
        (None, None) if !o.accept.is_empty() => {
            return fail("--accept needs --baseline and --update".into());
        }
        (None, None) => Verdict {
            result: "pass",
            code: 0,
            summary: String::new(),
            written: false,
            comparison: Comparison::default(),
            accepted: Vec::new(),
            unknown: unknown_count(&now),
            starts: Map::new(),
        },
    };
    save(&verdict_json(o, &v));
    let (code, summary) = (ExitCode::from(v.code), v.summary);
    let output = o.output.as_deref().unwrap_or("md");
    let gate =
        o.output.is_none() && o.out.is_none() && (o.baseline.is_some() || o.against.is_some());
    if let Some(p) = &o.html
        && let Err(e) = std::fs::write(p, html(&m, &d, &v.starts))
    {
        return fail(format!("{}: {e}", p.display()));
    }
    if output == "web" {
        let Some(dir) = &o.out else {
            return fail("-o web writes a directory; pass --out DIR".into());
        };
        if let Err(e) = write_web(dir, &m, &d, &v.starts) {
            return fail(e);
        }
        eprint!("{summary}");
        return code;
    }
    if gate {
        print!("{summary}");
        if v.code != 0 {
            println!("every scorecard: add -o md, or --out FILE");
        }
        return code;
    }
    let body = match output {
        "json" => {
            let mut doc = json!({
                "format": FORMAT,
                "snapshot": d.snapshot,
                "scores": d.scores,
                "results": now,
                "campaigns": campaign_defs(&m, &d, &v.starts),
            });
            if !d.comparisons.is_empty() {
                doc["comparisons"] = json!(d.comparisons);
            }
            toolbox_derive::pyjson::pretty(&doc)
        }
        "failures" => toolbox_derive::pyjson::pretty(&failures(&m, &d)),
        "standard" => standard(o, &m),
        "html" => html(&m, &d, &v.starts),
        "md" => markdown(&m, &d, o.all),
        other => {
            return fail(format!(
                "-o {other}: expected md, json, failures, standard, html or web"
            ));
        }
    };
    match &o.out {
        Some(p) => {
            if let Err(e) = std::fs::write(p, body) {
                return fail(format!("{}: {e}", p.display()));
            }
        }
        None => print!("{body}"),
    }
    eprint!("{summary}");
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[Change]) -> Vec<String> {
        v.iter().map(ToString::to_string).collect()
    }

    fn r(rows: Value) -> Value {
        json!({"card": rows})
    }

    #[test]
    fn inline_code_survives_backticks() {
        assert_eq!(inline_code("plain"), "`plain`");
        assert_eq!(inline_code("run `ruff` alone"), "``run `ruff` alone``");
        assert_eq!(inline_code("`x`"), "`` `x` ``");
    }

    #[test]
    fn ratchet_flags_new_failures_and_growing_evidence() {
        let before =
            r(json!({"a": {"level": "bronze", "failing": {"docs": 3}}, "b": {"level": "gold"}}));
        let after = r(
            json!({"a": {"level": "bronze", "failing": {"docs": 4}}, "b": {"level": "silver", "failing": {"tests": null}}}),
        );
        let c = compare(&before, &after, &NewPolicy::new());
        assert_eq!(
            s(&c.regressions),
            [
                "card · a: docs evidence grew 3 → 4",
                "card · b: tests now fails"
            ]
        );
        // The baseline keeps counts, so a grown list's new item can be anywhere: show it whole.
        assert!(c.regressions[0].whole_evidence && !c.regressions[1].whole_evidence);
        assert!(c.improvements.is_empty());
    }

    #[test]
    fn new_entities_below_the_scorecards_floor_are_regressions() {
        let before = r(json!({"a": {"level": "gold"}}));
        let after = r(
            json!({"a": {"level": "gold"}, "b": {"level": "none", "failing": {"tested": null}}, "c": {"level": "bronze"}}),
        );
        let policy: NewPolicy = [(
            "card".to_owned(),
            (vec!["bronze".into(), "silver".into()], "bronze".into()),
        )]
        .into();
        let c = compare(&before, &after, &policy);
        assert_eq!(
            s(&c.regressions),
            ["card · b: new, at none; new entities must reach bronze"]
        );
        assert_eq!(s(&c.other), ["card · c: new (bronze)"]);
        assert!(
            compare(&before, &after, &NewPolicy::new())
                .regressions
                .is_empty()
        );
    }

    #[test]
    fn unknown_outcomes_are_neither_worse_nor_better() {
        let before =
            r(json!({"a": {"level": "none", "failing": {"lint": 2}}, "b": {"level": "gold"}}));
        let after = r(
            json!({"a": {"level": "none", "unknown": ["lint"]}, "b": {"level": "none", "unknown": ["lint"]}}),
        );
        let c = compare(&before, &after, &NewPolicy::new());
        assert!(
            c.regressions.is_empty() && c.improvements.is_empty(),
            "{:?} {:?}",
            c.regressions,
            c.improvements
        );
        assert_eq!(
            s(&c.other),
            [
                "card · a: lint unknown now (its data is missing)",
                "card · b: lint unknown now (its data is missing)",
                "card · b: level \"gold\" → \"none\""
            ]
        );
        assert_eq!(unknown_count(&after), 2);
        // and back: a rule known again that fails is reported, not counted as a regression
        let c = compare(&after, &before, &NewPolicy::new());
        assert!(c.regressions.is_empty());
        assert_eq!(c.other[0].to_string(), "card · a: lint known again: fails");
    }

    #[test]
    fn compact_scorecards_omit_clean_entities_and_still_catch_regressions() {
        let compact: BTreeSet<String> = ["card".to_owned()].into();
        let policy: NewPolicy =
            [("card".to_owned(), (vec!["clean".into()], "clean".into()))].into();
        let now = r(
            json!({"a": {"level": "none", "failing": {"no-matches": 2}}, "b": {"level": "clean"},
                           "c": {"level": "skipped"}}),
        );
        let stored = compacted(&now, &compact);
        assert_eq!(
            stored,
            r(json!({"a": {"level": "none", "failing": {"no-matches": 2}}}))
        );
        // Against that baseline: a new clean file is no change; a clean file that gains a match,
        // or a new file with one, is a regression; a fixed file is better; an exempted one is neither.
        let after = r(
            json!({"a": {"level": "clean"}, "b": {"level": "none", "failing": {"no-matches": 1}},
                             "d": {"level": "clean"}, "e": {"level": "none", "failing": {"no-matches": 1}}}),
        );
        let c = compare_with(&stored, &after, &policy, &compact);
        assert_eq!(
            s(&c.regressions),
            [
                "card · b: no-matches now fails (it was clean, or is new)",
                "card · e: no-matches now fails (it was clean, or is new)"
            ]
        );
        assert_eq!(s(&c.improvements), ["card · a: no-matches now passes"]);
        assert!(c.other.is_empty(), "{:?}", c.other);
        let skipped = r(json!({"a": {"level": "skipped"}}));
        let c = compare_with(&stored, &skipped, &policy, &compact);
        assert_eq!(
            s(&c.other),
            ["card · a: now skipped (exempt or out of scope)"]
        );
        let gone = r(json!({}));
        let c = compare_with(&stored, &gone, &policy, &compact);
        assert_eq!(
            s(&c.improvements),
            ["card · a: no longer reported (fixed, or the file is gone)"]
        );
    }

    #[test]
    fn measures_are_compared_as_numbers() {
        // A rule with a `measure` records it (1070 lines), so one more line is a regression.
        let before = r(json!({"f": {"level": "none", "failing": {"size": 1070}}}));
        let grew = r(json!({"f": {"level": "none", "failing": {"size": 1071}}}));
        let shrank = r(json!({"f": {"level": "none", "failing": {"size": 1069.5}}}));
        let c = compare(&before, &grew, &NewPolicy::new());
        assert_eq!(
            s(&c.regressions),
            ["card · f: size evidence grew 1070 → 1071"]
        );
        let c = compare(&before, &shrank, &NewPolicy::new());
        assert_eq!(
            s(&c.improvements),
            ["card · f: size evidence shrank 1070 → 1069.5"]
        );
        assert_eq!(number(1070.0), json!(1070));
        assert_eq!(number(2.5), json!(2.5));
    }

    fn progress(
        in_scope: usize,
        meeting: usize,
        measure: Option<f64>,
    ) -> toolbox_derive::CampaignProgress {
        toolbox_derive::CampaignProgress {
            status: "active".into(),
            track: None,
            target_percent: 100,
            in_scope,
            meeting,
            percent: 0,
            on_target: false,
            measure,
            milestones: vec![],
            entities: BTreeMap::new(),
        }
    }

    #[test]
    fn paydown_is_measured_from_the_recorded_start() {
        let start = json!({"start": "2026-11-21", "recorded": "2026-11-21", "inScope": 400, "short": 40, "measure": 120});
        let p = paydown(&start, &progress(400, 361, Some(114.0)));
        assert_eq!(
            p,
            json!({"start": "2026-11-21", "recorded": "2026-11-21",
                   "entitiesShort": {"from": 40, "now": 39, "percentDone": 2.5},
                   "measure": {"from": 120, "now": 114, "percentDone": 5.0}})
        );
        // Backwards is negative, not hidden.
        assert_eq!(
            paydown(&start, &progress(400, 359, Some(126.0)))["measure"]["percentDone"],
            json!(-5.0)
        );
        // Without a measure, only entities short of the goal.
        let start = json!({"start": null, "inScope": 10, "short": 4});
        let p = paydown(&start, &progress(10, 7, None));
        assert!(p.get("measure").is_none());
        assert_eq!(p["entitiesShort"]["percentDone"], json!(25.0));
        // Nothing to pay down: done.
        assert_eq!(
            paydown(&json!({"short": 0}), &progress(3, 3, None))["entitiesShort"]["percentDone"],
            json!(100.0)
        );
    }

    #[test]
    fn ratchet_records_improvements_and_new_entities() {
        let before = r(json!({"a": {"level": "none", "failing": {"docs": 3, "tests": null}}}));
        let after = r(
            json!({"a": {"level": "none", "failing": {"docs": 1}}, "c": {"level": "none", "failing": {"x": null}}}),
        );
        let c = compare(&before, &after, &NewPolicy::new());
        assert!(c.regressions.is_empty());
        assert_eq!(
            s(&c.improvements),
            [
                "card · a: docs evidence shrank 3 → 1",
                "card · a: tests now passes"
            ]
        );
        assert_eq!(s(&c.other), ["card · c: new (none)"]);
    }

    #[test]
    fn the_html_page_is_the_element_with_slots_and_inlined_data() {
        let page = html_page(
            r#"{"portal":"Demo","snapshot":"s","catalog":{},"scorecards":{},"scores":{}}"#,
        );
        assert!(page.contains("<toolbox-report>"));
        assert!(page.contains(r#"<script type="application/json">{"portal":"Demo""#));
        assert!(page.contains(r#"<slot name="header">"#));
        assert!(page.contains(r#"<slot name="nav">"#));
        assert!(page.contains(r#"<slot name="footer">"#));
        assert!(page.contains("Overview"));
        assert!(page.contains("Catalog"));
        assert!(page.contains("Campaigns"));
        assert!(page.contains(r#"setAttribute("aria-label", "Status")"#));
        assert!(page.contains("Needs attention first"));
        assert!(page.contains("Most entities at none first"));
        assert!(page.contains("role: \"tab\""));
        assert!(!page.contains("{{"));
    }

    #[test]
    fn a_closing_script_tag_in_the_data_cannot_break_out_of_the_json_script() {
        let page = html_page(r#"{"portal":"</script>"}"#);
        let json = page
            .split(r#"<script type="application/json">"#)
            .nth(1)
            .unwrap()
            .split("</script>")
            .next()
            .unwrap();
        assert_eq!(json, r#"{"portal":"<\/script>"}"#);
    }

    #[test]
    fn web_output_writes_the_element_the_json_and_the_page() {
        let dir = std::env::temp_dir().join(format!("toolbox-report-web-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let data = r#"{"portal":"Demo"}"#;
        write_web_parts(&dir, data).unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("toolbox-report.js")).unwrap(),
            include_str!("toolbox-report.js")
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("report.json")).unwrap(),
            data
        );
        let page = std::fs::read_to_string(dir.join("index.html")).unwrap();
        assert!(page.contains("<toolbox-report>"));
        assert!(page.contains(data));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unassessed_lists_entities_no_assessment_points_at() {
        let m: toolbox_model::Manifest = serde_norway::from_str(
            r"
apiVersion: toolbox.dev/v1alpha1
portal: {name: t, owner: group:a, toolbox: '>=0.1'}
kinds:
  - name: Crate
    identityFrom: [connector:ws]
    sources: {default: [connector:ws]}
    fields:
      reviewFailing:
        type: list
        computed: 'inbound.assesses.size() == 0 ? [] : inbound.assesses[0].failing'
    map: [{from: ws/crate, id: record.id, set: record}]
  - name: Assessment
    identityFrom: [connector:ws]
    sources: {default: [connector:ws]}
    fields:
      failing: {type: list}
    relations:
      assesses: {to: Crate}
    map: [{from: ws/assessment, id: record.id, set: record, relations: {assesses: '[record.of]'}}]
scorecards:
  - name: crate
    kind: Crate
    owner: group:a
    levels: [bronze]
    rules:
      - {name: reviewed, level: bronze, check: entity.reviewFailing.size() == 0}
",
        )
        .unwrap();
        let rec = |kind: &str, data: Value| toolbox_store::Record {
            connector: "ws".into(),
            kind: kind.into(),
            external_id: data["id"].as_str().unwrap().into(),
            data,
        };
        let inputs = toolbox_store::InputSnapshot {
            version: 1,
            records: vec![
                rec("crate", json!({"id": "crate:a"})),
                rec("crate", json!({"id": "crate:b"})),
                rec(
                    "assessment",
                    json!({"id": "assessment:a", "of": "crate:a", "failing": []}),
                ),
            ],
            native: vec![],
        };
        let d = FullRecompute.recompute(&m, &inputs);
        assert_eq!(d.scores["crate"]["crate:b"].level, "bronze");
        assert_eq!(
            Value::Object(unassessed(&m, &d)),
            json!({"crate": ["crate:b"]})
        );
    }
}
