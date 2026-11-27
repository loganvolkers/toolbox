//! `seikika assess`: the push model for judgement (docs/design/scorecards-for-scorecards.md §6).
//! Seikika holds no model keys and calls no model. A developer's own coding agent reads a brief
//! (`brief`), writes an assessment file, and Seikika validates it (`check`) and reads it as data —
//! from the portal's `assessments/` directory through a connector, or pushed to a server
//! (`submit`).

use seikika_api::assess;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

type AssessmentKey = (String, String);

fn fail(m: impl std::fmt::Display) -> ExitCode {
    eprintln!("error: {m}");
    ExitCode::from(2)
}

/// The entity body `assess::brief` expects, from derived state.
fn entity_body(d: &seikika_derive::Derived, id: &str) -> Option<Value> {
    let e = d.entities.get(id)?;
    Some(json!({"id": id, "kind": e.kind, "fields": e.fields}))
}

/// Print the brief for `entity` (markdown for an agent, or JSON) from the portal's current state.
pub fn brief(
    portal: &Path,
    entity: &str,
    rubric: Option<&str>,
    fixtures: bool,
    skip: &[String],
    output: &str,
) -> ExitCode {
    let (_, m, d) = match crate::report::live(portal, fixtures, skip) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let Some(body) = entity_body(&d, entity) else {
        return fail(format!("no entity {entity}"));
    };
    match assess::brief(&m, &body, rubric) {
        Ok(b) if output == "json" => {
            println!("{}", seikika_derive::pyjson::pretty(&b));
            ExitCode::SUCCESS
        }
        Ok(b) => {
            println!("{}", assess::markdown(&b));
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}

/// Every assessment file under `dir` (YAML or JSON), parsed.
fn read_dir(dir: &Path) -> Vec<(String, Result<Value, String>)> {
    let mut out = Vec::new();
    for rel in seikika_connect::files::walk(dir) {
        if !(rel.ends_with(".yaml") || rel.ends_with(".yml") || rel.ends_with(".json")) {
            continue;
        }
        out.push((rel.clone(), read_file(&dir.join(&rel))));
    }
    out
}

fn read_file(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_norway::from_str::<Value>(&text).map_err(|e| e.to_string())
}

fn checked_file(
    m: &seikika_model::Manifest,
    d: &seikika_derive::Derived,
    doc: Result<Value, String>,
) -> Result<(AssessmentKey, Value), Vec<String>> {
    let doc = doc.map_err(|e| vec![e])?;
    let problems = assess::problems(m, &doc);
    if !problems.is_empty() {
        return Err(problems);
    }
    let name = doc["rubric"].as_str().unwrap_or_default();
    let id = doc["entity"].as_str().unwrap_or_default();
    let subject = d
        .entities
        .get(id)
        .ok_or_else(|| vec![format!("no entity {id}")])?;
    let rubric = assess::rubric(m, &subject.kind, Some(name)).map_err(|e| vec![e])?;
    if !assess::applies(rubric, &subject.fields) {
        return Err(vec![format!("rubric {name} does not apply to {id}")]);
    }
    Ok(((name.to_owned(), id.to_owned()), doc))
}

/// One entity's state against one rubric.
fn status(
    d: &seikika_derive::Derived,
    subject: &str,
    rubric_hash: &str,
    a: Option<&Value>,
    id: &str,
) -> Value {
    let now = d.entities[id].fields.get(subject).cloned();
    match a {
        None => json!({"entity": id, "state": "missing"}),
        Some(a) => {
            let r = seikika_connect::assessment::record(a, None);
            let rubric_matches = a["rubricHash"].as_str().is_none_or(|h| h == rubric_hash);
            let fresh = now.as_ref() == Some(&a["subjectHash"]) && rubric_matches;
            json!({"entity": id, "state": if fresh { "fresh" } else { "stale" },
                   "verdict": r["verdict"], "failing": r["failing"], "unsure": r["unsure"], "at": a["at"]})
        }
    }
}

/// Validate `<portal>/assessments/` and print, per rubric, each entity's state.
/// Invalid files always fail. Require freshness explicitly when the caller needs current judgments.
pub fn check(
    portal: &Path,
    fixtures: bool,
    skip: &[String],
    output: &str,
    require_fresh: bool,
) -> ExitCode {
    let (root, m, d) = match crate::report::live(portal, fixtures, skip) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let mut invalid = BTreeMap::new();
    let mut by_key = BTreeMap::new();
    for (rel, doc) in read_dir(&root.join("assessments")) {
        match checked_file(&m, &d, doc) {
            Ok((key, doc)) => match by_key.entry(key) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(doc);
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    let (rubric, subject) = entry.key();
                    invalid.insert(
                        format!("assessments/{rel}"),
                        vec![format!("duplicate assessment for {rubric} on {subject}")],
                    );
                }
            },
            Err(problems) => {
                invalid.insert(format!("assessments/{rel}"), problems);
            }
        }
    }
    let mut rubrics = serde_json::Map::new();
    for r in &m.rubrics {
        let hash = assess::rubric_hash(r);
        let rows: Vec<Value> = d
            .entities
            .iter()
            .filter(|(_, e)| e.kind == r.kind && assess::applies(r, &e.fields))
            .map(|(id, _)| {
                status(
                    &d,
                    r.subject(),
                    &hash,
                    by_key.get(&(r.name.clone(), id.clone())),
                    id,
                )
            })
            .collect();
        rubrics.insert(r.name.clone(), json!(rows));
    }
    let report = json!({"rubrics": rubrics, "invalid": invalid});
    if output == "json" {
        println!("{}", seikika_derive::pyjson::pretty(&report));
    } else {
        print!("{}", check_markdown(&report));
    }
    let stale = rubrics.values().any(|rows| {
        rows.as_array()
            .is_some_and(|rows| rows.iter().any(|r| r["state"] == "stale"))
    });
    if invalid.is_empty() && !(require_fresh && stale) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn check_markdown(report: &Value) -> String {
    use std::fmt::Write as _;
    let mut s = String::new();
    for (name, rows) in report["rubrics"].as_object().into_iter().flatten() {
        let rows = rows.as_array().cloned().unwrap_or_default();
        let count = |st: &str| rows.iter().filter(|r| r["state"] == st).count();
        let _ = writeln!(
            s,
            "## {name}: {} fresh, {} stale, {} missing",
            count("fresh"),
            count("stale"),
            count("missing")
        );
        for r in rows.iter().filter(|r| r["state"] != "missing") {
            let issues: Vec<String> = ["failing", "unsure"]
                .iter()
                .filter_map(|k| {
                    let l = r[*k].as_array().filter(|l| !l.is_empty())?;
                    Some(format!(
                        "{k}: {}",
                        l.iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                })
                .collect();
            let _ = writeln!(
                s,
                "- {} `{}` {}{}",
                r["state"].as_str().unwrap_or(""),
                r["entity"].as_str().unwrap_or(""),
                r["verdict"].as_str().unwrap_or(""),
                if issues.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", issues.join("; "))
                }
            );
        }
    }
    for (file, problems) in report["invalid"].as_object().into_iter().flatten() {
        let _ = writeln!(s, "✗ {file}");
        for p in problems.as_array().into_iter().flatten() {
            let _ = writeln!(s, "  - {}", p.as_str().unwrap_or(""));
        }
    }
    s
}

/// `seikika assess submit` options.
pub struct Submit {
    pub portal: PathBuf,
    pub files: Vec<PathBuf>,
    pub url: String,
    pub connector: String,
    pub token: Option<String>,
}

/// Validate assessment files against the portal's rubrics and push them to `/v1/ingest` as
/// `assessment` records of a remote connector.
pub fn submit(o: &Submit) -> ExitCode {
    let src = match seikika_model::load::load(&o.portal) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let m = match crate::portal::manifest_from(&src) {
        Ok(m) => m,
        Err(e) => return fail(e),
    };
    let mut lines = Vec::new();
    let mut submitted = BTreeSet::new();
    for f in &o.files {
        let doc = match read_file(f) {
            Ok(d) => d,
            Err(e) => return fail(format!("{}: {e}", f.display())),
        };
        let problems = assess::problems(&m, &doc);
        if !problems.is_empty() {
            return fail(format!("{}: {}", f.display(), problems.join("; ")));
        }
        if let Some(hash) = doc["rubricHash"].as_str()
            && m.rubrics.iter().any(|r| {
                Some(r.name.as_str()) == doc["rubric"].as_str() && assess::rubric_hash(r) != hash
            })
        {
            return fail(format!("{}: rubric changed since the brief", f.display()));
        }
        let external_id = seikika_connect::assessment::external_id(&doc);
        if !submitted.insert(external_id.clone()) {
            return fail(format!(
                "{}: duplicate assessment {external_id}",
                f.display()
            ));
        }
        let rec = json!({"type": "record", "kind": "assessment",
            "externalId": external_id,
            "data": seikika_connect::assessment::record(&doc, None)});
        lines.push(rec.to_string());
    }
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let mut req = reqwest::Client::new()
            .post(format!("{}/v1/ingest", o.url.trim_end_matches('/')))
            .header("content-type", "application/x-ndjson")
            .body(lines.join("\n"));
        req = match &o.token {
            Some(t) => req.bearer_auth(t),
            None => req.header("x-seikika-connector", &o.connector), // dev servers only
        };
        match req.send().await {
            Ok(r) if r.status().is_success() => {
                let v: Value = r.json().await.unwrap_or_default();
                println!(
                    "{} assessment(s) → input version {}",
                    lines.len(),
                    v["input_version"]
                );
                ExitCode::SUCCESS
            }
            Ok(r) => {
                eprintln!("{} {}", r.status(), r.text().await.unwrap_or_default());
                ExitCode::FAILURE
            }
            Err(e) => fail(e),
        }
    })
}
