# Dashboards, Trends and UI Customisation

*Status: decision (2026-11-19). **Replaces the earlier "canvases" design.** Addresses red-team gap
A2 (dashboards, reports, trends).*

## 1. Decision

1. **No user-authored dashboards inside Toolbox.** Personal and ad-hoc dashboards are built in the
   tools agents and people already use, such as **Claude Artifacts** or **Cursor Canvas**, connected
   to the **Toolbox MCP server**.
   - Those runtimes already provide sandboxed generated UI, sharing and iteration.
   - Toolbox provides the data, with permissions enforced by its MCP and API layer.
   - This removes the confused-deputy risks of user-authored canvases running inside the portal
     ([principal-review.md](../principal-review.md) §1.5).
2. **UI customisation is for Toolbox operators only, in code.** Operators extend the portal UI with
   **Vue views** on `@toolbox-dev/ui` ([frontend.md](./frontend.md)) in the portal repo. Those
   views are reviewed, tested (Storybook page stories) and versioned like every other part of the
   program.
3. **Toolbox ships good built-in views:**
   - portal home and team home
   - scorecard overview
   - initiative burndown
   - ownership coverage
   - portal health (meta catalog)
   - connector freshness
   - agent activity

## 2. What Toolbox provides so external dashboards are good

- **A history store, so trends exist** *(Phase 2; v1 keeps per-snapshot summary counts)*. It is an append-only archive of derived-output deltas
  ([incremental-computation.md](./incremental-computation.md) §3):
  - scores per (rule, entity) on change
  - daily roll-ups per (scorecard, group, level)
  - fact history
  - catalog counts and ownership coverage

  It records **the owner at the time**, so team trends survive reorganisations. Retention is set in
  code.
- **MCP tools and resources for analysis:**
  - `trend(scorecard | fact, group_by, range)`
  - `rollup(scorecard, by)`
  - `distribution(field, by)`
  - `history(ref, range)`
  - `as_of(ref, time)`

  These return compact tables designed to be dropped straight into a generated chart.
- **Permissions apply as usual.** Every MCP and API answer is shaped by the caller's ReBAC
  permissions and facet labels, including declassified summaries with minimum-cardinality rules
  ([authorization.md](../authorization.md) §4.2). An artifact built by one person and viewed by another
  runs its MCP calls as the viewer, in the viewer's own client.
- **Agent Skills** ship in the box, e.g. "build a scorecard dashboard", "initiative report",
  "ownership gaps by team". They teach agents which MCP tools to call and how to chart the results.

## 3. Operator-defined views (code)

```ts
// views/security-overview.tsx (operator-owned, in the portal repo)
export const meta = { title: "Security posture", owner: "group:default/security", slots: ["nav:security", "team-home"] };
export default function SecurityOverview() { /* uses @toolbox/ui-sdk hooks; same permission shaping as the API */ }
```

- Views are built by `toolbox build`, snapshot-tested with fixtures as two different principals, and
  are meta-catalog entities with owners.
- Views read data through the same typed hooks as the API (`useEntities`, `useScores`,
  `useScoreHistory`, `useGraph`). They have no extra privileges.

## 4. Reports

Scheduled reports (a weekly team digest, a monthly initiative report) are **notification templates**
rendered per recipient with the recipient's permissions ([notifications.md](./notifications.md)).
They are not dashboards.

## 5. Sources

- Cursor Canvas: [cursor.com/blog/canvas](https://cursor.com/en/blog/canvas)
- Claude Artifacts:
  [What are artifacts](https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them) ·
  [Publishing and sharing](https://support.claude.com/en/articles/9547008-publishing-and-sharing-artifacts)
