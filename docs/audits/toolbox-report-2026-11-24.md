# Design audit: the hosted report, 2026-11-24

*Status: record of two blind audits under the `design-audit` skill
([D01M85MK7FCRXH2TRMRKXTNA5GT](../decisions/D01M85MK7FCRXH2TRMRKXTNA5GT.md)). About the box: how
it reports.*

The redesign moved every usability and joy score up and none down. Usability rose from 13 to
29 of 40 and joy from 4 to 11 of 16. Four of the five reader tasks now succeed, against two before, and
serious axe-core nodes fell from 986 to 0. The two findings that misled readers are gone: failing
rules listed as "passing", and a campaign shown as both 0% and met. The second audit's own
severity-3 findings (rule ids without meanings, failure lists not ordered by level, and two
mobile Scorecards faults) are the next cycle's must-fix list, under *Findings still open*.

## Scope

| Item | Value |
|---|---|
| Surface | `<toolbox-report>` as rendered by `toolbox report portal --skip runner,github-actions -o web` for this repo's own portal (1,440 entities, 39 scorecards, 5 campaigns, snapshot 5) |
| Before | `0dcdaae` (main at branch point) |
| After | `c7381fd` (`3da2310` scorecard titles in report JSON, `d924e93` 37 scorecard titles, `635e6bb` element redesign) |
| Routes | `#overview`, `#catalog`, `#catalog/Skill`, `#entity/skill:plugins/toolbox-operator/skills/actions`, `#campaigns`, `#campaign/skill-silver`, `#scorecards`, `#scorecard/maintainer-skill` |
| Viewports | 1280×800 and 390×844, light and dark, plus a full-page desktop light shot |
| Auditors | Two fresh local agents, one per audit. Each saw only that audit's screenshots, `metrics.json`, the tasks and [the rubric](../../plugins/toolbox-operator/skills/design-audit/references/rubric.md): no source, no diff, and the second never saw the first's scores |

The tasks were written before either capture:

| Task | Reader | Question |
|---|---|---|
| T1 | VP of engineering | What level is the repository at, and what holds it there? |
| T2 | Platform owner | Which standard carries the most debt, and what is its most common failure? |
| T3 | Entity owner | What level is the `actions` skill at, what gets it to the next level, and what does it already pass? |
| T4 | Maintainer | Which campaign is furthest behind, and what would close it? |
| T5 | Newcomer or agent | What does `maintainer-skill` measure, and what does silver require? |

## Scores

Scores are 0–4 per the rubric. "Before" is the first auditor's, "after" the second's.

| Id | Before | After | What changed, in the auditors' words |
|---|---|---|---|
| U1 Answer first | 2 | **4** | "bronze" in a 13 px grey caption → medal, the word "Bronze", a one-sentence verdict and the standards holding it, all in the first viewport |
| U2 Reader's language | 1 | **2** | Human scorecard titles; rules, kinds and decision ids still appear raw |
| U3 Recognition over recall | 1 | **3** | No rule or level definitions → "What each level needs", counts with denominators, legends beside data |
| U4 Hierarchy and scent | 2 | **3** | Flat identical cards → a strong hero and question headings; list rows still uniform |
| U5 Data integrity | 1 | **3** | Failing rules listed as passing → proportional bars with counts; failures still all red |
| U6 Consistency | 2 | **3** | Five level renderings → one badge; entity rows still have three layouts |
| U7 Wayfinding and control | 2 | **3** | Silent wrong routes → not-found pages, highlighted selection, breadcrumbs |
| U8 Honest states | 0 | **3** | A campaign at 0% that "meets the goal" → "nothing in scope", "not scored", "✓ clear" |
| U9 Accessibility | 1 | **3** | 986 serious axe nodes → 0 |
| U10 Density and responsiveness | 1 | **2** | 16,857 px catalog → paged; mobile Scorecards rows clip and the picker sits below the list |
| **Usability total** | **13** | **29** | of 40 |
| J1 Visceral | 1 | **3** | One card style everywhere → medal, warm hero, level columns, rings |
| J2 Behavioral | 1 | **2** | Unstyled clickable rows → chevrons, underlined links, sticky sidebar (static evidence only) |
| J3 Reflective | 1 | **3** | Wins looked like debt → "✓ 31 scorecards have no entity at none", "Top level reached", milestone ticks |
| J4 Voice | 1 | **3** | "Inventory browse lives in Catalog…" → "Biggest lever", "Gold means well-formed, not good" |
| **Joy total** | **4** | **11** | of 16 |

| Task | Before | After |
|---|---|---|
| T1 VP: level and what holds it | Success, but the answer was a grey caption | Success in the first viewport |
| T2 Platform owner: most debt and commonest failure | Partial: the rule list was mislabelled and had to be tallied by hand | Success with effort: "Most common failures" answers it, but ranks all levels together |
| T3 Entity owner: level, next step, passes | Success, but the next level was never named | Success: "Next: gold needs has-evals" |
| T4 Maintainer: campaign furthest behind | Partial: it landed on a contradictory campaign | Success in one click, with the biggest lever named |
| T5 Newcomer: what a scorecard measures and what silver needs | Partial: no description and no rules | Partial: description and level matrix, but rules are bare ids |

## Metrics

From the capture script's `metrics.json` (axe-core 4.10.2, desktop light). Visual-variety
counts describe the page; they are not targets.

| Route | Serious axe nodes | Monospace share | Font sizes | SVG elements | Small targets | Page height (px) |
|---|---|---|---|---|---|---|
| overview | 0 → 0 | 31% → 4% | 6 → 11 | 0 → 15 | 1 → 2 | 3,519 → 2,179 |
| catalog | 453 → 0 | 16% → 43% | 6 → 6 | 0 → 1 | 1,359 → 1 | 16,857 → 4,762 |
| catalog/Skill | 40 → 0 | 9% → 55% | 6 → 6 | 0 → 1 | 120 → 0 | 2,195 → 3,584 |
| entity (actions) | 1 → 0 | 6% → 12% | 4 → 9 | 0 → 2 | 3 → 3 | 804 → 800 |
| campaigns | 0 → 0 | 0% → 0% | 5 → 9 | 0 → 7 | 0 → 6 | 800 → 1,077 |
| campaign/skill-silver | 0 → 0 | 0% → 0% | 6 → 9 | 0 → 7 | 5 → 6 | 908 → 1,077 |
| scorecards | 417 → 0 | 8% → 32% | 5 → 5 | 0 → 1 | 440 → 1 | 8,778 → 6,516 |
| scorecard/maintainer-skill | 75 → 0 | 9% → 32% | 5 → 5 | 0 → 1 | 80 → 0 | 2,272 → 3,637 |
| **Total** | **986 → 0** | | | **0 → 35** | **2,008 → 19** | |

Monospace share rose on the list routes because entity rows now show a readable name with the
full id beneath it in monospace, where they used to show only the id inside a clickable row
that axe did not count as text.

## Prioritised usability fixes

Every severity 3–4 finding was a must. The rest were ranked by the number of tasks they block,
then by reach across routes.

| Rank | Finding | Sev | Fix | Status |
|---|---|---|---|---|
| 1 | F1 failing rules listed under "passing" | 4 | Rows say "Fails: …" in fail ink with "passes N of M"; the entity page groups failing rules before passing ones | Fixed |
| 2 | F2 campaign at 0% and "every entity meets the goal" | 4 | An empty scope is its own state ("No entity is in scope…, check its selector"), excluded from "behind target" | Fixed |
| 3 | F3 verdict in a grey caption | 3 | Overview hero: level medal, level as `h2`, one-sentence verdict and the standards that hold it | Fixed |
| 4 | F4 internal shorthand | 3 | Human titles for 37 scorecards in the manifest, carried in report JSON; ids become secondary text | Fixed |
| 5 | F5 scorecard never says what it measures | 3 | Description plus a "What each level needs" matrix (each level also needs the ones below) | Fixed |
| 6 | F6 campaign rows name no rule | 3 | Each entity under a campaign names the rules it needs | Fixed |
| 7 | F7 no tally of failure reasons | 3 | "Most common failures" bar chart per scorecard; a bar filters the entity list to that rule | Fixed |
| 8 | F8 nested interactive rows | 3 | One link per row; `<summary>` rows hold no links | Fixed |
| 9 | F9 gold text at 3.45:1 | 3 | Separate fill and ink colours per level, ink at 4.5:1 or better in both themes; `--pass` darkened to `#21683e` | Fixed |
| 10 | F10 453-row pages | 3 | Lists show the lowest levels first and the first rows, with "Show all N" | Fixed |
| 11 | F12, F13, F32 skipped and level colours | 2 | Skipped takes no bar length and is stated as "N not in scope"; level fills differ in lightness; dark silver distinct from the track | Fixed |
| 12 | F14 non-standard level sets | 2 | Level names outside none/bronze/silver/gold map onto the same ramp by rank | Fixed |
| 13 | F15, F16, F27 inconsistent level rendering | 2 | One level badge everywhere; campaign titles in body font; stray ": " gone | Fixed |
| 14 | F17 invalid routes show Scorecards | 2 | "Not found" state with links back | Fixed |
| 15 | F18, F19 wayfinding | 2 | Selected scorecard highlighted and scrolled into view; entity name as heading and last breadcrumb item | Fixed |
| 16 | F20 "No rules match this filter" on skipped | 2 | "Not in this scorecard's scope." | Fixed |
| 17 | F21 no deadline | 2 | Campaign end date, days left and milestone ticks on the ring | Fixed |
| 18 | F22 entity rules unordered | 2 | "Next: gold needs …" callout, then failing, then passing rules | Fixed |
| 19 | F23, F28, F29, F30 voice and link styling | 2 | Rows styled as links with chevrons; keyboard instructions and implementation notes removed from copy; fleet health split into items; plurals fixed | Fixed |
| 20 | F11 why some scorecards do not count | 2 | The hero says how many repository standards there are and which hold the level | Fixed |
| 21 | F24 mobile kind tiles | 2 | Tabs fit on one row and kind tiles sit two to a row, but the entity list still starts below 15 rows of tiles | Partly fixed |
| 22 | F25 no level on catalog rows | 2 | Each scored row shows its lowest level badge | Fixed |
| 23 | F31 identical cards everywhere | 1 | Hero, campaign rings and level matrix each get their own treatment | Fixed |

## Joy opportunities

| Opportunity | Task | Decision |
|---|---|---|
| Next-level card on the entity page | T3 | Taken: "Next: gold needs has-evals", with earned rules beneath |
| Repository medal with a path to the next level | T1 | Taken: SVG medal and the standards holding the level, linked |
| Completed campaigns and milestone ticks | T4 | Taken: progress rings with milestone ticks, a "Done" state at 100% |
| "One fix, many wins" per scorecard | T2, T5 | Taken as the "Most common failures" chart: `has-evals` fails on 40 of 41 skills |
| Level ladder on the scorecard page | T5 | Taken as the "What each level needs" matrix with the entity distribution beneath |
| Deltas since the last snapshot | T1, T4 | Declined: the report JSON carries one snapshot. It needs the previous baseline in the report contract, which is a separate change |

## Findings still open

The second audit's findings, most severe first. Severity 3 is the next cycle's must-fix list.

| Id | Sev | Finding | Next step |
|---|---|---|---|
| A1 | 3 | Mobile scorecard rows overflow and clip their failure line | Wrap the failure line under the name at 390 px |
| A2 | 3 | On mobile the scorecard and campaign pickers sit below the selected item's full list | Put the picker above the detail on narrow screens |
| A3 | 3 | "What each level needs" lists bare rule ids; no view says what a rule checks | Show each rule's fix text beside its id |
| A4 | 3 | Row failures are alphabetical and all red, hiding the rule that blocks bronze | Order by level, lead with the blocker, mute failures above the next level |
| A5–A8 | 2 | Top failures ignore level; silver rows read as failing; Scorecards renders 222 rows; raw decision ids in descriptions | |
| A9–A10 | 2 | A stray space in a folded description path; Markdown backticks shown raw | |
| A11–A14 | 2 | Totals that do not reconcile (39 vs 44 scorecards, 1,250 vs 1,440 entities); example rules counted as repository debt; "behind target" with no pace; "nothing in scope" unexplained | |
| A15–A23 | 2 | Ring colours, three entity-row layouts, PascalCase kind names, colour-only kind bars, the `#catalog/<kind>` deep link landing above its list, ungrouped passes, scorecard ids missing from the sidebar, repeated monospace ids, small targets | |
| A24 | 2 | No change since the previous snapshot | Needs the previous baseline in the report contract |
| A25–A30 | 1 | Mobile attention table drops bars, "0% silver" caption, internal phrasing, "every rule passes" on a Rule, dark bronze and gold close in lightness, no summary above `#scorecards` | |

## Sources

- Nielsen, J. *10 Usability Heuristics for User Interface Design* (1994, updated 2024), and
  *Severity Ratings for Usability Problems*: Nielsen Norman Group.
- Wharton, C., Rieman, J., Lewis, C. and Polson, P. *The Cognitive Walkthrough Method: A
  Practitioner's Guide* (1994).
- Norman, D. *Emotional Design* (2004); Walter, A. *Designing for Emotion* (2011); Kano, N. et
  al. *Attractive Quality and Must-Be Quality* (1984).
- W3C, *Web Content Accessibility Guidelines (WCAG) 2.2* (2023); Deque, `axe-core` 4.10.2.
- The before and after evidence (screenshots, `metrics.json`) was captured with
  `plugins/toolbox-operator/skills/design-audit/scripts/capture.cjs`; it is not committed.
