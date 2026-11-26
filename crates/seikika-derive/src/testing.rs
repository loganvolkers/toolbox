//! `seikika test` for CEL expectation files (`tests/*.test.yaml`, docs/v1-spec.md §11.2): each
//! `expect` is CEL with `entity(id)`, `score(scorecard, id)`, `campaign(name)` (its progress:
//! `inScope`, `meeting`, `percent`, `measure`, …), `options(action, id)` and `orphans()`, and
//! passes when it evaluates to `true`.

use crate::Derived;
use seikika_expr::{Env, Expr};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

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

fn env(d: Arc<Derived>) -> Env {
    let mut env = Env::default();
    let e = d.clone();
    env.function1("entity", move |id| {
        e.entity_json(id).ok_or_else(|| format!("no entity {id}"))
    })
    .expect("entity is not a stdlib function");
    let s = d.clone();
    env.function2("score", move |card, id| {
        s.scores
            .get(card)
            .and_then(|m| m.get(id))
            .ok_or_else(|| format!("no score {card} for {id}"))
            .and_then(json)
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

/// Run the expectation files listed in the manifest's `tests` (paths relative to `root`).
pub fn run(
    root: &Path,
    files: &[String],
    derived: Arc<Derived>,
) -> Result<Vec<TestResult>, String> {
    let env = env(derived);
    let mut out = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))?;
        let suite: Suite = serde_norway::from_str(&text).map_err(|e| format!("{file}: {e}"))?;
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
