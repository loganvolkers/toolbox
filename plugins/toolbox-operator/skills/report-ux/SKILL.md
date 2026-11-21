---
name: report-ux
description: >-
  Designs and reviews the hosted toolbox report UI whenever the scorecard HTML,
  catalog browse, or rule-outcome presentation changes. Use when improving
  report -o html/web, auditing scorecard exploration, or judging whether a VP
  of engineering can assess portal quality from the static page alone.
---

# Hosted report UX

Judgement first. A visitor who opens the report must answer "is this portal healthy?"
before they pick a scorecard. Catalogue what exists, show time-bound campaigns,
then score continuous standards. Show every
rule outcome, not only failures. Drill from fleet → area → entity → check →
evidence and fix. Never hide the bar the portal already clears.

## When to use

1. Changing `crates/toolbox-cli/src/toolbox-report.js` or `report_json`.
2. Reviewing whether the published page is explorable without the live UI.
3. Comparing Toolbox's static report to Cortex, OpsLevel, Port, or OpenSSF Scorecard.

## Procedure

1. Name the primary reader for the change (VP, platform owner, entity owner, agent).
2. Check the first viewport answers one decision: overall health, with counts.
3. Confirm three modes exist and deep-link: Overview, Catalog, Scorecards.
4. Confirm an expanded entity lists pass, fail, unknown, and skipped outcomes.
5. Confirm failing rules still lead with fix text and evidence; passing rules stay compact.
6. Confirm filters cover level and outcome, and search matches entity id or any rule name.
7. Reject a layout that is only scorecard pills plus fail-only rows.

```
toolbox report portal --skip runner,github-actions -o web --out site/public/report
```

## Reader jobs

| Reader | First question | Must see without hunting |
|---|---|---|
| VP of engineering | Are we meeting our own bar? | Fleet summary, worst scorecards, catalog size by kind |
| Platform / standards owner | Where is debt concentrated? | Per-scorecard level bars, filter to `none` / failing |
| Entity owner | What do I fix next, and what already passes? | Entity page across scorecards; every rule outcome |
| Agent | What is the evidence and the fix? | Rule, outcome, fix, evidence list |

## Patterns adopted

| Source | Keep | Drop for this page |
|---|---|---|
| OpsLevel Maturity Report | Pass and fail checks together; level progress | Live "Fix this" actions (static report) |
| Cortex Bird's eye / All Scorecards | Fleet overview, drill-down, filter by level and failing rule | Org hierarchy grouping (repo portal has no teams tree) |
| Port | Catalog is the system of record; scorecards overlay it | Initiative dashboards and alerts |
| OpenSSF Scorecard viewer | Every check listed with reason / remediation | 0–10 weighted aggregate as the primary signal |
| Dashboard design patterns | Overview → detail-on-demand; URL state | Multi-page apps; personalization |

## Boundaries

Do not ship fail-only entity bodies: that erases the standard the team already meets.
Do not omit the catalog: a score without "what exists" is trivia.
Do not put thirty equal-weight pills in the first viewport; overview owns that space.
Limitation: this page is a snapshot with no facets; it is for repo portals, not a central portal's private data.
On error (missing `catalog` or `scores` in the JSON), show a clear load error and do not invent empty charts.
