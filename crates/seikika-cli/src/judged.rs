//! Judged-rule labels for the CLI text report.

pub use seikika_derive::judged::judged_rules;
use std::collections::BTreeSet;

/// What text reports, which can't show the web report's icons, write after a judged rule's name.
pub const MARK: &str = "✦";
/// The line that explains MARK, once per text report that uses it.
pub const LEGEND: &str = "✦ agent-judged: an agent's assessment decides the rule (`seikika assess`); \
     unmarked rules are measured by a tool or connector.";

/// `rule` as a text report names it: with MARK when it is judged.
pub fn marked(judged: &BTreeSet<(String, String)>, card: &str, rule: &str) -> String {
    match judged.contains(&(card.to_owned(), rule.to_owned())) {
        true => format!("{rule} {MARK}"),
        false => rule.to_owned(),
    }
}
