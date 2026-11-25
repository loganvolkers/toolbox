//! The executor types a manifest may name (docs/design/federated-actions.md §6.2): one table that
//! validation, the server and `toolbox-exec`'s parsers all read, and that a test holds equal to
//! the manifest schema's `executor.type` enum.

use crate::ci::CiType;
use crate::forge::Kind;

/// One executor `type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutorType {
    pub name: &'static str,
    /// The `executor:` fields this type needs.
    pub required: &'static [&'static str],
    /// Fields of which this type needs at least one.
    pub either: &'static [&'static str],
    /// A `dispatched` run is carried by a remote runner that reports later, so it outlives a
    /// restart of the portal that dispatched it. An `in-image` or `http` run is `dispatched` only
    /// when its executor answered `accepted` with an `executor_run_id` and no `result`.
    pub detached: bool,
    /// The forge type its `forge:` URL (default: that type's public host) must resolve to.
    pub forge: Option<Kind>,
    /// The type of `ci:` host it may name instead, for its endpoint and token.
    pub ci: Option<CiType>,
}

/// Every executor type; the manifest schema's `executor.type` enum must name exactly these.
pub const TYPES: &[ExecutorType] = &[
    ExecutorType {
        name: "command",
        required: &["run"],
        either: &[],
        detached: false,
        forge: None,
        ci: None,
    },
    ExecutorType {
        name: "in-image",
        required: &[],
        either: &["command", "package"],
        detached: true,
        forge: None,
        ci: None,
    },
    ExecutorType {
        name: "change",
        required: &["run"],
        either: &[],
        detached: false,
        forge: None,
        ci: None,
    },
    ExecutorType {
        name: "github-workflow",
        required: &["repo", "workflow"],
        either: &[],
        detached: true,
        forge: None,
        ci: Some(CiType::GithubActions),
    },
    ExecutorType {
        name: "gitlab-pipeline",
        required: &["project"],
        either: &[],
        detached: true,
        forge: Some(Kind::Gitlab),
        ci: Some(CiType::GitlabCi),
    },
    ExecutorType {
        name: "http",
        required: &["url"],
        either: &[],
        detached: true,
        forge: None,
        ci: None,
    },
    ExecutorType {
        name: "webhook",
        required: &["url"],
        either: &[],
        detached: false,
        forge: None,
        ci: None,
    },
];

/// The type a manifest names, if Toolbox has it.
pub fn get(name: &str) -> Option<&'static ExecutorType> {
    TYPES.iter().find(|t| t.name == name)
}

/// The names, for messages.
pub fn names() -> String {
    TYPES.iter().map(|t| t.name).collect::<Vec<_>>().join(", ")
}

/// Whether a run recorded with this executor survives a restart while `dispatched`.
pub fn detached(name: &str) -> bool {
    get(name).is_some_and(|t| t.detached)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn the_schema_names_exactly_the_executor_types() {
        let s: Value = serde_json::from_str(crate::schema::MANIFEST_SCHEMA).unwrap();
        let mut found = vec![];
        fn walk(v: &Value, found: &mut Vec<Vec<String>>) {
            match v {
                Value::Object(o) => {
                    if let Some(e) = o
                        .get("properties")
                        .and_then(|p| p.get("executor"))
                        .and_then(|e| e.get("properties"))
                        .and_then(|p| p.get("type"))
                        .and_then(|t| t.get("enum"))
                        .and_then(Value::as_array)
                    {
                        found.push(
                            e.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_owned)
                                .collect(),
                        );
                    }
                    o.values().for_each(|x| walk(x, found));
                }
                Value::Array(a) => a.iter().for_each(|x| walk(x, found)),
                _ => {}
            }
        }
        walk(&s, &mut found);
        assert_eq!(found.len(), 1, "one executor.type enum in the schema");
        let mut schema = found.remove(0);
        schema.sort();
        let mut ours: Vec<String> = TYPES.iter().map(|t| t.name.to_owned()).collect();
        ours.sort();
        assert_eq!(schema, ours);
    }
}
