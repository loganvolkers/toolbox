//! Read endpoints behind MCP tools, CLI commands and UI pages (v1-spec §7.2–§7.3, minimal-ui §6):
//! graph neighbourhoods, explanations (field provenance, ownership evidence, rule outcomes),
//! action input options, and the composite page views.

use crate::access::Caller;
use crate::routes::problem;
use crate::state::{AppState, Published};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type S = State<Arc<AppState>>;

/// Outgoing references of an entity: declared relations (inverse of inbound), `ref` fields and the
/// applied owner.
pub(crate) fn outgoing(s: &AppState, p: &Published, id: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (tid, t) in &p.derived.entities {
        for (rel, srcs) in &t.inbound {
            if srcs.iter().any(|x| x == id) {
                out.push((rel.clone(), tid.clone()));
            }
        }
    }
    if let Some(e) = p.derived.entities.get(id)
        && let Some(kind) = s.manifest.kinds.iter().find(|k| k.name == e.kind)
    {
        for (name, f) in &kind.fields {
            if f.ty == "ref"
                && let Some(target) = e.fields.get(name).and_then(Value::as_str)
                && p.derived.entities.contains_key(target)
            {
                out.push((name.clone(), target.to_owned()));
            }
        }
        if let Some(owner) = e
            .fields
            .get("ownership")
            .and_then(|o| o.get("owner"))
            .and_then(Value::as_str)
        {
            out.push(("ownedBy".into(), owner.to_owned()));
        }
    }
    out
}

fn incoming(s: &AppState, p: &Published, id: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = p
        .derived
        .entities
        .get(id)
        .map(|e| {
            e.inbound
                .iter()
                .flat_map(|(rel, srcs)| srcs.iter().map(move |x| (rel.clone(), x.clone())))
                .collect()
        })
        .unwrap_or_default();
    for other in p.derived.entities.keys() {
        if other != id {
            out.extend(
                outgoing(s, p, other)
                    .into_iter()
                    .filter(|(rel, t)| t == id && !p.derived.entities[id].inbound.contains_key(rel))
                    .map(|(rel, _)| (rel, other.clone())),
            );
        }
    }
    out
}

/// Query of `GET /v1/entities/{id}/graph`.
#[derive(Deserialize)]
pub struct GraphQuery {
    relation: Option<String>,
    direction: Option<String>,
    depth: Option<usize>,
}

/// `GET /v1/entities/{id}/graph` — neighbours up to `depth` (1–3), only what the caller may see.
pub async fn graph(
    State(s): S,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<GraphQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    if !p.derived.entities.contains_key(&id)
        || !s
            .visibility(&caller, &[id.as_str()])
            .await
            .map(|v| v[0].view)
            .unwrap_or(false)
    {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    }
    let dir = q.direction.as_deref().unwrap_or("both");
    let depth = q.depth.unwrap_or(1).clamp(1, 3);
    let mut edges: BTreeSet<(String, String, String)> = BTreeSet::new(); // (from, relation, to)
    let mut frontier = vec![id.clone()];
    let mut seen: BTreeSet<String> = BTreeSet::from([id.clone()]);
    for _ in 0..depth {
        let mut next = Vec::new();
        for node in &frontier {
            let mut add = |from: &str, rel: &str, to: &str, other: &str| {
                if q.relation.as_deref().is_none_or(|r| r == rel) {
                    edges.insert((from.to_owned(), rel.to_owned(), to.to_owned()));
                    if seen.insert(other.to_owned()) {
                        next.push(other.to_owned());
                    }
                }
            };
            if dir != "in" {
                for (rel, t) in outgoing(&s, &p, node) {
                    add(node, &rel, &t, &t);
                }
            }
            if dir != "out" {
                for (rel, src) in incoming(&s, &p, node) {
                    add(&src, &rel, node, &src);
                }
            }
        }
        frontier = next;
    }
    let ids: Vec<&str> = seen.iter().map(String::as_str).collect();
    let vis = match s.visibility(&caller, &ids).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let visible: BTreeSet<&str> = ids
        .iter()
        .zip(vis)
        .filter(|(_, v)| v.view)
        .map(|(i, _)| *i)
        .collect();
    let nodes: Vec<Value> = visible
        .iter()
        .map(|i| {
            let e = &p.derived.entities[*i];
            json!({"id": i, "kind": e.kind, "title": e.fields.get("title").or(e.fields.get("displayName"))})
        })
        .collect();
    let edges: Vec<Value> = edges
        .into_iter()
        .filter(|(a, _, b)| visible.contains(a.as_str()) && visible.contains(b.as_str()))
        .map(|(a, r, b)| json!({"from": a, "relation": r, "to": b}))
        .collect();
    Json(json!({"snapshot": p.snapshot_id, "root": id, "nodes": nodes, "edges": edges}))
        .into_response()
}

/// Query of `GET /v1/entities/{id}/explain`: one field or one rule.
#[derive(Deserialize)]
pub struct ExplainQuery {
    field: Option<String>,
    rule: Option<String>,
}

/// Field names a CEL expression reads from `entity` (`entity.<name>`).
fn fields_read(expr: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = expr;
    while let Some(i) = rest.find("entity.") {
        let tail = &rest[i + 7..];
        let name: String = tail
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.insert(name);
        }
        rest = tail;
    }
    out
}

/// `GET /v1/entities/{id}/explain?field=|rule=<scorecard>/<rule>` — why a value is what it is.
pub async fn explain(
    State(s): S,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(q): Query<ExplainQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let vis = match s.visibility(&caller, &[id.as_str()]).await {
        Ok(v) => v[0],
        Err(r) => return r,
    };
    let Some(shaped) = s
        .shape_entity(&p, &caller, &id, vis, None)
        .filter(|_| vis.view)
    else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    };
    let e = &p.derived.entities[&id];
    let kind = s.manifest.kinds.iter().find(|k| k.name == e.kind).unwrap();
    if let Some(field) = &q.field {
        let Some(spec) = kind.fields.get(field) else {
            return problem(
                StatusCode::NOT_FOUND,
                "not-found",
                format!("{} has no field {field}", e.kind),
            );
        };
        let mut body = json!({"entity": id, "field": field, "snapshot": p.snapshot_id,
            "value": shaped["fields"].get(field), "source": e.provenance.get(field),
            "precedence": kind.sources.fields.get(field).unwrap_or(&kind.sources.default)});
        if let Some(c) = &spec.computed {
            body["computed"] = json!(c);
        }
        if let Some(t) = crate::connectors::source_confinement(&s, e.provenance.get(field)) {
            body["sourceConfinement"] = json!(t);
        }
        if let Some(a) = &spec.authority {
            // Ownership evidence (D17): the claim, the CODEOWNERS rules for the path, and the rule.
            let owners = seikika_derive::codeowners_for(e.fields.get(&a.rules), &a.path);
            let status = e
                .fields
                .get(field)
                .and_then(|v| v.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("none");
            let claim = e.fields.get(&a.claim);
            let why = match status {
                "verified" => format!(
                    "the claim {} is backed by CODEOWNERS for {} ({})",
                    claim.unwrap_or(&Value::Null),
                    a.path,
                    owners.join(" ")
                ),
                "unverified" => format!(
                    "the claim {} is not applied: CODEOWNERS for {} names {}, not the claimed team",
                    claim.unwrap_or(&Value::Null),
                    a.path,
                    if owners.is_empty() {
                        "nobody".into()
                    } else {
                        owners.join(" ")
                    }
                ),
                "inferred" => format!(
                    "no catalog-info claim; CODEOWNERS for {} names exactly one team ({})",
                    a.path,
                    owners.join(" ")
                ),
                _ => format!(
                    "no catalog-info claim and no single CODEOWNERS team for {}",
                    a.path
                ),
            };
            let fix = match status {
                "verified" => Value::Null,
                "unverified" => json!(format!(
                    "Make the claimed team a code owner of {} in CODEOWNERS, or change spec.owner to a team that is",
                    a.path
                )),
                _ => json!(format!(
                    "Add {} with spec.owner, and make that team a code owner of the file",
                    a.path
                )),
            };
            body["authority"] = json!({"type": a.ty, "status": status, "claim": claim, "codeowners_for_path": owners,
                                       "path": a.path, "explanation": why, "fix": fix});
        }
        return Json(body).into_response();
    }
    if let Some(rule) = &q.rule {
        let (card, rname) = rule.split_once('/').unwrap_or((rule.as_str(), ""));
        let Some(sc) = s.manifest.scorecards.iter().find(|x| x.name == card) else {
            return problem(
                StatusCode::NOT_FOUND,
                "not-found",
                format!("no scorecard {card}"),
            );
        };
        let Some(r) = sc.rules.iter().find(|x| x.name == rname) else {
            return problem(
                StatusCode::NOT_FOUND,
                "not-found",
                format!("{card} has no rule {rname}"),
            );
        };
        let scores = s.shape_scores(&p, &caller, &id, vis);
        let outcome = &scores
            .get(card)
            .map(|c| c["rules"].clone())
            .unwrap_or(Value::Null);
        let hidden = outcome.get("redacted").is_some();
        let observed: BTreeMap<String, Value> = if hidden {
            BTreeMap::new()
        } else {
            fields_read(&r.check)
                .into_iter()
                .map(|f| {
                    (
                        f.clone(),
                        shaped["fields"].get(&f).cloned().unwrap_or(Value::Null),
                    )
                })
                .collect()
        };
        return Json(
            json!({"entity": id, "scorecard": card, "rule": rname, "snapshot": p.snapshot_id,
            "level": r.level, "check": r.check, "fix": r.fix,
            "outcome": if hidden { outcome.clone() } else { outcome[rname]["outcome"].clone() },
            "evidence": if hidden { Value::Null } else { outcome[rname].get("evidence").cloned().unwrap_or(Value::Null) },
            "observed": observed}),
        )
        .into_response();
    }
    problem(
        StatusCode::BAD_REQUEST,
        "bad-request",
        "pass field= or rule=<scorecard>/<rule>",
    )
}

/// Query of an action input's options: the entity and a search string.
#[derive(Deserialize)]
pub struct OptionsQuery {
    entity: String,
    q: Option<String>,
}

/// `GET /v1/actions/{name}/options/{input}?entity=&q=` — per-entity options (enumFrom).
pub async fn options(
    State(s): S,
    headers: HeaderMap,
    Path((name, input)): Path<(String, String)>,
    Query(q): Query<OptionsQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let Some(action) = s.manifest.actions.iter().find(|a| a.name == name) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no action {name}"),
        );
    };
    if !s
        .visibility(&caller, &[q.entity.as_str()])
        .await
        .map(|v| v[0].view)
        .unwrap_or(false)
        || !p.derived.entities.contains_key(&q.entity)
    {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {}", q.entity),
        );
    }
    let allowed = s
        .allowed(&caller, &action.permission, &q.entity)
        .await
        .unwrap_or(false);
    let needle = q.q.as_deref().map(str::to_lowercase);
    let opts: Vec<Value> = p
        .derived
        .options
        .get(&name)
        .and_then(|i| i.get(&input))
        .and_then(|e| e.get(&q.entity))
        .into_iter()
        .flatten()
        .filter(|o| needle.as_ref().is_none_or(|n| o.to_lowercase().contains(n)))
        .map(|o| {
            let policy = s.manifest.policies.values().find_map(|v| v.get(o.as_str()));
            json!({"value": o, "label": o, "steward": policy.and_then(|v| v.get("steward")),
                   "permissions": policy.and_then(|v| v.get("permissions"))})
        })
        .collect();
    Json(json!({"action": name, "input": input, "entity": q.entity, "snapshot": p.snapshot_id, "allowed": allowed,
                "disabled_reason": if allowed { Value::Null } else { json!(format!("you lack `{}`", action.permission)) },
                "options": opts}))
    .into_response()
}

fn connector_freshness(s: &AppState, p: &Published, id: &str) -> Vec<Value> {
    let mut seen = BTreeSet::new();
    p.derived.entities[id]
        .records
        .iter()
        .filter(|(c, k, _)| seen.insert((c.clone(), k.clone())))
        .map(|(c, k, _)| json!({"connector": c, "kind": k, "last_read": s.hub.last_read(c, k).map(seikika_connect::hub::rfc3339)}))
        .collect()
}

/// `GET /v1/views/entity/{id}` — the entity page in one request.
pub async fn view_entity(State(s): S, headers: HeaderMap, Path(id): Path<String>) -> Response {
    if crate::public_catalog::guest(&s, &headers) {
        return crate::public_catalog::entity(&s, &id, false);
    }
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let vis = match s.visibility(&caller, &[id.as_str()]).await {
        Ok(v) => v[0],
        Err(r) => return r,
    };
    let Some(entity) = s
        .shape_entity(&p, &caller, &id, vis, None)
        .filter(|_| vis.view)
    else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no entity {id}"),
        );
    };
    let scores = s.shape_scores(&p, &caller, &id, vis);
    let fixes: BTreeMap<String, BTreeMap<String, Option<String>>> = s
        .manifest
        .scorecards
        .iter()
        .map(|sc| {
            (
                sc.name.clone(),
                sc.rules
                    .iter()
                    .map(|r| (r.name.clone(), r.fix.clone()))
                    .collect(),
            )
        })
        .collect();
    let mut out_rel: Vec<Value> = outgoing(&s, &p, &id)
        .into_iter()
        .take(50)
        .map(|(r, t)| json!({"relation": r, "to": t}))
        .collect();
    let in_rel: Vec<Value> = incoming(&s, &p, &id)
        .into_iter()
        .take(50)
        .map(|(r, f)| json!({"relation": r, "from": f}))
        .collect();
    out_rel.sort_by_key(|v| v.to_string());
    let mut actions = Vec::new();
    for a in s
        .manifest
        .actions
        .iter()
        .filter(|a| a.kind == p.derived.entities[&id].kind)
    {
        let allowed = s
            .allowed(&caller, &a.permission, &id)
            .await
            .unwrap_or(false);
        actions.push(json!({"name": a.name, "description": a.description, "allowed": allowed, "permission": a.permission}));
    }
    Json(
        json!({"snapshot": p.snapshot_id, "entity": entity, "scorecards": scores, "fixes": fixes,
                "relations": {"out": out_rel, "in": in_rel}, "actions": actions,
                "freshness": connector_freshness(&s, &p, &id), "viewer": viewer(&caller)}),
    )
    .into_response()
}

fn viewer(c: &Caller) -> Value {
    json!({"user": c.user, "admin": c.admin, "method": c.method})
}

/// Selects the public catalog projection before loading the signed-in home view.
#[derive(Deserialize)]
pub struct HomeQuery {
    #[serde(default)]
    public_catalog: bool,
}

/// `GET /v1/views/home` — my teams, what they own, failing rules, and kinds to browse.
pub async fn view_home(
    State(s): S,
    headers: HeaderMap,
    Query(query): Query<HomeQuery>,
) -> Response {
    let guest = crate::public_catalog::guest(&s, &headers);
    if query.public_catalog {
        return if guest {
            crate::public_catalog::report(&s)
        } else {
            StatusCode::NO_CONTENT.into_response()
        };
    }
    if guest {
        return crate::public_catalog::home(&s);
    }
    view_authenticated_home(s, headers).await
}

async fn view_authenticated_home(s: Arc<AppState>, headers: HeaderMap) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let login = caller.user.trim_start_matches("user:");
    let teams: Vec<String> = p
        .derived
        .entities
        .iter()
        .filter(|(_, e)| {
            e.fields
                .get("members")
                .and_then(Value::as_array)
                .is_some_and(|m| m.iter().any(|x| x == login))
        })
        .map(|(id, _)| id.clone())
        .collect();
    let owned: Vec<&String> = p
        .derived
        .entities
        .iter()
        .filter(|(_, e)| {
            e.fields
                .get("ownership")
                .and_then(|o| o.get("owner"))
                .and_then(Value::as_str)
                .is_some_and(|o| teams.iter().any(|t| t == o))
        })
        .map(|(id, _)| id)
        .collect();
    let ids: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
    let vis = match s.visibility(&caller, &ids).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let mut items = Vec::new();
    for (id, v) in ids.iter().zip(vis) {
        let scores = s.shape_scores(&p, &caller, id, v);
        let failing: Vec<String> = scores
            .iter()
            .flat_map(|(card, sc)| {
                sc["rules"]
                    .as_object()
                    .into_iter()
                    .flatten()
                    .filter(|(_, r)| r["outcome"] == "fail")
                    .map(move |(r, _)| format!("{card}/{r}"))
            })
            .collect();
        let levels: BTreeMap<&String, &Value> =
            scores.iter().map(|(k, v)| (k, &v["level"])).collect();
        items.push(json!({"id": id, "kind": p.derived.entities[*id].kind, "levels": levels, "failing": failing}));
    }
    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    for e in p.derived.entities.values() {
        *kinds.entry(&e.kind).or_default() += 1;
    }
    Json(json!({"snapshot": p.snapshot_id, "viewer": viewer(&caller), "portal": s.manifest.portal,
                "teams": teams, "owned": items, "kinds": kinds,
                "scorecards": s.manifest.scorecards.iter().map(|sc| json!({"name": sc.name, "kind": sc.kind})).collect::<Vec<_>>(),
                "campaigns": home_campaigns(&s, &p)}))
    .into_response()
}

fn home_campaigns(s: &AppState, p: &Published) -> Vec<Value> {
    s.manifest
        .campaigns
        .iter()
        .filter_map(|c| {
            let pr = p.derived.campaigns.get(&c.name)?;
            Some(json!({
                "name": c.name,
                "title": c.title,
                "status": pr.status,
                "percent": pr.percent,
                "targetPercent": pr.target_percent,
                "onTarget": pr.on_target,
            }))
        })
        .collect()
}

/// `GET /v1/views/scorecard/{name}` — level distribution and worst rules over visible entities.
pub async fn view_scorecard(State(s): S, headers: HeaderMap, Path(name): Path<String>) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let Some(sc) = s.manifest.scorecards.iter().find(|x| x.name == name) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no scorecard {name}"),
        );
    };
    let ids: Vec<&str> = p
        .derived
        .scores
        .get(&name)
        .map(|m| m.keys().map(String::as_str).collect())
        .unwrap_or_default();
    let vis = match s.visibility(&caller, &ids).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let mut by_level: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut fails: BTreeMap<String, usize> = BTreeMap::new();
    for (id, v) in ids.iter().zip(vis) {
        if !v.view {
            continue;
        }
        let shaped = s.shape_scores(&p, &caller, id, v);
        let card = &shaped[&name];
        by_level
            .entry(card["level"].as_str().unwrap_or("hidden").to_owned())
            .or_default()
            .push(id);
        for (r, o) in card["rules"].as_object().into_iter().flatten() {
            if o["outcome"] == "fail" {
                *fails.entry(r.clone()).or_default() += 1;
            }
        }
    }
    Json(json!({"snapshot": p.snapshot_id, "scorecard": sc, "by_level": by_level, "failing_rules": fails})).into_response()
}

/// `GET /v1/scorecards` — every scorecard with its kind, levels and level counts over the entities
/// the caller may see.
pub async fn scorecards(State(s): S, headers: HeaderMap) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let mut out = Vec::new();
    for sc in &s.manifest.scorecards {
        let rows = match visible_rows(&s, &p, &caller, &sc.name).await {
            Ok(r) => r,
            Err(r) => return r,
        };
        let mut by_level: BTreeMap<String, usize> = BTreeMap::new();
        for r in &rows {
            *by_level.entry(level_name(&r.1)).or_default() += 1;
        }
        out.push(json!({"name": sc.name, "kind": sc.kind, "owner": sc.owner, "levels": sc.levels,
                        "rules": sc.rules.iter().map(|r| json!({"name": r.name, "level": r.level})).collect::<Vec<_>>(),
                        "by_level": by_level}));
    }
    Json(json!({"snapshot": p.snapshot_id, "scorecards": out})).into_response()
}

/// Query of `GET /v1/scorecards/{name}`: filters on one rule's outcome or on a level.
#[derive(Deserialize)]
pub struct ResultsQuery {
    /// Only entities where this rule has `outcome` (default `fail` when a rule is given).
    rule: Option<String>,
    outcome: Option<String>,
    /// Only entities at this level (`none`, `skipped`, or a scorecard level).
    level: Option<String>,
}

/// `GET /v1/scorecards/{name}?rule=&outcome=&level=` — one row per visible entity: its level, the
/// rules it fails and their evidence. Answers "which entities fail rule X, and why" in one call.
pub async fn scorecard_results(
    State(s): S,
    headers: HeaderMap,
    Path(name): Path<String>,
    Query(q): Query<ResultsQuery>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let Some(sc) = s.manifest.scorecards.iter().find(|x| x.name == name) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no scorecard {name}"),
        );
    };
    if let Some(r) = &q.rule
        && !sc.rules.iter().any(|x| &x.name == r)
    {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("{name} has no rule {r}"),
        );
    }
    let rows = match visible_rows(&s, &p, &caller, &name).await {
        Ok(r) => r,
        Err(r) => return r,
    };
    let want = q.outcome.as_deref().unwrap_or("fail");
    let mut out = Vec::new();
    let mut by_level: BTreeMap<String, usize> = BTreeMap::new();
    for (id, card) in rows {
        let level = level_name(&card);
        if q.level.as_deref().is_some_and(|l| l != level) {
            continue;
        }
        let rules = card["rules"].as_object();
        if let Some(r) = &q.rule {
            let outcome = rules.and_then(|m| m.get(r)).map(|o| o["outcome"].clone());
            if outcome != Some(json!(want)) {
                continue;
            }
        }
        *by_level.entry(level.clone()).or_default() += 1;
        let failing: BTreeMap<&String, &Value> = rules
            .into_iter()
            .flatten()
            .filter(|(_, o)| o["outcome"] != "pass")
            .collect();
        let hidden = card["rules"].get("redacted").is_some();
        out.push(json!({"id": id, "level": card["level"],
                        "failing": if hidden { card["rules"].clone() } else { json!(failing) }}));
    }
    Json(json!({"snapshot": p.snapshot_id, "scorecard": name, "kind": sc.kind, "levels": sc.levels,
                "filter": {"rule": q.rule, "outcome": q.rule.as_ref().map(|_| want), "level": q.level},
                "count": out.len(), "by_level": by_level, "entities": out}))
    .into_response()
}

/// `GET /v1/campaigns` — every campaign with progress over entities the caller may see.
pub async fn campaigns(State(s): S, headers: HeaderMap) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let mut out = Vec::new();
    for c in &s.manifest.campaigns {
        let row = match campaign_row(&s, &p, &caller, c).await {
            Ok(r) => r,
            Err(r) => return r,
        };
        out.push(row);
    }
    Json(json!({"snapshot": p.snapshot_id, "campaigns": out})).into_response()
}

/// `GET /v1/campaigns/{name}` — one campaign's progress and per-entity gaps the caller may see.
pub async fn campaign_results(
    State(s): S,
    headers: HeaderMap,
    Path(name): Path<String>,
) -> Response {
    let caller = match s.caller(&headers).await {
        Ok(c) => c,
        Err(r) => return r,
    };
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let Some(c) = s.manifest.campaigns.iter().find(|x| x.name == name) else {
        return problem(
            StatusCode::NOT_FOUND,
            "not-found",
            format!("no campaign {name}"),
        );
    };
    match campaign_row(&s, &p, &caller, c).await {
        Ok(row) => Json(json!({"snapshot": p.snapshot_id, "campaign": row})).into_response(),
        Err(r) => r,
    }
}

/// `GET /v1/views/campaign/{name}` — composite campaign page for the minimal UI.
pub async fn view_campaign(State(s): S, headers: HeaderMap, Path(name): Path<String>) -> Response {
    campaign_results(State(s), headers, Path(name)).await
}

async fn campaign_row(
    s: &AppState,
    p: &Published,
    caller: &Caller,
    c: &seikika_model::manifest::Campaign,
) -> Result<Value, Response> {
    let progress = p.derived.campaigns.get(&c.name);
    let ids: Vec<&str> = progress
        .map(|pr| pr.entities.keys().map(String::as_str).collect())
        .unwrap_or_default();
    let vis = s.visibility(caller, &ids).await?;
    let mut meeting = 0usize;
    let mut in_scope = 0usize;
    let mut entities = Vec::new();
    for (id, v) in ids.iter().zip(vis) {
        if !v.view {
            continue;
        }
        in_scope += 1;
        let ent = progress.and_then(|pr| pr.entities.get(*id));
        let ok = ent.is_some_and(|e| e.meeting);
        if ok {
            meeting += 1;
        }
        entities.push(json!({
            "id": id,
            "meeting": ok,
            "gaps": ent.map(|e| &e.gaps).cloned().unwrap_or_default(),
        }));
    }
    let target = c.target_percent();
    let percent = (meeting * 100).checked_div(in_scope).unwrap_or(0) as u8;
    Ok(json!({
        "name": c.name,
        "title": c.title,
        "track": c.track,
        "owner": c.owner,
        "description": c.description,
        "start": c.start,
        "end": c.end,
        "targetPercent": target,
        "includes": c.includes.iter().map(|i| json!({
            "scorecard": i.scorecard,
            "atLeast": i.at_least,
            "rules": i.rules,
        })).collect::<Vec<_>>(),
        "status": progress.map(|pr| pr.status.as_str()).unwrap_or("active"),
        "inScope": in_scope,
        "meeting": meeting,
        "percent": percent,
        "onTarget": percent >= target,
        "milestones": progress.map(|pr| &pr.milestones),
        "entities": entities,
    }))
}

/// The visible entities scored by `name`, with their shaped (facet-checked) scorecard.
async fn visible_rows(
    s: &AppState,
    p: &Published,
    caller: &Caller,
    name: &str,
) -> Result<Vec<(String, Value)>, Response> {
    let ids: Vec<&str> = p
        .derived
        .scores
        .get(name)
        .map(|m| m.keys().map(String::as_str).collect())
        .unwrap_or_default();
    let vis = s.visibility(caller, &ids).await?;
    Ok(ids
        .iter()
        .zip(vis)
        .filter(|(_, v)| v.view)
        .filter_map(|(id, v)| {
            let card = s.shape_scores(p, caller, id, v).remove(name)?;
            Some((id.to_string(), card))
        })
        .collect())
}

fn level_name(card: &Value) -> String {
    card["level"].as_str().unwrap_or("hidden").to_owned()
}

/// `GET /v1/views/ops` — operator page: snapshot, connectors, pending commands, diagnostics.
pub async fn view_ops(State(s): S, headers: HeaderMap) -> Response {
    match s.caller(&headers).await {
        Ok(c) if c.admin => {}
        Ok(c) => {
            return problem(
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("{} is not a portal admin", c.user),
            );
        }
        Err(r) => return r,
    }
    let Some(p) = s.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let unverified: Vec<&String> = p
        .derived
        .entities
        .iter()
        .filter(|(_, e)| {
            e.fields
                .get("ownership")
                .and_then(|o| o.get("status"))
                .is_some_and(|st| st == "unverified")
        })
        .map(|(id, _)| id)
        .collect();
    let regs = s.hub.registrations();
    let connectors: Vec<Value> = s
        .manifest
        .connectors
        .iter()
        .map(|c| {
            let last: BTreeMap<&String, Value> =
                c.records.iter().map(|k| (k, json!(s.hub.last_read(&c.name, k).map(seikika_connect::hub::rfc3339)))).collect();
            json!({"name": c.name, "runtime": c.runtime, "registered": regs.get(&c.name).map(|r| seikika_connect::hub::rfc3339(r.at)), "last_read": last})
        })
        .collect();
    Json(json!({"snapshot": p.snapshot_id, "input_version": p.input_version, "recompute_ms": p.recompute.as_millis() as u64,
                "connectors": connectors, "pending_commands": s.hub.pending(),
                "diagnostics": p.derived.diagnostics, "unverified_ownership": unverified, "orphans": p.derived.orphans}))
    .into_response()
}
