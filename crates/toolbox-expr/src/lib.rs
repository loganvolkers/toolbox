//! CEL support for Toolbox (docs/v1-spec.md §4, §6.2).
//!
//! M0 provides static checks used by `toolbox validate`: an expression must parse, and the top-level
//! variables it reads must be among those available at its site. Macro-bound variables (`i` in
//! `xs.map(i, i.app)`) are not top-level references and are ignored. Type checking is out of scope
//! (cel-rust has no checker; see Q10).

pub mod eval;
pub mod lazy;

pub use eval::{Expr, from_cel, map_value, to_cel, with_vals, with_vars};
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
            Site::Scorecard | Site::Tuples | Site::Options => Some(&["entity", "policy"]),
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
        }
    }
}

/// Parse `expr` and check its top-level variables against `site`.
pub fn check(expr: &str, site: Site) -> Result<(), CelIssue> {
    let program = cel::Program::compile(expr).map_err(|e| CelIssue::Syntax(e.to_string()))?;
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
                allowed: vec!["entity".into(), "policy".into()]
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
