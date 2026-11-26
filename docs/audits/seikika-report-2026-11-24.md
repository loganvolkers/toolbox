# Design audit: the hosted report, 2026-11-24

*Status: record of four blind audits and three fix cycles under the `design-audit` skill
([D01M85MK7FCRXH2TRMRKXTNA5GT](../decisions/D01M85MK7FCRXH2TRMRKXTNA5GT.md)). About the box: how
it reports.*

Three fix cycles took usability from 13 to 31 of 40 and joy from 4 to 12 of 16, with every
score at 3 or above. All five reader tasks now succeed, against two before, and serious axe-core
nodes fell from 986 to 0. The fourth auditor found no severity 3 or 4 finding. The second cycle
was not done when audited: usability fell from 29 to 27, one fall a regression it caused and two
findings the previous auditor missed. The third cycle fixed all three, and no usability score in
audit 4 is below its audit-2 value. What is left is trust and fluency: counts that differ between
views without saying why, no generation date, raw rule ids, and no momentum since the last
snapshot (*Findings still open*).

## Scope

| Item | Value |
|---|---|
| Surface | `<seikika-report>` as rendered by `seikika report portal --skip runner,github-actions -o web` for this repo's own portal |
| Content | Audits 1 and 2: 1,440 entities, 39 scorecards, 5 campaigns. Audits 3 and 4 ran after a rebase onto `main`, which had grown to 1,566 entities and 48 scorecards and added a Compare tab |
| Audit 1 (before) | `0dcdaae`, `main` at the branch point |
| Audit 2 (cycle 1) | `8ff6862` scorecard titles in report JSON, `5e3e5af` titles for every scorecard, `806a896` element redesign |
| Audit 3 (cycle 2) | `cc6b6c5` rule meanings, failures by level, mobile pickers, the lever; `68173d9` whole paths in folded descriptions; `cf00494` Compare ported to the redesign |
| Audit 4 (cycle 3) | `c2ca288` unscored entities, unmeasurable campaigns, drill-down focus, state before definitions, the level leading the entity page |
| Routes | `#overview`, `#catalog`, `#catalog/Skill`, `#entity/skill:plugins/seikika-operator/skills/actions`, `#campaigns`, `#campaign/skill-silver`, `#scorecards`, `#scorecard/maintainer-skill`; audits 3 and 4 also took their own extra screenshots |
| Viewports | 1280×800 and 390×844, light and dark, plus a full-page desktop light shot |
| Auditors | Four fresh local agents, one per audit. Each saw only its own capture, `metrics.json`, the tasks and [the rubric](../../plugins/seikika-operator/skills/design-audit/references/rubric.md): no source, no diff and no earlier scores |

The tasks were written before the first capture and kept for every audit:

| Task | Reader | Question |
|---|---|---|
| T1 | VP of engineering | What level is the repository at, and what holds it there? |
| T2 | Platform owner | Which standard carries the most debt, and what is its most common failure? |
| T3 | Entity owner | What level is the `actions` skill at, what gets it to the next level, and what does it already pass? |
| T4 | Maintainer | Which campaign is furthest behind, and what would close it? |
| T5 | Newcomer or agent | What does `maintainer-skill` measure, and what does silver require? |

## Scores

Scores are 0–4 per the rubric, one auditor per column.

| Id | Audit 1 | Audit 2 | Audit 3 | Audit 4 | What moved them, in the auditors' words |
|---|---|---|---|---|---|
| U1 Answer first | 2 | 4 | 3 | 4 | A grey "bronze" caption became a medal hero. Audit 3 marked it down because the level matrix pushed "Where entities stand" below the fold and the entity page led with a 92% ring |
| U2 Reader's language | 1 | 2 | 2 | 3 | Human scorecard titles; rule ids, kind names and decision ids are still raw |
| U3 Recognition over recall | 1 | 3 | 3 | 3 | Level definitions, denominators and legends beside the data |
| U4 Hierarchy and scent | 2 | 3 | 3 | 3 | A strong hero and question headings over flat identical cards |
| U5 Data integrity | 1 | 3 | 2 | 3 | Failing rules listed as passing were fixed in cycle 1. Audit 3 found kind bars that left unscored entities out; cycle 3 hatched them in |
| U6 Consistency | 2 | 3 | 3 | 3 | One level badge everywhere |
| U7 Wayfinding and control | 2 | 3 | 3 | 3 | Not-found pages, breadcrumbs, highlighted selection |
| U8 Honest states | 0 | 3 | 2 | 3 | A campaign at 0% that "met the goal" was fixed in cycle 1. Audit 3 found a campaign whose empty scope hid 14 entities at none; audit 4 called its "can't be measured" state better than most dashboards |
| U9 Accessibility | 1 | 3 | 3 | 3 | 986 serious axe-core nodes became 0 |
| U10 Density and responsiveness | 1 | 2 | 3 | 3 | Paged lists; cycle 2 fixed the clipped mobile rows and put a picker above the detail |
| **Usability total** | **13** | **29** | **27** | **31** | of 40 |
| J1 Visceral | 1 | 3 | 3 | 3 | Medal, warm hero, level columns, rings |
| J2 Behavioral | 1 | 2 | 2 | 3 | Chevrons, underlined links, a sticky sidebar; audit 4 saw hover and in-place expansion on the live page |
| J3 Reflective | 1 | 3 | 3 | 3 | Wins celebrated: "✓ 31 scorecards have no entity at none", passes grouped by level |
| J4 Voice | 1 | 3 | 3 | 3 | "Biggest lever", "Gold means well-formed, not good" |
| **Joy total** | **4** | **11** | **11** | **12** | of 16 |

| Task | Audit 1 | Audit 2 | Audit 3 | Audit 4 |
|---|---|---|---|---|
| T1 VP: level and what holds it | Success, but the answer was a grey caption | Success in the first viewport | Success | Success in the first viewport |
| T2 Platform owner: most debt and commonest failure | Partial | Success with effort: failures ranked across levels | Success with effort | Success in one click: debt, commonest failure and lever on one page |
| T3 Entity owner: level, next step, passes | Success, but the next level was never named | Success | Success | Success in three clicks |
| T4 Maintainer: campaign furthest behind | Partial: a contradictory campaign | Success in one click | Success | Success in one click; no pace marker |
| T5 Newcomer: what a scorecard measures and what silver needs | Partial: no description, no rules | Partial: rules were bare ids | Success | Success |

### Why audit 3 scored lower than audit 2

The skill's done rule is that no usability score falls, so cycle 2 did not pass it on audit 3,
even though every task succeeded. Each fall was traced to the evidence the auditor cited:

| Score | Cause | Kind |
|---|---|---|
| U1 4 → 3 | Cycle 2 put the level matrix, now with each rule's meaning, above the distribution: 378 and 440 words above the fold on the scorecard pages, against 187 and 184 before | Regression from cycle 2 |
| U5 3 → 2 | The Doc kind bar was all silver although half the docs were unscored | Missed by audit 2; the view did not change in cycle 2 |
| U8 3 → 2 | A campaign whose scorecards rate different kinds showed "scope is empty" while 14 entities sat at none; totals did not reconcile | Missed by audit 2; the view did not change in cycle 2 |

Both kinds were fixed in cycle 3 and the report was audited again. The skill now says so in
step 6: on a view the change touched a fall is a regression, on a view it did not it is a missed
finding, and auditors are never averaged away.

## Metrics

From the capture script's `metrics.json` (axe-core 4.10.2, desktop light), one value per
capture: before, cycle 1, cycle 2, cycle 3. Visual-variety counts describe the page; they are not
targets. Heights after cycle 2 also reflect `main`'s growth (48 scorecards against 39).

| Route | Serious axe nodes | Words above the fold | Monospace share | Page height (px) |
|---|---|---|---|---|
| overview | 0 · 0 · 0 · 0 | — · 190 · 192 · 202 | 31% · 4% · 4% · 7% | 3,519 · 2,179 · 2,179 · 2,443 |
| catalog | 453 · 0 · 0 · 0 | — · 152 · 152 · 159 | 16% · 43% · 43% · 43% | 16,857 · 4,762 · 4,762 · 4,875 |
| catalog/Skill | 40 · 0 · 0 · 0 | — · 152 · 152 · **42** | 9% · 55% · 55% · 54% | 2,195 · 3,584 · 3,584 · 3,697 |
| entity (actions) | 1 · 0 · 0 · 0 | — · 110 · 122 · 127 | 6% · 12% · 11% · 11% | 804 · 800 · 800 · 800 |
| campaigns | 0 · 0 · 0 · 0 | — · 189 · 191 · 195 | 0% · 0% · 0% · 0% | 800 · 1,077 · 1,077 · 1,077 |
| campaign/skill-silver | 0 · 0 · 0 · 0 | — · 189 · 191 · 195 | 0% · 0% · 0% · 0% | 908 · 1,077 · 1,077 · 1,077 |
| scorecards | 417 · 0 · 0 · 0 | — · 187 · 378 · **228** | 8% · 32% · 29% · 29% | 8,778 · 6,516 · 7,111 · 7,113 |
| scorecard/maintainer-skill | 75 · 0 · 0 · 0 | — · 184 · 440 · **314** | 9% · 32% · 24% · 23% | 2,272 · 3,637 · 4,312 · 4,336 |
| **Total** | **986 · 0 · 0 · 0** | | | |

The first capture predates the fold-word count. `catalog/Skill` fell to 42 words above the fold
because the drill-down now scrolls to the kind's list and focuses it. Monospace share rose on the
list routes in cycle 1 because rows show a readable name with the full id beneath it, where they
used to show only the id inside a row axe did not count as text.

## Cycle 1: audit 1's findings

Every severity 3–4 finding was a must. The rest were ranked by the number of tasks they block,
then by reach across routes.

| Rank | Finding | Sev | Fix | Status |
|---|---|---|---|---|
| 1 | F1 failing rules listed under "passing" | 4 | Rows say "Fails: …" in fail ink with "passes N of M"; the entity page groups failing rules before passing ones | Fixed |
| 2 | F2 campaign at 0% and "every entity meets the goal" | 4 | An empty scope is its own state, excluded from "behind target" (reworded again in cycle 3) | Fixed |
| 3 | F3 verdict in a grey caption | 3 | Overview hero: level medal, level as `h2`, one-sentence verdict and the standards that hold it | Fixed |
| 4 | F4 internal shorthand | 3 | Human titles for every scorecard in the manifest, carried in report JSON; ids become secondary text | Fixed |
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
| 21 | F24 mobile kind tiles | 2 | Tabs fit on one row and kind tiles sit two to a row | Partly fixed; finished in cycle 3 |
| 22 | F25 no level on catalog rows | 2 | Each scored row shows its lowest level badge | Fixed |
| 23 | F31 identical cards everywhere | 1 | Hero, campaign rings and level matrix each get their own treatment | Fixed |

## Cycle 2: audit 2's must-fix list

Audit 2's four severity-3 findings, plus the severity-2 findings that shared their views.

| Finding | Sev | Fix |
|---|---|---|
| A1 mobile scorecard rows clip their failure line | 3 | The tally wraps under the name at 390 px |
| A2 mobile pickers sit below the selected item's full list | 3 | Below 820 px a select sits above the detail and the side list is hidden |
| A3 "What each level needs" lists bare rule ids | 3 | Each rule's fix text sits beside its id, and passes are grouped by level ("all N met") |
| A4 row failures alphabetical and all red | 3 | Rows lead with what the next level needs ("Silver needs has-example"), then "N more for higher levels"; red only at none |
| A5 top failures ignore level | 2 | "Biggest lever" names the rule holding the most entities at the lowest occupied level |
| A9 a stray space in a folded description path | 2 | Folded descriptions keep paths whole in four standards files |
| A10 Markdown backticks shown raw | 2 | Backticks render as code |
| A21 scorecard ids missing from the sidebar search | 2 | The search matches ids as well as titles |
| A27 internal phrasing | 1 | "Most entities at none first", "owned by maintainers" |

## Cycle 3: audit 3's must-fix list

Audit 3's three severity-3 findings, the U1 regression, and the severity-2 findings on the same
views.

| Finding | Sev | Fix |
|---|---|---|
| F1 kind bars hide unscored entities; the Doc bar was all silver | 3 | A hatched "not scored" segment in every kind bar and legend, "N scored · M not scored" in the kind header, and "not scored" on rows without a scorecard |
| F2 a campaign whose scorecards rate different kinds says "scope is empty" | 3 | It says "can't be measured" in a warning pill, explains that no entity is rated by all its scorecards, lists each scorecard's open work (entities below the goal, of how many), says how to measure it, and counts in the overview's needs-attention list |
| F3 `#catalog/<kind>` lands above its list | 3 | The drill-down scrolls to the kind's list and focuses its heading |
| F8 rule definitions above the distribution (the U1 regression) | 2 | Scorecard pages show where entities stand, then the lever, then the failures, then what each level needs |
| F9 the entity page leads with a 92% ring | 2 | It leads with the level's medal and word; the check ratio is secondary |
| F6 gold failures shown in alarm red | 2 | Only first-level failures are red; the rest are neutral |
| F5 the lever and the commonest failure disagree without saying why | 2 | The lever reads "the rule that lifts the most entities a level" and is marked in the failure list, now headed "at any level" |
| F17 catalog kind cards have no legend | 2 | A level legend, including "not scored", under the catalog intro |
| F4 totals do not reconcile | 2 | Partly fixed: the header says "1,566 entities in the box (1,370 scored)" |

## Joy opportunities

| Opportunity | Task | Decision |
|---|---|---|
| Next-level card on the entity page | T3 | Taken in cycle 1: "Next: gold needs has-evals", with earned rules beneath; cycle 3 put the level medal first |
| Repository medal with a path to the next level | T1 | Taken in cycle 1: SVG medal and the standards holding the level, linked |
| Completed campaigns and milestone ticks | T4 | Taken in cycle 1: progress rings with milestone ticks, a "Done" state at 100% |
| "One fix, many wins" per scorecard | T2, T5 | Taken: "Most common failures" (cycle 1) and the biggest lever (cycle 2), marked in the list (cycle 3) |
| Level ladder on the scorecard page | T5 | Taken: the "What each level needs" matrix, with each rule's meaning since cycle 2 |
| Passes celebrated by level | T3 | Taken in cycle 2: "all N met" per level |
| Deltas since the last snapshot | T1, T4 | Declined: the report JSON carries one snapshot. It needs the previous baseline in the report contract, which is a separate change |

## Findings still open

Audit 4's findings, none above severity 2. They are the next cycle's candidates; momentum and the
generation date need the report contract to carry the previous snapshot and a timestamp.

| Id | Sev | Finding | Next step |
|---|---|---|---|
| F01 | 2 | Overview attention bars are all full width, so a 1-entity scorecard looks as alarming as a 270-entity one | Show the entity count beside each bar or scale length by it; group the product comparisons |
| F02 | 2 | "Biggest lever" and "Most common failures" still read as two answers; the bar colours have no legend | One line that relates them, and a legend for red and neutral bars |
| F03 | 2 | Campaigns show no pace: nothing says where a campaign should be today; the half-way tick is unexplained | An expected-today marker, a status word, a labelled milestone |
| F04, F23 | 2 | No generation date; "snapshot 5" is unexplained | Carry the generation time and commit in the report JSON and show them in the header |
| F05 | 2 | Counts differ between views (8 workflows in the catalog, 7 rated; 53 scorecard entities, 48 scorecards) | Say when a scorecard rates a subset, and why |
| F06 | 2 | Red "none" and bronze are hard to tell apart for colour-vision-deficient readers | A texture or separator on the none segment, counts on focus |
| F07, F16 | 2 | A kind deep link scrolls the tabs away; an entity reached from a campaign loses that context | Sticky tabs or a breadcrumb on the list; a back link to the campaign |
| F08 | 2 | No global entity search | A header search over names and ids |
| F09, F10 | 2 | Decision ids in descriptions; rules shown only by kebab-case id | A sources disclosure; human rule titles, as scorecards have |
| F11 | 2 | Catalog lists are not sorted lowest level first | The scorecard lists' sort and level filter |
| F12, F13 | 2 | Mobile hides campaign status in the select, and the tab bar clips "Compare" without a cue | Status in the option labels; an edge fade or wrapped tabs |
| F14 | 2 | An expanded entity row is not in the URL | Write it into the hash |
| F15 | 2 | Six small targets on Campaigns, three on the entity page | 24 px hit areas |
| F17–F22 | 1 | Mixed top-level captions, a blue ring beside a red pill, the sidebar sort label, missing "no deadline", silver close to the track grey, one sentence repeated per entity | |

## Sources

- Nielsen, J. *10 Usability Heuristics for User Interface Design* (1994, updated 2024), and
  *Severity Ratings for Usability Problems*: Nielsen Norman Group.
- Wharton, C., Rieman, J., Lewis, C. and Polson, P. *The Cognitive Walkthrough Method: A
  Practitioner's Guide* (1994).
- Norman, D. *Emotional Design* (2004); Walter, A. *Designing for Emotion* (2011); Kano, N. et
  al. *Attractive Quality and Must-Be Quality* (1984).
- W3C, *Web Content Accessibility Guidelines (WCAG) 2.2* (2023); Deque, `axe-core` 4.10.2.
- The evidence for each audit (screenshots, `metrics.json`) was captured with
  `plugins/seikika-operator/skills/design-audit/scripts/capture.cjs`; it is not committed.
