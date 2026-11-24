# Hosted report UX

*Status: decided 2026-11-21 (D01M7XY1QYKW2D0HMSG50R0XTQH); redesigned after a blind design audit
on 2026-11-24 (D01M85MK7FCRXH2TRMRKXTNA5GT, [audit](../audits/toolbox-report-2026-11-24.md)); tracks,
kind groups, rationale and measured vs agent-judged results added after a second audit the same day
(D01M868BY0KJNEYE9WBFZYEYFYH, [audit](../audits/toolbox-report-2026-11-24-2.md)). This document is
about the box: how it reports what the tools found.
Amends the static page in [repo-portals.md](./repo-portals.md) §4. The agent skill is
[`report-ux`](../../plugins/toolbox-operator/skills/report-ux/SKILL.md); changes are judged with
[`design-audit`](./design-audit.md).*

---

## 1. Conclusion

A VP of engineering opening Toolbox's own report must be able to **judge quality in one scroll**,
then **explore the catalog and every check** without switching to `serve`. The page that shipped
under D92/D93 was a scorecard tab strip that expanded **failing rules only**. That is a work list,
not a quality judgement.

| Mode | Job | Primary content |
|---|---|---|
| **Overview** (default) | Fleet health verdict | The repository level as a medal, a one-sentence verdict, the next level and the ladder of repository standards that hold it, with standards at their top level marked and a standard that caps the repository named in the next step; a strip of track rings, furthest from its goal first; tiles for checks passing, gold entities, scorecards at `none` and campaigns on target; campaign cards with progress rings; scorecards that need attention, with the clear ones collapsed; catalog kinds as chips |
| **Tracks** | Lines of work | One card per track, furthest from its goal first: a ring for the share of entities at the top level, the weakest scorecard, and the track's description; a comparison track gets a segmented ring instead, one segment per factor coloured ahead, at parity or behind, so it is never read as a share; a key above the cards replaces the colour prose; the comparison card names the factors it is behind on; a track page lists its scorecards and comparisons, with the other tracks as a compact strip rather than the grid again |
| **Catalog** | What exists | Kinds in their `kindGroups`; a selected kind's list starts with a breadcrumb back to the groups, and no kind is preselected; then the scored kinds' lists with one column per scorecard → entity ids (short name primary) → that entity's scores across every scorecard |
| **Campaigns** | Time-bound focus | Lowest progress first; progress vs target; milestones; entities not meeting includes ([campaigns.md](./campaigns.md), D01M7Y11G0YZT23W26Z9D3XW68Y) |
| **Scorecards** | Standards debt and proof | Ranked debt list (not equal pills); defaults to highest `none`; what each level needs; most common failures; level and outcome filters; every rule outcome on expand |
| **Compare** | Head to head | A verdict sentence (ahead, parity, behind as a tally of factors, which are not weighted), how it was measured, one card per factor; a factor page shows each rule, what it reads and both products' values |

## 2. Why the previous page failed the VP test

| Expectation | What the page did |
|---|---|
| "How healthy is Toolbox?" in the first viewport | Landed on the first scorecard name alphabetically |
| See the software inventory | No catalog; only entities that appear under the selected scorecard |
| See the bar we already clear | Expanded rows listed failing rules only; passing rules were invisible |
| Scan thirty-plus standards | Equal-weight pill buttons with no summary ranking |
| Share a deep link to a concern | Hash named a scorecard only |

## 3. Alternative products reviewed

Primary sources were product docs and the OpenSSF viewer (vendor marketing sites for Port/Cortex/OpsLevel
remain partially blocked in cloud sessions; docs pages and GitHub READMEs were used).

| Product | Pattern we keep | Pattern we skip on a static repo report |
|---|---|---|
| **OpsLevel Maturity Report** | Passing and failing checks on one ladder; fix path next to the fail | In-product "Fix this" mutations |
| **Cortex Bird's eye / All Scorecards** | Org-wide overview, drill-down, filters by level and failing rule | Domain/team hierarchy rollups |
| **Port scorecards** | Catalog as system of record; scorecards as overlay; distribution dashboards | Initiatives, alerts, deployment gates |
| **OpenSSF Scorecard viewer** | Every check listed with reason and remediation | Weighted 0–10 aggregate as the hero metric |
| **Dashboard design patterns** (Bach et al.) | Overview → detail-on-demand; URL-parameterized exploration | Multi-page personalization |

## 4. Skill adopted

No existing Cursor or Toolbox skill covered engineering-quality report UX. The new skill
`report-ux` encodes the reader jobs and the reject criteria above. Agents that change the HTML
element or `report_json` must follow it. `maintainer-skill` scores the file like every other skill.

## 5. Data contract

`report -o json|html|web` keeps `portal`, `snapshot`, `scorecards`, and `scores`, and adds:

```json
"catalog": { "Crate": ["crate:toolbox-cli", "..."], "Skill": ["..."] },
"tracks": { "maintainer": { "title": "…", "owner": "…" } },
"campaigns": { "skill-silver": { "status": "active", "percent": 88, "includes": [], "entities": {} } },
"scorecards": { "maintainer-skill": { "title": "Agent skills", "description": "…", "rationale": "…", "links": [],
  "levels": [], "rules": { "has-example": { "title": "…", "basis": "measured", "rationale": "…", "links": [] } } } },
"kindGroups": [{ "name": "code", "title": "Code", "description": "…", "kinds": ["Crate"] }],
"kinds": { "Crate": { "title": null, "description": "…" } },
"comparisons": { "vs-backstage-capabilities": { "facts": {}, "reads": {}, "fields": {} } },
"unassessed": { "review-crate": ["crate:toolbox-api", "..."] }
```

`unassessed` lists, per scorecard with judged rules, the scored entities no assessment points at.
Their judged rules pass for want of anything to fail on, so the element shows those passes as
"passes, not assessed yet" and says how many entities a scorecard's levels rest on no judgement.

Ids are sorted. Full entity fields stay out of the static page (size and facet caveats in
repo-portals §4). Catalog browse and entity score rollups are computed in the element from
`catalog` + `scores`.

## 6. Interaction rules

- Default hash is Overview (`#` / `#overview`).
- Deep links: `#tracks`, `#track/<name>`, `#scorecards`, `#catalog`, `#catalog/<Kind>`, `#entity/<id>`, `#campaign/<name>`,
  `#scorecard/<name>`, `#rule/<scorecard>/<rule>`, `#compare`, `#compare/<name>`. A scorecard's filters
  ride in the hash (`#scorecard/<name>?level=none&outcome=failing&rule=<rule>&q=<text>`), so a work list
  can be shared; so does the catalog's name filter (`#catalog/<Kind>?q=<text>`).
- The tab title names the page ("Agent skills · Toolbox for Toolbox"), and each history entry keeps its
  scroll position, so Back returns to the same place in a long list.
- Mode chrome uses `role="tab"` / `aria-selected` with arrow-key movement.
- Every row is one link or one `<summary>`, never a clickable row that contains a link (axe
  `nested-interactive`).
- Unknown scorecards, campaigns, kinds and entity ids show a Not found state with a way back.
- Scorecards mode opens the index of standards; Campaigns opens lowest progress; Catalog prefers scored kinds.
- Scorecard picker is a ranked debt list, not an equal-weight pill wall.
- Scorecard entity rows sort worst-first (`none` → bronze → … → gold).
- Outcome filter defaults to **all**; "Failing" is one click for a work list; empty filters offer Clear.
- Entity ids show a short label first; the full id stays muted beside it.
- Passing rules render as compact chips grouped by level ("Bronze: all 5 met"); failing rules keep fix + evidence.
- "What each level needs" shows each rule with its rationale (its fix when it has none) and the rule's own links,
  so a level's requirements read as sentences, not ids, and the evidence behind a rule is one click away.
- An entity row leads with what its next level needs ("Silver needs: Has example", with each rule's basis icon), then counts the failures above
  that level. Red is kept for entities at `none`.
- Each scorecard names its biggest lever: the rule holding the most entities at the lowest occupied level.
- Fix text renders backticks as code.
- Below 820 px the Scorecards and Campaigns lists become a select above the detail.
- A scorecard page shows state before definitions: where entities stand and the biggest lever, then the
  most common failures, then what each level needs. Only failures of the first level (the ones that hold
  entities at `none`) are drawn in red.
- Kind bars and kind headers count unscored entities as a hatched "not scored" segment, so a bar never
  implies every entity reached a level; unscored rows say "not scored".
- A drill-down (`#catalog/<kind>`) scrolls to and focuses the selected kind's list, which starts with "Catalog / group / kind".
- A campaign whose scope is empty is "can't be measured", not neutral: it says why, lists the debt of
  each included scorecard, and counts in the overview's needs-attention line.
- The entity page leads with the entity's level (medal and word); the check ratio is secondary text.
- Levels are one badge component everywhere: a dot, the level's name and `aria-label="level …"`,
  so a level is never colour-only. Each level has a fill colour and a darker ink colour for text
  that passes WCAG AA. Custom level names borrow the colour of the standard level at the same
  height from the top.
- Scorecards, campaigns and rules show their human `title`; the code name stays as muted monospace.
  A rule without a `title` spells its name out ("public-api-documented" reads "Public API documented"),
  and its page and failing rows show the name beside it.
- Skipped entities take no length in a distribution bar; the legend counts them as "not in scope".
- A campaign with nothing in scope says that it cannot be measured, instead of showing 0%.
- Motion (bars growing, rings filling) respects `prefers-reduced-motion`.
- Every rule and result carries its basis: **Measured** (a gauge icon, neutral) or **Agent-judged** (a
  sparkle, violet). Lists too dense for text show the icon or tint the chip, and a key above the list
  names both. Text outputs that can't show icons (`-o md`, which pull request comments carry, and
  the ratchet's gate summary) put `✦` after an agent-judged rule's name, add "every rule
  agent-judged" or "N of M rules agent-judged" to a scorecard's heading, and print one legend line.
- A rule shows **Why** (its rationale) beside its fix, and a link to the long story. Parentheticals that
  only cite decision, question or lesson ids, or doc paths, are stripped from prose, which readers
  can't resolve; links are titled by their claim ("Decision: judgement is pushed"), not by an id.
- An agent-judged pass on an entity no agent has assessed reads "passes, not assessed yet", never a
  plain pass. Catalog columns mark agent-judged scorecards with the sparkle. Such a level is tagged
  "not assessed" and drawn as a dashed pill wherever it is shown (catalog columns, scorecard rows,
  the entity's scorecard headers): a provisional level, not an earned one. The entity hero counts
  those scorecards in its "Provisional" line beside the checks not run. The notice above a
  scorecard with unassessed entities names which judged rules pass for want of anything to read and
  which fail until an assessment exists.
- Kind titles are human ("SDLC layer"); the kind id is muted monospace under the selected kind's
  heading only. Counts in running text use the plural title ("15 action dependencies"), and say how
  many are in scope when that differs.
- Compare defines its words in the hero: what *ahead* means on a rule and on a factor.
- A level resting on checks that were not run says "Provisional" and names them, and is drawn dashed with "provisional" wherever it is shown, like a level not assessed.
- Links and buttons in text have hit areas of at least 24 px.
- No page scrolls sideways at 390 px; the design-audit capture records `mobileOverflow` per route.
- A link title that still leads with a decision id renders as "Decision: <claim>", with the id on hover; long entity ids in rows are cut to one line, with the full id on hover.
- Track rings use one ordered ramp (red below 50%, olive, green from 80%) for the share at the top
  level. The ramp avoids the bronze, silver and gold hues, which belong to medals, and parity is
  neutral grey, not link blue.
- A kind's `title`, when the manifest sets one, is its reader-facing name; otherwise the name is split
  into words. A kind page whose scorecards leave some of its entities out says how many and why,
  with each scorecard's `appliesTo` one click away.
- A custom level name (OpenSSF's "ten") explains itself on hover and, when it is the scorecard's top
  level, says so wherever the weakest scorecard is named. A scorecard at its top level that still
  holds the repository's level is marked as the ceiling on every page that names it, never as a
  green "top level".
- The first use of ratchet, CEL, fixture or harness (or their plurals) in a description is a dotted
  term that keyboard focus reaches; its definition shows on hover and focus and is in its
  accessible name.
- A value that cannot be measured wears a neutral dashed pill, not the failing one; a check not run
  reads "not run in this build" everywhere.
- In "What each level needs", a column whose rules share one basis says so once in its header, and
  reasons are clamped to three lines behind one toggle.
- A "Skip to content" button comes first in the tab order and moves focus to the page body without
  changing the route.
- Long lists start short: a scorecard shows its 12 worst entities, then "Show all".
- The hero of a comparison says how it was measured (the track's description: releases, machine,
  setup, runs and who checked the scoring, with the date). Its headline names the factors it is
  behind on with their rule counts, and a rule-by-rule tally follows, because factors are not
  weighted. Only the leading value on a rule is coloured.
- A link to a hash that matches nothing says so and selects no tab.
- A factor's "How it is scored" says in words what the harness recorded for each product and which value is ahead, then the expression, then, where Toolbox is behind, what would close the gap.
- A check not run is neither a pass nor a failure. It is left out of the failure bars, the biggest
  lever, the failing filter and row tallies; a note under the bars names each not-run rule and its
  count, and the scorecard says how many entities' levels are provisional because of it. A kind
  column whose levels are all provisional says so once in its header.
- Cards (tracks, kinds, factors) show a description's first sentence whole, never a clamp that cuts
  mid-word; the rest is on the page the card opens.
- A factor card leads with the same kind of number on every card: "N of M rules behind", else
  ahead, else at parity, with the widest measured lead beneath. On a factor page, three or more
  rules at parity fold under one toggle.
- A comparison track's description opens with one plain sentence on how each kind of factor was
  measured, and goes on to the machine, versions, and who re-checked it and when. The Compare hero
  shows the first sentence as its method; commit hashes stay in the setup cards.
- An entity whose agent-judged rules no agent has assessed is drawn hatched in the scorecard's bar,
  outlined in the agent-judged colour, and counted beside it; the level key names the hatching.
- A route change opens the new page at its header when its focus target is near the top, and moves
  focus to the page's heading either way.
- An entity page names its agent-judged rules under the basis key, each linked to its scorecard's
  section, and each scorecard section counts its measured and agent-judged rules.
- A list of rules that all share one basis says so in one line; the two-column key appears only
  when both bases are present.
- An open kind replaces the grid of kind cards with one row of kind chips per group.
- Entity rows are a grid with a minimum width for the name, so a long "needs" list wraps below it
  rather than squeezing it. The last word of a row and its chevron never break apart.
- The Scorecard kind's count is split wherever it appears: standards, comparison factors and
  example portals' scorecards. The scorecard sidebar groups standards and comparison factors under
  their own heads, and a factor's row shows its verdict, not a debt.
- When a ceiling standard caps the repository level, the climb bar toward the next level is drawn
  blocked, with a lock, and its label names the standard.
- A custom level name (OpenSSF's "ten") is followed by the sentence of its scorecard's description
  that defines it.
- A basis badge is focusable and shows its description on hover and focus, like a glossary term.
  Inside a button it is a plain label, so no focus stop sits inside another control.
- A scorecard whose rules all share one basis says so in its "Rates every …" line rather than in a
  separate box; the not-assessed banner sits below the level bar, so the data starts within the
  first screen.
- Comparison tracks sit apart from the other tracks, under "Measured against a rival": their ring
  counts factors led, which is not a share of entities, so they take no place in the order.
  A track card says what its percentage averages over ("averaged over N scorecards").
- Stripes mean "not assessed" and nothing else; an entity with no result is a dotted fill, and so
  is a level that rests on checks not run in this build, outlined in the provisional colour.
- A scorecard id left in a lede or rationale links to that scorecard under its title.
- An entity one rule from its next level says so ("One rule from gold · pass “Has evals”"), and the
  scorecard's lever counts how many entities are within reach that way.
- A track card draws a bar per scorecard, filled to that scorecard's share at its top level, with a
  dashed line at their average, which is the ring's number, so the ring and the bars cannot
  disagree. Beneath them a medal shelf counts its scorecards by their weakest entity's level, and one
  "Next medal" line names the weakest scorecard that can still rise and how many entities it has to
  lift. The card link is the title, stretched over the card. The track page lists every scorecard
  and names the rule most of the lifted entities need.
- A basis badge has one form: icon and word. On a scorecard or factor whose rules are all
  measured, the measured badge goes quiet (no border or fill) but keeps its word. A level column
  whose rules share a basis says "All 5" or "1 rule" before the badge.
- A hollow level carries a glyph inside the pill, ✦ for not assessed and ◌ for checks not run, and
  its reason in words beside it or in its column's header; on a phone, beside every pill.
- A campaign whose scorecards rate different kinds leads with its open work per scorecard; why it
  has no single percentage is a collapsed note.
- A tab switch opens the new view at the top. Changes inside a view (filters, expands) keep the
  scroll position, and Back restores it.
- Track links use a slug of the track's title; the manifest name still resolves.
- A level name outside none, bronze, silver and gold that is its scorecard's top level reads
  "(top level)" in track summaries. Track cards size to their content.
- A scorecard page lists its entities before "What each level needs".
- A factor page restates the setup under its title ("Measured on: corpus … · runs … · version …").
  The Compare hero names the widest measured lead after the behind line, and on a phone the
  counters follow the headline.
- When a ceiling caps the repository level, the hero sentence names it first; there is no second
  "capped" box.
- A comparison's method sentence separates timed factors from capability and fact factors and
  carries any bias in the scenarios; the method section names who checked it and when.
- Below 820px an open kind is picked from a grouped select, not chips. On wider screens the tab bar
  sticks to the top. Tooltips shift left rather than run past the viewport.
- `#scorecards` opens an index of every standard, grouped by track and weakest first within each,
  with a search that rides in the hash (`#scorecards?q=…`). A scorecard page's breadcrumb returns
  to it. Comparison factors stay on Compare.
- Rings mean one thing: a track's share at the top level. A campaign is a bar with a tick at its
  target, which turns green once the bar passes it; the overview's tiles use a bar and pips.
- A custom level name is quoted ("“ten”") wherever it is counted, so it reads as a name.
- The overview's needs-attention line says it covers the whole box ("8 of 39 scorecards have
  entities at none"), apart from the repository ladder above it.
- A standard that caps the repository names its owner as the one to add the missing level, and
  links to its levels.
- A basis badge outside a link is a focus stop with its description on focus, even when small.
  A catalog table with hollow levels shows both hollow pills with their meaning above its rows.
- A factor with no timed rule reads "Probed once on" and leaves out the run count and corpus, which
  only timed factors used. The Compare hero carries one provenance line: the setup it was measured
  on and the evidence links (results, how it was run, adjudication, design).
- Every list of scorecards uses one caption: entities, how many are in scope, the share at the top
  level with that level named ("27% at the top level (gold)"), and how many are provisional or not
  assessed. A level bar draws entities whose level rests on checks not run dotted, as it draws
  unassessed ones hatched. A medal is always drawn plain; when at least half the entities holding
  the weakest level are hatched or dotted, the weakest-level pill beside it is hollow, with its
  glyph, and says the level may not hold. The report has two overlay states, not assessed and not
  run, each with one look. A list never reads better than the scorecard's page.
- A scorecard page opens with its status under the title: the medal, the weakest level, how many
  entities are at the top level, the biggest lever, and the level bar. The description and a
  "why" clamped to three lines (with "Read the full why") follow.
- A ratio uses the comparative a reader would say for its unit: fewer files, faster seconds, smaller
  megabytes; the measure comes first ("files the adopter keeps …, 22× fewer"). A factor page's
  standing chip carries its rule tally ("7 of 13 rules behind").
- A track card names its weakest scorecard and its level; when that scorecard is also the next
  medal, one line says both.
- A headline share of passing checks counts only passes on evidence. An agent-judged rule that
  passes on an entity no agent has assessed is counted apart ("346 more pass only because no agent
  has assessed them yet"), beside the checks not run.
- A row in the scorecard index carries its standard's why in full: the first sentence of its
  rationale ("Why: …"); the description stands in only where there is no rationale. On a narrow
  screen the row drops the scorecard's id, which its page still shows.
- Entities at a level that is their scorecard's top level are never debt: the scorecard sidebar
  ranks and labels a card by its entities at none, then at bronze below the top, and a card whose
  every entity is at the top reads "✓ top level".
- A count of scorecards says "scorecards" wherever it appears; "standards" is kept for the ones
  that set the repository level.
- An external link's ↗ belongs to its text and wraps with the last word.
- Every scorecard in a list carries its basis: the Agent-judged badge when an agent's judgement
  decides some rules, else the Measured badge. The scorecard sidebar shows the same as an icon,
  with its meaning under the sidebar's title.
- On an entity's card and in a scorecard's entity rows, rules needed beyond the next level are
  folded: one closed row each with its outcome, basis and level, which opens to the fix and the
  why. Three or more entities with the same result share one row, which opens to them.
- A level name that is its scorecard's only level carries "✓ top" inside its chip, so it never
  passes for a gold of another name.
- A tile states its number on one line and its caveats on a second, quieter one.
- A kind's bar in the catalog splits each level the way the table below it does: hatched where its
  scorecards' agent-judged rules have not been assessed, dotted where it is provisional, with both
  named in the catalog legend.
- A track's ring and bars run on one ramp, grey to green; red is kept for entities at none and for
  factors behind a rival. The bars are explained once, in the tracks legend, not under every card.
- A comparison factor that is behind says why it matters and what would close it on its row and
  at the top of its page, not only inside "How it is scored".
- A campaign with no single score draws no bar.
- In the scorecard index, scorecards whose every entity is at the top level fold under one row per
  track ("✓ 4 scorecards with every entity at the top level", naming them), so open work leads;
  a search shows every row.
- Compare's headline pairs the factor count with the rule count ("Ahead on 8 of 9 factors and 27
  of 48 rules; behind on 1 factor and 7 rules"), so a lead built on parity does not read as a
  sweep.
- An entity whose level is provisional says so in its headline ("Bronze (provisional)"), with the
  reasons beneath.
- Bar segments carry a thin outline in the text colour, so silver and gold keep 3:1 against the
  page and the track.
- A quoted level on a track card ("ten", "clean") is its scorecard's own name for a level, and the
  tracks legend says so.
- Wherever scorecards are listed (the overview's attention table, a track page, the Scorecards
  tab), each row carries its why on a full-width second line, so the why never squeezes into the
  name column.
- On a narrow screen the tab bar scrolls sideways with an edge fade and keeps the current tab in
  view, and the tracks legend folds under "How to read the rings, bars and medals".
- A scorecard whose rules are all agent-judged says that the assessment is the evidence, so
  whether one exists and still holds counts too.
- A comparison rule that is behind links to what to read right after its why, on the Compare
  summary as on its factor page; a yes/no rule shows both products' answers on one line.

Not built: a change since the last snapshot beside each track ring. The report carries one snapshot
and the committed baseline, which equals it on every published build; a pages build that also scored
the previous release would give it something to compare with.

## Sources

- [design-audit.md](./design-audit.md) and its rubric; the baseline and re-audit in
  [toolbox-report-2026-11-24.md](../audits/toolbox-report-2026-11-24.md), and the second loop in
  [toolbox-report-2026-11-24-2.md](../audits/toolbox-report-2026-11-24-2.md).
- OpsLevel Maturity Report docs; Cortex Bird's eye and All Scorecards docs; Port scorecards guide;
  OpenSSF Scorecard viewer (`scorecard.dev/viewer`); Bach et al., *Dashboard Design Patterns*
  (arXiv:2205.00757); [repo-portals.md](./repo-portals.md); [minimal-ui.md](./minimal-ui.md).
