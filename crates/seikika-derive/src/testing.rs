//! `seikika test` for CEL expectation files (`tests/*.test.yaml`, docs/v1-spec.md §11.2): each
//! `expect` is CEL with `entity(id)`, `score(scorecard, id)`, `campaign(name)` (its progress:
//! `inScope`, `meeting`, `percent`, `measure`, …), `options(action, id)` and `orphans()`, and
//! passes when it evaluates to `true`.

use crate::Derived;
use seikika_expr::{Env, Expr};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize)]
struct Suite {
    name: String,
    /// A file may hold only action cases (`runs`), which `seikika test` runs itself.
    #[serde(rename = "assert", default)]
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    name: String,
    expect: String,
}

/// The outcome of one expectation.
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub file: String,
    pub suite: String,
    pub name: String,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Serialize a derived value for an expectation.
fn json(v: impl Serialize) -> Result<serde_json::Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

type Lookups = Arc<Mutex<BTreeSet<String>>>;

fn lookup<T>(result: Result<T, String>, id: &str, lookups: &Option<Lookups>) -> Result<T, String> {
    if result.is_ok()
        && let Some(lookups) = lookups
    {
        lookups.lock().expect("fixture lookups").insert(id.into());
    }
    result
}

fn env(d: Arc<Derived>, lookups: Option<Lookups>) -> Env {
    let mut env = Env::default();
    let e = d.clone();
    let entity_lookups = lookups.clone();
    env.function1("entity", move |id| {
        lookup(
            e.entity_json(id).ok_or_else(|| format!("no entity {id}")),
            id,
            &entity_lookups,
        )
    })
    .expect("entity is not a stdlib function");
    let s = d.clone();
    env.function2("score", move |card, id| {
        let result = s
            .scores
            .get(card)
            .and_then(|m| m.get(id))
            .ok_or_else(|| format!("no score {card} for {id}"))
            .and_then(json);
        lookup(result, id, &lookups)
    })
    .expect("score is not a stdlib function");
    let c = d.clone();
    env.function1("campaign", move |name| {
        c.campaigns
            .get(name)
            .ok_or_else(|| format!("no campaign {name}"))
            .and_then(json)
    })
    .expect("campaign is not a stdlib function");
    let o = d.clone();
    env.function2("options", move |action, id| {
        o.options
            .get(action)
            .and_then(|inputs| inputs.values().next())
            .and_then(|by_entity| by_entity.get(id))
            .ok_or_else(|| format!("no options for {action} on {id}"))
            .and_then(json)
    })
    .expect("options is not a stdlib function");
    let r = d;
    env.function0("orphans", move || json(&r.orphans))
        .expect("orphans is not a stdlib function");
    env
}

/// Evidence collected from passing fixture assertions.
#[derive(Default)]
pub struct FixtureCoverage {
    pub scores: Vec<seikika_expr::analysis::ScoreAssertion>,
    pub entities: BTreeSet<String>,
}

/// Collect required score comparisons and successful entity and score lookups.
pub fn verified_coverage(
    root: &Path,
    files: &[String],
    derived: Arc<Derived>,
) -> Result<FixtureCoverage, String> {
    let lookups = Lookups::default();
    let env = env(derived.clone(), Some(lookups.clone()));
    let mut coverage = FixtureCoverage::default();
    for file in files {
        for case in read_suite(root, file)?.cases {
            if let Ok(expr) = Expr::compile(&case.expect)
                && expr
                    .eval_in(&env.scope())
                    .is_ok_and(|outcome| outcome.as_bool() == Some(true))
            {
                coverage
                    .scores
                    .extend(
                        expr.score_assertions()
                            .into_iter()
                            .filter_map(|mut assertion| {
                                let score = derived
                                    .scores
                                    .get(&assertion.card)?
                                    .get(&assertion.entity)?;
                                if score.level == "skipped" {
                                    return None;
                                }
                                if let Some((rule, outcome)) = &mut assertion.rule
                                    && outcome == "evidence"
                                {
                                    let actual = score.rules.as_ref()?.get(rule)?;
                                    if actual != "fail" {
                                        return None;
                                    }
                                    outcome.clone_from(actual);
                                }
                                Some(assertion)
                            }),
                    );
                coverage
                    .entities
                    .append(&mut lookups.lock().expect("fixture lookups"));
            } else {
                lookups.lock().expect("fixture lookups").clear();
            }
        }
    }
    Ok(coverage)
}

fn read_suite(root: &Path, file: &str) -> Result<Suite, String> {
    let text = std::fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))?;
    serde_norway::from_str(&text).map_err(|e| format!("{file}: {e}"))
}

/// Run the expectation files listed in the manifest's `tests` (paths relative to `root`).
pub fn run(
    root: &Path,
    files: &[String],
    derived: Arc<Derived>,
) -> Result<Vec<TestResult>, String> {
    let env = env(derived, None);
    let mut out = Vec::new();
    for file in files {
        let suite = read_suite(root, file)?;
        for case in suite.cases {
            let outcome = Expr::compile(&case.expect).and_then(|e| e.eval_in(&env.scope()));
            let (passed, error) = match outcome {
                Ok(v) if v.as_bool() == Some(true) => (true, None),
                Ok(other) => (false, Some(format!("evaluated to {other:?}"))),
                Err(e) => (false, Some(e)),
            };
            out.push(TestResult {
                file: file.clone(),
                suite: suite.name.clone(),
                name: case.name,
                passed,
                error,
            });
        }
    }
    Ok(out)
}
