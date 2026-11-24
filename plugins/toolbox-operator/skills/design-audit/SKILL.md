---
name: design-audit
description: >-
  Runs an independent design audit of a rendered portal UI (the hosted report, the minimal UI,
  a custom view): evidence capture, a blind heuristic and task review, severity-ranked usability
  fixes, prioritised joy opportunities, and a blind re-audit that shows the change moved the
  scores. Use before redesigning a page, after a UI change lands, or whenever readers say a page
  is hard to understand or dull.
---

# Design audit

An audit judges what a reader sees, not what the code intends. The auditor is independent: a
fresh agent or person who did not write the change, who sees only rendered evidence, the reader
jobs and the rubric. The implementer never scores their own work. Scores are 0–4 per dimension
in [references/rubric.md](references/rubric.md): ten usability dimensions (U1–U10), four joy
dimensions (J1–J4), and named reader tasks. Machines check first: for the hosted report, the
story probes in `report-ui` must pass before an auditor is spent (D01M87PSM9YQGGW3YQ5X4XCJP63).

## Procedure

1. **Name the readers and their tasks.** Three to six tasks, each with a reader, a question and
   what counts as the answer ("VP: what level is the repository at, and what holds it there?").
   Write them before looking at the page so the tasks are not fitted to the design. Use two
   reference portals, and put at least one task on the second, so the design is not fitted to
   one portal either. For the report: Toolbox for Toolbox and the OpenFGA portal.
2. **Probe.** For the report, `npm run build-storybook && npm run probe` in `report-ui`. It
   renders every story over the fixture portals at both viewports and schemes, and fails on
   script errors, serious axe violations and sideways scroll. Fix those before step 4: an
   auditor's time is for what a probe cannot judge.
3. **Capture evidence.** Serve the page or a Storybook build, and run the capture script over
   every route or story a task touches. It writes screenshots (desktop and mobile, light and dark), a full-page desktop
   shot, and `metrics.json` with axe-core violations (WCAG 2.0 to 2.2 AA and best practice, so
   `target-size` counts), visual-variety counts and
   `mobileOverflow` (pixels a 390 px page scrolls sideways; anything above 0 is a defect). It waits for
   animations to finish and takes the full-page shot with reduced motion, so a fade caught
   mid-way is not scored as low contrast; the auditor checks motion on the live page.
4. **Audit blind, as a panel.** Give three fresh auditors, separately, the tasks, the rubric,
   the screenshots and `metrics.json`, and nothing else: not the source, not the diff, not
   earlier scores, not each other's. Each auditor walks each task (cognitive walkthrough: will the reader know what to do, see how,
   and know it worked?), scores every dimension with cited evidence, and lists findings with
   severity 1–4. Record every auditor's scores and the median per dimension; the median is
   the panel's score.
5. **Prioritise.** Usability first: every severity 3–4 finding is a must, then by reach and
   effort. Joy second: Kano delighters that reinforce a task (progress, achievement, identity).
   Record the ranked list in the audit file with the reason for each cut.
6. **Fix.** Implement in priority order. Keep the data contract stable or extend it; never
   drop information a task needs to make a view prettier. Say whether each fix is the box's
   (the element, a package) or one portal's configuration.
7. **Re-audit blind.** Probe, recapture the same routes and stories, and give a new panel the
   same inputs as step 4. A round passes when the probes pass, no severity-3 or 4 finding is
   open, and no targeted dimension's median fell. Stop when two passing rounds in a row raise
   no targeted median, or every targeted dimension is at 4. When a median falls, trace its
   cited evidence: on a view the change touched it is a regression, and on a view it did not it
   is a finding earlier panels missed. Fix both in the next round. One auditor's score is not a
   verdict: in the report's second loop, a dimension moved by chance in 28% of round-to-round
   comparisons.
8. **Publish.** Commit the audit file as `docs/audits/<surface>-<yyyy-mm-dd>.md` with both
   score tables, the task outcomes, the metric deltas, the fixes made and the findings left open.

```bash
npm i --prefix /tmp/audit playwright axe-core && npx --prefix /tmp/audit playwright install chromium
NODE_PATH=/tmp/audit/node_modules node scripts/capture.cjs \
  --url http://127.0.0.1:8765/index.html \
  --routes '#overview,#catalog,#campaigns,#scorecards' --ready section --out /tmp/audit/before
# or from a Storybook build served on 6006 (report-ui: npm run build-storybook)
NODE_PATH=/tmp/audit/node_modules node scripts/capture.cjs \
  --storybook http://127.0.0.1:6006 --stories pages-openfga--overview,pages-toolbox-for-toolbox--compare \
  --out /tmp/audit/before
```

## Auditor brief

Send the auditor this, with the tasks and file paths filled in:

```text
You are an independent UX auditor. Judge only the attached screenshots and metrics.json; do not
read source code. Readers and tasks: <tasks>. Use the rubric: <rubric>. For each task give
outcome (success/partial/fail), steps from the landing view, and the biggest obstacle. Score
U1–U10 and J1–J4 from 0 to 4, citing a screenshot or metric for each. List findings as
id | dimension | severity 1–4 | evidence | recommendation, most severe first. Then name the
three joy opportunities with the most reach.
```

## Audit file

| Section | Content |
|---|---|
| Scope | Surface, commit, routes, viewports, readers and tasks |
| Scores | Each auditor's U1–U10, J1–J4 and task outcomes, and the panel median; rounds side by side |
| Metrics | `metrics.json` summary: serious axe nodes, monospace share, distinct font sizes and hues, words above the fold |
| Findings | Ranked, with severity, evidence and the fix or the reason it stays open |
| Joy | Opportunities taken and declined |

## Boundaries

- Never let the implementer score the work, and never show the re-auditor the first scores or
  one panellist another's: anchoring inflates both.
- Do not trade a usability score for a joy score. A delighter that hides data, slows a task or
  fails contrast is a regression.
- Visual-variety metrics describe the page; they are not targets. More hues or font sizes is
  not better on its own.
- Limitation: an agent auditor is a proxy for readers. For a release-defining change, add five
  real readers on the same tasks (Nielsen's five-user rule) and record what they did.
- Prerequisite: the page must render from a local server or file URL. On error (the capture
  prints `not ready`), fix the load or the `--ready` selector before auditing a blank page.
