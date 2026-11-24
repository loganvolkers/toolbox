# Design Audits: Independent, Rubric-Scored Reviews of Rendered UI

*Status: decided 2026-11-24 (D01M85MK7FCRXH2TRMRKXTNA5GT). The agent skill is
[`design-audit`](../../plugins/toolbox-operator/skills/design-audit/SKILL.md); the first record,
four audits and three fix cycles of the hosted report, is
[toolbox-report-2026-11-24.md](../audits/toolbox-report-2026-11-24.md).*

**About:** the box. It is how the maintainer SDLC judges what the box shows readers: the hosted
report, the minimal UI and any custom view. It wraps no tool. Playwright and axe-core are used
as they ship.

---

## 1. Conclusion

A UI change is judged by an **independent auditor who sees only rendered evidence**, against a
fixed rubric and named reader tasks. The person or agent who made the change fixes but never
scores. A change is done when a **second, blind audit** shows that:

- no usability score fell;
- the targeted scores rose;
- serious axe-core violations did not grow.

When a score falls, its cited evidence says why: on a view the change touched it is a regression,
on a view it did not it is a finding the last auditor missed. Both are fixed and the page is
audited again, by another new auditor; scores are never averaged across auditors. The first
record needed this: its third audit fell from 29 to 27 of 40, and the fourth, after a cycle that
fixed both kinds, scored 31.

| Step | Who | Output |
|---|---|---|
| Name readers and tasks | Implementer, before looking | 3–6 tasks with what counts as an answer |
| Capture | Script | Screenshots (desktop and mobile, light and dark) and `metrics.json` |
| Audit | Fresh auditor, blind | Task outcomes, U1–U10 and J1–J4 scores, ranked findings, joy ideas |
| Prioritise and fix | Implementer | Severity 3–4 findings first, then joy that reinforces a task |
| Re-audit | A new auditor each time, blind to earlier scores | Another score table, until the done rule holds |
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
- One auditor per round has variance. Treat a one-point move on one dimension as noise. A move
  across many dimensions, or a task going from partial to success, is signal.
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
- W3C, *Web Content Accessibility Guidelines 2.2* (2023); Deque axe-core 4.10.
- B. Bach et al., *Dashboard Design Patterns* (arXiv:2205.00757).
- [report-ux.md](./report-ux.md), [maintainer-sdlc.md](./maintainer-sdlc.md).
