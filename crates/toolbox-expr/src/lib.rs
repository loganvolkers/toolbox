//! CEL for Toolbox (docs/v1-spec.md §4 "CEL compile and evaluate", §6.2): one crate owns the
//! expression language, so the rest of the workspace never names a `cel` type.
//!
//! - **Static checks** ([`check`]), used by `toolbox validate`: an expression must parse, read only
//!   the top-level variables its manifest site provides ([`Site`]), and call only declared
//!   functions. Macro-bound variables (`i` in `xs.map(i, i.app)`) are not top-level references.
//!   Type checking is out of scope (cel-rust has no checker; see Q10).
//! - **Compile and evaluate** ([`Expr`], [`eval`]): JSON in and out through [`ExprValue`], variables
//!   bound once per entity with [`with_vars`], and extra functions through [`Env`].
//! - **Lazy lists** ([`lazy`]): `inbound.<relation>` converts only the elements an expression
//!   visits.
//! - **Analysis** ([`analysis`]): node count, depth, loop nesting and the fields an expression
//!   reads, for the meta standards' readability rules, and the field equalities a list filter
//!   requires, which the API narrows candidates by.
//!
//! The implementation is cel-rust (a default behind this contract, charter goal 3): swapping it
//! changes this crate only.

pub mod analysis;
pub mod eval;
pub mod lazy;

pub use eval::{Env, Expr, ExprValue, Lazy, Scope, with_vals, with_vars};
pub use lazy::lazy_map;

use std::fmt;

/// Variables available to CEL at each manifest site (docs/v1-spec.md §6.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site {
    /// `kinds[].map[].id`, `.set`, `.relations.*`
    Map,
    /// `kinds[].fields.*.computed`
    Computed,
    /// `scorecards[].appliesTo`, `scorecards[].rules[].check`
    Scorecard,
    /// `authz.tuples[].emit`
    Tuples,
    /// `actions[].inputs.**.enumFrom`
    Options,
    /// `actions[].approval`, `.preconditions[]`, `.expect.postcondition`
    Action,
    /// `actions[].context.*.where`
    ContextLookup,
    /// `actions[].expect.refresh[].selector` and `.ids`: record fields are bare names, so only
    /// syntax is checked.
    RefreshScope,
}

impl Site {
    /// The variables an expression at this site may read. `None` means any identifier is allowed.
    pub fn variables(self) -> Option<&'static [&'static str]> {
        match self {
            Site::Map => Some(&["record"]),
            Site::Computed => Some(&["entity", "inbound", "policy"]),
            // `cards` is bound only for scorecards that read other scorecards' levels (a second
            // pass). Other sites never see it.
            Site::Scorecard => Some(&["entity", "policy", "cards"]),
            Site::Tuples | Site::Options => Some(&["entity", "policy"]),
            Site::Action => Some(&["entity", "inputs", "policy"]),
            Site::ContextLookup => Some(&["candidate", "entity", "inputs", "policy"]),
            Site::RefreshScope => None,
        }
    }
}

/// A problem found in one CEL expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CelIssue {
    /// The expression doesn't parse.
    Syntax(String),
    /// The expression reads variables that aren't available at its site.
    UnknownVariables {
        found: Vec<String>,
        allowed: Vec<String>,
    },
    /// The expression calls functions the runtime doesn't declare (it would fail when evaluated).
    UndeclaredFunctions(Vec<String>),
}

impl fmt::Display for CelIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CelIssue::Syntax(msg) => {
                write!(f, "CEL syntax error: {}", msg.lines().next().unwrap_or(msg))
            }
            CelIssue::UnknownVariables { found, allowed } => write!(
                f,
                "CEL reads {} not available here (available: {})",
                found
                    .iter()
                    .map(|v| format!("`{v}`"))
                    .collect::<Vec<_>>()
                    .join(", "),
                allowed.join(", ")
            ),
            CelIssue::UndeclaredFunctions(names) => write!(
                f,
                "CEL calls {} which the runtime doesn't provide",
                names
                    .iter()
                    .map(|v| format!("`{v}()`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

/// Parse `expr` and check its top-level variables against `site`.
pub fn check(expr: &str, site: Site) -> Result<(), CelIssue> {
    let program = cel::Program::compile(expr).map_err(|e| CelIssue::Syntax(e.to_string()))?;
    let mut undeclared: Vec<String> = program
        .references()
        .functions()
        .into_iter()
        .filter(|f| f.chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
        .filter(|f| !analysis::declared(f))
        .map(str::to_owned)
        .collect();
    if !undeclared.is_empty() {
        undeclared.sort();
        return Err(CelIssue::UndeclaredFunctions(undeclared));
    }
    let Some(allowed) = site.variables() else {
        return Ok(());
    };
    let refs = program.references();
    let mut found: Vec<String> = refs
        .variables()
        .into_iter()
        .filter(|v| !allowed.contains(v))
        .map(str::to_owned)
        .collect();
    if found.is_empty() {
        return Ok(());
    }
    found.sort();
    found.dedup();
    Err(CelIssue::UnknownVariables {
        found,
        allowed: allowed.iter().map(|s| s.to_string()).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_variables_are_not_references() {
        assert_eq!(
            check("inbound.installedOn.map(i, i.app)", Site::Computed),
            Ok(())
        );
        assert_eq!(
            check(
                "policy.approvedApps.filter(a, !(a in entity.installedApps))",
                Site::Options
            ),
            Ok(())
        );
    }

    #[test]
    fn unknown_variables_are_reported() {
        let err = check("entity.archived && inputs.app == 'x'", Site::Scorecard).unwrap_err();
        assert_eq!(
            err,
            CelIssue::UnknownVariables {
                found: vec!["inputs".into()],
                allowed: vec!["entity".into(), "policy".into(), "cards".into()]
            }
        );
    }

    #[test]
    fn syntax_errors_are_reported() {
        assert!(matches!(
            check("entity.(", Site::Scorecard),
            Err(CelIssue::Syntax(_))
        ));
    }

    #[test]
    fn refresh_scopes_only_check_syntax() {
        assert_eq!(check("app == inputs.app", Site::RefreshScope), Ok(()));
    }
}
