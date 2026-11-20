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
use std::collections::{BTreeMap, HashMap};
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
}

/// One scorecard result for one entity.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Score {
    pub level: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<BTreeMap<String, String>>,
}

/// A problem found while deriving (never fatal: the pipeline always produces a result).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub location: String,
    pub message: String,
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
type Mapped = (String, Map<String, Value>, Vec<(String, String)>);

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
        let mut edges: Vec<(String, String, String)> = Vec::new();
        for kind in &self.m.kinds {
            for (ri, rule) in kind.map.iter().enumerate() {
                let (conn, rkind) = rule.from_parts();
                let source = rule.source_label();
                let mapped: Vec<Result<Mapped, String>> = inputs
                    .records
                    .par_iter()
                    .filter(|r| r.connector == conn && r.kind == rkind)
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
                                        rels.extend(ts.into_iter().filter_map(|t| {
                                            t.as_str().map(|t| (rel.clone(), t.to_owned()))
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
                for item in mapped {
                    match item {
                        Ok((id, values, rels)) => {
                            kind_of.insert(id.clone(), kind.name.clone());
                            let frag = fragments
                                .entry(id.clone())
                                .or_default()
                                .entry(source.clone())
                                .or_default();
                            frag.extend(values.into_iter().filter(|(_, v)| !v.is_null()));
                            edges.extend(rels.into_iter().map(|(rel, t)| (rel, id.clone(), t)));
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

        // 3–4. Existence (identity sources only) and merge by per-field precedence.
        for (id, by_source) in &fragments {
            let Some(kind) = kind_of.get(id).and_then(|k| kinds.get(k.as_str())) else {
                d.orphans.push(id.clone());
                continue;
            };
            if !kind.identity_from.iter().any(|s| by_source.contains_key(s)) {
                d.orphans.push(id.clone());
                continue;
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
                if let Some((src, v)) = order
                    .iter()
                    .find_map(|s| by_source.get(s).and_then(|f| f.get(name)).map(|v| (s, v)))
                {
                    fields.insert(name.clone(), v.clone());
                    provenance.insert(name.clone(), src.clone());
                }
            }
            d.entities.insert(
                id.clone(),
                Entity {
                    kind: kind.name.clone(),
                    fields,
                    provenance,
                    inbound: BTreeMap::new(),
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
        let mut inbound: HashMap<u32, BTreeMap<String, Vec<u32>>> = HashMap::new();
        for k in &self.m.kinds {
            for (rel, spec) in &k.relations {
                for (id, e) in &d.entities {
                    if e.kind == spec.to {
                        inbound
                            .entry(index[id.as_str()])
                            .or_default()
                            .entry(rel.clone())
                            .or_default();
                    }
                }
            }
        }
        for (rel, src, target) in &edges {
            if let (Some(&s), Some(&t)) = (index.get(src.as_str()), index.get(target.as_str())) {
                inbound
                    .entry(t)
                    .or_default()
                    .entry(rel.clone())
                    .or_default()
                    .push(s);
            }
        }
        drop(edges);
        inbound.par_iter_mut().for_each(|(_, rels)| {
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
        let empty = BTreeMap::new();
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
                let rels = inbound.get(&(i as u32)).unwrap_or(&empty);
                let lists = rels.iter().map(|(rel, srcs)| {
                    (
                        rel.clone(),
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
        for (i, rels) in inbound {
            let e = d.entities.get_mut(&ids[i as usize]).unwrap();
            e.inbound = rels
                .into_iter()
                .map(|(rel, srcs)| {
                    (
                        rel,
                        srcs.into_iter().map(|s| ids[s as usize].clone()).collect(),
                    )
                })
                .collect();
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
        d.tuples = tuples.into_values().collect();
        lap("scores+tuples+options");
        d.orphans.sort();
        d
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
            };
        }
        let mut outcomes = BTreeMap::new();
        for r in &sc.rules {
            let outcome = match applies
                .clone()
                .and_then(|_| truthy(self.expr(&r.check)?.eval_in(ctx)?))
            {
                Ok(true) => "pass",
                Ok(false) => "fail",
                Err(_) => "unknown",
            };
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
