# Canvases: Dashboards the Way Agents Build UI

*Status: design. Addresses red-team gap A2 (dashboards, reports, trends). Inspired by Cursor Canvas
and Claude Artifacts. Related: [plan.md](../plan.md) §4.5, [authorization.md](../authorization.md),
[sdlc.md](./sdlc.md), [meta-catalog.md](./meta-catalog.md).*

## 1. Why canvases instead of a dashboard builder

The market has converged on a new pattern for agent-era UI.

**Cursor Canvas** (Cursor 3.1, April 2026):
- Agents generate interactive dashboards and mini-tools from a **first-party React component library**
  (tables, charts, diagrams, boxes), rendered as durable artifacts beside the chat.
- **Skills** define canvas types.
- **Design Mode** (3.7) lets users annotate elements for the agent to revise.
- **Shared canvases** are read-only links. Cloud agents can build canvases too.

**Claude Artifacts:**
- Model-generated HTML/React runs in a **sandboxed iframe** isolated by CSP.
- Artifacts can call the model ("AI-powered artifacts"), use MCP, keep **persistent storage**, and be
  published or shared.

Port, Backstage and the others offer **widget builders**, which are configuration in a database. That
clashes with our non-goals and is not how agents work. Toolbox instead treats a dashboard as **a small
program over a typed, permission-scoped data API**, which is exactly what agents are good at writing.

## 2. Concepts

| Concept | What it is | Where it lives |
|---|---|---|
| **Canvas** | A TSX (or HTML) module rendered in a sandboxed iframe, built from `@toolbox/canvas` components, reading data only through the canvas bridge | Draft: state (per user). Promoted: code in the portal repo |
| **Query manifest** | The canvas's declared data needs (`queries`): kinds, filters (CEL), facets, history ranges | Exported by the canvas module and validated at build time |
| **Canvas bridge** | A `postMessage` API between the iframe and the host. The host executes queries **as the viewer** | Toolbox UI host + API |
| **Slot** | Where a promoted canvas appears: nav page, portal home, team home, entity tab (per kind), initiative page | Declared in code |
| **Canvas skill** | A template plus guidance for a canvas type (e.g. "scorecard roll-up", "initiative burndown", "ownership coverage") | Shipped by Toolbox and by packages |

## 3. Lifecycle: ephemeral by default, code when it matters

```
 agent / human asks ──► DRAFT canvas (state, private, expires) ──► share link (live, viewer's permissions)
                              │
                              └─ "Promote" ──► PR adds canvases/<name>.tsx + fixtures + snapshot test
                                                 └─ review → build → deploy → canvas in slots (code)
```

- **Drafts** are personal scratch work, like a chat artifact.
  - They are created through MCP (`canvas_create`) or the UI "Ask" panel.
  - They are stored as state with an owner and an expiry (default 30 days, set in code).
  - They are *not* portal behaviour, so the "no config in the DB" non-goal holds.
- **Promotion** turns a draft into code. `canvas_promote` opens a PR containing:
  - the module
  - a fixture-backed snapshot test
  - an `owner`, which is required (see [meta-catalog.md](./meta-catalog.md))
  - the slot binding
  Organisation-wide dashboards are therefore reviewed, versioned, tested and owned like everything else.
- **Design-mode loop:** users comment on elements ("make this a stacked bar", "group by tier").
  `canvas_feedback` returns the source plus annotations to the agent, which revises and re-previews.

## 4. Data access and security (the important part)

1. **No network from the iframe.** It runs with `sandbox="allow-scripts"` (no same-origin), a CSP of
   `default-src 'none'` with only inline/bundled script, and no `connect-src`. All data flows through
   the bridge.
2. **The bridge executes queries as the viewer.** Every query passes the same ReBAC checks and facet
   shaping as the API ([authorization.md](../authorization.md) §4.4). One canvas therefore shows
   different data to different people, and a canvas can never read more than its viewer could through
   the API.
3. **Declared queries only.** The host rejects queries outside the canvas's manifest. This bounds cost,
   enables caching and precomputation, and lets `toolbox plan` know which kinds, facets and scorecards
   each promoted canvas depends on. Canvases therefore appear in the meta-catalog dependency graph.
4. **Sharing never leaks the sharer's view.** Shared links **re-execute live with the recipient's
   identity**. A *static snapshot* (for reports) can only be published to an audience if every member
   of the audience can read every facet in it. Toolbox checks this with ListUsers over the snapshot's
   facets at publish time and refuses otherwise.
5. **Budgets:** per-canvas query cost limits, row caps and rate limits, all enforced host-side.
6. **AI inside canvases** (later phase): a canvas may call `ask()`, which routes to the portal's
   assistant with the viewer's identity and the same MCP tools. It never sees a model key.

## 5. The component library and data API

`@toolbox/canvas` (TypeScript, versioned with Toolbox). Its design goal is a small, opinionated set that
agents use correctly on the first try:

- **Layout:** `Page`, `Grid`, `Section`, `Tabs`, `Callout`
- **Data display:** `Table` (sortable, column facets), `Stat`, `Sparkline`, `LineChart`, `BarChart`,
  `Heatmap`, `Funnel`, `Timeline`, `Graph` (catalog-graph viewer), `EntityCard`, `ScoreBadge`,
  `ScorecardMatrix` (entities × rules), `InitiativeBurndown`
- **Interaction:** `Filter`, `Search`, `EntityPicker`, `ActionButton` (runs a Toolbox action through
  the normal action flow, with dry-run and approvals)
- **Data hooks:** `useEntities(query)`, `useEntity(ref)`, `useScores(query)`,
  `useScoreHistory(query, range, groupBy)`, `useFacts(query)`, `useGraph(ref, opts)`, `useRuns(query)`,
  `useMeta(kind)`. Hooks are typed from the portal's generated schema.

Every component follows the dataviz guidance shipped with the library: accessible colours, light and
dark themes, WCAG 2.2 AA keyboard and screen-reader support.

```tsx
// canvases/security-overview.tsx — promoted canvas
import { Page, Grid, Stat, ScorecardMatrix, LineChart, useScores, useScoreHistory } from "@toolbox/canvas";

export const meta = {
  title: "Security posture",
  owner: "group:default/security",
  slots: ["nav:security", "team-home"],
  queries: {
    current: { scorecard: "security", kind: "Service", facets: ["summary"] },
    trend:   { scorecard: "security", history: "180d", groupBy: "owner" },
  },
};

export default function SecurityOverview({ team }: { team?: string }) {
  const current = useScores("current", { owner: team });
  const trend = useScoreHistory("trend", { owner: team });
  return (
    <Page title="Security posture">
      <Grid cols={3}>
        <Stat label="Services at gold" value={current.count({ level: "gold" })} />
        <Stat label="Red tier-1 services" value={current.count({ level: "red", "spec.tier": "1" })} tone="critical" />
        <Stat label="Δ 30 days" value={trend.delta("30d")} />
      </Grid>
      <LineChart data={trend.series()} x="day" y="levelShare" series="level" />
      <ScorecardMatrix data={current} />   {/* details columns appear only where the viewer can see details */}
    </Page>
  );
}
```

## 6. History: the store that makes trends possible

Trends need history that a catalog alone doesn't keep. Toolbox adds a **compact daily snapshot store**:

| Table | Grain | Notes |
|---|---|---|
| `score_daily` | (day, scorecard, rule, entity) → outcome | Run-length encoded: one row per *change*, not per day. Includes the entity's **owner at that time**, so team trends survive reorganisations |
| `rollup_daily` | (day, scorecard, group, level) → counts | Precomputed at end of day for fast leadership views |
| `fact_history` | (entity, fact, ts) → value | Bounded per fact (`history: "180d"` in code) |
| `catalog_daily` | (day, kind, owner) → counts, ownership coverage | Supports meta and ownership trends |

Retention is configured in code (plan §6). History follows the same facet rules: a team-level rollup is
a *summary* facet, while rule-level history for an entity is a *details* facet.

## 7. Reports (scheduled canvases)

A promoted canvas can declare a schedule and audience, for example a weekly team digest or a monthly
initiative report. The scheduler renders it **per recipient** with each recipient's permissions.
Delivery goes through the notification system ([notifications.md](./notifications.md)), via email,
GitHub, or a webhook. Slack is out of scope for now.

## 8. Agents and MCP

| Tool | Purpose |
|---|---|
| `canvas_create(title, source, queries)` | Create a draft; returns an id and a preview URL |
| `canvas_preview(id)` | Render headlessly and return a screenshot, a DOM/text summary, query errors and empty-state warnings, so the agent can iterate without a human |
| `canvas_update(id, source)` / `canvas_feedback(id)` | Edit loop, including design-mode annotations |
| `canvas_promote(id, owner, slots)` | Open a PR with module + fixtures + snapshot test |
| `canvas_list(slot?)` | Discover existing canvases before creating duplicates |

Canvas skills ship as Agent Skills, so an agent asked for "a dashboard of tier-1 services failing
security, by team" starts from the right template and the right queries.

## 9. Testing

- **Snapshot tests:** `toolbox test` renders each promoted canvas against fixtures in headless Chromium
  and compares DOM text, plus an optional image diff.
- **Query validation:** at build time, every manifest query is type-checked against kinds and facets.
  Removing a field used by a canvas is a build error, and `plan` lists affected canvases.
- **Permission tests:** render as two fixture principals (e.g. owner vs. non-owner) and assert which
  details facets are present ([authorization.md](../authorization.md) Level 3).

## 10. Built-in canvases

These are promoted canvases shipped by Toolbox, owned by the portal's platform team, and overridable:

- Portal home
- Team home
- Scorecard overview
- Initiative burndown
- Ownership coverage
- **Portal health / meta catalog** ([meta-catalog.md](./meta-catalog.md))
- Connector freshness
- Agent activity

## 11. Phasing

| Phase | Scope |
|---|---|
| 2 | History store; `@toolbox/canvas` core components; promoted canvases in slots; built-in canvases for scorecards and ownership; snapshot tests |
| 3 | Drafts + MCP tools + preview loop; `ActionButton`; promotion PRs |
| 4 | Scheduled reports; shared links; design-mode feedback |
| 5 | `ask()` inside canvases; canvas skills from packages |

## Sources

- Cursor Canvas: [cursor.com/blog/canvas](https://cursor.com/en/blog/canvas) ·
  [shared canvases](https://aicatchup.com/news/cursor-shared-canvases-team-url) ·
  [Cursor 3.7 design mode](https://codepipelines.pages.dev/news/cursor-37-canvas-design-mode.html)
  (secondary coverage for dates)
- Claude Artifacts:
  [What are artifacts](https://support.claude.com/en/articles/9487310-what-are-artifacts-and-how-do-i-use-them) ·
  [Publishing and sharing](https://support.claude.com/en/articles/9547008-publishing-and-sharing-artifacts)
