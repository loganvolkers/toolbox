//! `toolbox report`: score a repository locally, without a server (docs/design/toolbox-for-toolbox.md
//! F1, F4; docs/design/scorecard-ratchet.md). Runs the portal's in-image connectors once (a resync
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

pub struct Options {
    pub path: PathBuf,
    pub fixtures: bool,
    pub skip: Vec<String>,
    pub output: String,
    pub out: Option<PathBuf>,
    pub html: Option<PathBuf>,
    pub baseline: Option<PathBuf>,
    pub update: bool,
    pub ci: bool,
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
                    let n = s.evidence.get(r).and_then(Value::as_array).map(Vec::len);
                    (r.clone(), json!(n))
                })
                .collect();
            // `unknown`: the rule couldn't be evaluated (its data is missing, e.g. a linter that
            // isn't installed). Never a regression or an improvement; `--ci` refuses it.
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
            rows.insert(id.clone(), row);
        }
        cards.insert(card.clone(), Value::Object(rows));
    }
    Value::Object(cards)
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
    regressions: Vec<String>,
    improvements: Vec<String>,
    other: Vec<String>,
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
            let at = format!("{card} · {id}");
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
            c.other.push(format!("{card} · {id}: removed"));
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
fn new_entity(c: &mut Comparison, at: &str, now: &Value, floor: Option<&(Vec<String>, String)>) {
    let level = now["level"].as_str().unwrap_or("?");
    let short = floor.and_then(|(levels, min)| {
        let (have, need) = (rank(levels, level)?, rank(levels, min)?);
        (have < need).then_some(min)
    });
    match short {
        Some(min) => c.regressions.push(format!(
            "{at}: new, at {level}; new entities must reach {min}"
        )),
        None => c.other.push(format!("{at}: new ({level})")),
    }
}

/// One entity in both: rules that started failing or whose evidence grew are worse; unknown
/// outcomes on either side are reported but not compared.
fn compare_entity(c: &mut Comparison, at: &str, was: &Value, now: &Value) {
    let (wf, nf) = (obj(&was["failing"]), obj(&now["failing"]));
    let (wu, nu) = (strings(&was["unknown"]), strings(&now["unknown"]));
    for rule in nu.iter().filter(|r| !wu.contains(r)) {
        c.other
            .push(format!("{at}: {rule} unknown now (its data is missing)"));
    }
    for (rule, n) in &nf {
        if wu.contains(rule) {
            c.other.push(format!("{at}: {rule} known again: fails"));
        } else if let Some(m) = wf.get(rule) {
            evidence_change(c, at, rule, m, n);
        } else if was["level"] == "skipped" {
            c.other
                .push(format!("{at}: no longer skipped; {rule} fails"));
        } else {
            c.regressions.push(format!("{at}: {rule} now fails"));
        }
    }
    for rule in wf
        .keys()
        .filter(|r| !nf.contains_key(*r) && !nu.contains(r))
    {
        c.improvements.push(format!("{at}: {rule} now passes"));
    }
    if was["level"] != now["level"] && wf.keys().eq(nf.keys()) {
        c.other
            .push(format!("{at}: level {} → {}", was["level"], now["level"]));
    }
}

/// A rule failing on both sides: its evidence count may have grown (worse) or shrunk (better).
fn evidence_change(c: &mut Comparison, at: &str, rule: &str, was: &Value, now: &Value) {
    match (was.as_u64(), now.as_u64()) {
        (Some(m), Some(n)) if n > m => c
            .regressions
            .push(format!("{at}: {rule} evidence grew {m} → {n}")),
        (Some(m), Some(n)) if n < m => c
            .improvements
            .push(format!("{at}: {rule} evidence shrank {m} → {n}")),
        _ => {}
    }
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

/// The report document the element renders: portal, snapshot, scorecard definitions, scores.
fn report_json(m: &toolbox_model::Manifest, d: &Derived) -> String {
    let fixes: Map<String, Value> = m
        .scorecards
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
                json!({"kind": sc.kind, "levels": sc.levels, "owner": sc.owner, "rules": rules}),
            )
        })
        .collect();
    json!({
        "portal": m.portal.title.as_deref().unwrap_or(&m.portal.name),
        "snapshot": d.snapshot,
        "scorecards": fixes,
        "scores": d.scores,
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

fn html(m: &toolbox_model::Manifest, d: &Derived) -> String {
    html_page(&report_json(m, d))
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
) -> Result<(), String> {
    write_web_parts(dir, &report_json(m, d))
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

pub fn run(o: &Options) -> ExitCode {
    let fail = |m: String| {
        eprintln!("error: {m}");
        ExitCode::from(2)
    };
    let (_, m, d) = match live(&o.path, o.fixtures, &o.skip) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let now = results(&d, &advisory(&m));
    let mut code = ExitCode::SUCCESS;
    let mut summary = String::new();
    if let Some(path) = &o.baseline {
        let before = std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok());
        let doc = json!({"format": FORMAT, "portal": m.portal.name, "results": now});
        let write = || std::fs::write(path, toolbox_derive::pyjson::pretty(&doc));
        match before {
            None => {
                if o.update {
                    if let Err(e) = write() {
                        return fail(format!("{}: {e}", path.display()));
                    }
                    summary = format!("baseline: created {}\n", path.display());
                } else {
                    summary = format!(
                        "baseline: {} doesn't exist; create it with --update\n",
                        path.display()
                    );
                    if o.ci {
                        code = ExitCode::FAILURE;
                    }
                }
            }
            Some(b) => {
                let c = compare(&b["results"], &now, &new_policy(&m));
                let mut lines = Vec::new();
                for r in &c.regressions {
                    lines.push(format!("✗ worse   {r}"));
                }
                for r in &c.improvements {
                    lines.push(format!("✓ better  {r}"));
                }
                for r in &c.other {
                    lines.push(format!("· changed {r}"));
                }
                let differs = b["results"] != now;
                let unknown = unknown_count(&now);
                let verdict = if !c.regressions.is_empty() {
                    code = ExitCode::FAILURE;
                    format!(
                        "{} regression(s) against {}",
                        c.regressions.len(),
                        path.display()
                    )
                } else if unknown > 0 && (o.ci || o.update) {
                    // Writing unknowns would silently drop those rules from the ratchet.
                    code = ExitCode::FAILURE;
                    format!(
                        "{unknown} rule outcome(s) unknown (missing tools or data); {} needs every rule evaluated",
                        if o.ci { "--ci" } else { "--update" }
                    )
                } else if unknown > 0 {
                    format!(
                        "no regressions; {unknown} rule outcome(s) unknown (missing tools or data), not compared"
                    )
                } else if differs && o.update {
                    if let Err(e) = write() {
                        return fail(format!("{}: {e}", path.display()));
                    }
                    format!("no regressions; updated {}", path.display())
                } else if differs {
                    if o.ci {
                        code = ExitCode::FAILURE;
                    }
                    format!(
                        "no regressions, but {} is out of date{}: run with --update and commit it",
                        path.display(),
                        if o.ci { " (--ci)" } else { "" }
                    )
                } else {
                    format!("matches {}", path.display())
                };
                summary = format!("{}\nbaseline: {verdict}\n", lines.join("\n"));
            }
        }
    }
    if o.output == "web" {
        let Some(dir) = &o.out else {
            return fail("-o web writes a directory; pass --out DIR".into());
        };
        if let Err(e) = write_web(dir, &m, &d) {
            return fail(e);
        }
        if let Some(p) = &o.html
            && let Err(e) = std::fs::write(p, html(&m, &d))
        {
            return fail(format!("{}: {e}", p.display()));
        }
        eprint!("{summary}");
        return code;
    }
    let body = match o.output.as_str() {
        "json" => toolbox_derive::pyjson::pretty(
            &json!({"format": FORMAT, "snapshot": d.snapshot, "scores": d.scores, "results": now}),
        ),
        "html" => html(&m, &d),
        _ => markdown(&m, &d),
    };
    if let Some(p) = &o.html
        && let Err(e) = std::fs::write(p, html(&m, &d))
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
            c.regressions,
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
            c.regressions,
            ["card · b: new, at none; new entities must reach bronze"]
        );
        assert_eq!(c.other, ["card · c: new (bronze)"]);
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
            c.other,
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
        assert_eq!(c.other[0], "card · a: lint known again: fails");
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
            c.improvements,
            [
                "card · a: docs evidence shrank 3 → 1",
                "card · a: tests now passes"
            ]
        );
        assert_eq!(c.other, ["card · c: new (none)"]);
    }

    #[test]
    fn the_html_page_is_the_element_with_slots_and_inlined_data() {
        let page = html_page(r#"{"portal":"Demo","snapshot":"s","scorecards":{},"scores":{}}"#);
        assert!(page.contains("<toolbox-report>"));
        assert!(page.contains(r#"<script type="application/json">{"portal":"Demo""#));
        assert!(page.contains(r#"<slot name="header">"#));
        assert!(page.contains(r#"<slot name="nav">"#));
        assert!(page.contains(r#"<slot name="footer">"#));
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
