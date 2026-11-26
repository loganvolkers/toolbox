//! Which rules an agent's judgement decides. A rule is *judged* when its check reads, directly or
//! through computed fields, a relation to entities mapped from `assessment` records (`seikika
//! assess`, D79): its outcome moves when an assessor's verdict does. Every other rule is a
//! deterministic check of facts a connector or tool measured. Reports mark the two apart.

use seikika_model::Manifest;
use std::collections::BTreeSet;

/// The kinds whose entities are mapped from `assessment` records.
pub fn assessment_kinds(m: &Manifest) -> BTreeSet<&str> {
    m.kinds
        .iter()
        .filter(|k| k.map.iter().any(|r| r.from_parts().1 == "assessment"))
        .map(|k| k.name.as_str())
        .collect()
}

/// `(scorecard, rule)` for every judged rule in `m`.
pub fn judged_rules(m: &Manifest) -> BTreeSet<(String, String)> {
    let assessed = assessment_kinds(m);
    let mut out = BTreeSet::new();
    if assessed.is_empty() {
        return out;
    }
    for sc in &m.scorecards {
        for r in &sc.rules {
            if reads_judgement(m, &assessed, &sc.kind, &r.check, &mut BTreeSet::new()) {
                out.insert((sc.name.clone(), r.name.clone()));
            }
        }
    }
    out
}

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

/// Whether `cel`, evaluated on a `kind` entity, reads an assessment relation, following computed
/// fields (`seen` stops cycles).
fn reads_judgement(
    m: &Manifest,
    assessed: &BTreeSet<&str>,
    kind: &str,
    cel: &str,
    seen: &mut BTreeSet<String>,
) -> bool {
    let Ok(a) = seikika_expr::analysis::analyse(cel) else {
        return false;
    };
    let Some(k) = m.kinds.iter().find(|k| k.name == kind) else {
        return false;
    };
    let inbound = cel.contains("inbound") && {
        m.kinds
            .iter()
            .filter(|s| assessed.contains(s.name.as_str()))
            .flat_map(|s| &s.relations)
            .any(|(name, rel)| rel.to == kind && a.selects.contains(name))
    };
    let outbound = k
        .relations
        .iter()
        .any(|(name, rel)| assessed.contains(rel.to.as_str()) && a.fields.contains(name));
    if inbound || outbound {
        return true;
    }
    a.fields.iter().any(|f| {
        let computed = k.fields.get(f).and_then(|fd| fd.computed.as_deref());
        match computed {
            Some(c) if seen.insert(f.clone()) => reads_judgement(m, assessed, kind, c, seen),
            _ => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORTAL: &str = r"
apiVersion: seikika.com/v1alpha1
portal: {name: t, owner: group:a, seikika: '>=0.1'}
kinds:
  - name: Crate
    identityFrom: [connector:ws]
    sources: {default: [connector:ws]}
    fields:
      docs: {type: integer}
      reviewFailing:
        type: list
        computed: 'inbound.assesses.size() == 0 ? [] : inbound.assesses[0].failing'
      clean: {type: boolean, computed: entity.reviewFailing.size() == 0}
    map: [{from: ws/crate, id: record.id, set: record}]
  - name: Assessment
    identityFrom: [connector:ws]
    sources: {default: [connector:ws]}
    fields:
      failing: {type: list}
      uncited: {type: list}
    relations:
      assesses: {to: Crate}
    map: [{from: ws/assessment, id: record.id, set: record}]
scorecards:
  - name: crate
    kind: Crate
    owner: group:a
    levels: [bronze]
    rules:
      - {name: documented, level: bronze, check: entity.docs > 0}
      - {name: reviewed, level: bronze, check: entity.reviewFailing.size() == 0}
      - {name: clean, level: bronze, check: entity.clean}
  - name: assessment
    kind: Assessment
    owner: group:a
    levels: [bronze]
    rules:
      - {name: cited, level: bronze, check: entity.uncited.size() == 0}
";

    #[test]
    fn judged_through_computed_fields_and_not_on_the_assessment_itself() {
        let m: Manifest = serde_norway::from_str(PORTAL).unwrap();
        let judged = judged_rules(&m);
        let names: Vec<String> = judged.iter().map(|(s, r)| format!("{s}/{r}")).collect();
        assert_eq!(names, ["crate/clean", "crate/reviewed"]);
        assert_eq!(marked(&judged, "crate", "reviewed"), "reviewed ✦");
        assert_eq!(marked(&judged, "crate", "documented"), "documented");
    }

    #[test]
    fn nothing_is_judged_without_assessment_records() {
        let m: Manifest =
            serde_norway::from_str(&PORTAL.replace("ws/assessment", "ws/review")).unwrap();
        assert!(judged_rules(&m).is_empty());
    }
}
