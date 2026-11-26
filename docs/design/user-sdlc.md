# User SDLC (Layer 4): Building Apps on a Portal

*Status: design, 2026-11-20 (D63). Layer 4 of [the four SDLC layers](../sdlc.md). A **user** is
anyone who builds on a running portal rather than on Seikika or the portal itself. Examples:*
- *an agent answering "who owns this?";*
- *a team's on-call bot;*
- *a dashboard in Claude Artifacts;*
- *a CI check that blocks deploys of services below bronze;*
- *an internal tool that installs approved GitHub Apps.*

---

## 1. Conclusion

Users need a user SDLC as mature as the operator SDLC: discover what's available, develop against
realistic data, test deterministically, ship with a proper identity, and get warned before
something they rely on changes.

The foundations exist today:
- MCP, REST and CLI over one authorization path;
- personal access tokens;
- snapshot ids that make reads reproducible, and read-your-writes (`min_snapshot`);
- a dev portal that runs from fixtures.

The parts that make layer 4 first-class are still to build:
- typed clients from a per-portal OpenAPI document;
- the generated user agent plugin;
- service identities for apps;
- action runs for apps that change things (M5);
- a published stability and deprecation policy.

## 2. Who builds what

| App | Interface | Example |
|---|---|---|
| Agent (in a harness) | MCP (`/mcp`, or `seikika mcp` over stdio) | "Who owns ledger, and why isn't it verified?" → `owner_of` |
| Dashboard / artifact | MCP or REST from Claude Artifacts, Cursor Canvas | A team scorecard view built on `query` and `scorecard_status` |
| Script / CI check | CLI (`seikika score … -o json`) or REST with a PAT | Fail a deploy when `ownership` is below bronze |
| Internal tool / bot | REST (`/v1/views/*`, `/v1/entities`) | On-call bot that posts owners and failing rules |
| Self-service flow | REST/MCP previews and runs (M5) | Install an approved GitHub App with preview, confirm and verification |
| Custom portal UI | UI extension contract (operator-owned bundle) | A rich action form; owned by operators, used by users |

## 3. The loop

| Step | What the user does | What Seikika provides | State |
|---|---|---|---|
| **Discover** | Find tools, entities and actions | MCP `tools/list` with schemas and annotations; `list_actions`; `/v1/views/home` | ✅ |
| | Get a typed contract | Per-portal OpenAPI 3.1 and generated TS/Python clients | Planned (v1-spec §7.2) |
| | Get guidance for agents | Generated user plugin (skills per promoted action, portal instructions) | Planned ([agent-plugins.md](./agent-plugins.md)) |
| **Develop** | Run against realistic data | `seikika serve --fixtures` on the portal repo (or the reference portal); preview portals per operator PR | ✅ local; previews in M6 |
| | Authenticate | PATs (`seikika token create`), OIDC bearer tokens, the identity-aware proxy; dev header locally | ✅ |
| | Authenticate an app | Service identities with their own relations (not a person's PAT) | Planned (§5) |
| **Test** | Get deterministic reads | Snapshot ids on every response; `min_snapshot` to read your own writes | ✅ |
| | Test the hidden-data path | Facet redaction markers that say which relation is needed, so apps handle "hidden" explicitly | ✅ |
| | Check contracts | Schema-checked MCP tool inputs; stable problem codes (`not-found`, `forbidden`, `snapshot-not-ready` …) | ✅ codes; every built-in MCP tool contract-tested over `tools/call` (`crates/seikika-api/tests/mcp.rs`, D65); a kit for users' own CI planned |
| | Test actions | Previews are side-effect free; confirm tokens; postconditions | Preview gate ✅ (M3); runs in M5 |
| **Deploy** | Run the app with least privilege | PAT or service identity scoped by OpenFGA relations | PATs ✅; service identities planned |
| **Monitor** | See usage and failures | `seikika_mcp_tool_calls_total`, audit of denials and runs | Partly (metrics planned per v1-spec §10.5) |
| | Hear about changes | Versioned `/v1`, deprecation headers and notices, `seikika plan` listing removed MCP tools for operators | `/v1` ✅; policy planned (§4) |

## 4. Stability contract (proposed)

Apps break when the portal changes under them. Operators change portals often (that's layer 3), so
layer 4 needs explicit promises:

| Surface | Promise | Enforced by |
|---|---|---|
| REST paths and response shapes under `/v1` | Additive changes only within `/v1` | API tests + OpenAPI diff in CI (layer 1) |
| MCP tool names, input schemas, `structuredContent` shapes | Additive within a major version | Tool-list snapshot tests (layer 1) |
| Problem `code` values | Never repurposed | Tests |
| Portal-defined kinds, fields, actions | Operators may change them; `seikika plan` shows removed fields, actions and MCP tools so the operator sees which apps could break | `plan` (layer 2) |
| Snapshot ids | Monotonic per portal; `min_snapshot` semantics stable | Acceptance tests |

Open: how users declare which portal fields they depend on, so `plan` can name the affected apps.

## 5. Gaps to close (proposed order)

1. **Per-portal OpenAPI + typed clients** (v1-spec §7.2 already promises the document).
2. **Service identities**: `service:<name>` principals with PAT-like tokens and their own OpenFGA
   relations, so apps don't borrow a person's access.
3. **Action runs (M5)**: preview → confirm → run → verify, usable from apps with the same tokens.
4. **Generated user plugin**: skills for promoted actions and portal-specific instructions.
5. **Contract test kit**: a fixtures-backed dev portal plus recorded MCP/REST responses users can
   run in their own CI.
6. **Change notices**: deprecation headers and a feed of manifest changes affecting public fields.

## Sources

[v1-spec.md](../v1-spec.md) §7 (interfaces), [design/agent-plugins.md](./agent-plugins.md),
[design/dashboards.md](./dashboards.md), [design/action-runs.md](./action-runs.md), and the
[reference portal](../../examples/reference-portal/README.md) acceptance scenarios, which are
written as layer-4 tasks.
