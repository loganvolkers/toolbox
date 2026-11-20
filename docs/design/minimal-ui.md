# Minimal UI (v1)

*Status: design, 2026-11-20 (decisions D56–D58). The built-in web UI that ships in the base image.
Builds on [frontend.md](./frontend.md) (UX bar, design system, Storybook) and the UI extension
contract in [v1-spec.md](../v1-spec.md) §7.6. Planned here because it changes the API.*

> **Implementation status (M4 done):** the read pages (home, entity, scorecard, ops) over the
> `/v1/views/*` endpoints, embedded in the binary with a strict CSP, with refresh and redaction
> markers. **Not yet:** the `/v1/events` SSE stream (the UI polls `/v1/snapshot`), generated action
> forms and approvals (M5), `@toolbox/ui` and Storybook stories.

---

## 1. Conclusion

The minimal UI is a **read-mostly single-page app** for people who aren't using an agent: find
things, see who owns them and why, see standards and how to fix them, refresh stale data, and run,
approve and follow actions. It is built with the default kit (React + `@toolbox/ui`), compiled into
the Toolbox release and served by the server. **It mounts through the same UI extension contract as
operator bundles**, so the contract is exercised from day one.

Planning it exposed four API needs that v1-spec lacked:

| # | Need | Decision |
|---|---|---|
| 1 | **Composite view endpoints**, so each page is one request within budget (GraphQL is deferred) | D56 |
| 2 | **Approvals API**, and approvals stay **human-only** in v1 (not exposed to agent tokens or MCP) | D57 |
| 3 | **A form-renderable subset of JSON Schema** for action inputs, enforced by `toolbox validate`, so every action gets a usable generated form | D58 |
| 4 | **A snapshot event stream** (SSE) so pages show "newer data available" and refresh results without polling | part of D56 |

## 2. Pages

| Route | Page | For | Data (one request) | Interactions |
|---|---|---|---|---|
| `/` | **Home** | Everyone | `GET /v1/views/home`: my teams, their entities with failing rules, my runs, approvals waiting for me | Search box; jump to items |
| `/search?q=` | **Search** | Everyone | `GET /v1/entities?q=` (Postgres full-text) | Filter by kind; keyboard navigation |
| `/c/:kind` | **Catalog list** | Everyone | `GET /v1/entities?kind=&filter=&fields=` | Column choice from the kind's fields; CEL filter box with examples; URL holds state |
| `/e/:id` | **Entity page** | Everyone | `GET /v1/views/entity/{id}`: fields with provenance, ownership evidence, relations (both directions, capped), scorecard summaries and details (facet-shaped), available actions, snapshot and per-connector freshness | **Refresh** (command → progress → new snapshot); edit **native** fields inline (revision, reason); open an action |
| `/e/:id/scorecards/:name` | **Rule details** | Owners (details facet) | Included in the entity view | Fix text, observed values, `explain` link |
| `/s/:name` | **Scorecard** | Everyone (summaries) | `GET /v1/views/scorecard/{name}`: level distribution, entities by level, worst rules | Filter by team |
| `/e/:id/actions/:action` | **Action** | Permitted users | `GET /v1/actions/{name}?entity=` + options + previews | Generated form (or an operator bundle in the `action:` slot), live preview, confirm, approval notice |
| `/runs/:id` | **Run** | Requester, approvers, owners | `GET /v1/runs/{id}` + SSE events | Timeline including "checking the result"; cancel; final state with snapshot |
| `/runs` | **My runs** | Everyone | `GET /v1/runs?mine=true` | — |
| `/approvals` | **Approvals** | Approvers | `GET /v1/approvals?mine=true` | Approve or reject with a comment (D57) |
| `/ops` | **Operations** | Portal admins (`operator` facet) | `GET /v1/views/ops`: snapshot lag and recompute time, connectors (registration, heartbeat, last sync, errors), pending commands, diagnostics (map errors, unverified claims, orphans, held deletes) | Trigger resync; open diagnostics |

**Not in the minimal UI (v1):** graph explorer, docs reader, dashboards (users build them over MCP,
D19), editing git- or connector-sourced fields (an explanation and a link to the source instead), and
admin configuration of any kind (D2).

## 3. Behaviour that must hold on every page

- **Snapshot honesty:** every page shows its snapshot "as of" time. When the event stream reports a
  newer snapshot, a non-intrusive "Newer data — update" appears; after a refresh or run, the page
  moves to the snapshot the command or run returned (read-your-writes, P8).
- **Redaction:** a hidden facet renders as a placeholder that says which relation is needed and who
  holds it (for example "Visible to members of group:payments"), never as an empty value (D15).
- **Provenance:** every field has a hover or disclosure showing its source and, for losing sources,
  the value they had (P3). Ownership shows its status and evidence (claim, CODEOWNERS rule).
- **Actions match the agent experience:** the same labels, disabled reasons, preview text and run
  states as MCP ([action-runs.md](./action-runs.md)).
- **Quality bar** from [frontend.md](./frontend.md) §1: keyboard-first, WCAG 2.2 AA, deep-linkable,
  dark mode, skeletons instead of spinners.

## 4. Generated action forms (D58)

Action inputs must use a **form-renderable subset** of JSON Schema, checked by `toolbox validate`:

| Allowed | Rendered as |
|---|---|
| `string` (+ `minLength`, `maxLength`, `pattern`, `format: date | email | uri | entity-ref`) | Text field; `entity-ref` becomes an entity combobox with search |
| `string` + `enum` or `enumFrom` | Combobox with labels, descriptions and disabled reasons from the options endpoint |
| `integer` / `number` (+ bounds) | Number field |
| `boolean` | Switch |
| `array` of an allowed scalar (+ `minItems`, `maxItems`, `uniqueItems`) | Multi-select or tag input |
| `object` one level deep, with `required` | Field group |
| `description`, `title`, `default`, `examples`; `x-toolbox-widget` for a registered widget | Help text, defaults, placeholders; field widgets (tier 2) |

Not allowed in v1: `oneOf`/`anyOf`/`allOf`, `if/then/else`, nested arrays, `$ref`, free-form
objects. Actions that need more use an operator UI bundle (tier 3).

## 5. Packaging and security

- Built with Vite in the Toolbox release pipeline and embedded in the binary; operators don't need
  Node unless they build their own bundles. The browser code is TypeScript, counted as the one
  front-end language outside D45's server-side budget (clarified by D56).
- Same-origin. Sessions come from the identity-aware proxy or OIDC. Mutations require a CSRF header
  token, and cookies are `SameSite=Strict`.
- Strict CSP: scripts only from the portal origin; operator bundles load from `/ui/bundles/<name>/`.
- The server enforces every permission; the UI only hides what the API already withholds.

## 6. API additions (into v1-spec §7.2)

| Method and path | Purpose |
|---|---|
| `GET /v1/views/home` | Home composite for the caller |
| `GET /v1/views/entity/{id}` | Entity page composite (facet-shaped; relations capped with cursors) |
| `GET /v1/views/scorecard/{name}` | Scorecard composite |
| `GET /v1/views/ops` | Operations composite (`operator` facet) |
| `GET /v1/runs?mine=true` | The caller's runs |
| `GET /v1/approvals?mine=true` · `POST /v1/approvals/{id}/decision` | Approvals waiting for the caller; approve or reject with a comment (human sessions only, D57) |
| `GET /v1/events` (SSE) | Snapshot published, command completed and run transition events visible to the caller |

View endpoints are compositions of the same service-layer calls the other endpoints use, so they add
no new authorization paths. Their response shapes are part of the generated OpenAPI and the typed
client that operator bundles also use.

## 7. Budgets and tests

| Measure | Budget |
|---|---|
| View endpoint p95 (reference scale) | ≤ 150 ms |
| Entity page interactive (cold, fast 4G profile) | ≤ 2.5 s |
| JS shipped for the shell (gzip) | ≤ 200 KB, routes code-split |

- **Stories:** one per page × persona × state, with fixtures generated from the reference portal's
  `expected/` outputs, so UI and engine share data semantics. Story-based tests check a11y and
  interactions ([frontend.md](./frontend.md) §5).
- **End-to-end:** Playwright against `toolbox dev` running the reference portal, covering A3/A4
  (redaction vs details), A6–A9 (action flow including a failed verification) and A11 (refresh).

## Sources

[frontend.md](./frontend.md), [action-runs.md](./action-runs.md),
[connector-control.md](./connector-control.md), [v1-spec.md](../v1-spec.md), and the
[reference portal](../../examples/reference-portal/README.md) acceptance scenarios.
