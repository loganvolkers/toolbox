//! Assessment briefs (docs/design/scorecards-for-scorecards.md §6): everything a coding agent needs
//! to judge one entity against one of the portal's rubrics and write the result down. Seikika never
//! calls a model: the agent the developer already runs reads the brief (from `seikika assess brief`
//! or the `assessment_brief` MCP tool), assesses, and writes an assessment file or pushes it.

pub use seikika_connect::assessment::{problems, rubric_hash};
use seikika_model::Manifest;
use seikika_model::manifest::Rubric;
use serde_json::{Map, Value, json};
use std::fmt::Write as _;

/// Where an assessment of `entity` against `rubric` lives, relative to the portal directory.
pub fn file_for(rubric: &str, entity: &str) -> String {
    format!("assessments/{rubric}/{}.yaml", entity.replace(':', "/"))
}

/// The rubric named `name`, or the only rubric for `kind`.
pub fn rubric<'a>(m: &'a Manifest, kind: &str, name: Option<&str>) -> Result<&'a Rubric, String> {
    if let Some(n) = name.filter(|n| !n.is_empty()) {
        return m
            .rubrics
            .iter()
            .find(|r| r.name == n && r.kind == kind)
            .ok_or_else(|| format!("no rubric {n} for kind {kind}"));
    }
    let mut for_kind = m.rubrics.iter().filter(|r| r.kind == kind);
    match (for_kind.next(), for_kind.next()) {
        (Some(r), None) => Ok(r),
        (None, _) => Err(format!("the portal has no rubric for kind {kind}")),
        (Some(_), Some(_)) => Err(format!(
            "several rubrics assess {kind}: name one ({})",
            m.rubrics
                .iter()
                .filter(|r| r.kind == kind)
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Whether `r` judges an entity with these fields. A rubric never judges an assessment made with
/// itself, so reviews stop at one level (docs/design/scorecards-for-scorecards.md §1, §6.1).
pub fn applies(r: &Rubric, fields: &Map<String, Value>) -> bool {
    fields.get("rubric").and_then(Value::as_str) != Some(r.name.as_str())
}

/// The brief for `entity` (a `GET /v1/entities/{id}` body: `{id, kind, fields}`).
pub fn brief(m: &Manifest, entity: &Value, rubric_name: Option<&str>) -> Result<Value, String> {
    let id = entity["id"].as_str().unwrap_or_default();
    let kind = entity["kind"].as_str().unwrap_or_default();
    let r = rubric(m, kind, rubric_name)?;
    if entity["fields"].as_object().is_some_and(|f| !applies(r, f)) {
        return Err(format!(
            "{id} was made with rubric {}, which does not review its own assessments",
            r.name
        ));
    }
    let subject = &entity["fields"][r.subject()];
    let hash = subject.as_str().map(str::to_owned).ok_or_else(|| {
        format!(
            "{id} has no `{}` to version the assessment by (rubric {})",
            r.subject(),
            r.name
        )
    })?;
    let verdicts: String = r
        .criteria
        .iter()
        .map(|c| {
            format!(
                "  - criterion: {}\n    verdict: pass   # pass | fail | unsure\n    reason: \"\"\n",
                c.name
            )
        })
        .collect();
    let rubric_hash = rubric_hash(r);
    let template = format!(
        "apiVersion: seikika.com/assessment/v1\nentity: {id}\nrubric: {}\nsubjectHash: \"{hash}\"\nrubricHash: \"{rubric_hash}\"\n\
         assessor: {{kind: agent, name: \"\", model: \"\"}}\nat: \"\"\nverdicts:\n{verdicts}summary: \"\"\n",
        r.name
    );
    Ok(json!({
        "entity": id, "kind": kind,
        "rubric": {"name": r.name, "description": r.description, "subject": r.subject(), "links": r.links,
                   "criteria": r.criteria.iter().map(|c| json!({"name": c.name, "question": c.question, "guidance": c.guidance})).collect::<Vec<_>>()},
        "subjectHash": hash,
        "rubricHash": rubric_hash,
        "file": file_for(&r.name, id),
        "template": template,
        "facts": entity["fields"],
    }))
}

/// The brief as instructions an agent can follow.
pub fn markdown(b: &Value) -> String {
    let mut s = String::new();
    let r = &b["rubric"];
    let _ = writeln!(
        s,
        "# Assess `{}` against `{}`\n",
        b["entity"].as_str().unwrap_or(""),
        r["name"].as_str().unwrap_or("")
    );
    if let Some(d) = r["description"].as_str() {
        let _ = writeln!(s, "{d}\n");
    }
    let _ = writeln!(
        s,
        "Answer each question from the facts below and the sources they point to. Say `unsure` rather \
         than guess, give a reason that cites what you read, and a suggestion for anything that fails.\n\n## Criteria\n"
    );
    for c in r["criteria"].as_array().into_iter().flatten() {
        let _ = writeln!(
            s,
            "- **{}**: {}",
            c["name"].as_str().unwrap_or(""),
            c["question"].as_str().unwrap_or("")
        );
        if let Some(g) = c["guidance"].as_str() {
            let _ = writeln!(s, "  {g}");
        }
    }
    let facts = serde_json::to_string_pretty(&b["facts"]).unwrap_or_default();
    let _ = writeln!(s, "\n## Facts\n\n```json\n{facts}\n```\n");
    let _ = writeln!(
        s,
        "## Write it down\n\nSave as `{}` in the portal directory (or push it with `seikika assess submit`). \
         `subjectHash` is the entity's `{}` now ({}); when that changes, the assessment is stale.\n\n```yaml\n{}```\n\n\
         Then run `seikika assess check` to validate it.",
        b["file"].as_str().unwrap_or(""),
        r["subject"].as_str().unwrap_or("hash"),
        b["subjectHash"].as_str().unwrap_or(""),
        b["template"].as_str().unwrap_or("")
    );
    s
}
