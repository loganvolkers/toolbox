# Action-Created Entities, Lifecycles (TTL) and Day-2 Actions

> **Status 2026-11-20 (D40):** action code can ship **inside the Seikika image** as in-image executors (child processes over the stdio transport, any language), alongside external executors such as GitHub Actions. Isolation for in-image executors is Q12. Run lifecycle, preview/confirm and verification: [action-runs.md](./action-runs.md) (D49). See [decisions.md](../decisions/README.md).

*Status: design. Addresses red-team gaps B4 (self-service results must exist immediately and expire)
and B3 (visible pending state for PR-based changes).
Related: [plan.md](../plan.md) §4.3, [identity-resolution.md](./identity-resolution.md),
[notifications.md](./notifications.md).*

## 1. Prior art

Port's action resource (Terraform provider schema) shows what customers expect:
- an `upsert_entity_method` (actions that write catalog entities directly)
- automations triggered by `timer_property_expired_event` (TTL expiry)
- self-service vs. automation triggers bound to blueprints

Backstage's Scaffolder can only *register* a new component via `catalog:register` after publishing a
repo, and records it with `backstage.io/source-template`.

## 2. Action scopes: how actions attach to entities

| Scope | Appears | Pre-filled | Permission checked on |
|---|---|---|---|
| `create` (kind-level) | "Create" menu of a kind; MCP `list_actions(kind)` | — | The kind's parent (e.g. `portal` or `system`) |
| `day2` (entity-bound) | The entity page's action bar; MCP `list_actions(ref)` | `entity` input bound to the entity | **That entity** (e.g. `service:x#can_run_actions`) |
| `delete` / `decommission` | Entity page (danger zone) | `entity` | That entity; destructive → confirmation + approval policy |
| `global` | Actions page | — | `portal` |

```ts
new seikika.Action("extend-env-ttl", {
  scope: { day2: env },                                   // binds to kind Environment
  owner: "group:default/platform",
  inputs: seikika.schema.object({ hours: seikika.schema.int({ min: 1, max: 72 }) }),
  executor: seikika.executor.builtin("lifecycle.extend"),
  policy: { permission: "can_run_actions" },
  annotations: { destructive: false, idempotent: true },
});
```

## 3. Action-sourced fragments

Executors (or their callbacks) may return **fragments**. These enter the catalog through the built-in
`actions` source, with full provenance (run id, action, requester, agent):

```json
{ "fragments": [{ "kind": "Environment", "name": "pr-1234-payments",
  "spec": { "owner": "group:default/payments", "service": "service:default/payments-api", "url": "https://pr-1234.preview.acme.dev" },
  "externalIds": ["k8s:namespace/preview/pr-1234-payments"],
  "lifecycle": { "ttl": "72h" } }] }
```

- **Immediate visibility:** the entity exists as soon as the run reports it, rather than after the next
  connector sync.
- **Reconciliation:** when a real connector later observes the same resource (matched via external ids,
  [identity-resolution.md](./identity-resolution.md)), the connector's fragment takes precedence. The
  action fragment becomes historical provenance.
- **Guardrails:**
  - Only kinds declaring `sources: { actions: true }` accept action fragments.
  - Per-team **quotas** in code, e.g. at most 10 live `Environment`s per team.
  - Fragments are validated against the kind schema and the action's declared output schema.

## 4. Lifecycles and TTL

```ts
new seikika.Kind("Environment", {
  props: { url: seikika.prop.string(), service: seikika.prop.ref(service) },
  lifecycle: {
    ttl: { default: "72h", max: "14d" },
    warnBefore: ["24h", "2h"],                         // → notifications to owner/requester
    onExpire: seikika.actionRef("destroy-env"),             // automation runs the destroy action
    gracePeriod: "1h",
  },
});
```

**State machine:** `active → expiring → expired → decommissioning → gone`.
- Moving to `gone` requires *confirmation* from the source: the connector no longer sees the
  resource, or the destroy run succeeded.
- Failed destroys alert the kind's owner, and the entity is never silently dropped.

**Timer engine:** a scheduler over a `lifecycle_timers` table, leader-elected, emitting events. Timers
can also be attached to properties (`prop.timer()`), e.g. for access grants or exception deadlines.

## 5. Pending state for PR-based changes (B3)

Actions whose executor opens a PR (templates, "edit catalog data", identity link acceptance) create a
**pending overlay**: a read-only shadow fragment marked `pending` with a PR link.
- UI and MCP show the entity or field as "pending review (PR #123)".
- Scorecards ignore pending overlays.
- The overlay disappears when the PR merges (the real source takes over) or closes.

**Bulk edits** (e.g. re-tier 200 services) produce **one PR** against the catalog-data repo, or one PR
per owning repo when the owners differ. Grouping is configurable.

## 5b. Entity-dependent inputs and user identity (from the GitHub Apps example)

- **`enumFrom` (CEL):** an input's allowed values are computed in the recompute pipeline for the
  target entity, e.g. "approved apps eligible for this repo". They appear in dry-run, the MCP input
  schema and the UI. They are re-validated at execution.
- **Executor identity `user`:** the executor acts with the requesting user's own token in the
  external system, so that system enforces its own rules too. Refresh tokens (e.g. GitHub Apps
  `offline_access`) are kept in the secrets vault and refreshed inside durable workflow steps, which
  survives long approval waits.
- Worked example: [examples/approved-github-apps.md](../examples/approved-github-apps.md).

## 6. Phasing

| Phase | Scope |
|---|---|
| 3 | Action scopes (`create`/`day2`/`delete`/`global`), action-sourced fragments with provenance, reconciliation with connectors, quotas |
| 3 | Lifecycle/TTL engine, expiry notifications, destroy automations |
| 3 | Pending overlays for PR-based actions; bulk-edit PRs |

## Sources

- [port-labs/terraform-provider-port-labs `action` resource docs](https://github.com/port-labs/terraform-provider-port-labs)
  (`upsert_entity_method`, `automation_trigger.timer_property_expired_event`)
- [Backstage descriptor format: Template `spec.owner`; `backstage.io/source-template`](https://backstage.io/docs/features/software-catalog/descriptor-format)
