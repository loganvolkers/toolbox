//! `seikika assess`: the push model for judgement (docs/design/scorecards-for-scorecards.md §6).
//! Seikika holds no model keys and calls no model. A developer's own coding agent reads a brief
//! (`brief`), writes an assessment file, and Seikika validates it (`check`) and reads it as data —
//! from the portal's `assessments/` directory through a connector, or pushed to a server
//! (`submit`).

mod queue;
mod run;

use seikika_api::assess;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

type AssessmentKey = (String, String);

struct AssessmentFile {
    path: String,
    doc: Value,
}

/// CLI options for assessment validation and the scoped work queue.
#[derive(clap::Args)]
pub struct Check {
    /// A portal directory (containing seikika.yaml) or a manifest file.
    #[arg(default_value = ".")]
    portal: PathBuf,
    /// Use the portal's fixtures instead of running its connectors.
    #[arg(long)]
    fixtures: bool,
    /// Connectors not to run (comma-separated), e.g. a slow runner.
    #[arg(long, value_delimiter = ',')]
    skip: Vec<String>,
    /// Fail when a recorded assessment or review in scope is stale.
    #[arg(long)]
    require_fresh: bool,
    /// Output: md (default) or json.
    #[arg(short = 'o', long, default_value = "md", value_parser = ["md", "json"])]
    output: String,
    /// Limit the result to this rubric.
    #[arg(long)]
    rubric: Option<String>,
    /// Limit the result to a subject ID. Repeat to select multiple subjects.
    #[arg(long = "entity")]
    entities: Vec<String>,
    /// Limit the result to the subjects of a campaign.
    #[arg(long)]
    campaign: Option<String>,
    /// Print the unresolved work, grouped by its next action.
    #[arg(long)]
    queue: bool,
    /// Limit the queue to this many items (default: 20).
    #[arg(long, requires = "queue", value_parser = clap::value_parser!(u32).range(1..))]
    limit: Option<u32>,
    /// Show only work with this next action.
    #[arg(long, requires = "queue", value_parser = ["assess", "repair", "collect-evidence", "independent-review", "hand-off-review"])]
    action: Option<String>,
    /// Add a brief to each eligible item using the same collected snapshot.
    #[arg(long, requires = "queue")]
    with_briefs: bool,
    /// Named reviewer to check for conflicts with original assessors.
    #[arg(long, requires = "queue")]
    reviewer: Option<String>,
    /// Write a JSON run record with input, output, and version digests.
    #[arg(long)]
    run_manifest: Option<PathBuf>,
}

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

/// Compare the two version inputs separately. Legacy assessments can bind only the subject.
fn version_changes(
    current_subject: Option<&Value>,
    current_rubric: &str,
    doc: &Value,
) -> (bool, bool) {
    (
        current_subject != Some(&doc["subjectHash"]),
        doc["rubricHash"]
            .as_str()
            .is_some_and(|hash| hash != current_rubric),
    )
}

/// One entity's state against one rubric.
fn status(subject: Option<&Value>, rubric_hash: &str, a: Option<&Value>, id: &str) -> Value {
    let Some(a) = a else {
        return json!({"entity": id, "state": "missing"});
    };
    let r = seikika_connect::assessment::record(a, None);
    let (subject_changed, rubric_changed) = version_changes(subject, rubric_hash, a);
    let state = if subject_changed || rubric_changed {
        "stale"
    } else {
        "fresh"
    };
    json!({"entity": id, "state": state, "verdict": r["verdict"],
           "failing": r["failing"], "unsure": r["unsure"], "at": a["at"]})
}

fn check_error(
    options: &Check,
    started: std::time::Instant,
    stage: &str,
    connector: Option<&str>,
    error: impl std::fmt::Display,
) -> ExitCode {
    if let Err(write_error) = run::write_failure(options, started, stage, connector) {
        eprintln!("error: {write_error}");
    }
    fail(error)
}

struct LocalAssessments {
    by_key: BTreeMap<AssessmentKey, AssessmentFile>,
    invalid: BTreeMap<String, Vec<String>>,
    digest: Option<String>,
}

fn local_assessments(
    root: &Path,
    m: &seikika_model::Manifest,
    d: &seikika_derive::Derived,
    record_run: bool,
) -> LocalAssessments {
    let files = read_dir(&root.join("assessments"));
    let digest = record_run.then(|| run::json_digest(&files));
    let mut by_key = BTreeMap::new();
    let mut invalid = BTreeMap::new();
    for (rel, doc) in files {
        let path = format!("assessments/{rel}");
        match checked_file(m, d, doc) {
            Ok((key, doc)) => match by_key.entry(key) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(AssessmentFile { path, doc });
                }
                std::collections::btree_map::Entry::Occupied(entry) => {
                    let (rubric, subject) = entry.key();
                    invalid.insert(
                        path,
                        vec![format!("duplicate assessment for {rubric} on {subject}")],
                    );
                }
            },
            Err(problems) => {
                invalid.insert(path, problems);
            }
        }
    }
    LocalAssessments {
        by_key,
        invalid,
        digest,
    }
}

fn check_report(
    options: &Check,
    m: &seikika_model::Manifest,
    d: &seikika_derive::Derived,
    files: &LocalAssessments,
) -> Result<(Value, u8), String> {
    let mut scope = queue::Scope::new(options, m, d)?;
    let mut rubrics = serde_json::Map::new();
    let mut work = Vec::new();
    let mut stale = false;
    for r in &m.rubrics {
        if !scope.rubric(r) {
            continue;
        }
        let rubric_hash = assess::rubric_hash(r);
        let mut rows = Vec::new();
        for (id, entity) in &d.entities {
            if entity.kind != r.kind || !assess::applies(r, &entity.fields) || !scope.includes(id) {
                continue;
            }
            scope.matched.insert(id);
            let file = files.by_key.get(&(r.name.clone(), id.clone()));
            let subject = entity.fields.get(r.subject());
            let row = status(subject, &rubric_hash, file.map(|f| &f.doc), id);
            stale |= row["state"] == "stale";
            if options.queue {
                work.extend(queue::pending(r, entity, file, row, &rubric_hash, &scope));
            } else {
                rows.push(row);
            }
        }
        rubrics.insert(r.name.clone(), json!(rows));
    }
    scope.check_matches()?;
    let code = u8::from(!files.invalid.is_empty() || options.require_fresh && stale);
    let report = if options.queue {
        queue::report(options, m, d, &scope, work, &files.invalid)
    } else {
        json!({"rubrics": rubrics, "invalid": files.invalid})
    };
    Ok((report, code))
}

/// Validate local assessment files and describe the scoped assessment work.
pub fn check(options: &Check) -> ExitCode {
    let started = std::time::Instant::now();
    let (root, m, inputs) =
        match crate::report::live_inputs(&options.portal, options.fixtures, &options.skip) {
            Ok(inputs) => inputs,
            Err(error) => {
                return check_error(
                    options,
                    started,
                    error.stage,
                    error.connector.as_deref(),
                    error.message,
                );
            }
        };
    let d = crate::report::derive_live(&m, &inputs);
    let files = local_assessments(&root, &m, &d, options.run_manifest.is_some());
    let (report, code) = match check_report(options, &m, &d, &files) {
        Ok(report) => report,
        Err(error) => return check_error(options, started, "scope", None, error),
    };
    let output = if options.output == "json" {
        format!("{}\n", seikika_derive::pyjson::pretty(&report))
    } else if options.queue {
        queue::markdown(&report)
    } else {
        check_markdown(&report)
    };
    if let (Some(path), Some(files_hash)) = (&options.run_manifest, files.digest)
        && let Err(error) = run::write(
            path,
            run::Record {
                options,
                root: &root,
                manifest: &m,
                inputs: &inputs,
                files_hash,
                output: &output,
                code,
                started,
            },
        )
    {
        return fail(error);
    }
    print!("{output}");
    ExitCode::from(code)
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
