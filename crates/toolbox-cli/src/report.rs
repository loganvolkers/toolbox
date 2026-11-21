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
    /// pass --out DIR for web.
    #[arg(short = 'o', long, default_value = "md")]
    pub output: String,
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
    /// With --update: record regressions in these scorecards (comma-separated) instead of
    /// failing, for an intended change to a standard (a tighter threshold, a new pattern).
    /// They are listed; regressions in other scorecards still fail.
    #[arg(long, value_delimiter = ',')]
    pub accept: Vec<String>,
    /// Write the ratchet's verdict as JSON to this file (result, exit code, regressions,
    /// improvements), for any CI system to read.
    #[arg(long)]
    pub verdict: Option<PathBuf>,
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
        for (id, s) in entities {
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

/// One difference from the baseline, on one entity of one scorecard.
#[derive(Debug)]
struct Change {
    scorecard: String,
    entity: String,
    message: String,
}

impl std::fmt::Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} · {}: {}", self.scorecard, self.entity, self.message)
    }
}

impl Change {
    fn to_json(&self) -> Value {
        json!({"scorecard": self.scorecard, "entity": self.entity, "message": self.message})
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

fn compare(before: &Value, after: &Value, policy: &NewPolicy) -> Comparison {
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
        for (id, now) in &a {
            let at = At { card, id };
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
            c.other.push(At { card, id }.say("removed".into()));
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
    let (wf, nf) = (obj(&was["failing"]), obj(&now["failing"]));
    let (wu, nu) = (strings(&was["unknown"]), strings(&now["unknown"]));
    for rule in nu.iter().filter(|r| !wu.contains(r)) {
        c.other
            .push(at.say(format!("{rule} unknown now (its data is missing)")));
    }
    for (rule, n) in &nf {
        if wu.contains(rule) {
            c.other.push(at.say(format!("{rule} known again: fails")));
        } else if let Some(m) = wf.get(rule) {
            evidence_change(c, at, rule, m, n);
        } else if was["level"] == "skipped" {
            c.other
                .push(at.say(format!("no longer skipped; {rule} fails")));
        } else {
            c.regressions.push(at.say(format!("{rule} now fails")));
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
        (Some(m), Some(n)) if n > m => c
            .regressions
            .push(at.say(format!("{rule} evidence grew {was} → {now}"))),
        (Some(m), Some(n)) if n < m => c
            .improvements
            .push(at.say(format!("{rule} evidence shrank {was} → {now}"))),
        _ => {}
    }
}

/// Each campaign's starting point, as the baseline records it: kept while the campaign's `start`
/// is unchanged, recorded from this run the first time the campaign is active. Changing `start`
/// restarts the campaign from the current state, and so does `--accept` on a scorecard the
/// campaign includes: an intended change to a standard changes what the campaign pays down.
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

fn markdown(m: &toolbox_model::Manifest, d: &Derived) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# {} — scorecards",
        m.portal.title.as_deref().unwrap_or(&m.portal.name)
    );
    for sc in &m.scorecards {
        let rows = d.scores.get(&sc.name).cloned().unwrap_or_default();
        let mut tally: BTreeMap<&str, usize> = BTreeMap::new();
        for r in rows.values() {
            *tally.entry(r.level.as_str()).or_default() += 1;
        }
        let counts: Vec<String> = tally.iter().map(|(l, n)| format!("{l}: {n}")).collect();
        let _ = writeln!(s, "\n## {}  ({})\n", sc.name, counts.join(", "));
        for (id, r) in rows.iter().filter(|(_, r)| r.level != "skipped") {
            let failing: Vec<&String> = r
                .rules
                .iter()
                .flatten()
                .filter(|(_, o)| *o != "pass")
                .map(|(n, _)| n)
                .collect();
            let _ = writeln!(
                s,
                "- **{}** `{id}`{}",
                r.level,
                if failing.is_empty() {
                    String::new()
                } else {
                    format!(
                        ": {}",
                        failing
                            .iter()
                            .map(|f| f.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            );
            for (rule, ev) in &r.evidence {
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
    }
    s
}

fn scorecard_defs(m: &toolbox_model::Manifest) -> Map<String, Value> {
    m.scorecards
        .iter()
        .map(|sc| {
            let rules: Map<String, Value> = sc
                .rules
                .iter()
                .map(|r| {
                    (
                        r.name.clone(),
                        json!({"level": r.level, "fix": r.fix, "check": r.check}),
                    )
                })
                .collect();
            (
                sc.name.clone(),
                json!({
                    "kind": sc.kind,
                    "levels": sc.levels,
                    "owner": sc.owner,
                    "track": sc.track,
                    "rules": rules
                }),
            )
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
                    "description": t.description
                }),
            )
        })
        .collect();
    let mut catalog: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, e) in &d.entities {
        catalog.entry(e.kind.clone()).or_default().push(id.clone());
    }
    json!({
        "portal": m.portal.title.as_deref().unwrap_or(&m.portal.name),
        "snapshot": d.snapshot,
        "catalog": catalog,
        "tracks": tracks,
        "campaigns": campaign_defs(m, d, starts),
        "scorecards": scorecard_defs(m),
        "scores": d.scores,
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
fn change_lines(v: &Verdict, d: &Derived) -> Vec<String> {
    let c = &v.comparison;
    let mut lines = Vec::new();
    for (mark, changes) in [
        ("✗ worse  ", &c.regressions),
        ("✓ accept ", &v.accepted),
        ("✓ better ", &c.improvements),
        ("· changed", &c.other),
    ] {
        lines.extend(changes.iter().map(|r| format!("{mark} {r}")));
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
    if unexpected > 0 && (o.ci || o.update) {
        // A connector that ran and still produced no data must not be written into the baseline.
        // Unknowns whose `dataFrom` connector was skipped are expected and are recorded.
        (v.result, v.code) = ("unknown", 1);
        let flag = if o.ci { "--ci" } else { "--update" };
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
    let starts = campaign_starts(m, d, before.as_ref().map(|b| &b["campaigns"]), &o.accept);
    let mut doc = json!({"format": FORMAT, "portal": m.portal.name, "results": now});
    if !starts.is_empty() {
        doc["campaigns"] = Value::Object(starts.clone());
    }
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
    v.comparison = compare(&b["results"], now, policy);
    (v.accepted, v.comparison.regressions) = std::mem::take(&mut v.comparison.regressions)
        .into_iter()
        .partition(|r| o.accept.contains(&r.scorecard));
    let recorded = b["campaigns"].as_object().cloned().unwrap_or_default();
    let differs = b["results"] != *now || recorded != v.starts;
    let unexpected = unexpected_unknowns(m, &o.skip, now);
    let text = decide(o, &mut v, path, differs, unexpected, &write)?;
    v.summary = format!("{}\nbaseline: {text}\n", change_lines(&v, d).join("\n"));
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
    let v = match &o.baseline {
        Some(path) => match ratchet(o, path, &m, &d, &now, &new_policy(&m)) {
            Ok(v) => v,
            Err(e) => return fail(e),
        },
        None if !o.accept.is_empty() => {
            return fail("--accept needs --baseline and --update".into());
        }
        None => Verdict {
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
    if o.output == "web" {
        let Some(dir) = &o.out else {
            return fail("-o web writes a directory; pass --out DIR".into());
        };
        if let Err(e) = write_web(dir, &m, &d, &v.starts) {
            return fail(e);
        }
        if let Some(p) = &o.html
            && let Err(e) = std::fs::write(p, html(&m, &d, &v.starts))
        {
            return fail(format!("{}: {e}", p.display()));
        }
        eprint!("{summary}");
        return code;
    }
    let body = match o.output.as_str() {
        "json" => toolbox_derive::pyjson::pretty(&json!({
            "format": FORMAT,
            "snapshot": d.snapshot,
            "scores": d.scores,
            "results": now,
            "campaigns": campaign_defs(&m, &d, &v.starts),
        })),
        "failures" => toolbox_derive::pyjson::pretty(&failures(&m, &d)),
        "html" => html(&m, &d, &v.starts),
        "md" => markdown(&m, &d),
        other => {
            return fail(format!(
                "-o {other}: expected md, json, failures, html or web"
            ));
        }
    };
    if let Some(p) = &o.html
        && let Err(e) = std::fs::write(p, html(&m, &d, &v.starts))
    {
        return fail(format!("{}: {e}", p.display()));
    }
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
        let start = json!({"start": "2026-11-21", "recorded": "2026-11-21", "inScope": 400, "short": 37, "measure": 102});
        let p = paydown(&start, &progress(400, 365, Some(99.0)));
        assert_eq!(
            p,
            json!({"start": "2026-11-21", "recorded": "2026-11-21",
                   "entitiesShort": {"from": 37, "now": 35, "percentDone": 5.4},
                   "measure": {"from": 102, "now": 99, "percentDone": 2.9}})
        );
        // Backwards is negative, not hidden.
        assert_eq!(
            paydown(&start, &progress(400, 360, Some(105.0)))["measure"]["percentDone"],
            json!(-2.9)
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
        assert!(page.contains("Fleet health"));
        assert!(page.contains("Needs attention first"));
        assert!(page.contains("Highest none debt first"));
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
}
