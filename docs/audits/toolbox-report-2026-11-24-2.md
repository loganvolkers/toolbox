# Design audit: tracks, kind groups, rationale and judged results, 2026-11-24

*Status: record of a second design loop on the hosted report under the `design-audit` skill
([D01M868BY0KJNEYE9WBFZYEYFYH](../decisions/D01M868BY0KJNEYE9WBFZYEYFYH.md)). About the box: how
it reports what the tools found.*

The loop is in progress: the baseline and round 1 are recorded below, and round 2 follows.

## Scope

| Item | Value |
|---|---|
| Surface | `<toolbox-report>` as rendered by `toolbox report portal --skip runner,github-actions -o web` for this repo's own portal |
| Goals | Tracks easy to see, understand and explore; kinds grouped, with a sensible grouping for this portal; a comparison understood without its docs; a rationale and links on every rule and scorecard; measured and agent-judged results told apart wherever a result is shown |
| Routes | `#overview`, `#tracks`, `#track/delivery`, `#catalog`, `#catalog/Crate`, `#campaigns`, `#scorecards`, `#scorecard/maintainer-skill`, `#scorecard/review-crate`, `#rule/maintainer-skill/has-example`, `#entity/crate:toolbox-cli`, `#compare`, `#compare/vs-backstage-capabilities`, `#compare/vs-backstage-performance` |
| Viewports | 1280×800 and 390×844, light and dark, plus a full-page desktop light shot |
| Auditors | A fresh local agent per audit. Each saw only its own capture, `metrics.json`, the tasks and [the rubric](../../plugins/toolbox-operator/skills/design-audit/references/rubric.md): no source, no diff and no earlier scores |

The tasks were written before the first capture and kept for every audit:

| Task | Reader | Question |
|---|---|---|
| T1 | New maintainer | What long-term tracks does this portal pursue, and which one is furthest from its goal? Name one scorecard in that track and its weakest level. |
| T2 | Platform lead, without reading any docs | Is Toolbox ahead of Backstage overall, how was it measured (versions, corpus, runs, who judged it), and where is it behind and why? |
| T3 | Contributor | Why does the rule `has-example` on Agent skills exist, beyond how to fix it? Find a link to read more. |
| T4 | Reviewer | On the "Crates, reviewed by agents" scorecard and the toolbox-cli crate page, which rules depend on an agent's judgement rather than a mechanical check? |
| T5 | Newcomer | Which kinds of thing describe documentation, and how many documentation entities are there? |
| T6 | VP (regression guard from the first loop) | What level is the repository at, and what holds it there? |

## Scores

| Id | Baseline | Round 1 |
|---|---|---|
| U1 Answer first | 3 | 4 |
| U2 Reader's language | 2 | 3 |
| U3 Recognition over recall | 2 | 3 |
| U4 Hierarchy and scent | 3 | 3 |
| U5 Data integrity | 3 | 3 |
| U6 Consistency | 2 | 3 |
| U7 Wayfinding and control | 2 | 3 |
| U8 Honest states | 1 | 3 |
| U9 Accessibility | 3 | 3 |
| U10 Density and responsiveness | 2 | 3 |
| **Usability total** | **23** | **31** |
| J1 Visceral | 3 | 3 |
| J2 Behavioral | 2 | 2 |
| J3 Reflective | 3 | 3 |
| J4 Voice | 2 | 3 |
| **Joy total** | **10** | **11** |
| Tasks succeeded | 1 of 6 (4 partial, 1 fail) | 6 of 6 |
| Serious axe-core nodes | 0 | 0 |

## Sources

- [design-audit.md](../design/design-audit.md), the skill
  [`design-audit`](../../plugins/toolbox-operator/skills/design-audit/SKILL.md) and its rubric.
- [report-ux.md](../design/report-ux.md); the first loop,
  [toolbox-report-2026-11-24.md](./toolbox-report-2026-11-24.md).
