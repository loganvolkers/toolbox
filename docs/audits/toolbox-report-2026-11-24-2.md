# Design audit: tracks, kind groups, rationale and judged results, 2026-11-24

*Status: record of a second design loop on the hosted report under the `design-audit` skill
([D01M868BY0KJNEYE9WBFZYEYFYH](../decisions/D01M868BY0KJNEYE9WBFZYEYFYH.md)). About the box: how
it reports what the tools found.*

ROUND_SUMMARY

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

| Id | Baseline | Round 1 | Round 2 | Round 3 | Round 4 | Round 5 | Round 6 | Round 7 | Round 8 | Round 9 | Round 10 | Round 11 | Round 12 | Round 13 | Round 14 | Round 15 | Round 16 | Round 17 | Round 18 | Round 19 | Round 20 | Round 21 | Round 22 | Round 23 | Round 24 | Round 25 | Round 26 | Round 27 | Round 28 | Round 29 | Round 30 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| U1 Answer first | 3 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| U2 Reader's language | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U3 Recognition over recall | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 4 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 |
| U4 Hierarchy and scent | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U5 Data integrity | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 |
| U6 Consistency | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 4 | 4 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U7 Wayfinding and control | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 4 | 4 | 3 | 4 | 4 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 3 | 3 | 4 | 4 | 3 | 4 | 4 | 4 |
| U8 Honest states | 1 | 3 | 2 | 3 | 3 | 3 | 3 | 4 | 3 | 2 | 3 | 3 | 4 | 4 | 4 | 3 | 3 | 4 | 4 | 3 | 3 | 4 | 4 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| U9 Accessibility | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 4 | 4 | 4 | 4 | 4 | 4 | 3 | 4 | 4 | 3 | 4 | 3 | 4 | 3 | 4 | 4 | 3 |
| U10 Density and responsiveness | 2 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 4 | 4 | 4 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 4 | 4 | 4 |
| **Usability total** | **23** | **31** | **30** | **31** | **31** | **31** | **31** | **33** | **32** | **30** | **31** | **32** | **33** | **34** | **35** | **33** | **37** | **34** | **37** | **34** | **34** | **36** | **35** | **30** | **33** | **34** | **34** | **32** | **36** | **36** | **34** |
| J1 Visceral | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| J2 Behavioral | 2 | 2 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| J3 Reflective | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 4 | 4 | 4 | 4 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| J4 Voice | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| **Joy total** | **10** | **11** | **11** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **13** | **13** | **12** | **13** | **13** | **14** | **14** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** | **12** |
| Tasks succeeded | 1 of 6 (4 partial, 1 fail) | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 | 6 of 6 |
| Serious axe-core nodes | 0 | 0 | 0 | 0 (5 in the capture, from an entry fade; see round 3) | 0 | 0 | 0 | 0 (WCAG 2.2 `target-size` found by the auditor; see round 7) | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included | 0, WCAG 2.2 included |

## Round 1: the baseline's findings

The baseline answered one task of six. Tracks did not exist as a page, kinds were an ungrouped list
of PascalCase ids, the comparison never said how it was measured, and an agent-judged result looked
exactly like a measured one. The model gained kind groups, a rationale and links on every rule and
scorecard, and a basis (measured or agent-judged) computed for every rule; the report gained a Tracks
page and track detail, catalog groups, rule pages, Measured and Agent-judged badges, and a compare
view that states its setup and publishes its losses. Round 1 answered all six tasks.

## Round 2: one regression, three severity-3 findings

Round 2 scored U8 at 2, below round 1, so the loop did not pass.

| Finding | Fix |
|---|---|
| F1 (sev 3). A track page marked a different track "Furthest from goal" than the Tracks page did | The furthest track is ranked over all tracks, on every page that marks it |
| F2 (sev 3). Clicking a failing rule filtered on its name as text, so entities that pass it were listed | A rule filter, shown as a removable chip, lists only the entities failing that rule |
| F3 (sev 3). An agent-judged rule passed on entities no agent had assessed, beside siblings saying "no current assessment" | The report JSON gains `unassessed`; such a pass reads "passes, not assessed yet", and the scorecard says how many entities no agent has assessed |
| F4. Filters were lost on reload | Level, outcome, rule and text filters live in the hash (`#scorecard/x?level=silver&rule=has-example`) |
| F5. Arrow keys between tabs dropped focus | Focus follows the selected tab |
| F7. Bar captions used kind ids ("15 ActionDependency") | Plural kind titles ("15 action dependencies") |
| F8. "Ahead" was undefined on Compare | The hero defines ahead, behind and parity per rule and per factor |
| F9. Counts differed between pages without a reason | Captions give both ("8 workflows, 7 in scope") |
| F11. Nothing moved | Short transitions on hover, filters and view changes, off under reduced motion |
| F13. "Silver needs assessed." | "Silver needs:" followed by rule links with their basis icons |

## Round 3: the loop passes

A new auditor found every task successful and no severity 3 or 4 finding. No score fell below
round 1, and Q1 to Q5 were all yes, Q4 and Q5 with caveats. Its severity 1 and 2 findings were then
fixed where they were cheap and clear:

| Finding | Fix |
|---|---|
| F1. A literal "null" after the outcome filter | An empty slot renders nothing |
| F2. Catalog columns showed an unassessed agent-judged level like a measured one | Agent-judged column headers carry the ✦ icon, and unassessed cells say "not assessed" |
| F4. "0 all clear" in green beside "0% gold" | The column reads "at none", red only above zero |
| F9 (Q4 caveat). No rule seemed to have its own link | 69 of 260 rules do; the scorecard's "What each level needs" list now shows them beside the rationale, not only on rule pages |
| F10. The weakest level was plain mono text | A level badge |
| F11. The rule breadcrumb lacked its root and name | "Scorecards / Agent skills / has-example" |
| F12. Five serious contrast nodes on Campaigns | They came from the capture, not the page: the full-page shot replayed an entry fade. The capture script now settles animations inside shadow roots and takes the full-page shot with reduced motion; axe is 0 on every route |
| F5. "harness 5bd608d" was unexplained | The Product kind's field descriptions, shown in the setup, say what the version and the harness are |

## Round 4: U1 falls, so the loop goes round again

A new auditor, on the state after round 3's fixes, found every task successful, no severity 3 or 4
finding, Q1 to Q5 all yes, and axe at 0 on all 14 routes; U10 rose to 4. But U1 fell from 4 to 3,
citing the Tracks page, which opened with a six-line legend before any card, so by the skill's
rule the change was not done. That finding (F8) and the caveats on the goals were fixed:

| Finding | Fix |
|---|---|
| F1. The review-crate notice said its judged rules "pass only because there is nothing to fail on", above a list where four of the five fail | The notice names which judged rules pass for want of anything to read and which fail until an assessment exists |
| F2. The Against Backstage ring reused the progress ring's red for a different measure | A comparison track gets a segmented ring, one segment per factor coloured ahead, at parity or behind, and a neutral card |
| F3. Track cards announced concatenated text ("Furthest from goal13%Proving value…") | Each card has a short label: title, standing, flag and weakest scorecard |
| F4 (Q5 caveat). Compact basis icons had only a hover title, and the ✦ vanished in the mobile catalog | Icon-only badges carry hidden text that joins their row's accessible name; the catalog explains the marker above the table; mobile cards keep the ✦ |
| F8 (the U1 fall). The Tracks page opened with a six-line paragraph of colour thresholds | One sentence, then a key of swatches for the ring and the segmented ring |
| F12. An unassessed "Bronze" looked earned | It is drawn as a dashed pill beside "not assessed" |
| Q5 beyond the page | The markdown report, which the GitHub Action posts on pull requests, and the ratchet's gate summary mark agent-judged rules with ✦, count them in each scorecard heading, and print a legend line |

## Round 5: U10 falls, so the loop goes round again

A new auditor, on the state after round 4's fixes, found every task successful, no severity 3 or 4
finding, Q1 to Q5 all yes, and axe at 0; U1 was back at 4. But U10 fell from 4 to 3, citing long
pages, the full track grid repeated under "Other tracks" on every track page, and the empty "None"
column on mobile, so the loop went round again. Those findings and the other severity 2 ones were
fixed, with the cheap severity 1 ones:

| Finding | Fix |
|---|---|
| F1. An unassessed agent-judged level was dashed in the catalog but solid on its scorecard and on the entity page, whose "Provisional" line ignored it | One pill everywhere: dashed with "not assessed" in catalog cells, scorecard rows and the entity's scorecard headers; the entity hero counts such scorecards beside the checks not run |
| F2. The mobile tab bar clipped Scorecards and hid Compare | The tabs wrap to a second row on narrow screens |
| F3 (the U10 fall). Track pages repeated the whole track grid, with the comparison's long description | "Other tracks" is a compact strip of small rings and titles; card descriptions are clamped to three lines |
| F4. Ids as reader text: "D71 one meta level", "Factors F3 F4 F5 F14", "(lesson L5)", and "install facts from v0.1.1" | 41 links are titled by their claim ("Decision: judgement is pushed", "Factor definitions: installability and footprint"); parentheticals that only cite ids or doc paths are dropped from prose; the version field says the install facts come from the v0.1.1 release download. Four titles keep their id because an assessment pins the scorecard's text |
| F5. The scorecard sidebar scrolled with no visible scrollbar | A thin scrollbar and fades at the edges that have more |
| F6. The Against Backstage card said "behind on 1" | "behind on Capabilities" |
| F7. "Most common failures" bars were red or grey with no key | Each bar takes the colour of the level its rule is needed for, and one line says so |
| F8. The ring key named thresholds | Ranges: "80–100% · 50–79% · 25–49% · below 25%" |
| F10. A kind card's count hid its unscored entities | "297 entities · 149 not scored" |
| F11. Kind cards showed a CamelCase id on some cards only | The id is shown once, under the selected kind's heading |
| F14. The empty "None" column took half the mobile width | An empty level is a single line on narrow screens |
| Earlier rounds | The catalog filter is kept in the URL (`#catalog/Crate?q=cli`), each page sets the tab title ("Agent skills · Toolbox for Toolbox"), and Back restores the scroll position |

## Round 6: U10 stays at 3, so the loop goes round again

A new auditor, on the state after round 5's fixes, found every task successful, no severity 3 or 4
finding, Q1 to Q5 all yes, and axe at 0. No score fell, but U10, the score round 5 lost, did not
come back to 4: the entity page scrolled sideways at 390 px. So the loop went round again, and the
capture now records `mobileOverflow` per route, so a page that scrolls sideways is a number in
`metrics.json`, not something an auditor has to notice.

| Finding | Fix |
|---|---|
| F1 (the U10 hold). `#entity/crate:toolbox-cli` was 540 px wide at 390 | The entity's level block may shrink and wrap; every route is 0 px over |
| F2. Two standards sat in the Silver column although Silver is their top level, and nothing said whether the repository could reach gold | A standard at its top level is marked "✓ top level". When one caps the repository, the hero says so ("Capped at bronze: Third-party actions pinned has no level above bronze …") and lists the rest of the work |
| F3. The catalog showed a solid Bronze that the entity page called provisional | A level whose checks did not run is dashed with "provisional" wherever it is shown, like "not assessed" |
| F4. Compare's "How it is scored" showed a bare expression and Toolbox's to-do | It says in words what the harness recorded for each product and which value is ahead, then the expression, then "What would close it" |
| F5. Scorecard lists on the overview and tracks had no Agent-judged mark | Every scorecard list marks a scorecard with agent-judged rules, with how many on hover |
| F6. Row badges were a bare, hidden glyph | Compact badges show a word ("measured", "judged") beside the icon |
| F7. Back after opening a rule returned to the top | The scroll position is saved before the click navigates |
| F8. `#catalog` preselected a kind and a kind's list hid the way back | No kind is preselected; a kind's list starts with "Catalog / Code / Crate" |
| F9. The ring ramp mixed blue into a progress scale | One ordered ramp from red through bronze and olive to green |
| F10. The Tracks tab repeated the overview's track cards | The overview shows a strip of rings, furthest first; the full cards are on the Tracks tab |
| F11. Campaign rows read "Agent skills is bronze; the goal is silver. Needs has-example." | "Needs has-example for silver on Agent skills; bronze now." |
| F12. The compare hero named no versions | The Measured line names each differing version |
| F13. Link titles such as "D71 one meta level", and long ids wrapping over three lines | A title that leads with a decision id renders as "Decision: one meta level", with the id on hover; ids in rows are cut to one line with the full id on hover |
| F14. Setup cards stretched to the tallest | Cards align to the start |

## Round 7: U10 stays at 3 again, so the loop goes round again

A new auditor, on the state after round 6's fixes, found every task successful, no severity 3 or 4
finding and Q1 to Q5 all yes; U7 and U8 rose to 4. U10 stayed at 3, citing scorecard pages of
3,224 to 3,947 px and evidence repeated under two rules. The auditor also ran axe with WCAG 2.2
tags and found `target-size` violations the capture had not counted, because axe leaves 2.2 rules
out unless their tag is asked for. The capture now asks for them, so the next round's metrics
count them.

| Finding | Fix |
|---|---|
| F1. "✓ top level" on the overview, but "weakest Bronze" with no mark on a track page | Every place that names a weakest scorecard says when that level is its top one; on the overview, a scorecard at its only top level that holds the repository is marked "ceiling: caps the repository" |
| F2. Compare named Toolbox by commit and the harness by hash | The hero's "How it was measured" leads with the track's description: Toolbox (a release build after v0.1.1) against Backstage 1.57.0 on the same machine and Postgres, medians of 5 runs on 10,000 components, and the independent agent that recomputed the numbers and upheld the verdict |
| F3. Four spellings of the basis badge | One label pair at every size: "Measured" and "Agent-judged", smaller in dense rows |
| F4. One dashed pill meant both "not assessed" and "provisional" | Two treatments: "not assessed" is a dashed violet outline, "provisional" a dotted olive one |
| F5. "Ten" and "ratchet" were undefined | A custom level explains itself on hover ("ten is a level OpenSSF names itself: its only level, so reaching it is the top"); the first ratchet, CEL, fixture or harness in a description is a dotted term with its definition on hover |
| F6. "Connector" and "Portal connector", "File" and "Python script", and "Observability" read alike or as qualities | Kinds take a `title`: "Connector binary", "Declared connector", "Test or script file", "Python file" (through the sarif package's `kindTitle`) and "Telemetry checklist" |
| F7 (the U10 hold). Tall scorecard pages; the same evidence under two rules | Evidence lists collapse after three items and evidence shared by rules shows once; a scorecard has an "On this page" jump bar; its entity list starts with the 12 worst; scorecard pages are about 3,150 px |
| F8. WCAG 2.2 `target-size` on inline links | Inline links and reference links are at least 24 px tall; serious axe nodes are 0 with WCAG 2.2 included |
| F10. Enter on the selected tab dropped focus to the body | Focus stays on the tab |
| F11. An unknown hash read as a missing scorecard, with Scorecards selected | "The link “#nope/xyz” doesn't match anything in snapshot 5", with no tab selected |
| F12. "+5 more" on a track card did not expand | The card is one link, so its chips cannot be links too; it reads "+5 more on the track page" |
| F13. "149 not scored" on Doc, with no reason | A kind page says how many entities its scorecards leave out and why, with each scorecard's `appliesTo` one click away; the Doc kind says decision records and append-only logs are listed but not scored |
| F14. Equal-height level columns | Columns size to their content |
| F9, declined. Drop the row badges on fully measured scorecards | The goal is that a rule's basis shows wherever a result is shown, so a row keeps its badge |

## Round 8: U8 falls, so the loop goes round again

A new auditor, on the state after round 7's fixes, found every task successful, no severity 3 or 4
finding, Q1 to Q5 all yes, and serious axe nodes at 0 with WCAG 2.2 included. But U8 fell from 4 to
3: the overview called a scorecard a ceiling that its track page drew as a green "✓ top level", an
unmeasurable value wore the failing pill, and a glossary term's hover read "undefined". U10 stayed
at 3, citing the dense level grid.

| Finding | Fix |
|---|---|
| F1. "Ahead on 8 of 9 factors" read as an overall win | The headline names the factor it is behind on with its rule count ("Capabilities (7 of 13 rules); check it first if it matters to you"), and a rule tally follows: "27 ahead · 14 at parity · 7 behind", with a note that factors are not weighted |
| F2. The "harness" term's hover read "undefined", and terms were hover-only | A plural finds its singular definition; a term is focusable and its definition is in its accessible name |
| F3 (part of the U8 fall). "Ceiling" on the overview, a green "✓ top level" on the track page | A scorecard at its top level that caps the repository says "▲ ceiling: caps the repository" on every page that names it |
| F4. 39 scorecards in the header, 53 Scorecard entities in the catalog | The Scorecard kind says its entities include the starter's and the reference portal's, by id prefix, and the comparison factors, and that the header counts this portal's only |
| F5. Backstage's losing value was green | Only the leader's value is coloured; whether a value meets the rule's absolute bar is in its hover |
| F6 (part of the U8 fall). "Can't be measured" wore the failing pill | A neutral dashed pill |
| F7. "How it was measured" did not name the machine or the adjudication | One VM with 4 vCPU (Intel Xeon, Linux 6.12), one Postgres 16, the load generator on the same CPUs, and the date of the independent agent's recomputation |
| F8 (the U10 hold). The level grid repeated a basis badge on every rule and every reason ran long | A column whose rules share a basis says so once in its header; reasons are clamped to three lines with one toggle for the full text |
| F9. "Provisional: not run" and "unknown: not checked in this build" | One phrase, "not run in this build", and the legend says what the dotted outline means |
| F10. Overview track chips did not respond to hover | They lift and tint like the track cards |
| F11. "Ten" unexplained; "Charter goal 1" in the Footprint copy | The OpenSSF scorecard's description says ten means a check scores 10 out of 10; Footprint says what Toolbox promises (one container image plus Postgres) |
| F12. "What would close it" under rules Toolbox already leads | Shown only where Toolbox is behind or the rule is unmeasured |
| F14. No skip link | A "Skip to content" button, first in the tab order, moves focus to the page body without changing the route |
| F13, declined. Track slugs differ from titles (`evidence` is "Proving value") | Slugs are ids that scorecards' `track` fields and existing links use; titles may change without breaking them |

## Round 9: U8 and U10 fall, so the loop goes round again

A new auditor, on the state after round 8's fixes, found every task successful and Q1 to Q5 all yes,
with serious axe nodes at 0. But it went one click past the captured routes, to scorecards the
captured pages link to, and found a severity-4 defect: a check that was not run counted as a failure.
The Rust crates scorecard told readers to "Fix the clippy warnings" when clippy had not run, and drew
"clippy-clean 15 of 15" failing beside crates at silver and gold. U8 fell to 2. U10 fell to 2
because the entity table collapsed to one letter per line on five scorecards at 1280px.

| Finding | Fix |
|---|---|
| F1 (sev 4, U8). Not-run checks counted as failures in the failure bars and the Biggest lever | A check that was not run is neither a pass nor a failure: it is left out of the bars, the lever, the failing filter and the row tallies. A note under the bars lists each not-run rule with its count, and a level that needs one is provisional: "N entities' levels rest on checks not run in this build (drawn dotted below)" |
| F2 (sev 3, U10). The entity name column collapsed on desktop and CI reliability overflowed its card | Entity rows are a grid with a minimum name column (`minmax(min(14rem,100%),1fr)`); the tally wraps under the name on narrow screens. A probe over 13 routes found no horizontal overflow and no name narrower than its minimum |
| F3. "Provisional: not run" on every cell of the Crate kind, missing from the scorecard and its summary | A kind column whose levels are all provisional says so once in its header and draws quiet pills; the scorecards table and the scorecard page carry the provisional count |
| F4. 39 scorecards and 31 kinds in the header, 53 and 38 in the catalog | The Scorecard and Kind kinds say they include the example portals' (`repo/…`, `acme/…`), so they count more than the tabs do |
| F5. Explanatory text cut mid-sentence on track, kind and factor cards | Cards show the description's first sentence whole; the clamps are gone |
| F6. The method was a grey paragraph without its checker | "How it was measured" leads with a facts strip (corpus, harness, runs, version) and "Every rule is measured by script"; the method section opens with the setup in prose, naming the machine and the independent agent that recomputed the verdict |
| F7. The capabilities factor and the Crate kind page were long and uniform | Rules at parity fold under "Show the N rules at parity" when there are three or more; an open kind replaces the 31-card grid with one row of kind chips per group |
| F8. "9 scorecards" on the Against Backstage track card | "9 factors" on its card and page |
| F9. Blue meant both "at parity" and "in progress" | In-progress campaign rings, cards and bars are ink, not accent |
| F10. "Clamp the reasons", "caps the repository", "the bars are absolute" | "Show shorter reasons"; "▲ ceiling: its top level is bronze, so the repository can't rise above it"; "pass thresholds were fixed before the run" |
| F11. On mobile the scoreboard followed the method prose | The method is now a one-line facts strip, so the scoreboard follows the verdict within a screen |
| F12. Small targets; 2 headings on the entity page | Scorecard sections are `h3` and rule names `h4`; the parity fold's summary is 24px tall |

Joy: each Compare factor card leads with its headline number ("7 of 13 behind", "12× lower p95").
Momentum since the last snapshot stays unbuilt, for the reason under *Joy opportunities*.

## Round 10: U7 falls, so the loop goes round again

A new auditor found every task successful, Q1 to Q5 all yes, no severity-3 or -4 finding and serious
axe nodes at 0. U8 and U10 recovered to 3, but the stop rule wants them at 4, and U7 fell to 3:
drill-downs opened scrolled past the site header, and focus was left on the page body after a stat
card or sidebar link.

| Finding | Fix |
|---|---|
| F1 (U8). Review scorecards' overview rows drew unassessed entities as solid bronze | Unassessed entities are a hatched segment, outlined in the agent-judged violet, and the row says "N not assessed (✦ agent-judged)"; the level key above each scorecard table names the hatching |
| F2 (U7). Drill-downs opened at about 110px down, hiding the portal's name | A route whose focus target sits near the top opens at the page header; a deep target still scrolls to itself |
| F3 (U2). The Compare method line read "harness 5bd608d · install facts from v0.1.1" | The track description now opens with one plain sentence (machine, runs, corpus, versions, who re-checked it, when), which the hero shows as "Method"; hashes stay in the setup cards, and the method section continues where the sentence stops |
| F4 (U5, U6). Factor headlines mixed ratios and counts | Every card leads with "N of M rules behind" (else ahead, else at parity), with "widest lead: 3.2× lower …" beneath when there is a measured one |
| F5 (U7, U9). Focus stayed on the body after a stat card or sidebar link | Every route change focuses the page's heading |
| F6 (U4). The entity page's agent-judged rules sat below six measured cards, unnamed | "Agent-judged here:" under the basis key names each scorecard's judged rules and jumps to it; each scorecard header counts its measured and agent-judged rules |
| F7 (U3). "✓ half 50%" unexplained; "Toward silver" grey on grey | Milestones say "reached" and explain themselves on hover; the climb bar is the level's ink on an outlined track, with its percentage |
| F8. The Scorecard kind's count was explained in a long clause | One sentence: the count adds the Scorecards tab's standards, the Compare tab's factors and the example portals' scorecards |
| F9. No medal in the Tracks key; the hollow medal faint in dark mode | "Medal: its weakest entity's level · none" in the key; the hollow medal is outlined in the muted ink |
| F10. "Show them" had no way back | "Hide them" collapses the list again, and both buttons say whether the list is open. Filters were already kept in the URL (`#catalog/Crate?q=cli`) |
| F11. The catalog ended with an empty "Pick a kind" card | The lede says to pick a kind; the card is gone |
| F12. "Every Rust crates level here is provisional" | "Every level on the Rust crates scorecard is provisional here" |
| U10. Scorecard pages near 2,800px | A scorecard whose rules share one basis says so in one line instead of the two-column key, and level reasons clamp at two lines; the Agent skills page fell from 2,815px to 2,679px |

## Round 11: U8 and U10 stay at 3, so the loop goes round again

U7 recovered to 4 and U1 held at 4, with every task successful, Q1 to Q5 all yes, no severity-3 or
-4 finding and serious axe nodes at 0. U8 stayed at 3 because three scorecard counts disagreed (39,
48 and 53), and U10 stayed at 3 because scorecard pages stacked four boxes before their data.

| Finding | Fix |
|---|---|
| F1 (U8). The header said 39 scorecards, the sidebar listed 48 and the catalog 53 | Wherever the Scorecard kind is counted it splits the number: "53 (39 standards, 9 comparison factors, 5 in example portals)". The sidebar groups its rows under "39 standards" and "9 comparison factors, also on the Compare tab", and a factor's row shows its verdict instead of a debt |
| F2 (U2, J4). Scorecard descriptions named other scorecards by id | The meta-rule, review-crate, review-scorecard and review-sdlc-stage descriptions are one plain sentence each; a hyphenated scorecard id left in a lede or rationale links to that scorecard under its title. meta-rule and review-crate were re-assessed against scorecard-quality in the same change |
| F3 (U4, U10). Four boxes before "Where entities stand" | A scorecard whose rules share one basis says so in its "Rates every …" line, the not-assessed banner moved below the level bar, and the data now starts near 600px instead of 640–765px |
| F4 (U3, U5). "On average" undefined; the comparison track sorted among the others | Track cards say "of entities at the top level, averaged over N scorecards"; comparison tracks sit in their own section, "Measured against a rival", whose lede says why it has no place in the order |
| F5 (U9). Basis badges explained only by `title` | Badges are focusable and show their description on hover and focus, like the glossary terms; inside a button the badge stays a plain label so no focus stop nests in a control |
| F6 (U6). Stripes meant both "not assessed" and "not scored" | Stripes mean not assessed only; no result is a dotted fill |
| F7, F8 (J4, U2). "See Version on the Compare page" on the Compare page; two version phrases | The vs-backstage description names "Toolbox 581be50 (a release build after v0.1.1)" once and drops the cross-reference |
| F9 (U5). The level "Ten" undefined | A custom level name is followed by the sentence of the scorecard's description that defines it: "Its one level, ten, means a check scores 10 out of 10" |
| F10 (U6). Title Case filters | Filter segments are sentence case |
| F11 (U10). The row chevron wrapped alone | The last word and the chevron never break apart |
| F12 (U2). "Scorecard: A scorecard." | "A named set of rules, in levels, that rates every entity of one kind, in this portal or in an example portal it describes" |
| F13 (U8). "Toward silver 62%" looked reachable | When a ceiling standard caps the level, the climb bar is drawn blocked, with a lock and "blocked", and its label names the standard |
| F14 (U7). "Back to the summary →" | "← Back to the summary" |
| F15 (U3). Rule pages linked only the standard's sources | `has-example` has its own link (Anthropic's skill best practices), shown before the standard's; 70 of 260 rules now have their own |

## Round 12: U10 stays at 3, so the loop goes round again

U8 rose to 4 and U1 and U7 held at 4, with every task successful, Q1 to Q5 all yes, no severity-3 or
-4 finding and serious axe nodes at 0. Only U10 stayed at 3: on a phone the tab counts disappeared,
the crate list spent its first screen on the kind chips, and Compare ran past 3,200px with no way
back to the tabs. The fixes below also take the round's severity-1 and -2 findings.

| Finding | Fix |
|---|---|
| F13 (U10). Mobile tab counts hidden; kind chips fill a phone's first screen; no navigation on long pages | Tab counts stay on a phone, in a smaller pill. Below 820px the kind chips give way to a "Kind" select at the top of the list, grouped like the chips. On wider screens the tab bar sticks to the top of the page, and headings scroll clear of it |
| F1 (U3). A track ring counts only the top level | Each track card has a level-mix bar under its description, "Entity levels: 3 none · 32 bronze · 27 silver · 11 gold", with the ring kept as the share at the top level |
| F5 (U9, U3). Chip medals were colour only at 14px | Each chip names its level: "DX Benchmarks · bronze" |
| F9 (U7, J2). A chip was not its own link | The card's title is the card link, stretched over the card; the chips, and the links in the next-medal line, sit above it and open their scorecard or rule |
| F14 (J1) and joy idea 2. Empty space on track cards | A "Next medal" line names the weakest scorecard that can still rise, how many entities it has to lift, and the rule most of them need; the track page repeats it under its goal |
| F15 (U6). Medal x-offsets jittered on the track scorecard table | The medal column has a fixed width |
| F2 (U3, J4). A paragraph explained outline styles | The paragraph says only "A hollow level may not hold; the words beside it, or above its column, say why", and on a phone every hollow pill carries its words, since the column header is hidden there |
| F7 (J4, U8). "To measure it, include scorecards that rate the same kind" shown to every reader | A campaign whose scorecards rate different kinds leads with "Open work, per scorecard" and its per-scorecard rows; the configuration hint is in a collapsed "For portal authors" note, and the pill reads "no single score" |
| F8 (U1). Two causes for the repository level | The hero leads with the binding one: "is at bronze, the lowest level among 13 repository standards, and capped there: Third-party actions pinned has no silver level yet. Once it does, 4 more standards need silver" |
| F4 (U3). One method sentence for timed and capability factors; the bias note only on Capabilities; an unnamed checker | The method's first sentence, in the hero, splits timed factors from capability and fact factors and says the capability scenarios come from the Backstage parity audit, so they favour what Backstage ships; the method names the benchmark's adjudicator and its date |
| F6 (U4). "Measured" on every row of an all-measured scorecard | On a scorecard or factor whose rules are all measured, rows keep the measured icon with its name for screen readers, and the header says "Measured · all 13 rules" once; agent-judged badges always keep their word |
| F3 (U9). Tooltips could run off a phone's screen | A tooltip shifts left when it would pass the viewport's right edge |
| F10 (U4, U6). Rule-page entity links looked like plain text | They carry the row chevron |
| F11 (U3). Level reasons cut mid-sentence | The short view shows each reason's first sentence whole |
| F12 (J2). The parity toggle kept saying "Show" | It says "Hide the 6 rules at parity" when open |
| F16 (U9). The row chevron at 3.47:1 | It uses the muted text colour |
| J4. "check it first if it matters to you" | "Behind on Capabilities (7 of 13 rules): if you need that, start there" |

## Round 13: a severity-3 scroll bug, so the loop goes round again

U5, U9 and J3 rose to 4, with every task successful, Q1 to Q5 all yes and serious axe nodes at 0.
The auditor found one severity-3 finding: a tab switch kept the previous page's scroll offset, so a
reader who had scrolled landed mid-page, past the verdict. That held U7 at 3. U10 stayed at 3 on
scorecard pages that reach the entity list only after every level's reasons.

| Finding | Fix |
|---|---|
| F1 (sev 3; U7, U1, J2). A tab switch kept the scroll offset | Every tab switch to another route opens at the top of the report; filters and inline expands inside a view keep their place |
| F2 (U2, U3). "13 clean" and "10 ten" on track bars | A level name outside none, bronze, silver and gold that is its scorecard's top level reads "10 ten (top level)" |
| F3 (U5, U8). Contributors counted among missing features | The Capabilities factor says so: "12 rules are features and one, community, counts contributors". The scoring stays as the benchmark published it |
| F4 (U6). Two hollow outlines, "Measured" as word and as icon, and "Agent-judged" against "All Agent-judged" | A hollow pill carries its glyph inside: ✦ for not assessed, ◌ for checks not run, and the kind page's note names both. "Measured" keeps its word everywhere, quieter on an all-measured scorecard. Level columns read "All 5" or "1 rule" before the badge |
| F5 (U9). The icon-only gauge explained itself only on mouse hover | There is no icon-only form any more |
| F6 (U4, U10). The entity list started near 2,100px on scorecard pages | Entities follow "Most common failures"; "What each level needs" comes last, and the jump bar follows the same order |
| F7 (U2, J4). "snapshot 5 · generated by toolbox report", "Rates every Rule", "For portal authors" | "made by toolbox report from data snapshot 5" (the report data carries no date), "Rates every rule", "Why there's no progress ring here" |
| F8 (U6). Checkbox squares in "Toward Silver" | ↑ for a standard to lift and ⊘ for one that needs the level first |
| F9 (U3). Factor pages did not restate the setup | A "Measured on" line under each factor's title: corpus, harness, runs and both versions, with a link to the method and who checked it |
| F10 (U10). On a phone the 8 / 0 / 1 counters fell below the definitions | On narrow screens the counters follow the headline, then the widest lead, the rule tally, the definitions and the method |
| F11 (J1). Equal-height track cards left empty blocks | Cards size to their content |
| F12 (U7). Track URLs did not match their titles | Links read `#track/ci-and-supply-chain`; the manifest name still resolves |
| Joy idea 2. Wins sat below the loss | The hero adds "Widest lead: 22× lower files the adopter keeps in their repository for a first portal (Toolbox 2 files, Backstage 44 files)" after the behind line |
| (found between rounds) The next-medal line named its rule without a basis | The rule carries its badge |

## Round 14: U10 stays at 3, so the loop goes round again

The first round with no severity-3 finding and U1, U7 and U8 all at 4; J2 rose to 4. U10 held
at 3, and the auditor named why: `#scorecards` opened one scorecard rather than an index, and
track cards carried a chip list, a next-medal box and a footer that repeated the weakest
scorecard. Every finding was severity 2 or 1.

| Finding | Fix |
|---|---|
| F1 (U6, U5). Rings meant four things: a track's share, a campaign's progress, a factor tally and a pass rate | Rings mean a track's share at the top level only. A campaign is a bar with a tick at its target that turns green once passed; the overview's tiles use a bar and pips |
| F2 (U2, U6). Rules were named only by ids | Rules may carry a `title`; without one the report spells the name out ("Public API documented"). Rule pages and failing rows show the id beside it in monospace |
| F3 (U3, U9). Two hollow pills told apart at 11 px; small basis badges explained themselves only on hover | A catalog table with hollow levels shows both pills and their meaning above its rows. A small badge outside a link is a focus stop with its description on focus |
| F4 (U10, U7). `#scorecards` had no index | `#scorecards` lists all 39 standards by track, weakest first, with a search that rides in the hash; each scorecard's breadcrumb returns to it |
| F5 (U5). Probe factors showed the corpus and run count of timed runs | A factor with no timed rule reads "Probed once on" with the builds only |
| F6 (U10, J1). Track cards were overloaded and their chips wrapped | A card keeps its ring, purpose and level bar, then a medal shelf (scorecards counted by weakest level) and one "Next medal" line; the list lives on the track page. The Tracks page fell from about 2,100px to 1,676px |
| F7 (U1, J4). The cap named no one who could lift it | "needs a silver level defined by its owner, maintainers: see its levels →" |
| F8 (U2). "ten" read like a typo | Custom level names are quoted wherever they are counted: "“ten” (top level)" |
| F9 (U5). The 25–49% band had the bronze hue; parity was link blue | The ring ramp is red below 50%, olive, then green, and skips the medal hues; parity is neutral grey |
| F10 (U3). "Needs attention: 8 scorecards" beside the ladder's "None 0" | "Needs attention across the whole box: 8 of 39 scorecards have entities at none" |
| F11 (U10). Provenance sat at the end of a long page | The Compare hero carries "Measured on" with both builds, corpus and runs, and links to the results, method, adjudication and design |

## Round 15: the new index reads better than its scorecards, so the loop goes round again

Every task succeeded and Q1 to Q5 were all yes, with U1, U7 and U9 at 4 and no small targets. The
auditor found one severity-3 finding in the round's own work: the Scorecards index summarised each
standard more kindly than its page, dropping "in scope" and "provisional", so "297 docs · 100%
silver" stood against "148 silver · 149 not in scope", and Rust crates drew solid although every
level was provisional. That held U8 at 3. U10 stayed at 3 on scorecard pages that open with
prose.

| Finding | Fix |
|---|---|
| F1 (sev 3; U8, U6). The index dropped scope and provisional status | The index and the overview share one caption: "297 docs, 148 in scope · 100% at the top level (silver)", then provisional and not-assessed counts. Bars draw provisional levels dotted, as they draw unassessed ones hatched, and the level key explains both |
| F2 (U4, U10). Scorecard pages led with the description and the why | A status block follows the title: medal, weakest level, entities at the top level, the biggest lever and the level bar. The why clamps to three lines with "Read the full why" |
| F3 (J4, U2). "22× lower files" | Ratios use the unit's comparative and put the measure first: "files the adopter keeps in their repository for a first portal, 22× fewer"; "Backstage: 707× more" |
| F4 (U3, U5). "Ten" and "Clean" read as gold; "0% silver" beside "% gold" | Captions name the level: "0% at the top level (silver)"; a weakest pill at its top level says "✓ top level" |
| F5 (U1). The Capabilities header said only "behind" | Its chip reads "Toolbox behind Backstage · 7 of 13 rules behind" |
| F9 (U2, J4). The Tracks page over-explained | One purpose sentence; the medal meaning stays in the key |
| F10 (U5). A bronze medal while 23 of 29 were not assessed | A medal fades and carries ✦ or ◌ when at least half the entities at its level are hatched or dotted |
| (T1 obstacle) A track card did not name its weakest scorecard | "Weakest, and the next medal: SDLC stages, reviewed by agents at None → bronze · 3 entities to lift" |
| (found between rounds) The Campaigns heading still said "a ring turns green" | "a bar turns green once it passes its target mark" |

F6 (no deltas) stays open for the reason under Joy opportunities. F7 (grey for silver, parity and
under-target) and F8 (the overview's stat strip at the fold) are severity 1 and left for now.

## Round 16: one tile still counts unread passes, so the loop goes round again

Every task succeeded, Q1 to Q5 were all yes, and there was no severity-3 finding. U1, U3, U5, U6,
U7, U9 and U10 were at 4, J3 at 4. Only U8 missed the bar: the overview's "82% · 8,304 of 10,100
checks pass" counted agent-judged rules that pass on entities no agent has assessed, which the
same page's tooltip says pass "for want of anything to read".

| Finding | Fix |
|---|---|
| F1 (U8). The checks tile counted passes nobody had judged | The percentage counts passes on evidence only: "7,958 of 10,100 checks pass on evidence; 346 more pass only because no agent has assessed them yet (✦); 72 not run in this build". The gold tile would add how many gold entities await an assessment; none do |
| F2 (U3, J1). The faded-medal key overlapped its label | The key item lays the medal and its text out side by side with a gap |
| F3 (U3). Scorecard rows showed no purpose | Each row carries the first sentence of its description in one muted line, in full on hover |
| F7 (U10). ", 22× fewer" began a mobile line | The ratio sits inside the link with the measure, so they wrap as one phrase |
| F8 (U4). The ↗ arrow floated away from wrapped link text | The arrow is part of the link text, joined to the last word by a no-break space and hidden from screen readers |
| F9 (J4). "If you need that, start there" | "If your teams rely on what that factor measures, read it first" |
| F11 (U10). Mobile status lines began with "·" | The separators are their own elements, hidden on narrow screens, where each fact takes its own line |

F4 (snapshot wording and non-metal top levels), F5 (the ring and the medals on one track card),
F6 (the Assessed rule's badge) and F10 (13 font sizes) are severity 1 or 2 and left for now.

## Round 17: the honest overlays had become a vocabulary, so the loop goes round again

Every task succeeded and Q1 to Q5 were yes, with no severity-3 finding. U8 reached 4 for the first
time since round 14, but U3, U5, U6 and U10 fell back to 3: the track ring and its pooled level bar
measured different things, four overlay codes had to be held at once, measured scorecards went
unmarked in lists, and entity and scorecard pages ran to 2,800 to 3,000px.

| Finding | Fix |
|---|---|
| F1 (U5, U3). The track ring and its pooled entity bar disagreed (96% over a mostly silver bar) | A bar per scorecard, filled to its share at its top level, with a dashed line at their average, which is the ring |
| F3 (U2, U3). Four overlay codes: hatched, dotted, faded medal and hollow chips | Two states, each with one look. Medals are always plain; when at least half the entities at the weakest level are unassessed or unrun, the weakest-level pill beside the medal is hollow with ✦ or ◌ and says the level may not hold |
| F4 (U3, Q4). List rows showed the what, not the why | A scorecard index row leads with the first sentence of its rationale ("Why: …") |
| F5 (U6, Q5). Lists marked only agent-judged scorecards | Every listed scorecard carries its basis: Agent-judged, or Measured when every rule is measured; the sidebar uses the icon, with its meaning under the sidebar title |
| F7 (U6). "1 rule ✦", "All 3 ✦" | Level grids say "All 3 rules" or "1 rule" before the badge |
| F8 (U4). The checks tile packed its caveats into the headline | The number and its sentence on one line, the caveats on a second, quieter line |
| F9 (J4). "made by toolbox report from data snapshot 5"; "Runner-backed cards are left out" | "generated by toolbox report from snapshot 5 of the portal's data"; the repository standard's why says "scorecards that need the CI runner" |
| F10 (U6). "Ten" and "Clean" passed for gold | A level that is its scorecard's only level reads "Ten ✓ top" inside its chip, and the separate "✓ top level" mark is dropped there |
| U10. Entity page 2,847px, scorecard pages about 3,000px | Rules needed beyond the next level fold into closed rows that keep outcome, basis and level; three or more entities with the same result share one row. The toolbox-cli entity page is 2,094px and Crates, reviewed by agents 2,388px |

F11 (the track card's focus ring) needed no change: the card's main link already outlines the whole
card, and the auditor's ring was on the inner "Next medal" link, a separate stop. F2 (the why on a
loss explains impact, not cause) and F6 (two Toolbox versions on one line) are data written by the
benchmark harness and stay open under *Findings still open*.

## Round 18: the stop rule holds

No score fell from round 17, the targeted U3 and U10 rose to 4 with U4, and J1 reached 4 for the
first time. Every task succeeded, Q1 to Q5 were all yes, there was no severity-3 or -4 finding, and
axe found no serious node on any route. U1, U7, U8 and U10 are at 4 and nothing is below 3, which
meets the bar set for this loop. Four severity-2 findings and one severity-1 were cheap, so they were
fixed after the audit:

| Finding | Fix |
|---|---|
| F1 (U5, U8). The scorecard sidebar ranked finished cards as debt ("33 at bronze" on a card whose only level is bronze) | Entities at their scorecard's top level are never debt: the sidebar counts bronze only below the top, and a card with every entity at the top reads "✓ top level" |
| F3 (U10, Q4). "Why:" lines on the Scorecards list were cut to one line | Two lines, with the full sentence on hover and on the scorecard's page |
| F4 (U2). "39 scorecards", "39 standards" and "Scorecard 53" for one idea | Counts of scorecards say "scorecards"; the kind count reads "53 (39 of this portal's own, 9 comparison factors, 5 in example portals)" |
| F6 (U6, Q5). The Crate table's "Rust crates" column had no basis mark | Every scorecard column carries its basis badge |

F2 (why each Backstage gap exists) stays open under *Findings still open*. F5 (a factor led on 2 of 8
rules with 6 at parity reads "ahead"), F7 (top level as a chip mark or a separate mark), F8 (a gloss
for "ten" and "clean" on track cards), F9 (the hero's repeated links), F10 (the ring's prose) and
F11 (motion on hover) are severity 1 and left for now.

## Round 19: the fixes after round 18 cost U3, U4 and U8, so the loop goes round again

Round 19 audited the build with round 18's fixes. Every task succeeded, Q1 to Q5 were yes, there was
no severity-3 finding and axe found no serious node, and J2 rose to 4. But U3, U4 and U8 fell back
to 3, so the stop rule failed:

- U8: the catalog's kind bars drew levels as settled that the table below marks unassessed or
  provisional, and a campaign with no single score drew an empty bar.
- U3: the sidebar described the basis icons in words only, and a loss on Compare said why the
  capability matters but not what would close the gap.
- U4: the Scorecards tab was a 4,229px run of 39 rows of equal weight.

| Finding | Fix |
|---|---|
| F1 (U8, U6). The Crate kind bar read "14 bronze · 1 gold" in solid colour over a table of hollow and provisional levels | Kind bars split each level as the scorecard bars do: hatched where agent-judged rules are not assessed, dotted where provisional. The Crate bar reads "14 bronze ✦ not assessed · 1 gold ◌ provisional · 2 not scored", and the catalog legend names both overlays |
| F2 (U3, T2). A behind rule's why explained the capability's value, not the gap | Each behind row on Compare adds "What would close it" from the rule's fix, and the factor page shows it outside "How it is scored". Why Toolbox lacks a capability stays open, under *Findings still open* |
| F3 (U9, Q4). The clamped why on Scorecards rows was in a tooltip only | It opens in full when the row has keyboard focus |
| F4 (U5, U6). Red meant under 50% on rings, none on pills and behind on Compare | Rings and track bars use one grey-to-green ramp; red is kept for none and behind |
| F5 (U2, J4). "generated by toolbox report from snapshot 5 of the portal's data"; "✦ some rules agent-judged, the gauge all measured" | "read from this repository's portal, snapshot 5"; the sidebar subtitle shows the two icons beside their meanings |
| F7 (U5, J1). The bar caption repeated under every track card | One key in the tracks legend, with a small sample of the bars |
| F8 (U8). An empty bar and target tick under a campaign with no single score | No bar |
| F9 (J4, U3). "Show the full reasons" did not say what it adds | "Show full reasons, with thresholds and sources" |
| F12 (U3, T4). "Assessed" passing on a file's existence read as mechanical | The Agent-judged definition adds that whether the assessment exists and holds counts too |
| U4. Scorecards tab of 39 equal rows | Scorecards whose every entity is at the top level fold under one row per track ("✓ 4 scorecards with every entity at the top level", naming them), so open work leads; a search shows every row. The page is 3,335px on desktop and 5,600px on mobile, from 4,229px and 7,109px |

F6 (name the adjudicating agent), F10 (two campaign bars in one mobile screen) and F11 (a docs
subtotal inside "Docs and examples") are severity 1 and left for now.

## Round 20: U9, J2 and J3 fall, so the loop goes round again

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. U6 rose to 4, but U9 fell to 3 on bar fills under 3:1 and the clamped why on the Scorecards
list. J2 fell to 3 because route changes seemed to have no motion, and J3 to 3 because no change
since the last snapshot is shown. U8 stayed at 3, on a provisional entity headed by a plain
"Bronze".

| Finding | Fix |
|---|---|
| F2 (U3, U8). "8 of 9 factors" read stronger than the rule tally | The headline pairs them: "Ahead on 8 of 9 factors and 27 of 48 rules; behind on 1 factor and 7 rules" |
| F3 (U5, U9). Silver 2.48:1 and gold 2.01:1 against the page | Bar segments carry a thin outline in the text colour: 3.8:1 for gold, 4.5:1 for silver |
| F4 (U8). The toolbox-cli page headed "Bronze" on two unrun bronze checks | "Bronze (provisional)", with the reasons beneath |
| F5 (U9, U4). Why lines clamped, the rest in a tooltip | The why shows in full; on a narrow screen the row drops the scorecard's id. The page is 3,822px on desktop and 6,856px on mobile, both shorter than round 18's |
| F6 (U3). "ten" (top) and "clean" (top) unexplained on Tracks | The tracks legend says a quoted level is its scorecard's own name for a level, and the shelf reads "✓ top" |
| F8 (U3). "149 not scored" on Doc, with no reason | The catalog legend: "not scored: out of every scorecard's scope, or none has run on it" |
| F13 (U9). The rail's scroll fade covered a whole item | A 14px fade |
| J2. No motion on route change | The entrance moves 8px over 0.3s with a short stagger across the first sections, and stays off under reduced motion |

F1 (internal ids such as "D64" and "v1-spec M5" in Compare's fix text) is data in the comparison,
which the report does not rewrite; it joins *Findings still open*. J3's change since the last
snapshot stays not built, for the reason under *Joy opportunities*. F7 and F9 to F12 are severity 1
and left for now.

## Round 21: U10 falls and Q4 is only mostly yes, so the loop goes round again

Every task succeeded, there was no severity-3 finding and axe found no serious node. U4, U8 and U9
rose to 4. U10 fell to 3 on page length (Scorecards 3,822px, Compare 3,567px), a mobile tab bar that
wrapped to two rows, and a tracks legend that pushed the first card down on mobile. Q4 was "mostly":
the overview's attention table and the track page's scorecard rows had no why, and two read-more
links were titled only "Thresholds". U2 stayed at 3 on Compare's fix text, which named decision ids,
spec milestones and repository paths.

| Finding | Fix |
|---|---|
| F1 (U2, J4). "once D64 lifts the pause", "v1-spec M5", "ui/src" in Compare | The comparison's standard is rewritten in plain words, and every Capabilities gap links to what to read |
| F3 (U3, Q4). No why on the attention table and the track page's rows | Both show the scorecard's why on a full-width second line, as the Scorecards tab does |
| F4 (Q4). Two links titled "Thresholds"; "It sits in the floor" | The links say what they explain ("Thresholds: why hidden text sits at bronze"), the rationale says "bronze, the lowest level", and both rules are re-assessed |
| F7 (U2). "snapshot 5" unexplained | The header reads "snapshot 5 of this repository's facts" |
| F9 (U10). Mobile tabs wrap; the tracks legend comes first | The tab bar scrolls with an edge fade, and the legend folds under "How to read the rings, bars and medals", closed on narrow screens |
| U10. Scorecards 3,822px on desktop | The why moves to a full-width second line in each row: 3,282px |
| F11 (U3). "Assessed" counted as agent-judged without saying why on the scorecard | The scorecard's basis line adds "an agent's assessment is the evidence, so whether one exists and still holds counts too" |

F2 (commit hashes and an unnamed adjudicator in "Measured on") is harness data and joins *Findings
still open*. F5 (track mini-bars labelled only by tooltip), F6 (the bottom band of the ring ramp is
grey), F8 (the medal and the pill repeat the level) and F10 (no change since the last snapshot) are
left for now: F5 and F6 are severity 2 and 1 and the legend now explains both; F10 needs history the
report does not keep.

## Round 22: U4, U6 and U10 hold at 3, so the loop goes round again

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. U3 rose to 4, but U4 and U6 fell to 3 and U10 stayed at 3. The auditor traced U6 to ✦, which
marked both an agent-judged rule and a level not assessed yet, and to a column headed "Standard" over
scorecards that set no repository level. U4 came from 14 font sizes and long runs of same-weight
rows, and U10 from page length (overview 3,137px, Compare 3,574px).

| Finding | Fix |
|---|---|
| F1 (U5, U6). "Most common failures" bars took the level's colour, so a 40-of-41 failure was a long gold bar | Failure bars are drawn in the fail hue; the level stays as the pill beside the rule |
| F2 (U6). ✦ meant both "agent-judged" and "not assessed yet" | ✦ means agent-judged only; a level not assessed yet carries a small hatched swatch, matching its hatched bar, and the legends say "hatched: not assessed by an agent yet" |
| F3 (U2, U6). "Standard" headed a column of scorecards; "Why this standard" on scorecard pages | Any scorecard is a "scorecard"; "repository standard" is kept for the 13 that set the repository level |
| U4. 14 font sizes | Half-pixel and one-off sizes fold into their neighbours: 11 sizes |
| F8 (U10, U4). The overview's attention table listed all eight scorecards | The three furthest behind, then "Show the other 5 that need attention", which expands in place and moves focus to the fourth row: the overview is 2,734px |
| F6 (U2, U4). Compare's "Measured on:" line of hashes sat above the plain method | The plain method sentence follows the rule tally; the builds, versions and harness fold under a disclosure |
| F7 (U5). "2 of 8 rules ahead" and "7 of 7" had the same "ahead" pill; the hero's big numbers did not say they count factors | A factor ahead on fewer than half its rules is "narrowly ahead", and the hero's numbers are captioned "Factors" |
| F9 (Q4). Collapsed "Further up" rules hid their why | Each collapsed rule shows its why's first sentence until opened |
| F10 (U9, J2). Track-card bars named their scorecard only in native tooltips | The bar group is focusable and shows a styled tooltip naming every bar, left to right, on hover or focus |

F4 (grey is the low end of the ring ramp) is kept: the warm hues are taken by none, bronze and gold,
and the ramp's legend explains it; it joins *Findings still open*. F5 (no change since the last
snapshot) stays not built, for the reason under *Joy opportunities*.

## Round 23: a stricter auditor scores every dimension 3, so the loop goes round again

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. But every score was 3, so U1, U3, U7, U8 and U9 fell from 4. Each fall traces to a finding:

- U1: the overview's verdict was a five-line sentence, and "the lowest level among 13 repository
  standards" read as if bronze were the bottom rung.
- U8: the entity page's "7 of 17 checks pass" counted a pass that holds only because nothing was
  assessed.
- U7 and U9: the new "Show the other 5" could not be undone and had no `aria-expanded`; picking a
  failing rule dropped the "Failing" filter; much secondary text was 11px.
- U3: the catalog legend drew "not scored" and "provisional" both as dots.

| Finding | Fix |
|---|---|
| F1 (U1, U2). Five-line verdict | Three short sentences: "Held at bronze by Third-party actions pinned, which has no silver level yet. After that, 4 more standards need silver. The repository is rated by the weakest of its 13 standards." |
| F2 (U8, U5). "7 of 17 checks pass" included a hollow pass | "6 of 17 checks pass on evidence · 1 more only because nothing was assessed" |
| F10 (U9, J2). One-way "Show the other 5" | A toggle with `aria-expanded`: "Show only the first 3" folds it again |
| F9 (U7). A rule filter replaced "Failing" | Picking a failing rule keeps the "Failing" filter |
| F11 (U9). 11px text | The smallest size is 12px |
| F6 (U3, U5). "Not scored" and "provisional" both dotted | "Not scored" is drawn in thin vertical stripes; dots mean provisional only |
| F7 (U6). Silver-topped scorecards had no "✓ top" on track cards | Track cards mark any level that is its scorecard's top, as the overview does |
| F12 (U4). "Measured" repeated on every passing chip | Passing chips keep the gauge icon, with the word for screen readers and on hover; agent-judged chips keep the full badge |
| F4, F5 (J4, U3). "point at them, or tab to them"; what "furthest" measures | "hover or focus the strip to name them"; Tracks is sorted by "Lowest share at the top level first" |

F3 (the version is a commit) is harness data, already under *Findings still open*. F8 (a why line on
every Scorecards row) is kept: Q4 asks for the why wherever a scorecard is listed, and earlier rounds
found clamped or hover-only whys worse. F13 to F15 are severity 1; the mobile tab strip already fades
at both edges.

## Round 24: U1 and U8 back at 4, U7 and U10 still at 3

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. No score fell against round 23: U1, U8 and U9 rose to 4. U7 and U10 stayed at 3, short of the
bar, so the loop goes round again.

| Finding | Fix |
|---|---|
| F1 (U5, U3). The failure caption said bars are coloured by level, but they are red | "Each bar is the share of entities failing the rule; the pill beside it names the level the rule is needed for." |
| F2 (U7). The scorecard sidebar opened scrolled past its top with no cue | It measured the active item from the page, not the list; it now scrolls only when the active item is out of view, and then shows "↑ Back to the top of the list" |
| F5 (U10, U1). Compare is long, and each factor page repeats the 9-card grid | Each factor page ends with the compact head-to-head chart, the current factor marked; the hero's Method line now reads as a seal: "Toolbox 581be50 against Backstage 1.57.0: timed factors are medians of 5 scripted runs on a 10,000-component catalog, and on 2026-11-23 an independent agent adjudicator recomputed every number and upheld the verdict." |
| F6. "Narrowly ahead" was undefined | The definitions say it means ahead on fewer than half of that factor's rules, at parity on the rest |
| F7. "gold on every scorecard" | "gold on every scorecard that rates them" |
| F8. A 0% slot looked empty | Every slot draws at least 3px |
| F9. Campaign "Not meeting yet" rows had no basis | Each row carries the basis of the rules that block it |
| F10 (U10). The mobile tab strip clipped the active label | The active tab is centred in the strip |
| F11. "No single score" looked like a warning | A neutral solid pill |

F3 (naming the two axes): where both appear, the catalog now labels them "How it's decided" and
"Has it run". F4 (a why on every Scorecards row) is kept for the
reason given in round 23. F12 (rules without a link of their own) is content, not layout: a rule
inherits its scorecard's links.

## Round 25: the bar is met except for one fall, U9

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. U7 and U10 rose to 4, so U1, U7, U8 and U10 are all at 4. But U9 fell from 4 to 3, and the stop
rule allows no fall, so the loop goes round again. The fall traces to F2: the glossary tooltip for
"harness" was invisible inside the collapsed "Why this scorecard" box, which clips what overflows it.

| Finding | Fix |
|---|---|
| F2 (U3, U9, J2). A glossary tooltip clipped inside a collapsed box | Every badge and glossary tooltip is now one element above the page, fixed to the viewport: no box clips it, it stays open while the pointer moves onto it, and Escape closes it |
| F1 (U1, U4). The Compare hero led with the tally and named the behind factor only | "Behind on Capabilities (7 of 13 rules): API docs, Community, Docs, Docs search, Kubernetes, Notifications and Templates. If your teams rely on those, read it first." |
| F3 (U2, J4). "no data from connector github-actions"; "the same 1 item as" | "Not measured: the github-actions connector did not run in this build, so the facts this check reads are missing."; "Evidence: the same as for “Fail rate under 20”, above." |
| F10 (U8, J3). The "no single score" campaign card showed only "—" | "2 scorecards, each scored on its own" |

F4 (change over time) needs a second snapshot in the report and is under *Findings still open*. F5
(low rings grey) is the ring ramp already listed there. F6 to F9 and F11 are severity 1.

## Round 26: one fall, U10, from page length

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. U9 came back to 4, but U10 fell from 4 to 3: the Scorecards index ran to 3,300 px of rows of
equal weight, and Compare to 3,599 px. The stop rule allows no fall, so the loop goes round again.

| Finding | Fix |
|---|---|
| F7 (U4, U10, J1). The Scorecards index lists every scorecard at the same weight | Each track's group shows its weakest four and a count per level ("5 at none · 3 at bronze · 2 at silver"); "Show the other 6 scorecards in …" opens the rest in place |
| F1 (U5, U2). "Weakest, and the next medal" named the scorecard nearest a medal, not the weakest | Scorecards at the same level now sort by their share at the top level, so the weakest leads every list and card |
| F2 (U6, U8). "Provisional" also meant "not assessed" on the entity header | The header reads "Bronze (may not hold)" when only an assessment is missing; "provisional" means checks not run |
| F3 (U5). The track ring read like a share of entities | "average share at the top level; each of 4 scorecards counts once" |
| F4 (U2, J4). A why-line cited a file, and a fix was a bare command | "A layer can score well stage by stage while a whole stage is missing …"; "Have an agent review the crate's current sources: …" |
| F5 (U3, U6). The medal row showed `1 "ten"` without "✓ top" | `1 "ten" ✓ top`, as on the track page |
| F6 (Q5). The adjudicator was not described | "The adjudicator was a fresh coding agent outside the benchmark: it designed none of it and decides no rule, so every rule stays measured" |
| F8, F9, F10, F11 (severity 1) | Compare's behind rows carry the Measured badge and their values sit beside the name; the parity toggle is underlined; the Failing filter's count says "failing a rule"; the sidebar glyph stays with the last word |

F12 (change over time) needs a second snapshot in the report and stays under *Findings still open*.

## Round 27: two falls, U7 and U9, traced to the phone tab bar and small type

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 finding and axe found no serious
node. U7 fell from 4 to 3 because the phone tab bar scrolled sideways and hid half the views, and U9
fell from 4 to 3 because most body copy, the "Why" lines included, was 12px muted grey. U10 stayed
at 3: on a phone the Scorecards intro and legend pushed the first scorecard below the fold.

| Finding | Fix |
|---|---|
| F2 (U7, U10). The phone tab bar hid half the views | On a phone the six tabs sit in two rows of three, all visible |
| F8 (U4, U9). Body copy at 12px | "Why" lines are 14px and row captions 13px; 12px is left to ids, legends and counts |
| F10 (U10). The phone legend pushed the first scorecard below the fold | The Scorecards legend folds under "How to read the bars" on a phone, as on Tracks |
| F3 (U6, J3). "Ceiling" on Overview but "✓ top level" on Scorecards | One phrase everywhere: "at its top level, so it caps the repository" |
| F5 (U6, Q5). The sidebar legend said "some rules agent-judged" beside a card whose rules all are | "has agent-judged rules · every rule measured" |
| F6 (U2). Two Toolbox versions on Compare | "Toolbox 581be50 is a commit after v0.1.1, compiled as a release build, and its install facts come from the published v0.1.1 download" |
| F7 (U2, U3). The ring definition was abstract | "14% of what each of its 4 scorecards rates is at the top level, on average" |
| F11, F12 (severity 1) | A rule's badge stays with its name and the comma follows it; the campaign scored per scorecard names its scorecards |

F1 (three bar textures) and F4 (the docs count) are open; F9 is severity 1.

## Round 28: the bar is met, with no fall

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 or 4 finding, and axe found no
serious node on any route. Nothing fell. U3, U7, U9 and U10 rose to 4, so U1, U7, U8 and U10 are all
at 4 and every other score is at least 3. The fixes below came after this round and were checked by
round 29.

| Finding | Fix |
|---|---|
| F1 (U8, U4). The standard that caps the repository was filed under "✓ every entity at the top level" | A capping standard never folds; its row stays in view with "▲ top level; caps the repository" |
| F2 (U5). Grey rings below 50% looked dormant, and two greens were close | Rings and bars are one green, so arc length carries the share; grey means only not scored and parity |
| F3 (U2). "of what each of its 4 scorecards rates is at the top level, on average" | "average share at the top level, across its 4 scorecards"; the Tracks lede says each counts once |
| F7 (U3). The documentation count was a group of 386 or a kind of 308 | Group headers count each kind: "386 entities: 308 Doc · 30 Site page · 45 Example · 3 Agent doc" |
| F11 (U6). Three bar textures | Two: hatched for not assessed and dotted for not run; not scored is a plain pale fill |
| Round 27 F9 (U2). Kind cards drew a bar with no numbers | Kind cards write the bar's counts under it ("1 bronze · 1 silver · 31 gold") |

F4 (momentum since the last snapshot) needs a second scored snapshot in the pages build and stays
open, as in §6 of the design. F5, F6, F8, F9, F10 and F12 are severity 1.

## Round 29: one fall, U3, traced to medal counts that hid top levels

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 or 4 finding and axe found no
serious node. U5 rose to 4 once the track rings were one green. U3 fell from 4 to 3: a track's
"4 bronze" and a group's "4 at bronze" did not say that one of them was already at its top level,
while the Scorecards index folded that scorecard away, so the counts and the rows disagreed.

| Finding | Fix |
|---|---|
| F1 (U3, U6). "4 bronze" hid a scorecard whose top level is bronze | "4 bronze (1 ✓ top)" on the track's medals, and "4 at bronze (1 at its top level)" on the Scorecards group |
| F2 (U8, U6). The campaign scored per scorecard showed "2 scorecards" where others show a percentage, with no status | "0 of 2 scorecards at target", a "14 to go" pill, a bar and a count to lift per scorecard, and the deadline; Overview counts it as behind target |
| F4 (U2). "meta-gold" and "in scope" | "reached gold on the form checks" in the two standards; "17 crates, 16 it rates" |
| F5 (J4, U2). The Overview tracks intro was ring mechanics | "7 long-running lines of work; Proving value is furthest from its goal. Each ring is its scorecards' average share of entities at the top level." |

F3 (words above the fold on scorecard pages) is severity 2 and open; F6 to F11 are severity 1.

## Round 30: U5 and U9 fall, traced to a capped scorecard counted as done

Every task succeeded, Q1 to Q5 were yes, there was no severity-3 or 4 finding and axe found no
serious node, so the bar held. The stop rule did not: U5 fell to 3 because the CI and supply chain
ring counted Third-party actions pinned, which is at its top level only because its ladder stops at
bronze, as 100% done while the Overview said it caps the repository. U9 fell to 3 on one small
target on the campaign page.

| Finding | Fix |
|---|---|
| F1 (U5). A capping scorecard counted as done in the track ring | Left out of the ring's average ("across 5 of its 6 scorecards, leaving out the one that caps the repository"); its bar is amber, the legend says why, and the scorecard sidebar marks it too |
| F2 (U6). ▲ meant "ahead" on Compare and "caps the repository" elsewhere | A lock marks the cap; ▲ and ▼ stay for ahead and behind |
| F3 (U3). "2 checks this level needs were not run" on a provisional silver, for two bronze checks | "2 checks were not run in this build: Clippy clean and Tests pass, both needed for bronze. Silver holds only if they pass, and so does bronze." |
| F5 (U4). The Tracks legend was open above the cards | Folded by default, as on Scorecards |
| F6 (U9). A 20 px disclosure on the campaign page | Padded to 26 px |
| F9 (U2). "8 workflows, 7 it rates" | "8 workflows, 1 not scored", the catalog's word |
| F10, F11, F12 | The strip tooltip opens above the strip; the Doc card gives the reason half of it is not scored; the sidebar scrolls to a whole row |

F4 (change since the last snapshot) stays open by design; F7 and F8 are severity 1 and open.

## Findings still open

| Finding | Why it is open |
|---|---|
| A loss's "Why" on Compare explains the capability's value; only templates says why Toolbox lacks it | The text is the comparison's rule rationale, written with the benchmark; a status per lost rule (planned, paused, by design) with its decision belongs in the comparison data, not the report |
| The Toolbox version in Compare's setup cards is a commit, not a release | The hero names the release build, but the facts are written by the benchmark harness; the next run should record the release |
| Compare's "Measured on" names builds by commit and the adjudicator only as "an agent that designed no part of it" | Both are facts the benchmark harness writes; the next run should record the release and the adjudicating model |
| Internal shorthand in rationale text beyond the four glossary terms | The report defines ratchet, CEL, fixture and harness on hover; other terms are data in each standard and need a pass over the standards |
| Four link titles still lead with a decision id in the data | An assessment pins those scorecards' text; the report renders them as "Decision: …", and the data changes when the assessments are next refreshed |
| 70 of 260 rules have their own link; the rest rely on their scorecard's | A rule earns a link when there is something specific to read; adding them is per-standard work |

## Joy opportunities

The auditors' highest-reach ideas, and what came of them:

| Idea | State |
|---|---|
| Progress toward the next level in the Overview hero | Built: "Toward silver: 8 of 13 standards there", a bar, and a list of the standards still to lift, marked ↑ when they need lifting or ⊘ when they need a higher level first |
| The largest measured wins on Compare, next to the published losses | Built: "Where Toolbox leads by the most", the three largest ratios among measured rules Toolbox leads, each with both values and its factor |
| The headline number first on each Compare factor card | Built: "7 of 13 rules behind" (rules where Backstage leads), else ahead or at parity, with the widest measured lead beneath ("widest lead: 14× higher entity reads by id per second") |
| One headline lead in the Compare hero, after the behind line | Built: "Widest lead: 22× lower files the adopter keeps in their repository for a first portal", with both values and a link to its factor; it now reads "files the adopter keeps in their repository for a first portal, 22× fewer" |
| A change since the last snapshot beside each track ring | Not built: the report carries one snapshot and the committed baseline, which equals it on every published build, so there is no earlier state to compare with |
| A dated seal when a scorecard reaches its top level | Not built, for the same reason: the report does not know when a level was first reached |

## Sources

- [design-audit.md](../design/design-audit.md), the skill
  [`design-audit`](../../plugins/toolbox-operator/skills/design-audit/SKILL.md) and its rubric.
- [report-ux.md](../design/report-ux.md); the first loop,
  [toolbox-report-2026-11-24.md](./toolbox-report-2026-11-24.md).
