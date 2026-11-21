//! The derived-state pipeline (docs/v1-spec.md §6): a full recompute of every derived value from
//! an [`InputSnapshot`], behind the [`DerivedState`] contract so an incremental engine can replace
//! it (D37). The executable reference is `examples/reference-portal/tools/reference.py`; the
//! outputs of [`Derived::reference_files`] must match its files byte for byte (P2).

pub mod plan;
pub mod pyjson;
pub mod testing;

use rayon::prelude::*;
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use toolbox_expr::{Expr, from_cel, lazy_map, to_cel, with_vals, with_vars};
use toolbox_model::Manifest;
use toolbox_model::manifest::{Kind, TupleDerivation};
use toolbox_store::InputSnapshot;

/// Derived state, recomputed from inputs (D37).
pub trait DerivedState: Send + Sync {
    fn recompute(&self, manifest: &Manifest, inputs: &InputSnapshot) -> Derived;
}

/// The v1 engine: everything, every time, in parallel where stages allow.
#[derive(Debug, Clone, Copy, Default)]
pub struct FullRecompute;

impl DerivedState for FullRecompute {
    fn recompute(&self, manifest: &Manifest, inputs: &InputSnapshot) -> Derived {
        Pipeline::new(manifest).run(inputs)
    }
}

/// One derived entity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Entity {
    pub kind: String,
    /// Merged, authority and computed fields (not `id`).
    pub fields: Map<String, Value>,
    /// Which source (or `authority:codeowners`, `computed`) produced each field.
    pub provenance: BTreeMap<String, String>,
    /// Inbound relation → source entity ids, sorted.
    #[serde(skip)]
    pub inbound: BTreeMap<String, Vec<String>>,
    /// The raw records that contributed fragments: (connector, record kind, external id), sorted.
    /// Used to route refresh commands to connectors (connector-control §6).
    #[serde(skip)]
    pub records: Vec<(String, String, String)>,
}

/// One scorecard result for one entity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Score {
    pub level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<BTreeMap<String, String>>,
    /// For failing rules with an `evidence` expression: what makes them fail.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub evidence: BTreeMap<String, Value>,
}

/// A problem found while deriving (never fatal: the pipeline always produces a result).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
}

/// One entity's standing on a campaign.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CampaignEntity {
    pub meeting: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<String>,
}

/// A milestone with whether the campaign has reached it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMilestoneResult {
    pub name: String,
    pub target_percent: u8,
    pub met: bool,
}

/// Progress of one campaign after scorecards are scored (D01M7Y11G0YZT23W26Z9D3XW68Y).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignProgress {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    pub target_percent: u8,
    pub in_scope: usize,
    pub meeting: usize,
    pub percent: u8,
    pub on_target: bool,
    pub milestones: Vec<CampaignMilestoneResult>,
    pub entities: BTreeMap<String, CampaignEntity>,
}

/// Everything the pipeline produces for one snapshot.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Derived {
    /// `<input version>.<manifest hash prefix>` once a caller sets it; the pipeline leaves the
    /// input version only.
    pub snapshot: String,
    pub entities: BTreeMap<String, Entity>,
    /// scorecard → entity → score
    pub scores: BTreeMap<String, BTreeMap<String, Score>>,
    /// campaign → progress
    pub campaigns: BTreeMap<String, CampaignProgress>,
    /// Sorted, de-duplicated OpenFGA tuples (`{user, relation, object}`).
    pub tuples: Vec<Value>,
    /// action → input → entity → options
    pub options: BTreeMap<String, BTreeMap<String, BTreeMap<String, Vec<String>>>>,
    pub orphans: Vec<String>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Derived {
    /// The entity as CEL sees it: fields plus `id`.
    pub fn entity_json(&self, id: &str) -> Option<Value> {
        self.entities.get(id).map(|e| {
            let mut m = e.fields.clone();
            m.insert("id".into(), id.into());
            Value::Object(m)
        })
    }

    /// The reference model's five output files, as (relative path, contents).
    pub fn reference_files(&self) -> Vec<(&'static str, String)> {
        let entities: Vec<Value> = self
            .entities
            .iter()
            .map(|(id, e)| json!({"id": id, "kind": e.kind, "fields": e.fields, "provenance": e.provenance}))
            .collect();
        vec![
            (
                "expected/entities.json",
                pyjson::pretty(&Value::Array(entities)),
            ),
            (
                "expected/scorecards.json",
                pyjson::pretty(&serde_json::to_value(&self.scores).unwrap()),
            ),
            (
                "expected/options.json",
                pyjson::pretty(&serde_json::to_value(&self.options).unwrap()),
            ),
            (
                "expected/orphans.json",
                pyjson::pretty(&serde_json::to_value(&self.orphans).unwrap()),
            ),
            (
                "authz/tuples.expected.json",
                pyjson::pretty(&Value::Array(self.tuples.clone())),
            ),
        ]
    }
}

/// Compiled expressions for one manifest.
struct Pipeline<'m> {
    m: &'m Manifest,
    exprs: HashMap<String, Result<Expr, String>>,
    policy: cel::Value,
}

/// A mapped record: (entity id, field values, relation edges as (relation, target)).
/// A mapped record: its entity id, its field values, and its relation edges as (name, target).
type Mapped<'a> = (String, Map<String, Value>, Vec<(&'a str, String)>);

type Fragments = BTreeMap<String, BTreeMap<String, Map<String, Value>>>;

impl<'m> Pipeline<'m> {
    fn new(m: &'m Manifest) -> Self {
        let mut sources: Vec<&str> = Vec::new();
        for k in &m.kinds {
            for r in &k.map {
                sources.push(&r.id);
                sources.push(&r.set);
                sources.extend(r.relations.values().map(String::as_str));
            }
            sources.extend(k.fields.values().filter_map(|f| f.computed.as_deref()));
        }
        for s in &m.scorecards {
            sources.extend(s.applies_to.as_deref());
            sources.extend(s.rules.iter().map(|r| r.check.as_str()));
            sources.extend(s.rules.iter().filter_map(|r| r.evidence.as_deref()));
        }
        for t in m.authz.iter().flat_map(|a| &a.tuples) {
            if let TupleDerivation::Kind { emit, .. } = t {
                sources.push(emit);
            }
        }
        let enum_from: Vec<String> = m
            .actions
            .iter()
            .flat_map(|a| enum_from_inputs(&a.inputs))
            .map(|(_, e)| e)
            .collect();
        sources.extend(enum_from.iter().map(String::as_str));
        let exprs = sources
            .into_iter()
            .map(|s| (s.to_owned(), Expr::compile(s)))
            .collect();
        let policy = to_cel(&Value::Object(m.policies.clone()));
        Pipeline { m, exprs, policy }
    }

    fn expr(&self, src: &str) -> Result<&Expr, String> {
        match self.exprs.get(src) {
            Some(Ok(e)) => Ok(e),
            Some(Err(e)) => Err(e.clone()),
            None => Err(format!("expression not compiled: {src}")),
        }
    }

    fn json_in(&self, src: &str, ctx: &cel::Context<'_, '_>) -> Result<Value, String> {
        from_cel(&self.expr(src)?.eval_in(ctx)?)
    }

    fn run(&self, inputs: &InputSnapshot) -> Derived {
        let timings = std::env::var_os("TOOLBOX_TIMINGS").is_some();
        let mut clock = std::time::Instant::now();
        let mut lap = |label: &str| {
            if timings {
                eprintln!("  {label:<22} {:>8.2?}", clock.elapsed());
            }
            clock = std::time::Instant::now();
        };
        let mut d = Derived {
            snapshot: inputs.version.to_string(),
            ..Derived::default()
        };
        let kinds: BTreeMap<&str, &Kind> =
            self.m.kinds.iter().map(|k| (k.name.as_str(), k)).collect();

        // 1. Map raw records to fragments per source, plus relation edges.
        let mut fragments: Fragments = BTreeMap::new();
        let mut kind_of: HashMap<String, String> = HashMap::new();
        let mut records_of: HashMap<String, Vec<(String, String, String)>> = HashMap::new();
        // Relation edges grouped by source: (source id, [(relation, target id)]). Relation names
        // borrow the manifest, so an edge allocates only its target.
        let mut edges: Vec<(String, Vec<(&str, String)>)> = Vec::new();
        for kind in &self.m.kinds {
            for (ri, rule) in kind.map.iter().enumerate() {
                let (conn, rkind) = rule.from_parts();
                let source = rule.source_label();
                let recs: Vec<&toolbox_store::Record> = inputs
                    .records
                    .iter()
                    .filter(|r| r.connector == conn && r.kind == rkind)
                    .collect();
                let mapped: Vec<Result<Mapped, String>> = recs
                    .par_iter()
                    .map(|r| {
                        let record = to_cel(&r.data);
                        with_vars(&[("record", &record)], |ctx| {
                            let id = match self.json_in(&rule.id, ctx)? {
                                Value::String(s) => s,
                                other => {
                                    return Err(format!(
                                        "{}: id must be a string, got {other}",
                                        r.external_id
                                    ));
                                }
                            };
                            let values = match self.json_in(&rule.set, ctx)? {
                                Value::Object(m) => m,
                                other => {
                                    return Err(format!(
                                        "{}: set must be a map, got {other}",
                                        r.external_id
                                    ));
                                }
                            };
                            let mut rels = Vec::new();
                            for (rel, e) in &rule.relations {
                                match self.json_in(e, ctx)? {
                                    Value::Array(ts) => {
                                        rels.extend(ts.into_iter().filter_map(|t| match t {
                                            Value::String(t) => Some((rel.as_str(), t)),
                                            _ => None,
                                        }))
                                    }
                                    other => {
                                        return Err(format!(
                                            "relation {rel} must be a list, got {other}"
                                        ));
                                    }
                                }
                            }
                            Ok((id, values, rels))
                        })
                    })
                    .collect();
                for (rec, item) in recs.iter().zip(mapped) {
                    match item {
                        Ok((id, values, rels)) => {
                            kind_of.insert(id.clone(), kind.name.clone());
                            records_of.entry(id.clone()).or_default().push((
                                rec.connector.clone(),
                                rec.kind.clone(),
                                rec.external_id.clone(),
                            ));
                            let frag = fragments
                                .entry(id.clone())
                                .or_default()
                                .entry(source.clone())
                                .or_default();
                            frag.extend(values.into_iter().filter(|(_, v)| !v.is_null()));
                            if !rels.is_empty() {
                                edges.push((id, rels));
                            }
                        }
                        Err(e) => d.diagnostics.push(Diagnostic {
                            code: "map.error",
                            location: format!("kinds[{}].map[{ri}]", kind.name),
                            message: e,
                        }),
                    }
                }
            }
        }

        lap("map");
        // 2. Native values are one more source.
        for n in &inputs.native {
            fragments
                .entry(n.entity.clone())
                .or_default()
                .entry("native".into())
                .or_default()
                .insert(n.field.clone(), n.value.clone());
        }

        // 3–4. Existence (identity sources only) and merge by per-field precedence. Entities merge
        //      independently, in parallel, moving each chosen value out of its fragment.
        type Merged<'k> = Option<(&'k Kind, Map<String, Value>, BTreeMap<String, String>)>;
        let merged: Vec<(String, Merged)> = fragments
            .into_par_iter()
            .map(|(id, mut by_source)| {
                let Some(kind) = kind_of
                    .get(&id)
                    .and_then(|k| kinds.get(k.as_str()))
                    .copied()
                else {
                    return (id, None);
                };
                if !kind.identity_from.iter().any(|s| by_source.contains_key(s)) {
                    return (id, None);
                }
                let mut fields = Map::new();
                let mut provenance = BTreeMap::new();
                for (name, spec) in &kind.fields {
                    if spec.computed.is_some() || spec.authority.is_some() {
                        continue;
                    }
                    let order = kind
                        .sources
                        .fields
                        .get(name)
                        .unwrap_or(&kind.sources.default);
                    if let Some((src, v)) = order.iter().find_map(|s| {
                        by_source
                            .get_mut(s)
                            .and_then(|f| f.remove(name))
                            .map(|v| (s, v))
                    }) {
                        fields.insert(name.clone(), v);
                        provenance.insert(name.clone(), src.clone());
                    }
                }
                (id, Some((kind, fields, provenance)))
            })
            .collect();
        for (id, m) in merged {
            let Some((kind, fields, provenance)) = m else {
                d.orphans.push(id);
                continue;
            };
            let mut records = records_of.remove(&id).unwrap_or_default();
            records.sort();
            d.entities.insert(
                id,
                Entity {
                    kind: kind.name.clone(),
                    fields,
                    provenance,
                    inbound: BTreeMap::new(),
                    records,
                },
            );
        }

        lap("native+merge");
        // 5. CODEOWNERS authority (D17).
        self.authority(&mut d, &kinds);

        lap("authority");
        // 6. Relations: inbound lists of source ids (sorted), empty when declared but absent.
        //    Entity ids are numbered in sorted order, so sorting numbers sorts ids.
        let index: HashMap<&str, u32> = d
            .entities
            .keys()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i as u32))
            .collect();
        let ids: Vec<String> = d.entities.keys().cloned().collect();
        // Inbound lists by entity number, keyed by relation names borrowed from the manifest.
        let mut inbound: Vec<BTreeMap<&str, Vec<u32>>> = vec![BTreeMap::new(); ids.len()];
        for k in &self.m.kinds {
            for (rel, spec) in &k.relations {
                for (i, e) in d.entities.values().enumerate() {
                    if e.kind == spec.to {
                        inbound[i].entry(rel.as_str()).or_default();
                    }
                }
            }
        }
        // Edges whose target isn't an entity are dropped, and reported per relation (a count and a
        // sample), so a mistyped reference doesn't vanish silently.
        let mut dropped: BTreeMap<&str, (usize, usize, Vec<&str>)> = BTreeMap::new();
        for (src, rels) in &edges {
            let s = index.get(src.as_str()).copied();
            for (rel, target) in rels {
                let d = dropped.entry(rel).or_default();
                d.1 += 1;
                match (s, index.get(target.as_str())) {
                    (Some(s), Some(&t)) => inbound[t as usize].entry(rel).or_default().push(s),
                    _ => {
                        d.0 += 1;
                        if d.2.len() < 3 {
                            d.2.push(target.as_str());
                        }
                    }
                }
            }
        }
        let ignored: std::collections::HashSet<&str> = self
            .m
            .kinds
            .iter()
            .flat_map(|k| &k.relations)
            .filter(|(_, r)| r.unresolved.as_deref() == Some("ignore"))
            .map(|(name, _)| name.as_str())
            .collect();
        for (rel, (n, total, sample)) in dropped
            .into_iter()
            .filter(|(rel, d)| d.0 > 0 && !ignored.contains(rel))
        {
            d.diagnostics.push(Diagnostic {
                code: "relation.dropped",
                location: format!("relation {rel}"),
                message: format!(
                    "{n} of {total} edges point at no entity and were dropped, e.g. {}",
                    sample.join(", ")
                ),
            });
        }
        drop(edges);
        inbound.par_iter_mut().for_each(|rels| {
            for v in rels.values_mut() {
                v.sort_unstable();
                v.dedup();
            }
        });
        drop(index);
        lap("relations");

        // 7. Computed fields: CEL over the entity (without computed fields), inbound (lazy lists)
        //    and policy.
        let base: Vec<cel::Value> = ids
            .par_iter()
            .map(|id| to_cel(&d.entity_json(id).unwrap()))
            .collect();
        let computed: Vec<(usize, String, Result<Value, String>)> = ids
            .par_iter()
            .enumerate()
            .flat_map_iter(|(i, id)| {
                let kind = kinds[d.entities[id].kind.as_str()];
                let fields: Vec<_> = kind
                    .fields
                    .iter()
                    .filter_map(|(n, f)| f.computed.as_ref().map(|c| (n, c)))
                    .collect();
                if fields.is_empty() {
                    return Vec::new();
                }
                let lists = inbound[i].iter().map(|(rel, srcs)| {
                    (
                        (*rel).to_owned(),
                        std::sync::Arc::new(
                            srcs.iter()
                                .map(|&s| base[s as usize].clone())
                                .collect::<Vec<_>>(),
                        ),
                    )
                });
                with_vals(
                    &[("entity", &base[i]), ("policy", &self.policy)],
                    vec![("inbound", lazy_map(lists))],
                    |ctx| {
                        fields
                            .into_iter()
                            .map(|(n, c)| (i, n.clone(), self.json_in(c, ctx)))
                            .collect::<Vec<_>>()
                    },
                )
            })
            .collect();
        drop(base);
        for (i, field, r) in computed {
            match r {
                Ok(v) => {
                    let e = d.entities.get_mut(&ids[i]).unwrap();
                    e.fields.insert(field.clone(), v);
                    e.provenance.insert(field, "computed".into());
                }
                Err(msg) => d.diagnostics.push(Diagnostic {
                    code: "computed.error",
                    location: format!("{}.{field}", ids[i]),
                    message: msg,
                }),
            }
        }
        // Back to ids, in parallel; `inbound` is in entity order, as `d.entities` iterates.
        let named: Vec<BTreeMap<String, Vec<String>>> = inbound
            .into_par_iter()
            .map(|rels| {
                rels.into_iter()
                    .map(|(rel, srcs)| {
                        let srcs = srcs.into_iter().map(|s| ids[s as usize].clone());
                        (rel.to_owned(), srcs.collect())
                    })
                    .collect()
            })
            .collect();
        for (e, rels) in d.entities.values_mut().zip(named) {
            e.inbound = rels;
        }
        lap("computed");

        // 8–10. One pass per entity: bind it once, then evaluate its scorecards, tuple derivations
        //       and action input options.
        struct PerKind<'a> {
            scorecards: Vec<&'a toolbox_model::manifest::Scorecard>,
            emits: Vec<&'a str>,
            options: Vec<(&'a str, String, String)>, // (action, input, CEL)
        }
        let mut per_kind: HashMap<&str, PerKind> = HashMap::new();
        for k in &self.m.kinds {
            per_kind.insert(
                &k.name,
                PerKind {
                    scorecards: vec![],
                    emits: vec![],
                    options: vec![],
                },
            );
        }
        for sc in &self.m.scorecards {
            if let Some(p) = per_kind.get_mut(sc.kind.as_str()) {
                p.scorecards.push(sc);
            }
        }
        let mut static_tuples: Vec<Value> = Vec::new();
        for t in self.m.authz.iter().flat_map(|a| &a.tuples) {
            match t {
                TupleDerivation::Static { tuples: ts } => static_tuples.extend(ts.iter().cloned()),
                TupleDerivation::Kind { kind, emit } => {
                    if let Some(p) = per_kind.get_mut(kind.as_str()) {
                        p.emits.push(emit);
                    }
                }
            }
        }
        for a in &self.m.actions {
            if let Some(p) = per_kind.get_mut(a.kind.as_str()) {
                p.options.extend(
                    enum_from_inputs(&a.inputs)
                        .into_iter()
                        .map(|(input, e)| (a.name.as_str(), input, e)),
                );
            }
        }

        struct EntityOut {
            id: String,
            scores: Vec<(String, Score)>,
            tuples: Vec<Value>,
            options: Vec<(String, String, Vec<String>)>,
            diagnostics: Vec<Diagnostic>,
        }
        let outs: Vec<EntityOut> = ids
            .par_iter()
            .filter_map(|id| {
                let p = &per_kind[d.entities[id].kind.as_str()];
                if p.scorecards.is_empty() && p.emits.is_empty() && p.options.is_empty() {
                    return None;
                }
                let entity = to_cel(&d.entity_json(id).unwrap());
                Some(with_vars(
                    &[("entity", &entity), ("policy", &self.policy)],
                    |ctx| {
                        let mut out = EntityOut {
                            id: id.clone(),
                            scores: vec![],
                            tuples: vec![],
                            options: vec![],
                            diagnostics: vec![],
                        };
                        for sc in &p.scorecards {
                            // A scorecard that reads `cards` runs after every other score exists.
                            if reads_cards(sc) {
                                continue;
                            }
                            out.scores.push((sc.name.clone(), self.score(sc, ctx)));
                        }
                        for emit in &p.emits {
                            match self.json_in(emit, ctx) {
                                Ok(Value::Array(xs)) => out.tuples.extend(xs),
                                Ok(other) => out.diagnostics.push(Diagnostic {
                                    code: "tuples.error",
                                    location: id.clone(),
                                    message: format!("emit must return a list, got {other}"),
                                }),
                                Err(m) => out.diagnostics.push(Diagnostic {
                                    code: "tuples.error",
                                    location: id.clone(),
                                    message: m,
                                }),
                            }
                        }
                        for (action, input, src) in &p.options {
                            match self.json_in(src, ctx) {
                                Ok(Value::Array(xs)) => {
                                    let mut opts: Vec<String> = xs
                                        .iter()
                                        .filter_map(|x| x.as_str().map(str::to_owned))
                                        .collect();
                                    opts.sort();
                                    out.options.push((action.to_string(), input.clone(), opts));
                                }
                                Ok(other) => out.diagnostics.push(Diagnostic {
                                    code: "options.error",
                                    location: format!("{action}:{id}"),
                                    message: format!("enumFrom must return a list, got {other}"),
                                }),
                                Err(m) => out.diagnostics.push(Diagnostic {
                                    code: "options.error",
                                    location: format!("{action}:{id}"),
                                    message: m,
                                }),
                            }
                        }
                        out
                    },
                ))
            })
            .collect();
        let mut tuples: BTreeMap<String, Value> = static_tuples
            .into_iter()
            .map(|t| (pyjson::compact(&t), t))
            .collect();
        for o in outs {
            for (card, score) in o.scores {
                d.scores
                    .entry(card)
                    .or_default()
                    .insert(o.id.clone(), score);
            }
            for t in o.tuples {
                tuples.insert(pyjson::compact(&t), t);
            }
            for (action, input, opts) in o.options {
                d.options
                    .entry(action)
                    .or_default()
                    .entry(input)
                    .or_default()
                    .insert(o.id.clone(), opts);
            }
            d.diagnostics.extend(o.diagnostics);
        }
        self.score_aggregates(&mut d);
        self.score_campaigns(&mut d);
        d.tuples = tuples.into_values().collect();
        lap("scores+tuples+options");
        d.orphans.sort();
        d
    }

    /// Campaign progress from scorecard outcomes (intersection scope, AND includes).
    fn score_campaigns(&self, d: &mut Derived) {
        if self.m.campaigns.is_empty() {
            return;
        }
        let as_of = utc_ymd_today();
        let cards: BTreeMap<&str, &toolbox_model::manifest::Scorecard> = self
            .m
            .scorecards
            .iter()
            .map(|s| (s.name.as_str(), s))
            .collect();
        for c in &self.m.campaigns {
            d.campaigns.insert(
                c.name.clone(),
                score_one_campaign(c, &d.scores, &cards, &as_of),
            );
        }
    }

    /// One scorecard for the entity bound in `ctx`.
    fn score(&self, sc: &toolbox_model::manifest::Scorecard, ctx: &cel::Context<'_, '_>) -> Score {
        let applies = sc
            .applies_to
            .as_deref()
            .map_or(Ok(true), |a| truthy(self.expr(a)?.eval_in(ctx)?));
        if let Ok(false) = applies {
            return Score {
                level: "skipped".into(),
                rules: None,
                evidence: BTreeMap::new(),
            };
        }
        let mut outcomes = BTreeMap::new();
        let mut evidence = BTreeMap::new();
        for r in &sc.rules {
            let outcome = match applies
                .clone()
                .and_then(|_| truthy(self.expr(&r.check)?.eval_in(ctx)?))
            {
                Ok(true) => "pass",
                Ok(false) => "fail",
                Err(_) => "unknown",
            };
            if outcome == "fail"
                && let Some(e) = &r.evidence
            {
                // Evidence is best-effort: an error is shown as such, never fails the rule twice.
                let v = self
                    .json_in(e, ctx)
                    .unwrap_or_else(|m| serde_json::json!({"error": m}));
                evidence.insert(r.name.clone(), v);
            }
            outcomes.insert(r.name.clone(), outcome.to_string());
        }
        let mut level = "none".to_string();
        for (i, lvl) in sc.levels.iter().enumerate() {
            let upto = &sc.levels[..=i];
            if sc
                .rules
                .iter()
                .filter(|r| upto.contains(&r.level))
                .all(|r| outcomes[&r.name] == "pass")
            {
                level = lvl.clone();
            } else {
                break;
            }
        }
        Score {
            level,
            rules: Some(outcomes),
            evidence,
        }
    }

    /// Scorecards whose checks read `cards` (other scorecards' levels). They run after the first
    /// pass, so a repository level can be the lowest constituent without a cycle.
    fn score_aggregates(&self, d: &mut Derived) {
        let deferred: Vec<&toolbox_model::manifest::Scorecard> = self
            .m
            .scorecards
            .iter()
            .filter(|sc| reads_cards(sc))
            .collect();
        if deferred.is_empty() {
            return;
        }
        let summaries = constituent_summaries(self.m, &d.scores);
        let ids: Vec<String> = d
            .entities
            .iter()
            .filter(|(_, e)| deferred.iter().any(|sc| sc.kind == e.kind))
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            let kind = d.entities[&id].kind.clone();
            let watch: Vec<String> = d.entities[&id]
                .fields
                .get("watch")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect();
            let selected: Vec<Value> = summaries
                .iter()
                .filter(|c| watch.iter().any(|w| Some(w.as_str()) == c["name"].as_str()))
                .cloned()
                .collect();
            let entity = to_cel(&d.entity_json(&id).unwrap());
            let cards = to_cel(&Value::Array(selected));
            let scores: Vec<(String, Score)> = with_vars(
                &[
                    ("entity", &entity),
                    ("policy", &self.policy),
                    ("cards", &cards),
                ],
                |ctx| {
                    deferred
                        .iter()
                        .filter(|sc| sc.kind == kind)
                        .map(|sc| (sc.name.clone(), self.score(sc, ctx)))
                        .collect()
                },
            );
            for (name, score) in scores {
                d.scores.entry(name).or_default().insert(id.clone(), score);
            }
        }
    }

    fn authority(&self, d: &mut Derived, kinds: &BTreeMap<&str, &Kind>) {
        let mut updates: Vec<(String, String, Value)> = Vec::new();
        let mut team_maps: HashMap<&str, HashMap<String, &str>> = HashMap::new();
        for (id, e) in &d.entities {
            for (fname, spec) in &kinds[e.kind.as_str()].fields {
                let Some(a) = &spec.authority else { continue };
                let team_to_group = team_maps.entry(a.team_field.as_str()).or_insert_with(|| {
                    let (team_kind, team_field) = a.team_field.split_once('.').unwrap_or_default();
                    d.entities
                        .iter()
                        .filter(|(_, g)| g.kind == team_kind)
                        .filter_map(|(gid, g)| {
                            g.fields
                                .get(team_field)
                                .and_then(Value::as_str)
                                .map(|t| (format!("@{t}"), gid.as_str()))
                        })
                        .collect()
                });
                let claim = e.fields.get(&a.claim).and_then(Value::as_str);
                let owning: Vec<&str> = codeowners_for(e.fields.get(&a.rules), &a.path)
                    .iter()
                    .filter_map(|t| team_to_group.get(t.as_str()).copied())
                    .collect();
                let value = match claim {
                    Some(c) if owning.contains(&c) => {
                        json!({"status": "verified", "owner": c, "claim": c})
                    }
                    Some(c) => json!({"status": "unverified", "claim": c}),
                    None if owning.len() == 1 => json!({"status": "inferred", "owner": owning[0]}),
                    None => json!({"status": "none"}),
                };
                updates.push((id.clone(), fname.clone(), value));
            }
        }
        for (id, field, value) in updates {
            let e = d.entities.get_mut(&id).unwrap();
            e.fields.insert(field.clone(), value);
            e.provenance.insert(field, "authority:codeowners".into());
        }
    }
}

/// Gaps for one entity against every include; `None` if the entity is out of scope (skipped).
fn campaign_entity(
    id: &str,
    includes: &[toolbox_model::manifest::CampaignInclude],
    scores: &BTreeMap<String, BTreeMap<String, Score>>,
    cards: &BTreeMap<&str, &toolbox_model::manifest::Scorecard>,
) -> Option<CampaignEntity> {
    let mut gaps = Vec::new();
    for inc in includes {
        let score = scores.get(&inc.scorecard).and_then(|m| m.get(id))?;
        if score.level == "skipped" {
            return None;
        }
        if let Some(at) = &inc.at_least {
            let Some(sc) = cards.get(inc.scorecard.as_str()) else {
                gaps.push(format!("{}: unknown scorecard", inc.scorecard));
                continue;
            };
            if level_rank(sc, &score.level) < level_rank(sc, at) {
                gaps.push(format!("{}:atLeast {at} ({})", inc.scorecard, score.level));
            }
        }
        for rule in &inc.rules {
            let outcome = score
                .rules
                .as_ref()
                .and_then(|m| m.get(rule))
                .map(String::as_str)
                .unwrap_or("unknown");
            if outcome != "pass" {
                gaps.push(format!("{}:rule {rule} ({outcome})", inc.scorecard));
            }
        }
    }
    Some(CampaignEntity {
        meeting: gaps.is_empty(),
        gaps,
    })
}

fn score_one_campaign(
    c: &toolbox_model::manifest::Campaign,
    scores: &BTreeMap<String, BTreeMap<String, Score>>,
    cards: &BTreeMap<&str, &toolbox_model::manifest::Scorecard>,
    as_of: &str,
) -> CampaignProgress {
    let mut candidates: BTreeSet<String> = BTreeSet::new();
    for inc in &c.includes {
        if let Some(rows) = scores.get(&inc.scorecard) {
            candidates.extend(rows.keys().cloned());
        }
    }
    let mut entities = BTreeMap::new();
    let mut in_scope = 0usize;
    let mut meeting = 0usize;
    for id in candidates {
        let Some(ent) = campaign_entity(&id, &c.includes, scores, cards) else {
            continue;
        };
        in_scope += 1;
        if ent.meeting {
            meeting += 1;
        }
        entities.insert(id, ent);
    }
    let target = c.target_percent();
    let percent = (meeting * 100).checked_div(in_scope).unwrap_or(0) as u8;
    CampaignProgress {
        status: campaign_status(c, as_of),
        track: c.track.clone(),
        target_percent: target,
        in_scope,
        meeting,
        percent,
        on_target: percent >= target,
        milestones: c
            .milestones
            .iter()
            .map(|m| CampaignMilestoneResult {
                name: m.name.clone(),
                target_percent: m.target_percent,
                met: percent >= m.target_percent,
            })
            .collect(),
        entities,
    }
}

/// Campaign lifecycle from draft / override / dates vs `as_of` (`YYYY-MM-DD`).
fn campaign_status(c: &toolbox_model::manifest::Campaign, as_of: &str) -> String {
    if c.draft {
        return "draft".into();
    }
    if let Some(s) = &c.status {
        return s.clone();
    }
    if c.start.as_deref().is_some_and(|s| s > as_of) {
        return "scheduled".into();
    }
    if c.end.as_deref().is_some_and(|e| e < as_of) {
        return "ended".into();
    }
    "active".into()
}

/// Today's UTC date as `YYYY-MM-DD` (Howard Hinnant civil_from_days).
fn utc_ymd_today() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The check or its evidence names the `cards` variable (a list of constituent scorecards).
fn reads_cards(sc: &toolbox_model::manifest::Scorecard) -> bool {
    fn token(s: &str) -> bool {
        s.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .any(|t| t == "cards")
    }
    sc.applies_to.as_deref().is_some_and(token)
        || sc
            .rules
            .iter()
            .any(|r| token(&r.check) || r.evidence.as_deref().is_some_and(token))
}

/// One entry per scorecard that has at least one counted entity: the lowest level, with unknown
/// rules dropped out of that level. Fixture samples under `/demo/` are not counted, so a ladder
/// of bad fixtures does not set the repository level.
fn constituent_summaries(
    m: &Manifest,
    scores: &BTreeMap<String, BTreeMap<String, Score>>,
) -> Vec<Value> {
    let mut out = Vec::new();
    for sc in &m.scorecards {
        if reads_cards(sc) {
            continue;
        }
        let Some(rows) = scores.get(&sc.name) else {
            continue;
        };
        let Some((level, rank)) = constituent_level(sc, rows) else {
            continue;
        };
        out.push(json!({"name": sc.name, "level": level, "rank": rank}));
    }
    out
}

fn constituent_level(
    sc: &toolbox_model::manifest::Scorecard,
    rows: &BTreeMap<String, Score>,
) -> Option<(String, i64)> {
    let mut best: Option<(usize, String)> = None;
    for (id, score) in rows {
        if id.contains("/demo/") || score.level == "skipped" {
            continue;
        }
        let Some(rules) = &score.rules else {
            continue;
        };
        let Some(level) = level_ignoring_unknown(sc, rules) else {
            continue;
        };
        let rank = level_rank(sc, &level);
        if best.as_ref().is_none_or(|(r, _)| rank < *r) {
            best = Some((rank, level));
        }
    }
    best.map(|(rank, level)| (level, rank as i64))
}

/// Highest level whose non-unknown rules all pass. `None` when every rule is unknown.
fn level_ignoring_unknown(
    sc: &toolbox_model::manifest::Scorecard,
    outcomes: &BTreeMap<String, String>,
) -> Option<String> {
    let known: Vec<&toolbox_model::manifest::Rule> = sc
        .rules
        .iter()
        .filter(|r| outcomes.get(&r.name).is_some_and(|o| o != "unknown"))
        .collect();
    if known.is_empty() {
        return None;
    }
    let mut level = "none".to_string();
    for (i, lvl) in sc.levels.iter().enumerate() {
        let upto = &sc.levels[..=i];
        let ok = known
            .iter()
            .filter(|r| upto.contains(&r.level))
            .all(|r| outcomes[&r.name] == "pass");
        if ok {
            level = lvl.clone();
        } else {
            break;
        }
    }
    Some(level)
}

fn level_rank(sc: &toolbox_model::manifest::Scorecard, level: &str) -> usize {
    if level == "none" {
        0
    } else {
        sc.levels
            .iter()
            .position(|l| l == level)
            .map_or(0, |i| i + 1)
    }
}

fn truthy(v: cel::Value) -> Result<bool, String> {
    match v {
        cel::Value::Bool(b) => Ok(b),
        other => Err(format!("expected a bool, got {other:?}")),
    }
}

/// Owners of `path`: the last matching CODEOWNERS rule wins (`*`, `/exact`, or a glob).
pub fn codeowners_for(rules: Option<&Value>, path: &str) -> Vec<String> {
    let mut owners = Vec::new();
    for rule in rules.and_then(Value::as_array).into_iter().flatten() {
        let pat = rule
            .get("pattern")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if pat == "*" || pat.strip_prefix('/') == Some(path) || glob(pat, path) {
            owners = rule
                .get("owners")
                .and_then(Value::as_array)
                .map(|o| {
                    o.iter()
                        .filter_map(|x| x.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
        }
    }
    owners
}

/// fnmatch-style matching with `*` and `?`.
fn glob(pat: &str, s: &str) -> bool {
    let (p, t): (Vec<char>, Vec<char>) = (pat.chars().collect(), s.chars().collect());
    let (mut pi, mut ti, mut star, mut mark) = (0, 0, None, 0);
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == t[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star = Some(pi);
            mark = ti;
            pi += 1;
        } else if let Some(sp) = star {
            pi = sp + 1;
            mark += 1;
            ti = mark;
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|&c| c == '*')
}

/// Top-level action input properties with `enumFrom`, as (input name, CEL).
pub fn enum_from_inputs(inputs: &Value) -> Vec<(String, String)> {
    inputs
        .get("properties")
        .and_then(Value::as_object)
        .map(|props| {
            props
                .iter()
                .filter_map(|(n, p)| {
                    p.get("enumFrom")
                        .and_then(Value::as_str)
                        .map(|e| (n.clone(), e.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Entity ids by kind, for callers that need them.
pub fn ids_of_kind<'a>(d: &'a Derived, kind: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    d.entities
        .iter()
        .filter(move |(_, e)| e.kind == kind)
        .map(|(id, _)| id.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_and_codeowners() {
        assert!(glob("*.yaml", "catalog-info.yaml"));
        assert!(!glob("docs/*", "catalog-info.yaml"));
        let rules = json!([{"pattern": "*", "owners": ["@a"]}, {"pattern": "/catalog-info.yaml", "owners": ["@b"]}]);
        assert_eq!(
            codeowners_for(Some(&rules), "catalog-info.yaml"),
            vec!["@b".to_string()]
        );
    }
}
