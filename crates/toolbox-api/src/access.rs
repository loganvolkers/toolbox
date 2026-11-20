//! The one policy enforcement point (docs/v1-spec.md §6.5, §9): who the caller is, what they may
//! see, and facet shaping with redaction markers. Every read path goes through here.

use crate::routes::problem;
use crate::state::{AppState, Published};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde_json::{Value, json};
use toolbox_authz::authn::AuthError;
use toolbox_authz::fga_dsl;

/// An authenticated caller, mapped to a catalog user.
#[derive(Debug, Clone)]
pub struct Caller {
    /// `user:<login>`
    pub user: String,
    pub subject: String,
    pub method: &'static str,
    pub admin: bool,
}

/// What a caller may see of one entity.
#[derive(Debug, Clone, Copy, Default)]
pub struct Visibility {
    pub view: bool,
    pub details: bool,
}

impl AppState {
    /// Authenticate the request and map the subject to a `user:` entity (by email for email
    /// subjects, else by login).
    pub async fn caller(&self, headers: &HeaderMap) -> Result<Caller, Response> {
        let get = |name: &str| {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned)
        };
        let id = match self.authn.authenticate(get).await {
            Ok(id) => id,
            Err(AuthError::Missing) => {
                return Err(problem(
                    StatusCode::UNAUTHORIZED,
                    "unauthenticated",
                    "sign in through the identity-aware proxy, an OIDC token or a personal access token",
                ));
            }
            Err(AuthError::Invalid(m)) => {
                return Err(problem(StatusCode::UNAUTHORIZED, "invalid-credentials", m));
            }
        };
        let user = self.user_for(&id.subject);
        let admin = self.is_admin(&user).await;
        Ok(Caller {
            user,
            subject: id.subject,
            method: id.method,
            admin,
        })
    }

    fn user_for(&self, subject: &str) -> String {
        if subject.starts_with("user:") {
            return subject.to_owned();
        }
        if subject.contains('@')
            && let Some(p) = self.snapshot()
            && let Some((id, _)) = p.derived.entities.iter().find(|(_, e)| {
                e.kind == "User" && e.fields.get("email").and_then(Value::as_str) == Some(subject)
            })
        {
            return id.clone();
        }
        format!("user:{}", subject.split('@').next().unwrap_or(subject))
    }

    pub fn portal_object(&self) -> String {
        format!("portal:{}", self.manifest.portal.name)
    }

    async fn is_admin(&self, user: &str) -> bool {
        let Some(authz) = &self.authz else {
            return true;
        };
        if !fga_dsl::has_relation(authz.model(), "portal", "admin") {
            return false;
        }
        authz
            .check(user, "admin", &self.portal_object())
            .await
            .unwrap_or(false)
    }

    /// Visibility of many entities for one caller (batched OpenFGA checks).
    pub async fn visibility(
        &self,
        caller: &Caller,
        ids: &[&str],
    ) -> Result<Vec<Visibility>, Response> {
        let Some(authz) = &self.authz else {
            return Ok(vec![
                Visibility {
                    view: true,
                    details: true
                };
                ids.len()
            ]);
        };
        let model = authz.model();
        let mut checks = Vec::new();
        let mut plan = Vec::with_capacity(ids.len()); // per id: (view check index or fixed, details check index or fixed)
        for id in ids {
            let ty = id.split(':').next().unwrap_or_default();
            let mut slot = |rel: &str, fallback: bool| -> Result<usize, bool> {
                if fga_dsl::has_relation(model, ty, rel) {
                    checks.push((caller.user.clone(), rel.to_owned(), id.to_string()));
                    Ok(checks.len() - 1)
                } else {
                    Err(fallback)
                }
            };
            // Public by default (D15): types the model doesn't cover are visible to signed-in users;
            // their owner-only facets are visible to portal admins.
            let view = slot("can_view", true);
            let details = slot("can_view_details", caller.admin);
            plan.push((view, details));
        }
        let results = authz
            .batch_check(&checks)
            .await
            .map_err(|e| problem(StatusCode::BAD_GATEWAY, "authz-unavailable", e))?;
        let pick = |s: &Result<usize, bool>| match s {
            Ok(i) => results[*i],
            Err(f) => *f,
        };
        Ok(plan
            .iter()
            .map(|(v, d)| Visibility {
                view: pick(v),
                details: pick(d) || caller.admin,
            })
            .collect())
    }

    /// Check one relation; `Ok(false)` when the model doesn't define it.
    pub async fn allowed(
        &self,
        caller: &Caller,
        relation: &str,
        object: &str,
    ) -> Result<bool, Response> {
        let Some(authz) = &self.authz else {
            return Ok(true);
        };
        let ty = object.split(':').next().unwrap_or_default();
        if !fga_dsl::has_relation(authz.model(), ty, relation) {
            return Ok(false);
        }
        authz
            .check(&caller.user, relation, object)
            .await
            .map_err(|e| problem(StatusCode::BAD_GATEWAY, "authz-unavailable", e))
    }
}

/// The marker that replaces a hidden facet: which relation is needed and who holds it.
pub fn redacted(p: &Published, id: &str, requires: &str) -> Value {
    let owner = p
        .derived
        .entities
        .get(id)
        .and_then(|e| e.fields.get("ownership"))
        .and_then(|o| o.get("owner"))
        .and_then(Value::as_str);
    let visible_to = match (requires, owner) {
        ("admin", _) | (_, None) => "portal admins".to_owned(),
        (_, Some(o)) => format!("members of {o} and portal admins"),
    };
    json!({"redacted": true, "requires": requires, "visible_to": visible_to})
}

/// Facet level for a facet name: `public`, `operator`, or owner-only (any other name, e.g. `contact`).
fn facet_needs(facet: Option<&str>) -> Option<&'static str> {
    match facet {
        None | Some("public") => None,
        Some("operator") => Some("admin"),
        Some(_) => Some("can_view_details"),
    }
}

impl AppState {
    /// An entity as `caller` may see it (fields with hidden facets replaced by markers).
    pub fn shape_entity(
        &self,
        p: &Published,
        caller: &Caller,
        id: &str,
        vis: Visibility,
        fields: Option<&[&str]>,
    ) -> Option<Value> {
        let e = p.derived.entities.get(id)?;
        let kind = self.manifest.kinds.iter().find(|k| k.name == e.kind)?;
        let mut out = serde_json::Map::new();
        for (name, value) in &e.fields {
            if fields.is_some_and(|fs| !fs.contains(&name.as_str())) {
                continue;
            }
            let needs = facet_needs(kind.fields.get(name).and_then(|f| f.facet.as_deref()));
            let ok = match needs {
                None => true,
                Some("admin") => caller.admin,
                Some(_) => vis.details,
            };
            out.insert(
                name.clone(),
                if ok {
                    value.clone()
                } else {
                    redacted(p, id, needs.unwrap())
                },
            );
        }
        Some(json!({"id": id, "kind": e.kind, "fields": out, "provenance": e.provenance}))
    }

    /// Scorecard results for one entity as `caller` may see them.
    pub fn shape_scores(
        &self,
        p: &Published,
        caller: &Caller,
        id: &str,
        vis: Visibility,
    ) -> serde_json::Map<String, Value> {
        let mut out = serde_json::Map::new();
        for sc in &self.manifest.scorecards {
            let Some(score) = p.derived.scores.get(&sc.name).and_then(|m| m.get(id)) else {
                continue;
            };
            let facet = |part: &str| {
                sc.facets
                    .as_ref()
                    .and_then(|f| f.get(part))
                    .and_then(Value::as_str)
                    .unwrap_or(if part == "summary" { "public" } else { "owner" })
                    .to_owned()
            };
            let can = |f: &str| match f {
                "public" => vis.view,
                "operator" => caller.admin,
                _ => vis.details,
            };
            let level = if can(&facet("summary")) {
                json!(score.level)
            } else {
                redacted(
                    p,
                    id,
                    if facet("summary") == "operator" {
                        "admin"
                    } else {
                        "can_view_details"
                    },
                )
            };
            let mut card = json!({"level": level});
            if let Some(rules) = &score.rules {
                let details = facet("details");
                card["rules"] = if can(&details) {
                    let fixes: serde_json::Map<String, Value> = sc
                        .rules
                        .iter()
                        .filter_map(|r| r.fix.as_ref().map(|f| (r.name.clone(), json!(f))))
                        .collect();
                    let detailed: serde_json::Map<String, Value> = rules
                        .iter()
                        .map(|(name, outcome)| (name.clone(), json!({"outcome": outcome, "fix": if outcome == "pass" { Value::Null } else { fixes.get(name).cloned().unwrap_or(Value::Null) }})))
                        .collect();
                    Value::Object(detailed)
                } else {
                    redacted(
                        p,
                        id,
                        if details == "operator" {
                            "admin"
                        } else {
                            "can_view_details"
                        },
                    )
                };
            }
            out.insert(sc.name.clone(), card);
        }
        out
    }
}
