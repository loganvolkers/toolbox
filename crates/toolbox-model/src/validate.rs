//! `toolbox validate` (docs/v1-spec.md §5): schema, cross-references, form subset, CEL, files and
//! fixtures. The cross-reference and form-subset checks mirror
//! `examples/reference-portal/tools/validate.py`; the CEL, duplicate-name, relation-target,
//! authority and file checks are additional.

use crate::diagnostics::{Diagnostic, Report};
use crate::load::{self, LoadError, Source};
use crate::manifest::{Manifest, TupleDerivation};
use crate::{messages, schema};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use toolbox_expr::Site;

/// Load and validate the portal at `path` (a manifest file or a portal directory).
pub fn validate_portal(path: &Path) -> Result<Report, LoadError> {
    let Source {
        root,
        manifest_file,
        included,
        document,
    } = load::load(path)?;
    let mut report = Report {
        manifest: manifest_file.clone(),
        included,
        ..Report::default()
    };
    let (manifest, mut errors) = validate_manifest(&document, &manifest_file);
    if let Some(m) = &manifest {
        errors.extend(check_files(&root, &manifest_file, m));
    }
    let (count, fixture_errors) = messages::validate_fixtures(&root, manifest.as_ref());
    report.messages = count;
    errors.extend(fixture_errors);
    report.errors = errors;
    Ok(report)
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
    v.cel(&manifest);
    (Some(manifest), v.errors)
}

struct Checker {
    file: String,
    errors: Vec<Diagnostic>,
}

impl Checker {
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

        for (si, s) in m.scorecards.iter().enumerate() {
            if !kinds.contains_key(s.kind.as_str()) {
                self.push(
                    "xref.kind",
                    format!("/scorecards/{si}/kind"),
                    format!("scorecard {}: unknown kind {}", s.name, s.kind),
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
            "x-toolbox-widget",
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
        let bad: Vec<&str> = obj
            .keys()
            .map(String::as_str)
            .filter(|k| !KEYS.contains(k))
            .collect();
        if !bad.is_empty() {
            self.push(
                "form.subset",
                ptr.to_string(),
                format!("{label}: keywords outside the form subset: {bad:?}"),
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
                    Site::Action,
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
        if let Err(issue) = toolbox_expr::check(expr, site) {
            let code = match issue {
                toolbox_expr::CelIssue::Syntax(_) => "cel.syntax",
                toolbox_expr::CelIssue::UnknownVariables { .. } => "cel.variable",
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
