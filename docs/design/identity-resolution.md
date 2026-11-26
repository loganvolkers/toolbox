# Identity Resolution: One Service, Many Names

*Status: design. Addresses red-team gaps B1 (identity resolution) and B2 (renames and merges).
Related: [plan.md](../plan.md) §4.1, [ocean.md](../ocean.md) (lookups), [meta-catalog.md](./meta-catalog.md).*

## 1. Problem

The same service is `acme/payments-api` in GitHub, `Payments API (PXK2Q1)` in PagerDuty, `payments` in
Datadog, `svc-payments` in Snyk and `payments-prod` in Kubernetes. If these aren't matched, the catalog
is wrong on day one: scores read the wrong facts, ownership is split, and agents get partial context.

## 2. What the industry does

| Approach | Used by | Strength | Weakness |
|---|---|---|---|
| **Well-known annotations** on the entity (`github.com/project-slug`, `pagerduty.com/service-id`, `sentry.io/project-slug`, `sonarqube.org/project-key`…) | Backstage (plain string keys that plugins match), also read by Datadog's and Roadie's importers | Deterministic; lives with the owner in `catalog-info.yaml` | Manual; coverage decays; nothing reports what is *unlinked* |
| **Search identifiers/relations** (match by query, e.g. by email or property) | Port / Ocean (`IngestSearchQuery`) | Flexible, no manual ids | Silent mismatches; logic is per-mapping |
| **Code locations + telemetry** (path globs, APM service names) | Datadog (`codeLocations`, auto-detected dependencies, `inheritFrom`) | Automatic and fresh | Needs telemetry; naming drift across tools |
| **Record linkage** (deterministic + probabilistic matching, review queues) | MDM/data tooling generally | Handles the long tail | Needs human review to be trusted |

Seikika combines all four, and adds what none of them have: **explicit unmatched/ambiguous states,
coverage metrics, and PR-based resolution.**

## 3. Model

- Every entity has a canonical **ref** (`service:default/payments-api`) plus a set of **external
  identifiers**: typed, namespaced and unique across the catalog.
  ```
  github:repo/acme/payments-api      pagerduty:service/PXK2Q1     datadog:service/payments
  snyk:project/9f1c…                 k8s:workload/prod/payments/payments-api
  ```
- Every connector record and fragment carries the external ids of its source object. The connector
  protocol's `discover` declares each record kind's id namespace.
- A unique index `external_ids(namespace, id) → entity` makes linking and `rel.lookup` O(1).
- Backstage well-known annotations are translated automatically into external ids
  (`pagerduty.com/service-id: PXK2Q1` → `pagerduty:service/PXK2Q1`), so existing `catalog-info.yaml`
  files work unchanged.

## 4. Matching pipeline (per incoming record, in order)

1. **Explicit:** an entity already declares the record's external id (annotation, `links/*.yaml`, or
   a previous accepted link). Match, confidence 1.0.
2. **Declared matchers** (code, per connector record kind; CEL over record and candidate entity):
   ```ts
   new seikika.Matcher("pagerduty-service-to-service", {
     record: "pagerduty.service",
     target: service,
     rules: [
       { when: "record.description.contains('catalog: ' + entity.ref)", confidence: 1.0 },
       { when: "norm(record.name) == norm(entity.metadata.name)",     confidence: 0.9 },
       { when: "entity.metadata.aliases.exists(a, norm(a) == norm(record.name))", confidence: 0.9 },
     ],
     autoLinkAbove: 0.85,          // below this → proposal, never auto-applied
   });
   ```
3. **Structural inference** (shipped by connector packages): repo → service via `catalog-info.yaml`
   location or code-location globs; k8s workload → service via image labels; Datadog service → service
   via a `service` tag. Each carries an inference-specific confidence.
4. **Suggestions:** fuzzy name similarity and co-occurrence (e.g. the same team owns both, or the same
   repo URL appears) produce ranked candidates. These are **never auto-applied**.

| Outcome | What happens |
|---|---|
| `matched` (≥ threshold) | The fragment merges into the entity, with link provenance (method, confidence, matcher) |
| `ambiguous` (several candidates above threshold) | No merge. A proposal lists the candidates. Both owners are notified |
| `unmatched` | The record becomes an **unlinked entity** of a source kind (e.g. `PagerDutyService`). It is visible, searchable and counted, but excluded from scorecards on target kinds |
| `conflict` (two entities claim one external id) | Error status on both entities; both owners notified; the later claim is ignored until resolved |

## 5. Resolving proposals: PRs, not clicks

- The **Unlinked & ambiguous** view (built-in UI view + MCP `identity_proposals`) lists proposals with
  evidence.
- Accepting a proposal (by a human or an agent) opens a PR. The preferred target is the
  **owner's `catalog-info.yaml`**, by adding the annotation, so identity lives with the owner. The
  fallback for entities without a repo is the central **catalog-data repo** (`links/*.yaml`).
- **Bulk resolution:** one PR can resolve many proposals, which suits an agent clearing the backlog.
- **Rejected proposals** are recorded as negative links (also in code) so they don't reappear.

## 6. Renames, merges, splits (gap B2)

| Event | Handling |
|---|---|
| **Entity rename** (same external ids, new name) | Detected as a rename, not delete+create. The old ref becomes an **alias**, and refs, links, score history, notifications and ReBAC tuples follow the new ref |
| **Merge** (two entities are the same) | `mergedInto` redirect; histories are concatenated; external ids are unioned; conflicts are surfaced |
| **Split** (one service becomes two) | Declared in code (`split: { from, into: [...] , by: "codeLocations" }`); history stays with the original, which is marked `splitInto` |
| **Kind or property rename** (model change) | `seikika plan` detects it and requires a declared migration (`rename`, `retype`, `move`). Data is migrated in an online expand/contract migration |
| **Rule rename** | Rule ids are stable, and `renamedFrom` keeps history continuous |

Aliases resolve everywhere: API, MCP, search, ReBAC subject and object refs, and views.

## 7. Metrics, scorecards and plan

- **Link coverage** per connector and record kind (`matched / total`), the ambiguous count and the
  conflict count are exported as metrics and evaluated by the meta-catalog **Connector** scorecard
  (e.g. silver ≥ 90% coverage).
- `seikika plan` shows coverage deltas for matcher changes, e.g. "+212 PagerDuty services linked,
  3 new ambiguities".
- `explain` for any fact shows its link chain: fact ← record ← external id ← link method ← proposal PR.

## 8. Phasing

| Phase | Scope |
|---|---|
| 1 | External ids + unique index; annotation translation; explicit + declared matchers; unmatched/conflict states; aliases on rename |
| 2 | Proposals queue; PR-based resolution; coverage metrics + meta scorecard |
| 3 | Structural inference packs per connector; merges/splits; model migrations in `plan` |

## Sources

- [Backstage well-known annotations](https://backstage.io/docs/features/software-catalog/well-known-annotations)
- [Datadog: importing Backstage annotations](https://docs.datadoghq.com/internal_developer_portal/software_catalog/set_up/import_entities)
- [Datadog entity model (`codeLocations`, `inheritFrom`)](https://docs.datadoghq.com/internal_developer_portal/catalog/entity_model)
- Port Ocean `IngestSearchQuery` (source: `port_ocean/core/handlers/port_app_config/models.py`)
