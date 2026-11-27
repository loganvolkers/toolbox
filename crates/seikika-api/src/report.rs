//! The shared read model for the live portal report and a generated snapshot.

use crate::access::{Caller, Visibility, facet_needs, redacted};
use crate::routes::problem;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use seikika_derive::{Derived, judged};
use seikika_model::Manifest;
use serde::Deserialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Whether this score belongs to a comparison peer instead of the subject.
pub fn is_peer(d: &Derived, card: &str, id: &str) -> bool {
    d.comparisons
        .get(card)
        .is_some_and(|c| c.peers.iter().any(|p| p == id))
}

fn own_scores(d: &Derived) -> Map<String, Value> {
    d.scores
        .iter()
        .map(|(card, rows)| {
            let rows: Map<String, Value> = rows
                .iter()
                .filter(|(id, _)| !is_peer(d, card, id))
                .map(|(id, s)| (id.clone(), json!(s)))
                .collect();
            (card.clone(), Value::Object(rows))
        })
        .collect()
}

/// Represent whole measures as integer JSON values where they can be transmitted exactly.
pub fn number(x: f64) -> Value {
    if x.fract() == 0.0 && x.abs() < 9.0e15 {
        json!(x as i64)
    } else {
        json!(x)
    }
}

fn progress_towards_zero(from: f64, now: f64) -> Value {
    let percent = if from > 0.0 {
        ((from - now) / from * 1000.0).round() / 10.0
    } else if now > 0.0 {
        0.0
    } else {
        100.0
    };
    json!({"from": number(from), "now": number(now), "percentDone": percent})
}

/// Compare campaign progress against the baseline captured at its start.
pub fn paydown(start: &Value, p: &seikika_derive::CampaignProgress) -> Value {
    let mut out = json!({
        "start": start["start"],
        "recorded": start["recorded"],
        "entitiesShort": progress_towards_zero(
            start["short"].as_f64().unwrap_or(0.0),
            (p.in_scope - p.meeting) as f64
        ),
    });
    if let (Some(a), Some(b)) = (start["measure"].as_f64(), p.measure) {
        out["measure"] = progress_towards_zero(a, b);
    }
    out
}

/// Shape comparisons with their definitions, evidence, and participant scores.
pub fn comparison_defs(m: &seikika_model::Manifest, d: &Derived) -> Map<String, Value> {
    d.comparisons
        .iter()
        .map(|(name, c)| {
            let mut v = json!(c);
            let ids = || std::iter::once(&c.subject).chain(&c.peers);
            v["titles"] = ids()
                .map(|id| (id.clone(), json!(title_of(d, id))))
                .collect::<Map<String, Value>>()
                .into();
            v["facts"] = ids()
                .filter_map(|id| {
                    let entity = d.entities.get(id)?;
                    let fields: Map<_, _> = entity
                        .fields
                        .iter()
                        .filter(|(field, _)| !matches!(field.as_str(), "report" | "reportError"))
                        .map(|(field, value)| (field.clone(), value.clone()))
                        .collect();
                    Some((id.clone(), json!(fields)))
                })
                .collect::<Map<String, Value>>()
                .into();
            let sc = m.scorecards.iter().find(|s| &s.name == name);
            let kind = sc.and_then(|sc| m.kinds.iter().find(|k| k.name == sc.kind));
            if let (Some(sc), Some(kind)) = (sc, kind) {
                let reads: Map<String, Value> = sc
                    .rules
                    .iter()
                    .map(|r| {
                        let mut fields: Vec<String> = std::iter::once(r.check.as_str())
                            .chain(r.value.as_deref())
                            .filter_map(|cel| seikika_expr::analysis::analyse(cel).ok())
                            .flat_map(|a| a.fields)
                            .filter(|f| kind.fields.contains_key(f))
                            .collect();
                        fields.sort();
                        fields.dedup();
                        (r.name.clone(), json!(fields))
                    })
                    .collect();
                v["reads"] = reads.into();
                v["fields"] = kind
                    .fields
                    .iter()
                    .map(|(f, fd)| (f.clone(), json!(fd.description.as_deref().map(str::trim))))
                    .collect::<Map<String, Value>>()
                    .into();
            }
            let scores = d.scores.get(name);
            v["scores"] = ids()
                .filter_map(|id| Some((id.clone(), json!(scores?.get(id)?))))
                .collect::<Map<String, Value>>()
                .into();
            (name.clone(), v)
        })
        .collect()
}

/// Resolve an entity's display title, or use its identity when it has no title.
pub fn title_of<'a>(d: &'a Derived, id: &'a str) -> &'a str {
    d.entities
        .get(id)
        .and_then(|e| e.fields.get("title"))
        .and_then(Value::as_str)
        .unwrap_or(id)
}

fn scorecard_defs(m: &seikika_model::Manifest) -> Map<String, Value> {
    let judged = judged::judged_rules(m);
    m.scorecards
        .iter()
        .map(|sc| {
            let rules: Map<String, Value> = sc
                .rules
                .iter()
                .map(|r| {
                    let basis = if judged.contains(&(sc.name.clone(), r.name.clone())) {
                        "judged"
                    } else {
                        "measured"
                    };
                    let mut def = json!({
                        "level": r.level,
                        "fix": r.fix,
                        "check": r.check,
                        "basis": basis
                    });
                    if let Some(t) = &r.title {
                        def["title"] = json!(t.trim());
                    }
                    if let Some(why) = &r.rationale {
                        def["rationale"] = json!(why.trim());
                    }
                    if !r.links.is_empty() {
                        def["links"] = json!(r.links);
                    }
                    if r.value.is_some() {
                        def["value"] = json!(r.value);
                        def["better"] = json!(r.better);
                        def["unit"] = json!(r.unit);
                    }
                    (r.name.clone(), def)
                })
                .collect();
            let mut def = json!({
                "title": sc.title,
                "description": sc.description.as_deref().map(str::trim),
                "rationale": sc.rationale.as_deref().map(str::trim),
                "links": sc.links,
                "kind": sc.kind,
                "levels": sc.levels,
                "owner": sc.owner,
                "track": sc.track,
                "rules": rules
            });
            if let Some(c) = &sc.compare {
                def["compare"] = json!(c);
            }
            if let Some(a) = &sc.applies_to {
                def["appliesTo"] = json!(a.trim());
            }
            (sc.name.clone(), def)
        })
        .collect()
}

/// Combine campaign definitions, current progress, and any recorded starting point.
pub fn campaign_defs(
    m: &seikika_model::Manifest,
    d: &Derived,
    starts: &Map<String, Value>,
) -> Map<String, Value> {
    m.campaigns
        .iter()
        .map(|c| {
            let progress = d.campaigns.get(&c.name);
            let since = starts
                .get(&c.name)
                .zip(progress)
                .map(|(s, p)| paydown(s, p));
            let includes: Vec<Value> = c
                .includes
                .iter()
                .map(|i| {
                    json!({
                        "scorecard": i.scorecard,
                        "atLeast": i.at_least,
                        "rules": i.rules
                    })
                })
                .collect();
            (
                c.name.clone(),
                json!({
                    "title": c.title,
                    "track": c.track,
                    "owner": c.owner,
                    "description": c.description,
                    "start": c.start,
                    "end": c.end,
                    "targetPercent": c.target_percent(),
                    "draft": c.draft,
                    "includes": includes,
                    "status": progress.map(|p| &p.status),
                    "inScope": progress.map(|p| p.in_scope),
                    "meeting": progress.map(|p| p.meeting),
                    "percent": progress.map(|p| p.percent),
                    "onTarget": progress.map(|p| p.on_target),
                    "measure": progress.and_then(|p| p.measure).map(number),
                    "sinceStart": since,
                    "milestones": progress.map(|p| &p.milestones),
                    "entities": progress.map(|p| &p.entities),
                }),
            )
        })
        .collect()
}

/// Identify scored entities that are still missing a judgment-based assessment.
pub fn unassessed(m: &seikika_model::Manifest, d: &Derived) -> Map<String, Value> {
    let kinds = judged::assessment_kinds(m);
    let cards: BTreeSet<String> = judged::judged_rules(m)
        .into_iter()
        .map(|(sc, _)| sc)
        .collect();
    let assessed = |id: &str| {
        d.entities.get(id).is_some_and(|e| {
            e.inbound.values().flatten().any(|src| {
                d.entities
                    .get(src)
                    .is_some_and(|s| kinds.contains(s.kind.as_str()))
            })
        })
    };
    cards
        .iter()
        .filter_map(|card| {
            let ids: Vec<&String> = d
                .scores
                .get(card)?
                .keys()
                .filter(|id| !is_peer(d, card, id) && !assessed(id))
                .collect();
            (!ids.is_empty()).then(|| (card.clone(), json!(ids)))
        })
        .collect()
}

fn rubric_defs(m: &Manifest) -> Map<String, Value> {
    m.rubrics
        .iter()
        .map(|rubric| {
            (
                format!("rubric:{}/{}", m.portal.name, rubric.name),
                json!(rubric),
            )
        })
        .collect()
}

/// Build the shared report document for the viewer's audience or an offline publication.
pub fn snapshot(
    m: &Manifest,
    d: &Derived,
    starts: &Map<String, Value>,
    audience: Option<&Audience<'_>>,
) -> Value {
    let filtered = audience.map(|a| restrict(m, d, a));
    let collection = d;
    let d = filtered.as_ref().unwrap_or(d);
    let tracks: Map<String, Value> = m
        .tracks
        .iter()
        .map(|t| {
            (
                t.name.clone(),
                json!({
                    "title": t.title,
                    "owner": t.owner,
                    "description": t.description.as_deref().map(str::trim),
                    "links": t.links
                }),
            )
        })
        .collect();
    let mut catalog: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, e) in &d.entities {
        catalog.entry(e.kind.clone()).or_default().push(id.clone());
    }
    let kind_groups: Vec<Value> = m
        .kind_groups
        .iter()
        .map(|g| {
            json!({
                "name": g.name,
                "title": g.title,
                "description": g.description.as_deref().map(str::trim),
                "kinds": g.kinds
            })
        })
        .collect();
    let kinds: Map<String, Value> = m
        .kinds
        .iter()
        .map(|k| {
            (
                k.name.clone(),
                json!({"title": k.title, "description": k.description.as_deref().map(str::trim)}),
            )
        })
        .collect();
    json!({
        "portal": m.portal.title.as_deref().unwrap_or(&m.portal.name),
        "organization": organization(m),
        "snapshot": d.snapshot,
        "catalog": catalog,
        "entityOverview": public_entity_overviews(m, d),
        "entities": entity_details(m, d, collection, audience),
        "actions": actions(m, collection, d, audience),
        "children": children(m, collection, audience),
        "kinds": kinds,
        "kindGroups": kind_groups,
        "tracks": tracks,
        "campaigns": campaign_defs(m, d, starts),
        "scorecards": scorecard_defs(m),
        "scores": own_scores(d),
        "assessmentKinds": judged::assessment_kinds(m),
        "rubrics": rubric_defs(m),
        "unassessed": unassessed(m, d),
        "comparisons": comparison_defs(m, d),
        "repoRollup": m.policies.get("repoRollup").cloned().unwrap_or(Value::Null),
    })
}

/// Collect public fields needed on catalog rows and entity page headers.
pub fn public_entity_overviews(m: &seikika_model::Manifest, d: &Derived) -> Map<String, Value> {
    const FIELDS: &[&str] = &[
        "title",
        "displayName",
        "name",
        "description",
        "owner",
        "ownership",
        "links",
        "repositoryUrl",
        "sourceUrl",
        "visibility",
        "defaultBranch",
        "repository",
        "commit",
        "portalPresent",
        "scanError",
        "docsUrl",
        "runbookUrl",
        "grafanaUrl",
        "dashboardUrl",
        "onCallUrl",
        "registryUrl",
    ];
    let kinds: BTreeMap<_, _> = m
        .kinds
        .iter()
        .map(|kind| (&kind.name, &kind.fields))
        .collect();
    d.entities
        .iter()
        .filter_map(|(id, entity)| {
            let defs = kinds.get(&entity.kind)?;
            let mut overview = Map::new();
            for name in FIELDS {
                let Some(field) = defs.get(*name) else {
                    continue;
                };
                if !matches!(field.facet.as_deref(), None | Some("public")) {
                    continue;
                }
                let Some(value) = entity.fields.get(*name) else {
                    continue;
                };
                let presentation = if *name == "ownership" {
                    json!({ "owner": value["owner"].as_str(), "status": value["status"].as_str() })
                } else {
                    value.clone()
                };
                overview.insert((*name).to_owned(), presentation);
            }
            (!overview.is_empty()).then(|| (id.clone(), Value::Object(overview)))
        })
        .collect()
}

/// Per-entity access for an authenticated report request.
pub struct Audience<'a> {
    caller: &'a Caller,
    entities: BTreeMap<&'a str, Visibility>,
}

impl Audience<'_> {
    fn visibility(&self, id: &str) -> Visibility {
        self.entities.get(id).copied().unwrap_or_default()
    }

    fn facet(&self, id: &str, facet: Option<&str>) -> bool {
        let access = self.visibility(id);
        access.view
            && match facet {
                None | Some("public") => true,
                Some("operator") => self.caller.admin,
                Some(_) => access.details,
            }
    }
}

fn score_facet<'a>(card: &'a seikika_model::manifest::Scorecard, part: &str) -> &'a str {
    card.facets
        .as_ref()
        .and_then(|f| f.get(part))
        .and_then(Value::as_str)
        .unwrap_or(if part == "summary" { "public" } else { "owner" })
}

/// Remove hidden entities and project score and campaign data before it reaches the browser.
fn restrict(m: &Manifest, d: &Derived, audience: &Audience<'_>) -> Derived {
    let mut d = d.clone();
    d.entities.retain(|id, entity| {
        if !audience.visibility(id).view {
            return false;
        }
        if let Some(kind) = m.kinds.iter().find(|kind| kind.name == entity.kind) {
            entity.fields.retain(|name, _| {
                audience.facet(id, kind.fields.get(name).and_then(|f| f.facet.as_deref()))
            });
            entity
                .provenance
                .retain(|name, _| entity.fields.contains_key(name));
        }
        true
    });
    for card in &m.scorecards {
        if let Some(scores) = d.scores.get_mut(&card.name) {
            scores.retain(|id, score| {
                if !audience.facet(id, Some(score_facet(card, "summary"))) {
                    return false;
                }
                if !audience.facet(id, Some(score_facet(card, "details"))) {
                    score.rules = None;
                    score.evidence.clear();
                    score.measures.clear();
                    score.values.clear();
                }
                true
            });
        }
    }
    d.comparisons.retain(|name, comparison| {
        let view = m
            .scorecards
            .iter()
            .find(|card| &card.name == name)
            .is_some_and(|card| {
                std::iter::once(&comparison.subject)
                    .chain(&comparison.peers)
                    .all(|id| {
                        audience.facet(id, Some(score_facet(card, "summary")))
                            && audience.facet(id, Some(score_facet(card, "details")))
                    })
            });
        if !view && let Some(scores) = d.scores.get_mut(name) {
            for peer in &comparison.peers {
                scores.remove(peer);
            }
        }
        view
    });
    for campaign in &m.campaigns {
        if let Some(progress) = d.campaigns.get_mut(&campaign.name) {
            restrict_campaign(m, campaign, progress, audience);
        }
    }
    d
}

fn restrict_campaign(
    m: &Manifest,
    campaign: &seikika_model::manifest::Campaign,
    progress: &mut seikika_derive::CampaignProgress,
    audience: &Audience<'_>,
) {
    progress.entities.retain(|id, _| {
        audience.visibility(id).view
            && campaign.includes.iter().all(|include| {
                m.scorecards
                    .iter()
                    .find(|card| card.name == include.scorecard)
                    .is_some_and(|card| {
                        audience.facet(id, Some(score_facet(card, "summary")))
                            && (include.rules.is_empty()
                                || audience.facet(id, Some(score_facet(card, "details"))))
                    })
            })
    });
    if progress.entities.len() != progress.in_scope {
        progress.in_scope = progress.entities.len();
        progress.meeting = progress
            .entities
            .values()
            .filter(|entity| entity.meeting)
            .count();
        progress.percent = (100 * progress.meeting)
            .checked_div(progress.in_scope)
            .unwrap_or(0) as u8;
        progress.on_target = progress.in_scope > 0 && progress.percent >= progress.target_percent;
        progress.measure = None;
        for milestone in &mut progress.milestones {
            milestone.met = progress.in_scope > 0 && progress.percent >= milestone.target_percent;
        }
    } else if progress.entities.keys().any(|id| {
        campaign.includes.iter().any(|include| {
            m.scorecards
                .iter()
                .find(|card| card.name == include.scorecard)
                .is_some_and(|card| !audience.facet(id, Some(score_facet(card, "details"))))
        })
    }) {
        progress.measure = None;
    }
}

fn entity_details(
    m: &Manifest,
    d: &Derived,
    collection: &Derived,
    audience: Option<&Audience<'_>>,
) -> Map<String, Value> {
    let mut outgoing: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    let mut incoming: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for (id, entity) in &d.entities {
        for (relation, sources) in &entity.inbound {
            for from in sources.iter().filter(|id| d.entities.contains_key(*id)) {
                outgoing
                    .entry(from)
                    .or_default()
                    .push(json!({"relation": relation, "to": id}));
                incoming
                    .entry(id)
                    .or_default()
                    .push(json!({"relation": relation, "from": from}));
            }
        }
    }
    let kinds: BTreeMap<_, _> = m
        .kinds
        .iter()
        .map(|kind| (&kind.name, &kind.fields))
        .collect();
    d.entities
        .iter()
        .map(|(id, entity)| {
            let children_kind = m.federation.as_ref().and_then(|f| f.portals.as_ref());
            let fields = entity_fields(
                collection,
                id,
                kinds.get(&entity.kind).copied(),
                children_kind == Some(&entity.kind),
                audience,
            );
            let provenance: Map<String, Value> = fields
                .keys()
                .filter_map(|k| Some((k.clone(), json!(entity.provenance.get(k)?))))
                .collect();
            (
                id.clone(),
                json!({
                    "fields": fields, "provenance": provenance,
                    "relations": {
                        "in": incoming.remove(id.as_str()).unwrap_or_default(),
                        "out": outgoing.remove(id.as_str()).unwrap_or_default(),
                    },
                }),
            )
        })
        .collect()
}

// Only entity details get redaction markers. Aggregates use the restricted fields.
fn entity_fields(
    collection: &Derived,
    id: &str,
    definitions: Option<&BTreeMap<String, seikika_model::manifest::Field>>,
    is_child_portal: bool,
    audience: Option<&Audience<'_>>,
) -> Map<String, Value> {
    collection.entities[id]
        .fields
        .iter()
        .filter_map(|(field, value)| {
            if matches!(field.as_str(), "report" | "reportError")
                || is_child_portal
                    && matches!(
                        field.as_str(),
                        "actions" | "runner" | "dir" | "git" | "path" | "error"
                    )
            {
                return None;
            }
            let facet = definitions
                .and_then(|fields| fields.get(field))
                .and_then(|field| field.facet.as_deref());
            let Some(needs) = facet_needs(facet) else {
                return Some((field.clone(), value.clone()));
            };
            audience.map(|a| {
                let visible = if a.facet(id, facet) {
                    value.clone()
                } else {
                    redacted(collection, id, needs)
                };
                (field.clone(), visible)
            })
        })
        .collect()
}

fn organization(m: &Manifest) -> Option<&str> {
    m.connectors.iter().find_map(|connector| {
        connector
            .config
            .as_ref()?
            .get("githubOrg")?
            .get("organization")?
            .as_str()
    })
}

fn children(m: &Manifest, d: &Derived, audience: Option<&Audience<'_>>) -> Vec<Value> {
    let kind = m.federation.as_ref().and_then(|f| f.portals.as_deref());
    let github = organization(m).is_some();
    let own = format!("portal:{}", m.portal.name);
    d.entities.iter().filter(|(id, e)| {
        let github_report = github && e.kind == "Repository"
            && (e.fields.get("report").is_some_and(Value::is_object)
                || e.fields.get("reportError").is_some_and(Value::is_string));
        (kind == Some(&e.kind) && **id != own) || github_report
    })
    .filter(|(id, _)| audience.is_none_or(|a| {
        let v = a.visibility(id);
        v.view && (v.details || a.caller.admin)
    }))
    .map(|(id, e)| {
        let f = &e.fields;
        let name = f.get("key").or_else(|| f.get("name")).and_then(Value::as_str).unwrap_or(id);
        json!({
            "key": name,
            "title": f.get("title").and_then(Value::as_str).unwrap_or(name),
            "commit": f.get("git").and_then(|git| git["commit"].as_str())
                .or_else(|| f.get("commit").and_then(Value::as_str)),
            "report": f.get("report"),
            "error": f.get("reportError").and_then(Value::as_str)
                .or_else(|| f.get("error").and_then(Value::as_str))
                .map(|_| "The child snapshot could not be read.")
                .or_else(|| (!f.get("report").is_some_and(Value::is_object)).then_some("No child snapshot has been published.")),
        })
    }).collect()
}

fn actions(
    m: &Manifest,
    collection: &Derived,
    d: &Derived,
    audience: Option<&Audience<'_>>,
) -> Vec<Value> {
    seikika_exec::federation::offered(m, collection)
        .iter()
        .filter(|offered| {
            offered.entity.as_ref().is_none_or(|id| {
                audience.is_none_or(|a| {
                    let v = a.visibility(id);
                    v.view && (v.details || a.caller.admin)
                })
            })
        })
        .map(|offered| {
            let mut action = seikika_exec::federation::describe(offered, d);
            action["scope"] = json!(offered.action.scope);
            action["permission"] = json!(offered.action.permission);
            if let Some(child) = &offered.child {
                action["portal"] = json!({"name": child.portal, "key": child.key});
            } else if let Some(options) = d.options.get(&offered.action.name) {
                let choices: Map<_, _> = options
                    .iter()
                    .map(|(input, targets)| {
                        let targets: Map<_, _> = targets
                            .iter()
                            .filter(|(id, _)| d.entities.contains_key(*id))
                            .map(|(id, values)| (id.clone(), json!(values)))
                            .collect();
                        (input.clone(), Value::Object(targets))
                    })
                    .collect();
                action["choices"] = json!(choices);
            }
            action
        })
        .collect()
}

fn federation_problem(
    state: &AppState,
    input_version: u64,
) -> Option<(StatusCode, &'static str, &'static str)> {
    if crate::connectors::disabled() {
        return None;
    }
    let pending = |connector, record| {
        state
            .hub
            .first_sync(connector, record)
            .is_none_or(|version| input_version < version)
    };
    let mut waiting = crate::connectors::reads_children(state)
        && pending(seikika_model::federation::CONNECTOR, "portal");
    for connector in state.manifest.connectors.iter().filter(|connector| {
        connector.runtime == "in-image" && crate::connectors::is_github_org(connector)
    }) {
        if state
            .hub
            .first_sync(&connector.name, "federated-repo")
            .is_some()
            && state
                .hub
                .last_read(&connector.name, "federated-repo")
                .is_none()
        {
            return Some((
                StatusCode::BAD_GATEWAY,
                "organization-read-failed",
                "Unable to read the GitHub organization. Check the connector status and GitHub credential.",
            ));
        }
        waiting |= connector
            .records
            .iter()
            .any(|kind| pending(&connector.name, kind));
    }
    waiting.then_some((
        StatusCode::SERVICE_UNAVAILABLE,
        "reading-children",
        "Waiting for the first read of the federation sources",
    ))
}

#[derive(Default, Deserialize)]
pub(crate) struct ReportQuery {
    #[serde(default)]
    partial: bool,
}

/// The signed-in report, with read permissions applied before publishing any JSON.
pub(crate) async fn view(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReportQuery>,
    headers: HeaderMap,
) -> Response {
    let caller = match state.caller(&headers).await {
        Ok(caller) => caller,
        Err(response) => return response,
    };
    let Some(published) = state.snapshot() else {
        return problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "starting",
            "no snapshot yet",
        );
    };
    let federation = federation_problem(&state, published.input_version);
    if !query.partial
        && let Some((status, code, detail)) = federation
    {
        return problem(status, code, detail);
    }
    let ids: Vec<_> = published
        .derived
        .entities
        .keys()
        .map(String::as_str)
        .collect();
    let access = match state.visibility(&caller, &ids).await {
        Ok(access) => access,
        Err(response) => return response,
    };
    let audience = Audience {
        caller: &caller,
        entities: ids.into_iter().zip(access).collect(),
    };
    let mut report = snapshot(
        &state.manifest,
        &published.derived,
        &Map::new(),
        Some(&audience),
    );
    report["snapshot"] = json!(published.snapshot_id);
    report["viewer"] = json!(caller.user);
    if let Some((status, code, detail)) = federation {
        report["federationStatus"] = json!({
            "code": code, "detail": detail, "pending": status == StatusCode::SERVICE_UNAVAILABLE
        });
    }
    ([(header::CACHE_CONTROL, "private, no-store")], Json(report)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use seikika_derive::{Entity, Score};

    #[test]
    fn report_includes_rubrics_before_any_assessments_are_published() {
        let manifest: Manifest = serde_json::from_value(json!({
            "apiVersion": "seikika.com/v1alpha1",
            "portal": {"name": "review", "owner": "group:operators", "seikika": "0.2"},
            "kinds": [],
            "rubrics": [{"name": "design-quality", "kind": "Service",
                "criteria": [{"name": "documented", "question": "Can readers find the design?",
                    "guidance": "Pass when the design cites its sources."}],
                "links": [{"title": "Design guide", "url": "https://example.org/design"}]}],
        }))
        .unwrap();
        let report = snapshot(&manifest, &Derived::default(), &Map::new(), None);
        assert_eq!(report["catalog"], json!({}));
        assert_eq!(report["assessmentKinds"], json!([]));
        assert_eq!(
            report["rubrics"]["rubric:review/design-quality"]["criteria"][0]["question"],
            "Can readers find the design?"
        );
        assert_eq!(
            report["rubrics"]["rubric:review/design-quality"]["links"][0]["url"],
            "https://example.org/design"
        );
    }

    #[test]
    fn report_does_not_publish_hidden_children_or_score_evidence() {
        let manifest: Manifest = serde_json::from_value(json!({
            "apiVersion": "seikika.com/v1alpha1",
            "portal": {"name": "parent", "owner": "group:operators", "seikika": "0.2"},
            "federation": {"portals": "Portal"},
            "kinds": [{"name": "Portal", "identityFrom": [], "sources": {"default": []}, "fields": {
                "contact": {"type": "string", "facet": "owner"}
            }}],
            "scorecards": [{"name": "hygiene", "kind": "Portal", "owner": "group:operators",
                "levels": ["bronze"], "rules": []}],
        }))
        .unwrap();
        let mut derived = Derived::default();
        for name in ["public", "private"] {
            let id = format!("portal:{name}");
            derived.entities.insert(
                id.clone(),
                Entity {
                    kind: "Portal".into(),
                    fields: Map::from_iter([
                        ("report".into(), json!({"sensitive": name})),
                        ("contact".into(), json!("owner contact")),
                    ]),
                    provenance: BTreeMap::from([("contact".into(), "private source".into())]),
                    inbound: BTreeMap::new(),
                    records: Vec::new(),
                },
            );
            derived.scores.entry("hygiene".into()).or_default().insert(
                id,
                Score {
                    level: "none".into(),
                    rules: Some(BTreeMap::from([("rule".into(), "fail".into())])),
                    evidence: BTreeMap::from([("rule".into(), json!("owner evidence"))]),
                    measures: BTreeMap::new(),
                    values: BTreeMap::new(),
                },
            );
        }
        let caller = Caller {
            user: "user:reader".into(),
            subject: "reader".into(),
            method: "test",
            admin: false,
        };
        let audience = Audience {
            caller: &caller,
            entities: BTreeMap::from([(
                "portal:public",
                Visibility {
                    view: true,
                    details: false,
                },
            )]),
        };
        let report = snapshot(&manifest, &derived, &Map::new(), Some(&audience));
        assert_eq!(report["catalog"]["Portal"], json!(["portal:public"]));
        assert_eq!(report["children"], json!([]));
        assert_eq!(
            report["scores"]["hygiene"],
            json!({"portal:public": {"level": "none"}})
        );
        let entity = &report["entities"]["portal:public"];
        assert_eq!(
            entity["fields"],
            json!({"contact": {"redacted": true, "requires": "can_view_details", "visible_to": "portal admins"}})
        );
        assert_eq!(entity["provenance"], json!({}));
        let offline = snapshot(&manifest, &derived, &Map::new(), None);
        assert_eq!(offline["entities"]["portal:public"]["fields"], json!({}));
    }

    #[tokio::test]
    async fn partial_portal_report_keeps_local_data_while_waiting_for_federation() {
        let document = json!({
            "apiVersion": "seikika.com/v1alpha1",
            "portal": {"name": "parent", "owner": "group:operators", "seikika": "0.2"},
            "federation": {"children": [{"git": "../child", "key": "child"}]},
            "kinds": []
        });
        let manifest = serde_json::from_value(document.clone()).unwrap();
        let store = Arc::new(seikika_store::SharedMem(Default::default()));
        let state = crate::app_state(manifest, &document, store, None, Default::default());
        let derived = Derived {
            entities: BTreeMap::from([(
                "service:local".into(),
                Entity {
                    kind: "Service".into(),
                    fields: Map::new(),
                    provenance: BTreeMap::new(),
                    inbound: BTreeMap::new(),
                    records: Vec::new(),
                },
            )]),
            ..Default::default()
        };
        state.current.send_replace(Some(Arc::new(crate::Published {
            snapshot_id: "1.fixture".into(),
            derived: Arc::new(derived),
            input_version: 1,
            recompute: Default::default(),
            published_at: std::time::SystemTime::now(),
            decisions: None,
            by_field: Default::default(),
        })));
        let mut headers = HeaderMap::new();
        headers.insert("x-seikika-user", "reader".parse().unwrap());

        let complete = view(
            State(state.clone()),
            Query(ReportQuery::default()),
            headers.clone(),
        )
        .await;
        assert_eq!(complete.status(), StatusCode::SERVICE_UNAVAILABLE);
        let partial = view(State(state), Query(ReportQuery { partial: true }), headers).await;
        assert_eq!(partial.status(), StatusCode::OK);
        assert_eq!(
            partial.headers()[header::CACHE_CONTROL],
            "private, no-store"
        );
        let body = axum::body::to_bytes(partial.into_body(), 16_384)
            .await
            .unwrap();
        let report: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(report["catalog"]["Service"], json!(["service:local"]));
        assert_eq!(report["federationStatus"]["pending"], true);
    }
}
