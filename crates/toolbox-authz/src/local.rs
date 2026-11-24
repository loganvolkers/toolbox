//! Checks without OpenFGA, for a portal's checkout (`toolbox run --approve` with no server).
//!
//! Exact for relations built from direct types (with usersets and wildcards), `or`, computed
//! relations and `from`: membership then follows from the tuples alone. A relation that uses
//! `and` or `but not` is refused rather than guessed, so a checkout never grants more than the
//! server's OpenFGA would.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// OpenFGA's default resolution depth.
const MAX_DEPTH: usize = 25;

/// Tuples indexed by (object, relation) → users.
pub struct Local<'a> {
    model: &'a Value,
    users: BTreeMap<(&'a str, &'a str), Vec<&'a str>>,
}

impl<'a> Local<'a> {
    /// `tuples` are `{user, relation, object}` objects, as the derived state holds them.
    pub fn new(model: &'a Value, tuples: &'a [Value]) -> Self {
        let mut users: BTreeMap<(&str, &str), Vec<&str>> = BTreeMap::new();
        for t in tuples {
            if let (Some(u), Some(r), Some(o)) = (
                t["user"].as_str(),
                t["relation"].as_str(),
                t["object"].as_str(),
            ) {
                users.entry((o, r)).or_default().push(u);
            }
        }
        Self { model, users }
    }

    /// Whether `user` has `relation` on `object`.
    pub fn check(&self, user: &str, relation: &str, object: &str) -> Result<bool, String> {
        self.relation(user, relation, object, &mut BTreeSet::new(), 0)
    }

    fn relation(
        &self,
        user: &str,
        relation: &str,
        object: &str,
        seen: &mut BTreeSet<(String, String)>,
        depth: usize,
    ) -> Result<bool, String> {
        if depth > MAX_DEPTH {
            return Err(format!(
                "resolving {relation} on {object} went deeper than {MAX_DEPTH}"
            ));
        }
        if !seen.insert((relation.to_owned(), object.to_owned())) {
            return Ok(false);
        }
        let ty = object.split(':').next().unwrap_or_default();
        let rewrite = self.model["type_definitions"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|t| t["type"] == ty)
            .and_then(|t| t["relations"].get(relation))
            .ok_or_else(|| format!("the model defines no relation {relation} on type {ty}"))?;
        let found = self.rewrite(user, rewrite, relation, object, seen, depth)?;
        seen.remove(&(relation.to_owned(), object.to_owned()));
        Ok(found)
    }

    fn rewrite(
        &self,
        user: &str,
        rw: &Value,
        relation: &str,
        object: &str,
        seen: &mut BTreeSet<(String, String)>,
        depth: usize,
    ) -> Result<bool, String> {
        if rw.get("this").is_some() {
            return self.direct(user, relation, object, seen, depth);
        }
        if let Some(r) = rw["computedUserset"]["relation"].as_str() {
            return self.relation(user, r, object, seen, depth + 1);
        }
        if let Some(ts) = rw["tupleToUserset"]["tupleset"]["relation"].as_str() {
            let r = rw["tupleToUserset"]["computedUserset"]["relation"]
                .as_str()
                .unwrap_or_default();
            for parent in self.users.get(&(object, ts)).into_iter().flatten() {
                if !parent.contains('#') && self.relation(user, r, parent, seen, depth + 1)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        if let Some(children) = rw["union"]["child"].as_array() {
            for c in children {
                if self.rewrite(user, c, relation, object, seen, depth)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        Err(format!(
            "{relation} on {object} uses `and` or `but not`, which only the server's OpenFGA checks: serve the portal"
        ))
    }

    fn direct(
        &self,
        user: &str,
        relation: &str,
        object: &str,
        seen: &mut BTreeSet<(String, String)>,
        depth: usize,
    ) -> Result<bool, String> {
        let user_type = user.split(':').next().unwrap_or_default();
        for u in self.users.get(&(object, relation)).into_iter().flatten() {
            if u.eq_ignore_ascii_case(user) || u.strip_suffix(":*") == Some(user_type) {
                return Ok(true);
            }
            if let Some((set, r)) = u.split_once('#')
                && self.relation(user, r, set, seen, depth + 1)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fga_dsl::to_json;
    use serde_json::json;

    fn t(user: &str, relation: &str, object: &str) -> Value {
        json!({"user": user, "relation": relation, "object": object})
    }

    const GROUPS: &str = "model\n  schema 1.1\ntype user\ntype group\n  relations\n    define member: [user, user:*, group#member]\n";

    #[test]
    fn members_directly_and_through_nested_groups() {
        let m = to_json(GROUPS).unwrap();
        let tuples = vec![
            t("user:dave", "member", "group:security"),
            t("group:leads#member", "member", "group:security"),
            t("user:erin", "member", "group:leads"),
            t("group:security#member", "member", "group:leads"),
        ];
        let l = Local::new(&m, &tuples);
        assert!(l.check("user:dave", "member", "group:security").unwrap());
        assert!(l.check("user:Erin", "member", "group:security").unwrap());
        assert!(l.check("user:dave", "member", "group:leads").unwrap());
        assert!(!l.check("user:carol", "member", "group:security").unwrap());
        assert!(!l.check("user:dave", "member", "group:other").unwrap());
    }

    #[test]
    fn wildcard_counts_users_of_its_type() {
        let m = to_json(GROUPS).unwrap();
        let tuples = vec![t("user:*", "member", "group:everyone")];
        let l = Local::new(&m, &tuples);
        assert!(l.check("user:anyone", "member", "group:everyone").unwrap());
        assert!(!l.check("bot:x", "member", "group:everyone").unwrap());
    }

    #[test]
    fn computed_and_from_relations() {
        let m = to_json(
            "model\n  schema 1.1\ntype user\ntype team\n  relations\n    define lead: [user]\n    define member: [user] or lead\ntype repo\n  relations\n    define owner: [team]\n    define writer: member from owner\n",
        )
        .unwrap();
        let tuples = vec![
            t("user:ann", "lead", "team:web"),
            t("team:web", "owner", "repo:site"),
        ];
        let l = Local::new(&m, &tuples);
        assert!(l.check("user:ann", "member", "team:web").unwrap());
        assert!(l.check("user:ann", "writer", "repo:site").unwrap());
        assert!(!l.check("user:bob", "writer", "repo:site").unwrap());
    }

    #[test]
    fn exclusion_and_unknown_relations_are_refused() {
        let m = to_json(
            "model\n  schema 1.1\ntype user\ntype group\n  relations\n    define blocked: [user]\n    define member: [user] but not blocked\n",
        )
        .unwrap();
        let tuples = vec![t("user:dave", "member", "group:security")];
        let l = Local::new(&m, &tuples);
        assert!(
            l.check("user:dave", "member", "group:security")
                .unwrap_err()
                .contains("but not")
        );
        assert!(
            l.check("user:dave", "admin", "group:security")
                .unwrap_err()
                .contains("no relation admin")
        );
    }
}
