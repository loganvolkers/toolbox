//! Assessment briefs (docs/design/scorecards-for-scorecards.md §6): everything a coding agent needs
//! to judge one entity against one of the portal's rubrics and write the result down. Toolbox never
//! calls a model: the agent the developer already runs reads the brief (from `toolbox assess brief`
//! or the `assessment_brief` MCP tool), assesses, and writes an assessment file or pushes it.

use serde_json::{Value, json};
use std::fmt::Write as _;
use toolbox_model::Manifest;
use toolbox_model::manifest::Rubric;

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

/// The brief for `entity` (a `GET /v1/entities/{id}` body: `{id, kind, fields}`).
pub fn brief(m: &Manifest, entity: &Value, rubric_name: Option<&str>) -> Result<Value, String> {
    let id = entity["id"].as_str().unwrap_or_default();
    let kind = entity["kind"].as_str().unwrap_or_default();
    let r = rubric(m, kind, rubric_name)?;
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
    let template = format!(
        "apiVersion: toolbox.dev/assessment/v1\nentity: {id}\nrubric: {}\nsubjectHash: \"{hash}\"\n\
         assessor: {{kind: agent, name: \"\", model: \"\"}}\nat: \"\"\nverdicts:\n{verdicts}summary: \"\"\n",
        r.name
    );
    Ok(json!({
        "entity": id, "kind": kind,
        "rubric": {"name": r.name, "description": r.description, "subject": r.subject(),
                   "criteria": r.criteria.iter().map(|c| json!({"name": c.name, "question": c.question, "guidance": c.guidance})).collect::<Vec<_>>()},
        "subjectHash": hash,
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
        "## Write it down\n\nSave as `{}` in the portal directory (or push it with `toolbox assess submit`). \
         `subjectHash` is the entity's `{}` now ({}); when that changes, the assessment is stale.\n\n```yaml\n{}```\n\n\
         Then run `toolbox assess check` to validate it.",
        b["file"].as_str().unwrap_or(""),
        r["subject"].as_str().unwrap_or("hash"),
        b["subjectHash"].as_str().unwrap_or(""),
        b["template"].as_str().unwrap_or("")
    );
    s
}

/// Problems with an assessment document against the schema and the portal's rubrics.
pub fn problems(m: &Manifest, doc: &Value) -> Vec<String> {
    let mut out: Vec<String> =
        toolbox_model::schema::errors(toolbox_model::schema::assessment(), doc)
            .into_iter()
            .map(|(p, e)| format!("{}: {e}", if p.is_empty() { "/" } else { &p }))
            .collect();
    let Some(r) = m
        .rubrics
        .iter()
        .find(|r| Some(r.name.as_str()) == doc["rubric"].as_str())
    else {
        out.push(format!("unknown rubric {}", doc["rubric"]));
        return out;
    };
    let named: Vec<&str> = doc["verdicts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v["criterion"].as_str())
        .collect();
    for c in &r.criteria {
        if !named.contains(&c.name.as_str()) {
            out.push(format!("no verdict for criterion {}", c.name));
        }
    }
    for n in named {
        if !r.criteria.iter().any(|c| c.name == n) {
            out.push(format!("{n} is not a criterion of {}", r.name));
        }
    }
    out
}
