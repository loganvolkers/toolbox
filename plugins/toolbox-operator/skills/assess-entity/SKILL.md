---
name: assess-entity
description: Assess a Toolbox entity (a rule, scorecard, crate, workflow or anything a portal has a rubric for) against the portal's rubric, and record the judgement as an assessment file or in the portal's database. Use when asked to review, judge or assess standards, scorecards, rules or components, or when a review-* scorecard says an entity is unassessed or stale.
metadata:
  status: pre-release
---

# Assess an entity

Toolbox calls no model. You are the assessor: the portal gives you a rubric and the facts, you judge,
and the portal stores your judgement as data that scorecards read.

1. **Get the brief.** Run `toolbox assess brief <entity-id> --portal <dir>` (add `--fixtures` for a
   portal you can't run connectors for), or call the `assessment_brief` MCP tool on a running portal.
   The brief lists the rubric's criteria, the entity's facts, its `subjectHash`, the file to write and
   a template.
2. **Read the sources the facts point to,** not just the facts: the manifest entry, the code, the design
   doc a scorecard cites. A criterion is a question a deterministic check can't answer, so the answer
   is in what you read.
3. **Judge each criterion** `pass`, `fail` or `unsure`:
   - `reason`: one or two sentences that cite what you read (a file, a decision, a number).
   - `suggestion`: for `fail` and `unsure`, the change that would make it pass.
   - Say `unsure` rather than guess. An `unsure` with a good reason is more useful than a wrong
     `pass`.
4. **Write the assessment** to the path the brief names (under the portal's `assessments/`), keeping
   `subjectHash` exactly as given. Put your harness in `assessor.name` and today's date in `at`.
5. **Check it:** `toolbox assess check <dir>` validates every file against
   `schema/assessment.schema.json` and the rubric, and lists entities as fresh, stale or missing.
6. **Portals without a repository:** push instead of committing, with
   `toolbox assess submit <file> --portal <dir> --url <server>` and the connector token in
   `TOOLBOX_CONNECTOR_TOKEN`.

## Rules

- Assess the version you were briefed on. If the entity changes, the assessment goes stale; brief and
  assess again rather than editing `subjectHash`.
- When you change an assessed entity yourself, re-assess it in the same change.
- Never write an assessment you didn't make. If a criterion needs access you don't have, say `unsure`
  and name what you couldn't read.
- Assessments are advisory: `review-*` scorecards report them but never gate a merge. Don't weaken a
  verdict to raise a level.
