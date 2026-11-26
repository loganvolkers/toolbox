//! Run OpenFGA's `.fga.yaml` test format through the Authorizer (docs/v1-spec.md §11.2): each
//! test gets a fresh store on an in-memory OpenFGA, the model (DSL → JSON) and the file's and the
//! test's tuples; then every `check` and `list_objects` assertion is evaluated.

use crate::fga_dsl;
use crate::openfga::{Datastore, OpenFga, Tuple};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct File {
    #[serde(default)]
    name: Option<String>,
    model_file: Option<String>,
    model: Option<String>,
    tuple_file: Option<String>,
    #[serde(default)]
    tuple_files: Vec<String>,
    #[serde(default)]
    tuples: Vec<TupleKey>,
    #[serde(default)]
    tests: Vec<Test>,
}

#[derive(Debug, Clone, Deserialize)]
struct TupleKey {
    user: String,
    relation: String,
    object: String,
}

#[derive(Debug, Deserialize)]
struct Test {
    name: String,
    #[serde(default)]
    tuples: Vec<TupleKey>,
    #[serde(default)]
    check: Vec<Check>,
    #[serde(default)]
    list_objects: Vec<ListObjects>,
}

#[derive(Debug, Deserialize)]
struct Check {
    user: String,
    object: String,
    assertions: BTreeMap<String, bool>,
}

#[derive(Debug, Deserialize)]
struct ListObjects {
    user: String,
    #[serde(rename = "type")]
    ty: String,
    assertions: BTreeMap<String, Vec<String>>,
}

/// One failed assertion.
#[derive(Debug, Clone, Serialize)]
pub struct Failure {
    pub test: String,
    pub assertion: String,
    pub expected: Value,
    pub got: Value,
}

/// Results for one `.fga.yaml` file, counted like `fga model test`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Report {
    pub file: String,
    pub tests: usize,
    pub tests_passed: usize,
    pub checks: usize,
    pub checks_passed: usize,
    pub list_objects: usize,
    pub list_objects_passed: usize,
    pub failures: Vec<Failure>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }
}

fn read_tuples(dir: &Path, rel: &str) -> Result<Vec<TupleKey>, String> {
    let text = std::fs::read_to_string(dir.join(rel)).map_err(|e| format!("{rel}: {e}"))?;
    if rel.ends_with(".json") {
        serde_json::from_str(&text).map_err(|e| format!("{rel}: {e}"))
    } else {
        serde_norway::from_str(&text).map_err(|e| format!("{rel}: {e}"))
    }
}

fn key(t: &TupleKey) -> Tuple {
    (t.user.clone(), t.relation.clone(), t.object.clone())
}

/// Run one test file (paths inside it resolve against its directory).
pub async fn run_file(path: &Path) -> Result<Report, String> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file: File =
        serde_norway::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let dsl = match (&file.model, &file.model_file) {
        (Some(m), _) => m.clone(),
        (None, Some(f)) => std::fs::read_to_string(dir.join(f)).map_err(|e| format!("{f}: {e}"))?,
        _ => return Err(format!("{}: needs model or model_file", path.display())),
    };
    let model = fga_dsl::to_json(&dsl)?;
    let mut base: Vec<TupleKey> = file.tuples.clone();
    for f in file.tuple_file.iter().chain(file.tuple_files.iter()) {
        base.extend(read_tuples(dir, f)?);
    }
    let mut report = Report {
        file: file.name.unwrap_or_else(|| path.display().to_string()),
        ..Report::default()
    };
    let mut fga = OpenFga::start(Datastore::Memory, "seikika-test", model.clone()).await?;
    for test in &file.tests {
        fga.fresh_store(&format!("test-{}", test.name)).await?;
        let tuples: Vec<Tuple> = base.iter().chain(test.tuples.iter()).map(key).collect();
        fga.write_tuples(&tuples, &[]).await?;
        let failures_before = report.failures.len();
        for c in &test.check {
            for (relation, expected) in &c.assertions {
                report.checks += 1;
                let got = fga.check(&c.user, relation, &c.object).await?;
                if got == *expected {
                    report.checks_passed += 1;
                } else {
                    report.failures.push(Failure {
                        test: test.name.clone(),
                        assertion: format!("check {} {relation} {}", c.user, c.object),
                        expected: (*expected).into(),
                        got: got.into(),
                    });
                }
            }
        }
        for l in &test.list_objects {
            for (relation, expected) in &l.assertions {
                report.list_objects += 1;
                let got = fga.list_objects(&l.user, relation, &l.ty).await?;
                let mut want = expected.clone();
                want.sort();
                if got == want {
                    report.list_objects_passed += 1;
                } else {
                    report.failures.push(Failure {
                        test: test.name.clone(),
                        assertion: format!("list_objects {} {relation} {}", l.user, l.ty),
                        expected: want.into(),
                        got: got.into(),
                    });
                }
            }
        }
        report.tests += 1;
        if report.failures.len() == failures_before {
            report.tests_passed += 1;
        }
    }
    Ok(report)
}
