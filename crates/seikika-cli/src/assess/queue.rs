//! Scoped assessment work from the same snapshot as the validation check.

use super::{AssessmentFile, Check, assess, entity_body, version_changes};
use seikika_derive::{CampaignEntity, CampaignProgress, Derived, Entity};
use seikika_model::{Manifest, manifest::Rubric};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

pub(super) struct Scope<'a> {
    options: &'a Check,
    campaign: Option<&'a CampaignProgress>,
    pub(super) matched: BTreeSet<&'a str>,
}

impl<'a> Scope<'a> {
    pub(super) fn new(options: &'a Check, m: &Manifest, d: &'a Derived) -> Result<Self, String> {
        if let Some(name) = &options.rubric
            && !m.rubrics.iter().any(|r| &r.name == name)
        {
            return Err(format!("no rubric {name}"));
        }
        let campaign = options
            .campaign
            .as_ref()
            .map(|name| {
                d.campaigns
                    .get(name)
                    .ok_or_else(|| format!("no campaign {name}"))
            })
            .transpose()?;
        Ok(Self {
            options,
            campaign,
            matched: BTreeSet::new(),
        })
    }

    pub(super) fn rubric(&self, rubric: &Rubric) -> bool {
        self.options
            .rubric
            .as_ref()
            .is_none_or(|r| r == &rubric.name)
    }

    pub(super) fn includes(&self, id: &str) -> bool {
        (self.options.entities.is_empty() || self.options.entities.iter().any(|e| e == id))
            && self.campaign.is_none_or(|c| c.entities.contains_key(id))
    }

    fn campaign_row(&self, id: &str) -> Option<&CampaignEntity> {
        self.campaign.and_then(|c| c.entities.get(id))
    }

    pub(super) fn check_matches(&self) -> Result<(), String> {
        for id in &self.options.entities {
            if !self.matched.contains(id.as_str()) {
                return Err(format!("no assessment subject {id} in the selected scope"));
            }
        }
        Ok(())
    }
}

fn same_name(a: Option<&str>, b: Option<&str>) -> bool {
    matches!((a.map(str::trim), b.map(str::trim)), (Some(a), Some(b))
        if !a.is_empty() && a.to_lowercase() == b.to_lowercase())
}

struct ReviewRouting<'a> {
    is_review: bool,
    author: Option<&'a str>,
    needed: bool,
    self_review: bool,
    wrong_reviewer: bool,
}

impl<'a> ReviewRouting<'a> {
    fn new(
        entity: &'a Entity,
        rubric: &Rubric,
        doc: Option<&Value>,
        reviewer: Option<&str>,
    ) -> Self {
        let is_review = entity.kind == "Assessment" && rubric.name == "assessment-quality";
        let author = entity
            .fields
            .get("assessorName")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty());
        let assessed_by = doc.and_then(|doc| doc["assessor"]["name"].as_str());
        Self {
            is_review,
            author,
            needed: is_review
                && entity
                    .fields
                    .get("reviewVerified")
                    .is_some_and(|v| v != true),
            self_review: is_review && same_name(assessed_by, author),
            wrong_reviewer: is_review && same_name(reviewer, author),
        }
    }

    fn missing_author(&self) -> bool {
        self.is_review && self.author.is_none()
    }
}

fn reasons(
    subject: Option<&Value>,
    doc: Option<&Value>,
    rubric_hash: &str,
    review: &ReviewRouting<'_>,
    fresh_verdict: Option<&str>,
) -> Vec<&'static str> {
    let (subject_changed, rubric_changed) = doc
        .map(|doc| version_changes(subject, rubric_hash, doc))
        .unwrap_or_default();
    [
        (
            subject.and_then(Value::as_str).is_none(),
            "missing-subject-hash",
        ),
        (doc.is_none(), "missing-assessment"),
        (subject_changed, "subject-changed"),
        (rubric_changed, "rubric-changed"),
        (
            review.is_review && doc.is_some_and(|d| d["rubricHash"].is_null()),
            "unbound-review-rubric",
        ),
        (review.missing_author(), "missing-original-assessor"),
        (review.self_review, "self-review"),
        (review.wrong_reviewer, "reviewer-is-original-assessor"),
        (review.needed, "independent-review-required"),
        (fresh_verdict == Some("fail"), "failing-criteria"),
        (fresh_verdict == Some("unsure"), "unsure-criteria"),
    ]
    .into_iter()
    .filter_map(|(present, reason)| present.then_some(reason))
    .collect()
}

fn next_action(
    subject: Option<&Value>,
    review: &ReviewRouting<'_>,
    verdict: Option<&str>,
    passed: bool,
) -> &'static str {
    if subject.and_then(Value::as_str).is_none() || review.missing_author() {
        "collect-evidence"
    } else if review.self_review || review.wrong_reviewer && !passed {
        "hand-off-review"
    } else if verdict == Some("fail") || passed {
        "repair"
    } else if verdict == Some("unsure") {
        "collect-evidence"
    } else if review.is_review {
        "independent-review"
    } else {
        "assess"
    }
}

pub(super) fn pending(
    rubric: &Rubric,
    entity: &Entity,
    file: Option<&AssessmentFile>,
    mut row: Value,
    rubric_hash: &str,
    scope: &Scope<'_>,
) -> Option<Value> {
    let id = row["entity"].as_str()?.to_owned();
    let campaign = scope.campaign_row(&id);
    let fields = &entity.fields;
    let doc = file.map(|file| &file.doc);
    let review = ReviewRouting::new(entity, rubric, doc, scope.options.reviewer.as_deref());
    let verdict = row["verdict"].as_str().filter(|_| row["state"] == "fresh");
    let passed = verdict == Some("pass") && !review.needed;
    if campaign.map_or(passed, |c| c.meeting) {
        return None;
    }

    let subject = fields.get(rubric.subject());
    let mut reasons = reasons(subject, doc, rubric_hash, &review, verdict);
    let action = next_action(subject, &review, verdict, passed);
    if let Some(campaign) = campaign {
        row["gaps"] = json!(campaign.gaps);
        if reasons.is_empty() {
            reasons.push("campaign-evidence-gap");
        }
    }

    let path = file.map_or_else(
        || assess::file_for(&rubric.name, &id),
        |file| file.path.clone(),
    );
    row["rubric"] = json!(rubric.name);
    row["action"] = json!(action);
    row["reasons"] = json!(reasons);
    row["assessmentPath"] = json!(path);
    row["versions"] = json!({
        "subject": { "field": rubric.subject(), "current": subject, "recorded": doc.map(|d| &d["subjectHash"]) },
        "rubric": { "current": rubric_hash, "recorded": doc.map(|d| &d["rubricHash"]) },
    });
    if review.is_review {
        row["originalAssessor"] = json!(review.author);
    }
    if let Some(path) = fields.get("path") {
        row["subjectPath"] = path.clone();
    }
    Some(row)
}

pub(super) fn report(
    options: &Check,
    manifest: &Manifest,
    derived: &Derived,
    scope: &Scope<'_>,
    mut queue: Vec<Value>,
    invalid: &BTreeMap<String, Vec<String>>,
) -> Value {
    let mut counts = BTreeMap::new();
    for item in &queue {
        *counts
            .entry(item["action"].as_str().unwrap_or_default().to_owned())
            .or_insert(0usize) += 1;
    }
    if let Some(action) = &options.action {
        queue.retain(|row| row["action"] == action.as_str());
    }
    queue.sort_by(|a, b| {
        let key = |row: &Value| match row["action"].as_str() {
            Some("repair") => 0,
            Some("collect-evidence") => 1,
            Some("hand-off-review") => 2,
            Some("independent-review") => 3,
            _ => 4,
        };
        key(a)
            .cmp(&key(b))
            .then_with(|| a["rubric"].as_str().cmp(&b["rubric"].as_str()))
            .then_with(|| a["entity"].as_str().cmp(&b["entity"].as_str()))
    });
    let total_pending = queue.len();
    queue.truncate(options.limit.unwrap_or(20) as usize);
    if options.with_briefs {
        for row in &mut queue {
            if row["action"] == "hand-off-review" {
                continue;
            }
            let id = row["entity"].as_str().unwrap_or_default();
            if let Some(body) = entity_body(derived, id) {
                match assess::brief(manifest, &body, row["rubric"].as_str()) {
                    Ok(mut brief) => {
                        brief["file"] = row["assessmentPath"].clone();
                        row["brief"] = brief;
                    }
                    Err(error) => row["briefUnavailable"] = json!(error),
                }
            }
        }
    }
    let unmatched = scope.campaign.map(|campaign| {
        campaign
            .entities
            .keys()
            .filter(|id| scope.includes(id) && !scope.matched.contains(id.as_str()))
            .collect::<Vec<_>>()
    });
    json!({
        "scope": {"campaign": options.campaign, "rubric": options.rubric, "entities": options.entities, "action": options.action},
        "pendingByAction": counts,
        "totalPending": total_pending,
        "returned": queue.len(),
        "unmatchedCampaignEntities": unmatched,
        "queue": queue,
        "invalid": invalid,
    })
}

pub(super) fn markdown(report: &Value) -> String {
    let mut out = format!(
        "## Assessment queue\n\n{} pending, {} shown\n",
        report["totalPending"], report["returned"]
    );
    for row in report["queue"].as_array().into_iter().flatten() {
        let reasons = row["reasons"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            out,
            "- {} `{}` against `{}` ({reasons}). Assessment file `{}`.",
            row["action"].as_str().unwrap_or_default(),
            row["entity"].as_str().unwrap_or_default(),
            row["rubric"].as_str().unwrap_or_default(),
            row["assessmentPath"].as_str().unwrap_or_default()
        );
        if let Some(author) = row["originalAssessor"].as_str()
            && row["action"] == "hand-off-review"
        {
            let _ = writeln!(out, "  Assign a reviewer independent of {author}.");
        }
        for key in ["failing", "unsure", "gaps"] {
            let values = row[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>();
            if !values.is_empty() {
                let _ = writeln!(out, "  {key}: {}", values.join(", "));
            }
        }
        if let Some(brief) = row.get("brief") {
            out.push('\n');
            out.push_str(&assess::markdown(brief));
        }
    }
    if let Some(unmatched) = report["unmatchedCampaignEntities"].as_array()
        && !unmatched.is_empty()
    {
        let _ = writeln!(
            out,
            "\n{} selected campaign entities have no applicable rubric.",
            unmatched.len()
        );
    }
    out.push_str(&super::check_markdown(report));
    out
}
