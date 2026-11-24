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
| **Overview** (default) | Fleet health verdict | The repository level as a medal, a one-sentence verdict, the next level and the ladder of repository standards that hold it; tiles for checks passing, gold entities, scorecards at `none` and campaigns on target; campaign cards with progress rings; scorecards that need attention, with the clear ones collapsed; catalog kinds as chips |
| **Tracks** | Lines of work | One card per track, furthest from its goal first: a ring for the share of entities at the top level, the weakest scorecard, and the track's description; a track page lists its scorecards and comparisons |
| **Catalog** | What exists | Kinds in their `kindGroups`, then the scored kinds' lists with one column per scorecard → entity ids (short name primary) → that entity's scores across every scorecard |
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
  "levels": [], "rules": { "has-example": { "basis": "measured", "rationale": "…", "links": [] } } } },
"kindGroups": [{ "name": "code", "title": "Code", "description": "…", "kinds": ["Crate"] }],
"kinds": { "Crate": { "description": "…" } },
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
- Deep links: `#tracks`, `#track/<name>`, `#catalog`, `#catalog/<Kind>`, `#entity/<id>`, `#campaign/<name>`,
  `#scorecard/<name>`, `#rule/<scorecard>/<rule>`, `#compare`, `#compare/<name>`. A scorecard's filters
  ride in the hash (`#scorecard/<name>?level=none&outcome=failing&rule=<rule>&q=<text>`), so a work list
  can be shared.
- Mode chrome uses `role="tab"` / `aria-selected` with arrow-key movement.
- Every row is one link or one `<summary>`, never a clickable row that contains a link (axe
  `nested-interactive`).
- Unknown scorecards, campaigns, kinds and entity ids show a Not found state with a way back.
- Scorecards mode opens the highest-`none` card; Campaigns opens lowest progress; Catalog prefers scored kinds.
- Scorecard picker is a ranked debt list, not an equal-weight pill wall.
- Scorecard entity rows sort worst-first (`none` → bronze → … → gold).
- Outcome filter defaults to **all**; "Failing" is one click for a work list; empty filters offer Clear.
- Entity ids show a short label first; the full id stays muted beside it.
- Passing rules render as compact chips grouped by level ("Bronze: all 5 met"); failing rules keep fix + evidence.
- "What each level needs" shows each rule with its fix text, so a level's requirements read as sentences, not ids.
- An entity row leads with what its next level needs ("Silver needs: has-example", with each rule's basis icon), then counts the failures above
  that level. Red is kept for entities at `none`.
- Each scorecard names its biggest lever: the rule holding the most entities at the lowest occupied level.
- Fix text renders backticks as code.
- Below 820 px the Scorecards and Campaigns lists become a select above the detail.
- A scorecard page shows state before definitions: where entities stand and the biggest lever, then the
  most common failures, then what each level needs. Only failures of the first level (the ones that hold
  entities at `none`) are drawn in red.
- Kind bars and kind headers count unscored entities as a hatched "not scored" segment, so a bar never
  implies every entity reached a level; unscored rows say "not scored".
- A drill-down (`#catalog/<kind>`) scrolls to and focuses the selected kind's list.
- A campaign whose scope is empty is "can't be measured", not neutral: it says why, lists the debt of
  each included scorecard, and counts in the overview's needs-attention line.
- The entity page leads with the entity's level (medal and word); the check ratio is secondary text.
- Levels are one badge component everywhere: a dot, the level's name and `aria-label="level …"`,
  so a level is never colour-only. Each level has a fill colour and a darker ink colour for text
  that passes WCAG AA. Custom level names borrow the colour of the standard level at the same
  height from the top.
- Scorecards and campaigns show their human `title`; the code name stays as muted monospace.
- Skipped entities take no length in a distribution bar; the legend counts them as "not in scope".
- A campaign with nothing in scope says that it cannot be measured, instead of showing 0%.
- Motion (bars growing, rings filling) respects `prefers-reduced-motion`.
- Every rule and result carries its basis: **Measured** (a gauge icon, neutral) or **Agent-judged** (a
  sparkle, violet). Lists too dense for text show the icon or tint the chip, and a key above the list
  names both.
- A rule shows **Why** (its rationale) beside its fix, and a link to the long story. Decision ids are
  stripped from prose, which readers can't resolve.
- An agent-judged pass on an entity no agent has assessed reads "passes, not assessed yet", never a
  plain pass. Catalog columns mark agent-judged scorecards with the sparkle and tag such levels
  "not assessed".
- Kind titles are human ("SDLC layer"); the kind id stays as muted monospace. Counts in running text
  use the plural title ("15 action dependencies"), and say how many are in scope when that differs.
- Compare defines its words in the hero: what *ahead* means on a rule and on a factor.
- A level resting on checks that were not run says "Provisional" and names them.
- Links and buttons in text have hit areas of at least 24 px.

## Sources

- [design-audit.md](./design-audit.md) and its rubric; the baseline and re-audit in
  [toolbox-report-2026-11-24.md](../audits/toolbox-report-2026-11-24.md), and the second loop in
  [toolbox-report-2026-11-24-2.md](../audits/toolbox-report-2026-11-24-2.md).
- OpsLevel Maturity Report docs; Cortex Bird's eye and All Scorecards docs; Port scorecards guide;
  OpenSSF Scorecard viewer (`scorecard.dev/viewer`); Bach et al., *Dashboard Design Patterns*
  (arXiv:2205.00757); [repo-portals.md](./repo-portals.md); [minimal-ui.md](./minimal-ui.md).
