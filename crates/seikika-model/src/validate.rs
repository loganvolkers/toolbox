//! `seikika validate` (docs/v1-spec.md §5): schema, cross-references, form subset, CEL, files and
//! fixtures. The cross-reference and form-subset checks mirror
//! `examples/reference-portal/tools/validate.py`; the CEL, duplicate-name, relation-target,
//! authority and file checks are additional.

use crate::diagnostics::{Diagnostic, Report};
use crate::load::{self, LoadError, Source};
use crate::manifest::{Manifest, TupleDerivation};
use crate::{messages, schema};
use seikika_expr::Site;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Load and validate the portal at `path` (a manifest file or a portal directory).
pub fn validate_portal(path: &Path) -> Result<Report, LoadError> {
    let src = load::load(path)?;
    let mut report = Report {
        manifest: src.manifest_file.clone(),
        included: src.included.clone(),
        ..Report::default()
    };
    let (manifest, mut errors) = validate_source(&src);
    if let Some(m) = &manifest {
        let mut file_errors = check_files(&src.root, &src.manifest_file, m);
        src.relocate(&mut file_errors);
        errors.extend(file_errors);
    }
    let (count, fixture_errors) = messages::validate_fixtures(&src.root, manifest.as_ref());
    report.messages = count;
    errors.extend(fixture_errors);
    report.errors = errors;
    Ok(report)
}

/// [`validate_manifest`] on a loaded source, with each diagnostic pointing at the file (and the
/// pointer in it) its value came from, not the merged document.
pub fn validate_source(src: &Source) -> (Option<Manifest>, Vec<Diagnostic>) {
    let (m, mut errors) = validate_manifest(&src.document, &src.manifest_file);
    src.relocate(&mut errors);
    (m, errors)
}

/// Validate a manifest document. Returns the typed manifest when the schema passed, plus every
/// diagnostic. `file` labels diagnostics.
pub fn validate_manifest(doc: &Value, file: &str) -> (Option<Manifest>, Vec<Diagnostic>) {
    let mut v = Checker {
        file: file.to_string(),
        errors: Vec::new(),
    };
    for (ptr, msg) in schema::errors(schema::manifest(), doc) {
        v.push("schema", ptr, msg);
    }
    if !v.errors.is_empty() {
        return (None, v.errors); // shapes aren't guaranteed, so skip the typed checks
    }
    let manifest: Manifest = match serde_json::from_value(doc.clone()) {
        Ok(m) => m,
        Err(e) => {
            v.push(
                "schema",
                String::new(),
                format!("manifest doesn't match the typed model: {e}"),
            );
            return (None, v.errors);
        }
    };
    v.cross_references(&manifest);
    v.form_subset(&manifest);
    v.executors(&manifest);
    v.cel(&manifest);
    (Some(manifest), v.errors)
}

struct Checker {
    file: String,
    errors: Vec<Diagnostic>,
}

impl Checker {
    /// Rubrics: the kind exists, it has the subject field, and criterion names are unique.
    /// Each kind group names known kinds, and no kind is in two groups.
    fn kind_groups(&mut self, m: &Manifest, kinds: &BTreeMap<&str, usize>) {
        self.duplicates(
            "kind group",
            "/kindGroups",
            m.kind_groups.iter().map(|g| g.name.as_str()),
        );
        let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
        for (gi, g) in m.kind_groups.iter().enumerate() {
            for (ki, k) in g.kinds.iter().enumerate() {
                let at = format!("/kindGroups/{gi}/kinds/{ki}");
                if !kinds.contains_key(k.as_str()) {
                    self.push(
                        "xref.kind",
                        at.clone(),
                        format!("kind group {}: unknown kind {k}", g.name),
                    );
                }
                if let Some(first) = owner.insert(k, &g.name) {
                    self.push(
                        "xref.kind-group",
                        at,
                        format!("kind {k} is in kind groups {first} and {}", g.name),
                    );
                }
            }
        }
    }

    fn rubrics(&mut self, m: &Manifest, kinds: &BTreeMap<&str, usize>) {
        for (ri, r) in m.rubrics.iter().enumerate() {
            let Some(k) = kinds.get(r.kind.as_str()).map(|&i| &m.kinds[i]) else {
                self.push(
                    "xref.kind",
                    format!("/rubrics/{ri}/kind"),
                    format!("rubric {}: unknown kind {}", r.name, r.kind),
                );
                continue;
            };
            if !k.fields.contains_key(r.subject()) {
                self.push(
                    "xref.field",
                    format!("/rubrics/{ri}/subject"),
                    format!(
                        "rubric {}: {} has no field {} to version assessments by",
                        r.name,
                        r.kind,
                        r.subject()
                    ),
                );
            }
            let mut seen = BTreeSet::new();
            for (ci, c) in r.criteria.iter().enumerate() {
                if !seen.insert(c.name.as_str()) {
                    self.push(
                        "rubric.duplicate",
                        format!("/rubrics/{ri}/criteria/{ci}/name"),
                        format!("rubric {}: criterion {} is repeated", r.name, c.name),
                    );
                }
            }
        }
    }

    fn push(&mut self, code: &'static str, path: String, message: String) {
        self.errors.push(Diagnostic {
            code,
            file: self.file.clone(),
            path,
            message,
        });
    }

    fn cross_references(&mut self, m: &Manifest) {
        let connectors: BTreeMap<&str, BTreeSet<&str>> = m
            .connectors
            .iter()
            .map(|c| {
                (
                    c.name.as_str(),
                    c.records.iter().map(String::as_str).collect(),
                )
            })
            .collect();
        let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
        for (i, k) in m.kinds.iter().enumerate() {
            kinds.entry(k.name.as_str()).or_insert(i); // first definition wins; duplicates are reported
        }
        let declared =
            |conn: &str, rec: &str| connectors.get(conn).is_some_and(|r| r.contains(rec));

        self.duplicates(
            "connector",
            "/connectors",
            m.connectors.iter().map(|c| c.name.as_str()),
        );
        self.duplicates("kind", "/kinds", m.kinds.iter().map(|k| k.name.as_str()));
        self.duplicates(
            "scorecard",
            "/scorecards",
            m.scorecards.iter().map(|s| s.name.as_str()),
        );
        self.duplicates("track", "/tracks", m.tracks.iter().map(|t| t.name.as_str()));
        self.duplicates(
            "campaign",
            "/campaigns",
            m.campaigns.iter().map(|c| c.name.as_str()),
        );
        self.duplicates(
            "action",
            "/actions",
            m.actions.iter().map(|a| a.name.as_str()),
        );

        for (ci, c) in m.connectors.iter().enumerate() {
            for rec in c.refresh.keys().chain(c.freshness.keys()) {
                if !c.records.contains(rec) {
                    self.push(
                        "xref.connector-record",
                        format!("/connectors/{ci}"),
                        format!(
                            "connector {}: refresh or freshness names undeclared record kind {rec}",
                            c.name
                        ),
                    );
                }
            }
        }

        for (ki, k) in m.kinds.iter().enumerate() {
            let mut provided: BTreeSet<String> = BTreeSet::from(["native".to_string()]);
            for (ri, r) in k.map.iter().enumerate() {
                let (conn, rec) = r.from_parts();
                if !declared(conn, rec) {
                    self.push(
                        "xref.map-from",
                        format!("/kinds/{ki}/map/{ri}/from"),
                        format!(
                            "kind {}: map from {} names an undeclared connector/record",
                            k.name, r.from
                        ),
                    );
                }
                provided.insert(r.source_label());
                for rel in r.relations.keys() {
                    if !k.relations.contains_key(rel) {
                        self.push(
                            "xref.relation",
                            format!("/kinds/{ki}/map/{ri}/relations/{rel}"),
                            format!("kind {}: map rule sets undeclared relation {rel}", k.name),
                        );
                    }
                }
            }
            let lists = std::iter::once(("default", &k.sources.default))
                .chain(k.sources.fields.iter().map(|(f, l)| (f.as_str(), l)));
            for (field, list) in lists {
                if field != "default" && !k.fields.contains_key(field) {
                    self.push(
                        "xref.sources-field",
                        format!("/kinds/{ki}/sources/fields/{field}"),
                        format!("kind {}: sources for undeclared field {field}", k.name),
                    );
                }
                for s in list {
                    if !provided.contains(s) {
                        let at = if field == "default" {
                            "default".to_string()
                        } else {
                            format!("fields/{field}")
                        };
                        self.push(
                            "xref.source-provided",
                            format!("/kinds/{ki}/sources/{at}"),
                            format!(
                                "kind {}: source {s} for {field} is provided by no map rule",
                                k.name
                            ),
                        );
                    }
                }
            }
            for s in &k.identity_from {
                if !provided.contains(s) {
                    self.push(
                        "xref.source-provided",
                        format!("/kinds/{ki}/identityFrom"),
                        format!(
                            "kind {}: identity source {s} is provided by no map rule",
                            k.name
                        ),
                    );
                }
            }
            for (rel, r) in &k.relations {
                if !kinds.contains_key(r.to.as_str()) {
                    self.push(
                        "xref.kind",
                        format!("/kinds/{ki}/relations/{rel}"),
                        format!(
                            "kind {}: relation {rel} targets unknown kind {}",
                            k.name, r.to
                        ),
                    );
                }
            }
            for (fname, f) in &k.fields {
                if let Some(to) = &f.to
                    && !kinds.contains_key(to.as_str())
                {
                    self.push(
                        "xref.kind",
                        format!("/kinds/{ki}/fields/{fname}"),
                        format!(
                            "kind {}: field {fname} references unknown kind {to}",
                            k.name
                        ),
                    );
                }
                if let Some(a) = &f.authority {
                    for (role, target) in [("claim", &a.claim), ("rules", &a.rules)] {
                        if !k.fields.contains_key(target) {
                            self.push(
                                "xref.authority",
                                format!("/kinds/{ki}/fields/{fname}/authority/{role}"),
                                format!(
                                    "kind {}: authority {role} names undeclared field {target}",
                                    k.name
                                ),
                            );
                        }
                    }
                    let ok = a.team_field.split_once('.').is_some_and(|(kn, fld)| {
                        kinds
                            .get(kn)
                            .is_some_and(|&i| m.kinds[i].fields.contains_key(fld))
                    });
                    if !ok {
                        self.push(
                            "xref.authority",
                            format!("/kinds/{ki}/fields/{fname}/authority/teamField"),
                            format!(
                                "kind {}: authority teamField {} is not a declared Kind.field",
                                k.name, a.team_field
                            ),
                        );
                    }
                }
            }
        }

        self.rubrics(m, &kinds);
        self.kind_groups(m, &kinds);
        let tracks: BTreeSet<&str> = m.tracks.iter().map(|t| t.name.as_str()).collect();
        let scorecards: BTreeMap<&str, &crate::manifest::Scorecard> =
            m.scorecards.iter().map(|s| (s.name.as_str(), s)).collect();
        for (si, s) in m.scorecards.iter().enumerate() {
            if !kinds.contains_key(s.kind.as_str()) {
                self.push(
                    "xref.kind",
                    format!("/scorecards/{si}/kind"),
                    format!("scorecard {}: unknown kind {}", s.name, s.kind),
                );
            }
            if let Some(t) = &s.track
                && !tracks.contains(t.as_str())
            {
                self.push(
                    "xref.track",
                    format!("/scorecards/{si}/track"),
                    format!("scorecard {}: unknown track {t}", s.name),
                );
            }
            for (ri, r) in s.rules.iter().enumerate() {
                if !s.levels.contains(&r.level) {
                    self.push(
                        "xref.level",
                        format!("/scorecards/{si}/rules/{ri}/level"),
                        format!("scorecard {}: rule {} has unknown level", s.name, r.name),
                    );
                }
                if let Some(src) = &r.data_from
                    && !connectors.contains_key(src.as_str())
                {
                    self.push(
                        "xref.connector",
                        format!("/scorecards/{si}/rules/{ri}/dataFrom"),
                        format!(
                            "scorecard {}: rule {} reads unknown connector {src}",
                            s.name, r.name
                        ),
                    );
                }
            }
            if let Some(c) = &s.compare {
                let mut seen = BTreeSet::new();
                for (pi, id) in c.peers.iter().enumerate() {
                    if *id == c.subject || !seen.insert(id.as_str()) {
                        self.push(
                            "xref.compare",
                            format!("/scorecards/{si}/compare/peers/{pi}"),
                            format!("scorecard {}: compares {id} twice", s.name),
                        );
                    }
                }
            }
        }
        for (ci, c) in m.campaigns.iter().enumerate() {
            if let Some(t) = &c.track
                && !tracks.contains(t.as_str())
            {
                self.push(
                    "xref.track",
                    format!("/campaigns/{ci}/track"),
                    format!("campaign {}: unknown track {t}", c.name),
                );
            }
            for date in [&c.start, &c.end] {
                if let Some(d) = date
                    && !is_ymd(d)
                {
                    self.push(
                        "campaign.date",
                        format!("/campaigns/{ci}"),
                        format!("campaign {}: date {d} is not YYYY-MM-DD", c.name),
                    );
                }
            }
            if let (Some(start), Some(end)) = (&c.start, &c.end)
                && start > end
            {
                self.push(
                    "campaign.date",
                    format!("/campaigns/{ci}"),
                    format!("campaign {}: start {start} is after end {end}", c.name),
                );
            }
            if c.includes.is_empty() {
                self.push(
                    "campaign.includes",
                    format!("/campaigns/{ci}/includes"),
                    format!("campaign {}: needs at least one include", c.name),
                );
            }
            self.duplicates(
                "milestone",
                &format!("/campaigns/{ci}/milestones"),
                c.milestones.iter().map(|m| m.name.as_str()),
            );
            for (ii, inc) in c.includes.iter().enumerate() {
                let Some(sc) = scorecards.get(inc.scorecard.as_str()) else {
                    self.push(
                        "xref.scorecard",
                        format!("/campaigns/{ci}/includes/{ii}/scorecard"),
                        format!("campaign {}: unknown scorecard {}", c.name, inc.scorecard),
                    );
                    continue;
                };
                if inc.at_least.is_none() && inc.rules.is_empty() {
                    self.push(
                        "campaign.include",
                        format!("/campaigns/{ci}/includes/{ii}"),
                        format!(
                            "campaign {}: include of {} needs atLeast and/or rules",
                            c.name, inc.scorecard
                        ),
                    );
                }
                if let Some(lvl) = &inc.at_least
                    && lvl != "none"
                    && !sc.levels.iter().any(|l| l == lvl)
                {
                    self.push(
                        "xref.level",
                        format!("/campaigns/{ci}/includes/{ii}/atLeast"),
                        format!(
                            "campaign {}: atLeast {lvl} is not a level of {}",
                            c.name, inc.scorecard
                        ),
                    );
                }
                for (ri, rule) in inc.rules.iter().enumerate() {
                    if !sc.rules.iter().any(|r| &r.name == rule) {
                        self.push(
                            "xref.rule",
                            format!("/campaigns/{ci}/includes/{ii}/rules/{ri}"),
                            format!(
                                "campaign {}: rule {rule} is not on scorecard {}",
                                c.name, inc.scorecard
                            ),
                        );
                    }
                }
            }
        }

        let mut seen_waiver = BTreeSet::new();
        for (wi, w) in m.waivers.iter().enumerate() {
            if !seen_waiver.insert((w.scorecard.as_str(), w.entity.as_str(), w.rule.as_str())) {
                self.push(
                    "waiver.duplicate",
                    format!("/waivers/{wi}"),
                    format!(
                        "waiver for {} {} {} is repeated",
                        w.scorecard, w.entity, w.rule
                    ),
                );
            }
            let Some(sc) = scorecards.get(w.scorecard.as_str()) else {
                self.push(
                    "xref.scorecard",
                    format!("/waivers/{wi}/scorecard"),
                    format!("waiver: unknown scorecard {}", w.scorecard),
                );
                continue;
            };
            if !sc.rules.iter().any(|r| r.name == w.rule) {
                self.push(
                    "xref.rule",
                    format!("/waivers/{wi}/rule"),
                    format!(
                        "waiver: rule {} is not on scorecard {}",
                        w.rule, w.scorecard
                    ),
                );
            }
        }

        if let Some(authz) = &m.authz {
            for (ti, t) in authz.tuples.iter().enumerate() {
                if let TupleDerivation::Kind { kind, .. } = t
                    && !kinds.contains_key(kind.as_str())
                {
                    self.push(
                        "xref.kind",
                        format!("/authz/tuples/{ti}/kind"),
                        format!("authz tuples: unknown kind {kind}"),
                    );
                }
            }
        }

        let bundles: BTreeSet<&str> =
            m.ui.iter()
                .flat_map(|u| u.bundles.iter().map(|b| b.name.as_str()))
                .collect();
        if let Some(ui) = &m.ui {
            self.duplicates(
                "UI bundle",
                "/ui/bundles",
                ui.bundles.iter().map(|b| b.name.as_str()),
            );
        }
        for (ai, a) in m.actions.iter().enumerate() {
            if !kinds.contains_key(a.kind.as_str()) {
                self.push(
                    "xref.kind",
                    format!("/actions/{ai}/kind"),
                    format!("action {}: unknown kind {}", a.name, a.kind),
                );
            }
            for (name, lk) in &a.context {
                if !kinds.contains_key(lk.kind.as_str()) {
                    self.push(
                        "xref.kind",
                        format!("/actions/{ai}/context/{name}/kind"),
                        format!("action {}: context {name} names unknown kind", a.name),
                    );
                }
            }
            if let Some(ui) = &a.ui
                && !bundles.contains(ui.bundle.as_str())
            {
                self.push(
                    "xref.ui-bundle",
                    format!("/actions/{ai}/ui/bundle"),
                    format!("action {}: unknown UI bundle", a.name),
                );
            }
            if let Some(expect) = &a.expect {
                for (ri, r) in expect.refresh.iter().enumerate() {
                    if !declared(&r.connector, &r.kind) {
                        self.push(
                            "xref.refresh",
                            format!("/actions/{ai}/expect/refresh/{ri}"),
                            format!(
                                "action {}: refresh names an undeclared connector/record",
                                a.name
                            ),
                        );
                    }
                }
            }
        }
    }

    fn duplicates<'a>(&mut self, what: &str, base: &str, names: impl Iterator<Item = &'a str>) {
        let mut seen = BTreeSet::new();
        for (i, n) in names.enumerate() {
            if !seen.insert(n) {
                self.push(
                    "xref.duplicate",
                    format!("{base}/{i}"),
                    format!("duplicate {what} name {n}"),
                );
            }
        }
    }

    /// Forges: a type for each host that isn't known, and one entry per URL (docs/design/forges.md).
    fn forges(&mut self, m: &Manifest) {
        let mut seen = BTreeSet::new();
        for (i, f) in m.forges.iter().enumerate() {
            if crate::forge::kind_of(f).is_none() {
                self.push(
                    "forge",
                    format!("/forges/{i}/type"),
                    format!(
                        "forge {}: say its type (github or gitlab); only https://github.com and https://gitlab.com are known without one",
                        f.url
                    ),
                );
            }
            if !seen.insert(f.url.trim_end_matches('/').to_ascii_lowercase()) {
                self.push(
                    "forge",
                    format!("/forges/{i}/url"),
                    format!("forge {} is listed twice", f.url),
                );
            }
        }
        if let Some((i, f)) = m
            .forges
            .iter()
            .enumerate()
            .filter(|(_, f)| f.identity)
            .nth(1)
        {
            self.push(
                "forge",
                format!("/forges/{i}/identity"),
                format!(
                    "forge {}: only one forge says who people are (identity: true)",
                    f.url
                ),
            );
        }
    }

    /// CI hosts: unique names, an endpoint for a service, a project template with only
    /// `{project}` in it, and every `forges[].ci` naming one (docs/design/forges.md §4.7).
    fn ci_hosts(&mut self, m: &Manifest) {
        let mut seen = BTreeSet::new();
        for (i, h) in m.ci.iter().enumerate() {
            if h.name == crate::ci::NONE || !seen.insert(h.name.as_str()) {
                self.push(
                    "ci",
                    format!("/ci/{i}/name"),
                    format!("ci {}: the name is taken (none, or listed twice)", h.name),
                );
            }
            if let Err(e) = crate::ci::target(h, "x") {
                self.push("ci", format!("/ci/{i}"), e);
            }
            if let Some(p) = &h.project
                && p.replace("{project}", "").contains(['{', '}'])
            {
                self.push(
                    "ci",
                    format!("/ci/{i}/project"),
                    format!("ci {}: project {p} may hold only {{project}}", h.name),
                );
            }
        }
        for (i, f) in m.forges.iter().enumerate() {
            if let Some(c) = &f.ci
                && c != crate::ci::NONE
                && !m.ci.iter().any(|h| &h.name == c)
            {
                self.push(
                    "ci",
                    format!("/forges/{i}/ci"),
                    format!(
                        "forge {}: ci {c} is not declared under ci: (or say none)",
                        f.url
                    ),
                );
            }
        }
    }

    /// An executor's `ci:` names a declared host of the type it speaks, instead of `forge`,
    /// `api` or `token`.
    fn executor_ci(&mut self, m: &Manifest, ai: usize, t: &crate::executors::ExecutorType) {
        let a = &m.actions[ai];
        let Some(name) = a.executor.get("ci").and_then(Value::as_str) else {
            return;
        };
        let at = format!("/actions/{ai}/executor/ci");
        let Some(want) = t.ci else {
            self.push(
                "action.executor",
                at,
                format!("action {}: a {} executor names no ci", a.name, t.name),
            );
            return;
        };
        match m.ci.iter().find(|h| h.name == name) {
            Some(h) if h.kind == want.name() => {}
            Some(h) => self.push(
                "action.executor",
                at.clone(),
                format!(
                    "action {}: ci {name} is {}, and a {} executor needs {}",
                    a.name,
                    h.kind,
                    t.name,
                    want.name()
                ),
            ),
            None => self.push(
                "action.executor",
                at.clone(),
                format!("action {}: ci {name} is not declared under ci:", a.name),
            ),
        }
        if let Some(k) = ["forge", "api", "token"]
            .into_iter()
            .find(|k| a.executor.get(*k).is_some())
        {
            self.push(
                "action.executor",
                at,
                format!(
                    "action {}: name ci or {k}, not both; ci {name} supplies the endpoint and token",
                    a.name
                ),
            );
        }
    }

    /// Each executor type's own required fields (docs/design/federated-actions.md §5).
    fn executors(&mut self, m: &Manifest) {
        self.forges(m);
        self.ci_hosts(m);
        for (ai, a) in m.actions.iter().enumerate() {
            let ty = a
                .executor
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("command");
            // An unknown type is the schema's error: its enum is `executors::TYPES`.
            let Some(t) = crate::executors::get(ty) else {
                continue;
            };
            self.executor_ci(m, ai, t);
            if let Some(kind) = t.forge.filter(|_| a.executor.get("ci").is_none()) {
                let forge = a
                    .executor
                    .get("forge")
                    .and_then(Value::as_str)
                    .or_else(|| crate::forge::known_host(kind))
                    .unwrap_or_default();
                let project = a
                    .executor
                    .get("project")
                    .and_then(Value::as_str)
                    .unwrap_or("x");
                let url = format!("{}/{project}.git", forge.trim_end_matches('/'));
                if crate::forge::resolve(&m.forges, &url).is_none_or(|r| r.kind != kind) {
                    self.push(
                        "action.executor",
                        format!("/actions/{ai}/executor/forge"),
                        format!(
                            "action {}: {forge} is not a {} forge; list it under forges: with type: {}",
                            a.name,
                            kind.title(),
                            kind.name()
                        ),
                    );
                }
            }
            for k in t.required {
                if a.executor.get(*k).is_none() {
                    self.push(
                        "action.executor",
                        format!("/actions/{ai}/executor"),
                        format!("action {}: a {ty} executor needs `{k}`", a.name),
                    );
                }
            }
            if !t.either.is_empty() && !t.either.iter().any(|k| a.executor.get(*k).is_some()) {
                self.push(
                    "action.executor",
                    format!("/actions/{ai}/executor"),
                    format!(
                        "action {}: a {ty} executor needs `{}`",
                        a.name,
                        t.either.join("` or `")
                    ),
                );
            }
            if let Some(p) = a.executor.get("package").and_then(Value::as_str)
                && crate::artifacts::parse(p).is_none()
            {
                self.push(
                    "action.executor",
                    format!("/actions/{ai}/executor/package"),
                    format!(
                        "action {}: package {p} is not github:owner/repo[/app]@version, a release seikika install can pin",
                        a.name
                    ),
                );
            }
            if ty == "change" {
                self.placeholders(ai, a);
            }
        }
    }

    /// A `change` executor's `branch` takes `{action}` and `{run}`; its `message` also takes
    /// `{entity}` and `{inputs.<name>}` for a declared input. Anything else in braces would be
    /// committed literally.
    fn placeholders(&mut self, ai: usize, a: &crate::manifest::Action) {
        let declared = |n: &str| a.inputs.pointer(&format!("/properties/{n}")).is_some();
        for (field, inputs) in [("branch", false), ("message", true)] {
            let Some(t) = a.executor.get(field).and_then(Value::as_str) else {
                continue;
            };
            let mut rest = t;
            while let Some(open) = rest.find('{') {
                let Some(len) = rest[open..].find('}') else {
                    break;
                };
                let name = &rest[open + 1..open + len];
                rest = &rest[open + len + 1..];
                let ok = matches!(name, "action" | "run" | "run_id")
                    || (inputs && name == "entity")
                    || (inputs && name.strip_prefix("inputs.").is_some_and(declared));
                if !ok {
                    let takes = if inputs {
                        "{action}, {run}, {entity} and {inputs.<name>} for a declared input"
                    } else {
                        "{action} and {run} (inputs go only in `message`)"
                    };
                    self.push(
                        "action.executor",
                        format!("/actions/{ai}/executor/{field}"),
                        format!(
                            "action {}: `{field}` has the unknown placeholder {{{name}}}; it takes {takes}",
                            a.name
                        ),
                    );
                }
            }
        }
    }

    /// D58: action inputs must stay within the form-renderable subset (docs/design/minimal-ui.md §4).
    fn form_subset(&mut self, m: &Manifest) {
        for (ai, a) in m.actions.iter().enumerate() {
            let base = format!("/actions/{ai}/inputs");
            if a.inputs.get("type").and_then(Value::as_str) != Some("object") {
                self.push(
                    "form.inputs",
                    base.clone(),
                    format!("action {}: inputs must be an object", a.name),
                );
            }
            self.form_node(&a.inputs, &format!("action {} inputs", a.name), &base, 0);
        }
    }

    fn form_node(&mut self, node: &Value, label: &str, ptr: &str, depth: usize) {
        const KEYS: &[&str] = &[
            "type",
            "title",
            "description",
            "default",
            "examples",
            "enum",
            "enumFrom",
            "minLength",
            "maxLength",
            "pattern",
            "format",
            "minimum",
            "maximum",
            "items",
            "minItems",
            "maxItems",
            "uniqueItems",
            "properties",
            "required",
            "x-seikika-widget",
        ];
        const SCALAR: &[&str] = &["string", "integer", "number", "boolean"];
        let Some(obj) = node.as_object() else {
            self.push(
                "form.subset",
                ptr.to_string(),
                format!("{label}: must be a schema object"),
            );
            return;
        };
        // A form never sends fields it doesn't show, so closing an object is within the subset.
        let closed = obj.get("type").and_then(Value::as_str) == Some("object")
            && obj.get("additionalProperties") == Some(&Value::Bool(false));
        let bad: Vec<&str> = obj
            .keys()
            .map(String::as_str)
            .filter(|k| !KEYS.contains(k) && !(closed && *k == "additionalProperties"))
            .collect();
        if !bad.is_empty() {
            let hints: Vec<String> = bad
                .iter()
                .filter_map(|k| crate::split_value_hint(k, &obj[*k]))
                .collect();
            self.push(
                "form.subset",
                ptr.to_string(),
                if hints.is_empty() {
                    format!("{label}: keywords outside the form subset: {bad:?}")
                } else {
                    format!("{label}: {}", hints.join("; "))
                },
            );
        }
        match obj.get("type").and_then(Value::as_str) {
            Some("object") => {
                if depth > 1 {
                    self.push(
                        "form.subset",
                        ptr.to_string(),
                        format!("{label}: objects may be nested one level only"),
                    );
                }
                if let Some(props) = obj.get("properties").and_then(Value::as_object) {
                    for (name, sub) in props {
                        self.form_node(
                            sub,
                            &format!("{label}.{name}"),
                            &format!("{ptr}/properties/{name}"),
                            depth + 1,
                        );
                    }
                }
            }
            Some("array") => {
                let item = obj
                    .get("items")
                    .and_then(|i| i.get("type"))
                    .and_then(Value::as_str);
                if !item.is_some_and(|t| SCALAR.contains(&t)) {
                    self.push(
                        "form.subset",
                        ptr.to_string(),
                        format!("{label}: arrays must hold scalars"),
                    );
                }
            }
            Some(t) if SCALAR.contains(&t) => {}
            other => self.push(
                "form.subset",
                ptr.to_string(),
                format!("{label}: unsupported type {other:?}"),
            ),
        }
    }

    /// Every CEL expression parses and reads only the variables available at its site (§6.2).
    fn cel(&mut self, m: &Manifest) {
        let mut exprs: Vec<(String, &str, Site)> = Vec::new();
        for (ki, k) in m.kinds.iter().enumerate() {
            for (ri, r) in k.map.iter().enumerate() {
                exprs.push((format!("/kinds/{ki}/map/{ri}/id"), &r.id, Site::Map));
                exprs.push((format!("/kinds/{ki}/map/{ri}/set"), &r.set, Site::Map));
                for (rel, e) in &r.relations {
                    exprs.push((
                        format!("/kinds/{ki}/map/{ri}/relations/{rel}"),
                        e,
                        Site::Map,
                    ));
                }
            }
            for (fname, f) in &k.fields {
                if let Some(e) = &f.computed {
                    exprs.push((
                        format!("/kinds/{ki}/fields/{fname}/computed"),
                        e,
                        Site::Computed,
                    ));
                }
            }
        }
        for (si, s) in m.scorecards.iter().enumerate() {
            if let Some(e) = &s.applies_to {
                exprs.push((format!("/scorecards/{si}/appliesTo"), e, Site::Scorecard));
            }
            for (ri, r) in s.rules.iter().enumerate() {
                exprs.push((
                    format!("/scorecards/{si}/rules/{ri}/check"),
                    &r.check,
                    Site::Scorecard,
                ));
                if let Some(e) = &r.evidence {
                    exprs.push((
                        format!("/scorecards/{si}/rules/{ri}/evidence"),
                        e,
                        Site::Scorecard,
                    ));
                }
                if let Some(e) = &r.measure {
                    exprs.push((
                        format!("/scorecards/{si}/rules/{ri}/measure"),
                        e,
                        Site::Scorecard,
                    ));
                }
                if let Some(e) = &r.value {
                    exprs.push((
                        format!("/scorecards/{si}/rules/{ri}/value"),
                        e,
                        Site::Scorecard,
                    ));
                }
            }
        }
        if let Some(authz) = &m.authz {
            for (ti, t) in authz.tuples.iter().enumerate() {
                if let TupleDerivation::Kind { emit, .. } = t {
                    exprs.push((format!("/authz/tuples/{ti}/emit"), emit, Site::Tuples));
                }
            }
        }
        let mut options: Vec<(String, String)> = Vec::new();
        for (ai, a) in m.actions.iter().enumerate() {
            if let Some(e) = &a.approval {
                exprs.push((format!("/actions/{ai}/approval"), e, Site::Action));
            }
            for (pi, e) in a.preconditions.iter().enumerate() {
                exprs.push((format!("/actions/{ai}/preconditions/{pi}"), e, Site::Action));
            }
            for (name, lk) in &a.context {
                exprs.push((
                    format!("/actions/{ai}/context/{name}/where"),
                    &lk.where_,
                    Site::ContextLookup,
                ));
            }
            if let Some(x) = &a.expect {
                exprs.push((
                    format!("/actions/{ai}/expect/postcondition"),
                    &x.postcondition,
                    Site::Postcondition,
                ));
                for (ri, r) in x.refresh.iter().enumerate() {
                    if let Some(e) = &r.selector {
                        exprs.push((
                            format!("/actions/{ai}/expect/refresh/{ri}/selector"),
                            e,
                            Site::RefreshScope,
                        ));
                    }
                    if let Some(e) = &r.ids {
                        exprs.push((
                            format!("/actions/{ai}/expect/refresh/{ri}/ids"),
                            e,
                            Site::RefreshScope,
                        ));
                    }
                }
            }
            collect_enum_from(&a.inputs, &format!("/actions/{ai}/inputs"), &mut options);
        }
        for (ptr, expr, site) in exprs {
            self.cel_one(ptr, expr, site);
        }
        for (ptr, expr) in options {
            self.cel_one(ptr, &expr, Site::Options);
        }
    }

    fn cel_one(&mut self, ptr: String, expr: &str, site: Site) {
        if let Err(issue) = seikika_expr::check(expr, site) {
            let code = match issue {
                seikika_expr::CelIssue::Syntax(_) => "cel.syntax",
                seikika_expr::CelIssue::UnknownVariables { .. } => "cel.variable",
                seikika_expr::CelIssue::UndeclaredFunctions(_) => "cel.function",
            };
            self.push(code, ptr, issue.to_string());
        }
    }
}

fn collect_enum_from(node: &Value, ptr: &str, out: &mut Vec<(String, String)>) {
    if let Some(obj) = node.as_object() {
        if let Some(e) = obj.get("enumFrom").and_then(Value::as_str) {
            out.push((format!("{ptr}/enumFrom"), e.to_string()));
        }
        if let Some(props) = obj.get("properties").and_then(Value::as_object) {
            for (name, sub) in props {
                collect_enum_from(sub, &format!("{ptr}/properties/{name}"), out);
            }
        }
        if let Some(items) = obj.get("items") {
            collect_enum_from(items, &format!("{ptr}/items"), out);
        }
    }
}

/// UTC calendar date `YYYY-MM-DD` (schema pattern; month/day digits only, not calendar validity).
fn is_ymd(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

/// Files the manifest names must exist (relative to the portal root).
fn check_files(root: &Path, file: &str, m: &Manifest) -> Vec<Diagnostic> {
    let mut wanted: Vec<(String, String)> = Vec::new();
    if let Some(a) = &m.authz {
        wanted.push(("/authz/model".into(), a.model.clone()));
        wanted.extend(
            a.tests
                .iter()
                .enumerate()
                .map(|(i, t)| (format!("/authz/tests/{i}"), t.clone())),
        );
    }
    wanted.extend(
        m.tests
            .iter()
            .enumerate()
            .map(|(i, t)| (format!("/tests/{i}"), t.clone())),
    );
    if let Some(ui) = &m.ui {
        wanted.extend(
            ui.bundles
                .iter()
                .enumerate()
                .map(|(i, b)| (format!("/ui/bundles/{i}/path"), b.path.clone())),
        );
    }
    wanted.extend(m.waivers.iter().enumerate().map(|(i, w)| {
        (
            format!("/waivers/{i}/decision"),
            format!("../docs/decisions/{}.md", w.decision),
        )
    }));
    wanted
        .into_iter()
        .filter(|(_, p)| !root.join(p).exists())
        .map(|(ptr, p)| Diagnostic {
            code: "file.missing",
            file: file.to_string(),
            path: ptr,
            message: format!("{p} does not exist"),
        })
        .collect()
}
