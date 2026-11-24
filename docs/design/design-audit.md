# Design Audits: Independent, Rubric-Scored Reviews of Rendered UI

*Status: decided 2026-11-24 (D01M85MK7FCRXH2TRMRKXTNA5GT); evidence, probes, panel and stop rule
revised the same day (D01M87PSM9YQGGW3YQ5X4XCJP63) after a 36-round loop measured one auditor's
noise ([design-loop-efficiency-2026-11-24.md](../audits/design-loop-efficiency-2026-11-24.md)).
The agent skill is [`design-audit`](../../plugins/toolbox-operator/skills/design-audit/SKILL.md);
the records are in [docs/audits/](../audits/).*

**About:** the box. It is how the maintainer SDLC's `design-review` stage judges what the box
shows readers: the hosted report, the minimal UI and any custom view. It wraps no tool. Playwright and axe-core are used
as they ship.

---

## 1. Conclusion

A UI change is judged in two tiers. **Probes** check what a machine can: every Storybook story,
over fixtures from at least two reference portals, at 1280 and 390 px in light and dark, with no
script error, no serious axe-core violation (WCAG 2.2 A and AA, so 24 px targets) and no sideways
scroll. They run in CI in under a minute. **A panel of three independent auditors**, who see only
rendered evidence, judges the rest against a fixed rubric and named reader tasks. The person or
agent who made the change fixes but never scores.

A round passes when:

- the probes pass;
- no severity-3 or 4 finding is open;
- on every dimension the round targeted, the panel's median did not fall.

The loop stops when two passing rounds in a row raise no targeted median, or when every targeted
dimension is at 4. A median over three auditors is the verdict, not any one score. In the report's
second loop, which used one auditor per round and required that no dimension fall, a dimension
fell in 14% of round-to-round comparisons and rose in 14% over 22 plateau rounds. A round with no
fall then had a 22% chance, so that rule rarely passed and the loop ran 36 rounds
([design-loop-efficiency-2026-11-24.md](../audits/design-loop-efficiency-2026-11-24.md)).

When a median falls, the cited evidence says why: on a view the change touched it is a
regression, and on a view it did not it is a finding earlier panels missed. Both are fixed in the
next round.

| Step | Who | Output |
|---|---|---|
| Name readers and tasks | Implementer, before looking | 3–6 tasks with what counts as an answer, at least one on the second reference portal |
| Probe | `npm run probe` in `report-ui` (CI) | Pass or the failing story, view and rule |
| Capture | Script, from a Storybook build or a served page | Screenshots (desktop and mobile, light and dark) and `metrics.json` |
| Audit | Three fresh auditors, blind, independently | Task outcomes, U1–U10 and J1–J4 scores, ranked findings, joy ideas; the record keeps each and the median |
| Prioritise and fix | Implementer | Severity 3–4 findings first, then joy that reinforces a task; each fix says whether it is the box's or a portal's |
| Re-audit | A new panel each round, blind to earlier scores | Another median table, until the stop rule holds |
| Publish | Implementer | `docs/audits/<surface>-<date>.md` |

## 2. Why this process, and not a checklist

The hosted report had a design skill (`report-ux`) with reader jobs and reject criteria. The
maintainers who built the page applied it, and the page still read as "functional but joyless,
hard to understand, homogeneous". A checklist written by the builders checks the builders'
intent. Three things were missing:

| Gap | Remedy | Basis |
|---|---|---|
| The builder judged their own page | A separate auditor who never sees the source, the diff or earlier scores | Evaluator effect in heuristic evaluation (Hertzum and Jacobsen); anchoring |
| Nothing measured delight | Four joy dimensions alongside ten usability dimensions | Norman's three levels of emotional design; Walter's hierarchy of user needs; Kano delighters |
| "Better" was a feeling | Scores 0–4 with cited evidence, task outcomes, and metrics such as axe-core nodes and monospace share | Nielsen's heuristics and severity ratings; WCAG 2.2; cognitive walkthrough |

## 3. The rubric

The rubric is in the skill's
[references/rubric.md](../../plugins/toolbox-operator/skills/design-audit/references/rubric.md). Its
dimensions:

| Usability | Joy |
|---|---|
| U1 Answer first · U2 Reader's language · U3 Recognition over recall · U4 Hierarchy and scent · U5 Data integrity · U6 Consistency · U7 Wayfinding and control · U8 Honest states · U9 Accessibility · U10 Density and responsiveness | J1 Visceral · J2 Behavioral · J3 Reflective (progress and pride) · J4 Voice |

Joy never outranks usability: a delighter that hides data, slows a task or fails contrast counts
as a regression. Visual-variety metrics, such as distinct font sizes, hues and SVG count, describe
a page. They are not targets.

## 4. Relation to `report-ux`

`report-ux` stays the design brief for the hosted report: reader jobs, modes, patterns adopted.
`design-audit` is how a change to that page, or any other rendered surface, is judged. A report
change follows `report-ux` and then a `design-audit` pass.

## 5. Limits

- An agent auditor stands in for readers; it does not replace them. Before a release that
  defines a surface, add five real readers on the same tasks, following Nielsen's five-user rule.
- Auditors have variance, which is why the verdict is a panel's median. Even so, treat a
  one-point move on one dimension as noise. A move across many dimensions, or a task going from
  partial to success, is signal.
- Probes catch only what they measure. Contrast inside images, motion, wording and whether a
  task can be done stay with the panel.
- Two reference portals are better than one, but both are repository portals. A central portal
  with many teams is a third reference, not yet built.
- The rubric scores the page as served. Performance on large portals is out of scope.

## Sources

- J. Nielsen, *10 Usability Heuristics for User Interface Design* (1994, updated 2020) and
  *Severity Ratings for Usability Problems*, Nielsen Norman Group.
- J. Nielsen, *Why You Only Need to Test with 5 Users* (2000), Nielsen Norman Group.
- M. Hertzum and N. E. Jacobsen, "The Evaluator Effect", *International Journal of
  Human–Computer Interaction* 13(4), 2001.
- C. Wharton et al., "The Cognitive Walkthrough Method: A Practitioner's Guide" (1994).
- D. Norman, *Emotional Design* (2004); A. Walter, *Designing for Emotion* (2011).
- N. Kano et al., "Attractive quality and must-be quality" (1984).
- S. Few, *Information Dashboard Design* (2nd ed., 2013); E. Tufte, *The Visual Display of
  Quantitative Information* (2001).
- W3C, *Web Content Accessibility Guidelines 2.2* (2023); Deque axe-core 4.10 (capture) and 4.14
  (story probes).
- B. Bach et al., *Dashboard Design Patterns* (arXiv:2205.00757).
- [report-ux.md](./report-ux.md), [maintainer-sdlc.md](./maintainer-sdlc.md),
  [report-ui/README.md](../../report-ui/README.md).
