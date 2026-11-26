# Frontend: Delightful Self-Service UIs Defined in Code

*Status:* Astro and Vue are the maintained frontend stack (D01M8A5518E6GYEY9EG8Z7FPXK2).

## Current architecture

The box's first-party UI uses Astro and Vue under one PNPM workspace. The hosted portal builds into
the Rust release. The static report is the `@seikika/astro-report` integration. Both use
`@seikika/ui` for the theme, page frame, entity overview, and scorecard presentation. Customize
through slots and props. Publish the component and page stories with the Astro docs.

Operators compose the shared components in their own Astro builds. The hosted UI gets
facet-shaped data and uses the existing Rust APIs for refresh and actions. The static report uses
a snapshot approved for its audience. Loading independent UI modules into the hosted portal is
part of the proposed extension design below.

The sections below record the earlier self-service design. Framework recommendations there were
superseded by [the Astro and Vue decision](../decisions/D01M8A5518E6GYEY9EG8Z7FPXK2.md).

## 1. Designer's view: the ideal "add an approved GitHub App" experience

*Written as if knowing nothing about Seikika: what should this feel like?*

**Principles:**
- **Start from where the user is.**
- **Show consequences before commitment.**
- **Explain every "no".**
- **Never make the user hunt for an id.**
- **Make the safe path the fast path.**

**Entry points.** The same flow is reachable from four places, pre-filled with context:
- an **"Apps" tab** on the repository page
- the **command palette** (⌘K → "Add app to…")
- a link in the repo's GitHub dashboard issue
- the user's coding agent

**One screen, progressive disclosure:**

1. **Repository.** Pre-filled from context. Otherwise a searchable combobox of repositories *the user
   can act on*. Each option shows:
   - `owner/name`
   - owning team
   - tier badge
   - visibility
   - last activity

   Repositories the user can't act on appear dimmed, with the reason ("You're not an admin. Ask
   @payments"). There's **multi-select** for team-wide rollouts.
2. **App.** A rich, searchable picker:
   - Searchable by name, publisher **and capability** ("dependency updates", "code scanning").
   - Grouped: *Recommended for your team*, *Popular in acme*, *All approved*.
   - Each option shows:
     - icon, name, publisher and one-line purpose
     - steward team
     - "on 37 repos in acme"
     - **permission chips** colour-coded by risk (read / write / admin)
     - an **eligibility badge for this repo**: *Ready*, *Needs security review*, or *Not eligible*
       (with the reason)
   - Already-installed apps show *Installed ✓* with a "Manage" link.
   - The list ends with **"Can't find it? Request a new app."**
3. **Review.** A live panel that updates as choices change; no "preview" button:
   - **Plain language first:** "Renovate will be able to *read and write code* and *open pull
     requests* in `payments-api`". Details expand on demand.
   - **What changes:** current versus after (37 → 38 repos).
   - **Approval path:** "No approval needed", or "Security will review — usually within 4 h", with
     reviewer avatars.
   - **Identity:** "This will run as you (Alice)."
   - **Justification:** appears only when approval is required, with suggested reasons.
4. **Submit** becomes an **inline progress timeline**: requested → approved → added in GitHub →
   verified. It updates live and stays on the page after navigation, with notifications.
   - On success: "Undo" (removes the app) for a short window, plus "Copy link" to share.
   - On failure: what happened, whether anything changed, and a one-click retry.

**Quality bar:**
- Search results in under 100 ms (feels instant), with skeletons rather than spinners.
- Keyboard-first: every control reachable, with type-ahead.
- WCAG 2.2 AA: screen-reader-announced loading, errors and option counts.
- Deep-linkable URL state.
- Responsive.
- Dark mode.
- Copy reviewed like product copy.
- Error states that say how to fix the problem.
- **Consistent with the agent experience:** the same labels, reasons and dry-run text appear in the
  MCP tool's responses.

## 2. Can auto-generated forms reach that bar?

| Requirement | Generated JSON-Schema form (Backstage Scaffolder, Port) | Verdict |
|---|---|---|
| Structure, required fields, validation | Strong (RJSF / JSON Schema) | ✓ |
| Searchable dropdowns over catalog entities | Backstage: custom field extensions (React code). Port: entity selectors with `dataset` rules and jq | ◐ (needs code, or limited filters) |
| **Rich options** (icons, two-line labels, badges, risk chips, grouping) | Not expressible in JSON Schema; needs custom widgets | ✗ by default |
| **Per-entity eligibility with "disabled because…"** | Port can filter datasets, but not explain exclusions. Contextual and dynamic defaults are open feature requests on Port's roadmap | ✗ |
| **Live consequence preview** (dry-run panel) | Not a form concept | ✗ |
| Conditional reveal (justification only when needed) | Port `visible` + jq; JSON Schema `if/then`. Possible but awkward, and it can't depend on a server dry-run | ◐ |
| Progress timeline, undo, multi-select rollouts with per-repo status | Outside form scope | ✗ |
| Validation enforced server-side | Port has a public roadmap request saying dataset filters and required fields are **UI-side only** | Seikika enforces everything server-side regardless |

**Conclusion:**
- **Generated forms are the right *baseline*.** Every action gets a decent, accessible form for free,
  and most long-tail actions never need more.
- **They can't deliver the flagship experiences** that make a portal feel great. Those need code,
  defined by operators, on a shared design system, without giving up server-side guarantees.

## 3. The architecture: three tiers of action UI, one design system

| Tier | What the operator writes | Result |
|---|---|---|
| **1. Generated** | Nothing beyond the action's input schema | A form built from `@seikika/ui` widgets. `enumFrom` fields automatically become **rich entity comboboxes** with server-side search and disabled reasons. The dry-run panel is included by default |
| **2. Field widgets** | `ui: { app: AppPickerOption }`: a TSX component for one field's option rendering or input | Custom option cards and chips inside the generated layout |
| **3. Custom action UI** | `ui: "./ui/install-approved-app.tsx"`: a full React component | Any layout and interaction, built from the same hooks and components |

### 3.1 Design system and hooks (`@seikika/ui`, maintained in this repo)

- **React 19 + TypeScript.**
- **React Aria Components** provide behaviour and accessibility. Their ComboBox supports async
  loading and infinite scrolling, with correct ARIA semantics.
- **Design tokens** are CSS variables, with themeable light and dark modes.
- **Primitives:** Button, ComboBox, Select, Dialog, Toast, Tabs, Table, Badge.
- **Domain components:**
  - `EntityCombobox`, `EntityCard`, `OwnerBadge`, `TierBadge`
  - `PermissionChips`, `ScoreBadge`
  - `DryRunPanel`, `ApprovalPath`, `RunTimeline`
  - `RedactedField` (renders the `REDACTED` marker with "ask @team")
  - `CommandPalette`
- **Typed hooks:** these are presentation only, and every decision is made by the server.
  - `useAction(id, { entity })` returns `{ inputs, setInput, options(field, query), dryRun, submit, run }`.
  - `options()` calls the **option-search API** (§3.3) with debouncing and caching.
  - `dryRun` re-runs automatically on input change (debounced, cancellable).
  - `run` streams the run timeline over SSE.
- Read context for richer layouts uses the composite view endpoints ([minimal-ui.md](./minimal-ui.md) §6); GraphQL is deferred (D43).

```tsx
// portal repo: ui/install-approved-app.tsx (tier 3, operator-owned, reviewed like any code)
import { useAction, EntityCombobox, AppOptionCard, DryRunPanel, ApprovalPath, RunTimeline, Button } from "@seikika/ui";

export default function InstallApprovedApp({ entity }: { entity?: string }) {
  const a = useAction("install-approved-app", { entity });
  return (
    <a.Form>
      <EntityCombobox field="entity" label="Repository" kinds={["Repository"]} multiple />
      <a.Combobox field="app" label="App" renderOption={(o) => <AppOptionCard option={o} />}
                  groupBy="group" emptyAction={{ label: "Request a new app", action: "request-new-app" }} />
      <DryRunPanel result={a.dryRun} />
      <ApprovalPath result={a.dryRun} />
      {a.dryRun.data?.approvalRequired && <a.TextArea field="justification" suggestions />}
      <Button onPress={a.submit} isDisabled={!a.canSubmit}>Add app</Button>
      {a.run && <RunTimeline run={a.run} undo="remove-app" />}
    </a.Form>
  );
}
```

### 3.2 Packaging, loading and security
- Operator UI code lives in the portal repo (`ui/`).
- `seikika build` bundles it with **Vite 8**, whose default bundler since March 2026 is Rolldown,
  Rust-based. The bundle is code-split per action and view, and embedded in the image.
- There is **no runtime loading of remote code**, so the CSP stays strict.
- **Trust:** the code is operator-authored and reviewed, so it runs same-origin in the SPA. Users
  can't author UI ([dashboards.md](./dashboards.md)).
- **The UI is never a security boundary:**
  - authorization, eligibility, approvals and validation are enforced server-side
  - the option-search API and dry-run are permission-shaped
  - submissions are re-validated at execution

### 3.3 Server capabilities this requires (and the current architecture already supports)

| Capability | Status | Detail |
|---|---|---|
| Entity-dependent options (`enumFrom`) | Planned ([action-entities.md](./action-entities.md) §5b) | **Add rich option metadata in code:** `label`, `description`, `icon`, `badges`, `group`, `searchOn`, `disabledWhen: { expr, reason }`, all computed in the recompute pipeline, so options stay current |
| **Option-search API** | **New** | `GET /v1/actions/{id}/options/{field}?entity=…&q=…&cursor=…` returns ranked, permission-shaped options with disabled reasons. The same data is exposed to agents via an MCP `action_options` tool, so agents and humans see identical labels and reasons |
| Dry-run | Planned (§4.3 of plan) | **Add a latency SLO** (p95 ≤ 150 ms) and caching by `(inputs hash, snapshot id)`, so live preview is viable |
| Run timeline | Planned (SSE, durable workflows) | Step events map to timeline states |
| Read context | Composite view endpoints (D56) | View and action UIs use the typed client generated from the portal's OpenAPI |

**Validated against the current architecture: nothing here conflicts.** Dry-run, `enumFrom`, durable
runs with SSE, composite view endpoints, operator-only code-defined UI and no per-portal native
codegen all already exist in the design. The additions are the option-search API with rich option
metadata, a dry-run latency SLO, the `ui` field on actions, and committing to React (§4).

## 4. Framework choice: why not be framework-agnostic?

| Option | Benefits | Costs for Seikika |
|---|---|---|
| **Astro with framework islands** | Ships little JS; mix React, Vue, Svelte per island; excellent for content sites | Islands aren't one component tree, so React context (router, auth, query cache, theming) doesn't cross islands, and Astro's docs point to a separate store library (nanostores, with per-framework adapters) for shared state. **Every framework doubles** the runtime, the design-system implementation, Storybook renderers, tests, and agent context. Partial hydration helps content pages, but the portal is an authenticated, highly interactive app |
| **Web Components as the extension contract** | Framework-neutral boundary; React 19 supports custom elements | Form participation (ElementInternals), shadow-DOM styling and SSR add friction. The design system would be built twice (WC + React) or as WCs that every operator wraps. Agents write React far more fluently |
| **One framework: React + TypeScript** (recommended) | One design system, one test stack, one Storybook, one mental model; the largest ecosystem; React Aria's accessibility depth; agent fluency | Operators preferring Vue or Svelte can't use them natively |

**Decision:**
- **React (TSX) is the single first-party and operator UI framework.**
- **Escape hatch:** a `<seikika-island>` mount that can host a Web Component, for teams with existing
  non-React widgets. It's documented as outside design-system guarantees, for interop rather than
  primary use.
- This refines earlier wording ("Web Components / TSX"). The Lightning Web Components lesson was
  *standard, widely known web technology*, and for an app UI in 2026 that means React and TypeScript
  on web standards.

## 5. Storybook as a product surface, not just a component gallery

**Why:** Storybook stories are executable specifications. With Storybook 10:
- Stories run as **Vitest** tests: `play` interaction tests in browser mode via the Vitest addon. The
  older test-runner is reported as deprecated since Storybook 9.
- The **a11y addon** fails tests on violations.
- Storybook 10.3 ships **an MCP server (React) so AI agents can reuse real components, write stories
  and run focused tests.**

### 5.1 What gets stories

| Level | Examples | Owner |
|---|---|---|
| Tokens and primitives | Button, ComboBox, Badge | Maintainers |
| Domain components | EntityCombobox, PermissionChips, DryRunPanel, RunTimeline, RedactedField | Maintainers |
| **Generated action forms** | One story *per action* is generated automatically from its schema, against fixtures | Generated (operators get them free) |
| **Page-level** | EntityPage(Service), RepositoryAppsTab, InstallApprovedApp (empty, eligible, needs-approval, not-eligible, running, failed, undo) | Maintainers (built-in), operators (custom) |
| **Personas** | Every page story renders for owner / non-owner / security / admin / agent-on-behalf-of, via a toolbar switcher | Generated |

### 5.2 Fixture data with real semantics, without a backend

- `seikika ui fixtures` runs the **real engine** in process over the portal's fixtures and
  `seikika-synth` data:
  - per persona
  - for every view endpoint, action option search and preview
  - recording the JSON responses
- Stories serve those recordings through generated request handlers (MSW-style).
- **Effect:** stories render **real permission shaping and REDACTED markers** with zero backend at
  story time. Recordings regenerate incrementally when the manifest or fixtures change.
- For exploratory work, stories can instead point at a live `seikika dev` backend.

### 5.3 Tests from stories

- **Interaction:** `play` functions as Vitest browser tests. Examples: type "renov", pick Renovate, see
  the dry-run, justification appears only when approval is required.
- **Accessibility:** a11y checks at `error` level for primitives and pages.
- **Visual regression:** Playwright screenshots of affected stories, open source by default. Chromatic
  is optional.
- **Portable stories** are reused in unit and E2E tests, so there's no duplicate setup.

```tsx
// ui/install-approved-app.stories.tsx (page-level story + interaction test)
import { expect, userEvent, within } from "storybook/test";
import InstallApprovedApp from "./install-approved-app";
export default { component: InstallApprovedApp, parameters: { persona: "owner", fixtures: "acme-small" } };

export const NeedsSecurityApproval = {
  args: { entity: "repository:default/payments-api" },
  play: async ({ canvasElement }) => {
    const c = within(canvasElement);
    await userEvent.type(await c.findByRole("combobox", { name: "App" }), "snyk");
    await userEvent.click(await c.findByRole("option", { name: /Snyk Security/ }));
    await expect(await c.findByText(/Security will review/)).toBeVisible();
    await expect(c.getByRole("textbox", { name: "Justification" })).toBeRequired();
  },
};
```

## 6. Fast loops for maintainers and operators (SDLC layers 1–2)

| Loop | Maintainers (Seikika repo) | Operators (portal repo) |
|---|---|---|
| Edit → see | Vite HMR with React Fast Refresh in Storybook and the app | `seikika dev` runs Vite as a child of the daemon; API calls go to the in-process engine; HMR for `ui/` |
| Type check | TypeScript 7 native `tsc` (8–12× faster) in the background | Same, against **generated portal types** (kinds, actions, view endpoints) |
| Tests | Vitest browser mode, **affected stories only** (Vitest's module graph + changed files), in parallel | `seikika test --affected` includes affected stories and generated action-form stories |
| Visual and a11y | Affected stories on PR; all on the merge queue | Same, via the `seikika/test` Action |
| Fixtures | `seikika ui fixtures` incremental regeneration | Same, from portal fixtures |
| Agents | Storybook MCP (component reuse, story writing, focused tests) + `seikika mcp --dev` | Both exposed through the operator agent plugin. The `add-action-ui` skill: write the story first, then the component |

**Additional dev-loop SLOs** ([developer-velocity.md](./developer-velocity.md) §6):

| Loop | Target (p50) |
|---|---|
| UI edit → HMR update | ≤ 200 ms |
| Storybook cold start (Vite 8) | ≤ 5 s |
| Affected story tests after editing one component | ≤ 10 s |
| Option search (server) | p95 ≤ 100 ms |
| Dry-run (server) | p95 ≤ 150 ms |

## 7. Changes to the plan

1. **Commit to React + TypeScript** for first-party and operator UI. A Web Component island is the
   documented escape hatch.
2. **`@seikika/ui`** design system (React Aria + tokens + domain components + typed hooks) and
   **Storybook 10** in this repo. They are released with Seikika and consumed by portals.
3. **Actions gain `ui`:**
   - generated (default)
   - field widgets
   - a custom component

   `enumFrom` gains rich option metadata (`label`, `description`, `icon`, `badges`, `group`,
   `searchOn`, `disabledWhen`).
4. **New option-search API** and MCP `action_options` tool. Dry-run latency SLO and caching.
5. **`seikika dev` runs Vite**; `seikika storybook` and `seikika ui fixtures`; generated stories per
   action and per persona.
6. **Operator agent plugin:** an `add-action-ui` skill; the Storybook MCP alongside the authoring MCP.
7. **Phasing:**
   - **Phase 1:** `@seikika/ui` primitives, Storybook, generated action forms, page stories for the
     minimal UI.
   - **Phase 3 (actions):** option-search API, rich `enumFrom`, field widgets, custom action UIs,
     RunTimeline, the install-approved-app reference UI.

## Sources

- Storybook:
  [Storybook 10.3 (MCP for React, CSF Factories)](https://storybook.js.org/blog/whats-new/storybook-10-3/) ·
  [Accessibility testing (Storybook 9)](https://storybook.js.org/docs/9/writing-tests/accessibility-testing.md)
- Port action forms:
  [Advanced input configurations](https://docs.port.io/actions-and-automations/create-self-service-experiences/setup-ui-for-action/advanced-form-configurations) ·
  [Entity inputs](https://www.docs.port.io/actions-and-automations/create-self-service-experiences/setup-ui-for-action/user-inputs/entity)
  - Roadmap requests:
    [backend enforcement of UI validations](https://roadmap.port.io/ideas/p/enforce-ui-level-input-validations-at-the-backend-api-level-for-self-service-act) ·
    [dynamic filters and defaults](https://roadmap.getport.io/ideas/p/entity-and-user-team-pickers-should-allow-dynamic-filters-and-defaults) ·
    [repeatable sections](https://roadmap.port.io/ideas/p/add-structured-list-of-objects-form-inputs-repeatable-sections-for-selfservice-a)
- Backstage Scaffolder:
  [writing custom field extensions](https://backstage.io/docs/features/software-templates/writing-custom-field-extensions) ·
  [migrating to RJSF v5](https://backstage.io/docs/features/software-templates/migrating-to-rjsf-v5)
- Astro: [sharing state between islands](https://docs.astro.build/es/core-concepts/sharing-state)
- React Aria: [ComboBox](https://react-spectrum.adobe.com/react-aria/ComboBox.html) ·
  [useComboBox](https://react-spectrum.adobe.com/react-aria/useComboBox.html)
- Vite: [Vite 8 announcement (Rolldown default)](https://vite.dev/blog/announcing-vite8)
- TypeScript 7: [announcement](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)
