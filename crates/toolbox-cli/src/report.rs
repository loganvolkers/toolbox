//! `toolbox report`: score a repository locally, without a server (docs/design/toolbox-for-toolbox.md
//! F1, F4; docs/design/scorecard-ratchet.md). Runs the portal's in-image connectors once (a resync
//! each), recomputes, and prints every scorecard with the evidence for failing rules — as Markdown,
//! JSON, or one self-contained HTML page.
//!
//! With `--baseline FILE` it is a ratchet in the manner of Betterer's `.betterer.results`: the file
//! records each entity's level and failing rules with their evidence counts. A rule that starts
//! failing, or a failing rule whose evidence grows, is a regression (exit 1). `--update` records
//! improvements; `--ci` also fails when the file is out of date, so the committed file always
//! matches the code.

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
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
    pub baseline: Option<PathBuf>,
    pub update: bool,
    pub ci: bool,
}

const FORMAT: &str = "toolbox-report/v1";

/// Run one in-image connector's resync and return its record and delete messages.
fn resync(name: &str, program: &str, config: &Value) -> Result<Vec<Value>, String> {
    let mut child = Command::new(program)
        .env("TOOLBOX_CONNECTOR_CONFIG", config.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("connector {name}: cannot start {program}: {e}"))?;
    let cmd = json!({"type": "command", "command": "resync", "command_id": "report"});
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
    o: &Options,
    root: &std::path::Path,
    m: &toolbox_model::Manifest,
) -> Result<toolbox_store::MemStore, String> {
    if o.fixtures {
        return toolbox_store::load_fixtures(root).map_err(|e| e.to_string());
    }
    let jobs: Vec<_> = m
        .connectors
        .iter()
        .filter(|c| c.runtime == "in-image" && !o.skip.contains(&c.name))
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
                    resync(&c.name, &program, &config).map(|r| (c.name.clone(), r))
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
fn results(d: &Derived) -> Value {
    let mut cards = Map::new();
    for (card, entities) in &d.scores {
        let mut rows = Map::new();
        for (id, s) in entities {
            let failing: Map<String, Value> = s
                .rules
                .iter()
                .flatten()
                .filter(|(_, o)| *o != "pass")
                .map(|(r, _)| {
                    let n = s.evidence.get(r).and_then(Value::as_array).map(Vec::len);
                    (r.clone(), json!(n))
                })
                .collect();
            let mut row = json!({"level": s.level});
            if !failing.is_empty() {
                row["failing"] = Value::Object(failing);
            }
            rows.insert(id.clone(), row);
        }
        cards.insert(card.clone(), Value::Object(rows));
    }
    Value::Object(cards)
}

#[derive(Default)]
struct Comparison {
    regressions: Vec<String>,
    improvements: Vec<String>,
    other: Vec<String>,
}

fn compare(before: &Value, after: &Value) -> Comparison {
    let mut c = Comparison::default();
    let empty = Map::new();
    let obj = |v: &Value| v.as_object().cloned().unwrap_or_default();
    let cards: std::collections::BTreeSet<&String> = before
        .as_object()
        .unwrap_or(&empty)
        .keys()
        .chain(after.as_object().unwrap_or(&empty).keys())
        .collect();
    for card in cards {
        let (b, a) = (obj(&before[card]), obj(&after[card]));
        for (id, now) in &a {
            let Some(was) = b.get(id) else {
                c.other.push(format!(
                    "{card} · {id}: new ({})",
                    now["level"].as_str().unwrap_or("?")
                ));
                continue;
            };
            let (wf, nf) = (obj(&was["failing"]), obj(&now["failing"]));
            for (rule, n) in &nf {
                match wf.get(rule) {
                    None if was["level"] != "skipped" => c
                        .regressions
                        .push(format!("{card} · {id}: {rule} now fails")),
                    None => c
                        .other
                        .push(format!("{card} · {id}: no longer skipped; {rule} fails")),
                    Some(m) => match (m.as_u64(), n.as_u64()) {
                        (Some(m), Some(n)) if n > m => c
                            .regressions
                            .push(format!("{card} · {id}: {rule} evidence grew {m} → {n}")),
                        (Some(m), Some(n)) if n < m => c
                            .improvements
                            .push(format!("{card} · {id}: {rule} evidence shrank {m} → {n}")),
                        _ => {}
                    },
                }
            }
            for rule in wf.keys().filter(|r| !nf.contains_key(*r)) {
                c.improvements
                    .push(format!("{card} · {id}: {rule} now passes"));
            }
            if was["level"] != now["level"] && wf.keys().eq(nf.keys()) {
                c.other.push(format!(
                    "{card} · {id}: level {} → {}",
                    was["level"], now["level"]
                ));
            }
        }
        for id in b.keys().filter(|id| !a.contains_key(*id)) {
            c.other.push(format!("{card} · {id}: removed"));
        }
    }
    c
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

/// One self-contained HTML page: the report as embedded JSON, rendered by a small inline script.
fn html(m: &toolbox_model::Manifest, d: &Derived) -> String {
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
    let data = json!({
        "portal": m.portal.title.as_deref().unwrap_or(&m.portal.name),
        "snapshot": d.snapshot,
        "scorecards": fixes,
        "scores": d.scores,
    });
    // `</` can't appear inside the script element.
    let embedded = data.to_string().replace("</", "<\\/");
    include_str!("report.html").replace("/*DATA*/null", &embedded)
}

pub fn run(o: &Options) -> ExitCode {
    let fail = |m: String| {
        eprintln!("error: {m}");
        ExitCode::from(2)
    };
    let src = match toolbox_model::load::load(&o.path) {
        Ok(s) => s,
        Err(e) => return fail(e.to_string()),
    };
    let m = match crate::portal::manifest_from(&src.document, &src.manifest_file) {
        Ok(m) => m,
        Err(e) => return fail(e),
    };
    let store = match inputs(o, &src.root, &m) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let d = FullRecompute.recompute(&m, &store.snapshot());
    for diag in &d.diagnostics {
        eprintln!("WARN  {}: {} [{}]", diag.location, diag.message, diag.code);
    }
    let now = results(&d);
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
                let c = compare(&b["results"], &now);
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
                let verdict = if !c.regressions.is_empty() {
                    code = ExitCode::FAILURE;
                    format!(
                        "{} regression(s) against {}",
                        c.regressions.len(),
                        path.display()
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
    let body = match o.output.as_str() {
        "json" => toolbox_derive::pyjson::pretty(
            &json!({"format": FORMAT, "snapshot": d.snapshot, "scores": d.scores, "results": now}),
        ),
        "html" => html(&m, &d),
        _ => markdown(&m, &d),
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
        let c = compare(&before, &after);
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
    fn ratchet_records_improvements_and_new_entities() {
        let before = r(json!({"a": {"level": "none", "failing": {"docs": 3, "tests": null}}}));
        let after = r(
            json!({"a": {"level": "none", "failing": {"docs": 1}}, "c": {"level": "none", "failing": {"x": null}}}),
        );
        let c = compare(&before, &after);
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
}
