//! `toolbox test` for CEL expectation files (`tests/*.test.yaml`, docs/v1-spec.md §11.2): each
//! `expect` is CEL with `entity(id)`, `score(scorecard, id)`, `options(action, id)` and `orphans()`,
//! and passes when it evaluates to `true`.

use crate::Derived;
use cel::{Context, ExecutionError, ResolveResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use toolbox_expr::{Expr, to_cel};

#[derive(Debug, Deserialize)]
struct Suite {
    name: String,
    #[serde(rename = "assert")]
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

fn context(d: Arc<Derived>) -> Context<'static, 'static> {
    let mut ctx = Context::default();
    let e = d.clone();
    ctx.add_function("entity", move |id: Arc<String>| -> ResolveResult {
        e.entity_json(&id)
            .map(|v| to_cel(&v))
            .ok_or_else(|| ExecutionError::function_error("entity", format!("no entity {id}")))
    })
    .expect("entity is not a stdlib function");
    let s = d.clone();
    ctx.add_function(
        "score",
        move |card: Arc<String>, id: Arc<String>| -> ResolveResult {
            s.scores
                .get(card.as_str())
                .and_then(|m| m.get(id.as_str()))
                .map(|score| to_cel(&serde_json::to_value(score).unwrap()))
                .ok_or_else(|| {
                    ExecutionError::function_error("score", format!("no score {card} for {id}"))
                })
        },
    )
    .expect("score is not a stdlib function");
    let o = d.clone();
    ctx.add_function(
        "options",
        move |action: Arc<String>, id: Arc<String>| -> ResolveResult {
            o.options
                .get(action.as_str())
                .and_then(|inputs| inputs.values().next())
                .and_then(|by_entity| by_entity.get(id.as_str()))
                .map(|opts| to_cel(&serde_json::to_value(opts).unwrap()))
                .ok_or_else(|| {
                    ExecutionError::function_error(
                        "options",
                        format!("no options for {action} on {id}"),
                    )
                })
        },
    )
    .expect("options is not a stdlib function");
    let r = d;
    ctx.add_function("orphans", move || -> ResolveResult {
        Ok(to_cel(&serde_json::to_value(&r.orphans).unwrap()))
    })
    .expect("orphans is not a stdlib function");
    ctx
}

/// Run the expectation files listed in the manifest's `tests` (paths relative to `root`).
pub fn run(
    root: &Path,
    files: &[String],
    derived: Arc<Derived>,
) -> Result<Vec<TestResult>, String> {
    let ctx = context(derived);
    let mut out = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))?;
        let suite: Suite = serde_norway::from_str(&text).map_err(|e| format!("{file}: {e}"))?;
        for case in suite.cases {
            let outcome = Expr::compile(&case.expect).and_then(|e| e.eval_in(&ctx));
            let (passed, error) = match outcome {
                Ok(cel::Value::Bool(true)) => (true, None),
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
