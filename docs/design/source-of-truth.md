# Source of Truth: When Toolbox Owns the Data

*Status: research + recommendation. Research date 2026-11-18. Related: [plan.md](../plan.md) §1
(config vs. state), §4.1; [identity-resolution.md](./identity-resolution.md);
[action-entities.md](./action-entities.md); [authorization.md](../authorization.md);
[context-lake.md](./context-lake.md).*

## 1. The question

Today's design is read-mostly:
- Entities come from git (`catalog-info.yaml`, a catalog-data repo) or from connectors.
- Every human edit becomes a pull request.

Port's value proposition includes **mixed sources of truth**:
- Some entities are mirrored from elsewhere (GitHub repos, Kubernetes pods).
- Some are **owned by the portal** (teams).
- Some *fields* on mirrored entities are owned by the portal (e.g. a repo's owner or tier).

Should Toolbox be the canonical store for some entities and fields?

**Short answer: yes, but only for *data*, never for *configuration*.** Each field's source of truth
should be **declared in code** and enforced per field, so conflicts are rejected and explained instead
of silently overwritten.

## 2. How the market handles it

| Product | Model | Granularity | Conflict behaviour |
|---|---|---|---|
| **Backstage** | Git and providers only; the UI is read-only. `backstage.io/edit-url` links to the source YAML. The catalog holds "rarely changing, human curated data… managed by the owners" | Entity (one provider per entity) | No conflicts by construction, but **no place for data without a repo** |
| **Port** | The DB is the source of truth by default. Integrations become an entity's "maintainer" when they register it; UI/API edits allowed; manual entities are a documented pattern ("not every catalog entity has a single clean source of truth"). Can **push portal-owned properties to GitHub external custom properties** | Entity + property (by mapping scope) | **Syncs overwrite mapped fields on every run.** With two integrations writing one relation, **the last sync wins**. Advice: keep manual properties out of mappings |
| **Cortex** | GitOps (`cortex.yaml`) *or* UI editing, chosen **per entity type**. If UI editing is enabled, git changes are not processed. Hybrid: UI editing on, UI import off | Entity type (all-or-nothing) | Avoided by forbidding one side |
| **OpsLevel** | `opslevel.yml` manages selected fields; **those fields are locked in the UI** (services: name, description, owner, tier, lifecycle, language, framework; repos: owner, tier). List properties: YAML entries locked, UI may *add* entries | **Field** | Locks + lock icon |
| **Atlassian Compass** | `compass.yml`-managed fields are locked in the UI; some fields aren't supported by config-as-code. A documented bug: a UI-set slug was **overwritten** by a sync from a config without it | Field | Locks; data-loss edge cases when field sets differ |
| **ServiceNow CMDB (IRE)** | **Attribute-level authoritative sources**: reconciliation rules per source and attribute, numeric **precedence** (lower wins), timestamps for conflicts. Without rules, "data sources can overwrite each other" | **Attribute** + precedence | Rule-based arbitration; no staging/preview before commit |
| **Kubernetes Server-Side Apply** (analogy) | `managedFields` records which **field manager** owns each field. Conflicting applies are **rejected** with the owning manager named, unless forced; forcing transfers ownership | **Field** + named manager | Explicit rejection; explicit takeover |

**What we learn:**
1. **Field-level ownership is the mature answer** (OpsLevel, Compass, ServiceNow, Kubernetes).
   Entity-level (Backstage, Cortex) is too coarse. Port's mapping-scope approach produces silent
   overwrites, a frequent complaint pattern in its own docs.
2. **Silent last-writer-wins is the failure mode to avoid.** Reject, explain, and offer the right
   path, the way Kubernetes SSA does.
3. **Arbitration rules belong somewhere reviewable.** ServiceNow keeps them as admin configuration with
   no preview. We can keep them **in code, with `toolbox plan`** previewing their effect.
4. **Write-back closes the loop.** Port pushing owner and tier into GitHub custom properties makes the
   portal the system of record that other tools consume. This matches our "ownership system of record"
   goal ([context-lake.md](./context-lake.md) §5.5).

## 3. Pros and cons for Toolbox

| Pros | Cons |
|---|---|
| **Data with no natural home gets one:** team charter, notification channels, business criticality, data classification, compliance scope, cost centre, vendor SaaS entries, Systems and Domains curated by architects | **Drift and conflicts** with git and connectors, unless field ownership is enforced (the Port problem) |
| **Low-friction edits** for non-engineers and **bulk edits** without PR latency (red-team B3) | Weakens "everything is reviewed in git": approvals, history and audit move into Toolbox for those fields |
| **Agent-first writes:** agents can update facts directly through MCP, within ReBAC and approval guardrails, instead of opening PRs for trivial data | `toolbox plan` can't preview runtime data writes. **Changesets** with their own preview are needed (§4.5) |
| **Transactional workflows:** ownership verification, attestations, acknowledgements and exception requests are natural state, not code | **Backups matter more:** native data is irreplaceable state, not rebuildable from sources |
| **Market expectation** (Port, OpsLevel, Compass); removes a sales objection | **Slippery slope** toward "config in the DB" ("let me edit the scorecard threshold in the UI") unless the line is explicit and enforced |
| **Write-back:** Toolbox can own repo owner and tier and push them to GitHub custom properties and security tools, becoming the ownership system of record | Previews and staging need native data too (redacted samples; [sdlc.md](./sdlc.md) §5) |
| Concurrency, validation, history and undo are well-understood database problems | Portability: data can't be locked in, so export and import are required |

## 4. Recommendation

### 4.1 The line: configuration is code; *data* may be native

| Always code (never native) | May be native (declared per field, in code) |
|---|---|
| Kinds/schemas, which fields are native, precedence rules | Values of fields declared `native` |
| Scorecards, rules, levels, thresholds | Ownership verification attestations, acknowledgements |
| Actions, executors, automations, notification policies | Team metadata (charter, notify, accountable lead) |
| Authorization model and derivation rules | Business metadata (tier, criticality, data classification, cost centre) |
| Operator UI views, plan gates, connectors | Entities of kinds declared `native` (e.g. vendor SaaS, Systems, Domains, non-IdP teams) |

The non-goal is restated as: **"Portal configuration (schema and behaviour) is never stored in the
database. Catalog data may be, when the schema declares it."**

### 4.2 Field-level source of truth, declared in code

Every kind declares, per field (or field group), **who owns the value**, in precedence order:

```ts
new tb.Kind("Repository", {
  sources: {
    default: ["connector:github"],                         // mirrored: read-only in Toolbox
    fields: {
      "spec.owner": ["git:catalog-info", "native"],        // catalog-info wins if present, else Toolbox
      "spec.tier":  ["native"],                            // Toolbox is the source of truth
    },
  },
  writeBack: { "spec.owner": "github:custom-property/owner", "spec.tier": "github:custom-property/tier" },
});

new tb.Kind("Group", {
  sources: {
    default: ["native"],                                   // teams are owned by Toolbox…
    fields: { "spec.members": ["connector:okta"],          // …except membership, owned by the IdP
              "metadata.name": ["connector:okta", "native"] },  // IdP teams named by IdP; ad-hoc teams native
  },
});

new tb.Kind("Service", {
  sources: {
    default: ["git:catalog-info"],
    fields: { "spec.businessCriticality": ["native"], "spec.dataClassification": ["native"] },
  },
  approvals: { "spec.dataClassification": { approvers: "security from portal" } },   // ReBAC expression
});
```

**Source types:**
- `git:<pattern>` (catalog-info, catalog-data repo)
- `connector:<name>`
- `native` (Toolbox database)
- `computed` (calc/mirror/aggregate)
- `action` (action-sourced fragments, [action-entities.md](./action-entities.md))

The precedence list replaces today's merge precedence ([plan.md](../plan.md) §4.1), so there is one
mechanism for both.

### 4.3 Write semantics (Kubernetes-SSA style, routed for humans and agents)

One write path serves API, MCP, CLI and UI: `update_entity(ref, changes, reason)`. For each field it:

| Field's effective owner | Result |
|---|---|
| `native` | Applied as a revision: actor, on-behalf-of, agent, reason. Approvals apply if declared. Optimistic concurrency via ETag |
| `git:*` | Not applied. A **proposed change** is created: a PR to the owning file (pending overlay, [action-entities.md](./action-entities.md) §5). The response names the file and the PR |
| `connector:*` | **Rejected** with an explanation and a link to the source system ("owned by Okta; change it there"). Optionally runs the connector's `act` capability if one is declared, e.g. "request group change" |
| `computed` | Rejected; the response shows the expression and its inputs |

The UI therefore shows per-field affordances (inline edit, "propose PR", or a lock icon naming the
source, like OpsLevel's), and agents get one tool that always does the right thing. Forced takeovers,
moving a field's ownership, are **only** possible by changing the declaration in code, so they are
reviewed and visible in `plan`.

### 4.4 The native store

- **Versioned:** every change is a revision (who, on whose behalf, which agent, why, when), with full
  history, diffs and **undo**. Revisions feed the audit log and the daily history store.
- **Permissions:**
  - ReBAC write permissions per facet (`can_edit_<facet>`, defaulting to the entity's owners plus
    admins).
  - Declared approvals reuse the action approval engine.
  - Sensitive fields such as data classification require approvers from the declared relation.
- **Validation:** against the kind schema at write time. Bad writes are rejected, not stored as
  errors (unlike soft errors from external sources).
- **Write-back:** declared `writeBack` targets are pushed through connectors' `act` capability, with
  retries and drift detection. If someone edits the GitHub custom property directly, Toolbox flags the
  drift and re-asserts or alerts, per policy.

### 4.5 Changesets for bulk and agent writes

Large writes (bulk re-tiering, an agent reclassifying 300 services) go through a **changeset**:
1. **Preview** (`dry_run`): score deltas, authz deltas, notification volume, write-back targets.
   These are the same sections and gates as `toolbox plan` ([sdlc.md](./sdlc.md) §4, §6).
2. **Approval** if gates trip (e.g. > 50 entities, or any data-classification change).
3. **Atomic apply** and **one-click revert** of the whole changeset.

### 4.6 Portability and git escape hatches

- `toolbox data export|import`: native data as `catalog-info`-compatible YAML. This avoids lock-in and
  supports seeding previews and migrations between portals.
- **Optional git mirror:** native data is continuously committed (bot commits) to a read-only data repo
  for diffing, backup and review culture. Teams that want git as the *storage* can declare a field
  `git:data-repo` with `mode: direct` (bot commit, no PR) instead of `native`. This gives the same UX
  with git persistence.
- **Ownership-change migrations:** when `plan` sees a field move between sources (e.g.
  `native → git:catalog-info`), it requires a migration. Options: export native values into PRs, or
  seed native values from the current source on the reverse move.

### 4.7 Defaults Toolbox ships

| Kind / field | Default source of truth | Why |
|---|---|---|
| `Group` metadata (charter, notify, accountable lead) | `native` | No natural home elsewhere; teams own it |
| `Group` membership, names of IdP teams | IdP connector | The IdP is authoritative (SCIM, Okta, Entra) |
| `Service`/`Component` core fields | `git:catalog-info` | Lives with the code and owner, Backstage-compatible |
| Business metadata (tier, criticality, data classification, cost centre) | `native` (with approvals) or catalog-info: **portal's choice** | Often owned by people outside the repo |
| `Repository` owner/tier when no catalog-info exists | `native` + write-back to GitHub custom properties | Ownership system of record ([context-lake.md](./context-lake.md)) |
| Identity links for entities without a repo | `native` (with proposal review) | Avoids a PR per link for no-repo entities |
| Ownership verification, acknowledgements | `native` | Transactional |
| Waivers / exceptions | Code by default (reviewed by rule owner); `native` with rule-owner approval as an option | Some orgs want PR review, others want speed |
| Mirrored infrastructure (repos, pods, cloud resources) | Connectors | Never edited in Toolbox |

### 4.8 Meta catalog and operations

- **Meta catalog:** native-owned fields become visible in the meta graph ("which fields does Toolbox
  own, and who can edit them?"). The new default scorecard `native-data-hygiene` checks:
  - native fields have owners and write permissions
  - stale native values are re-verified on a cadence
  - write-back drift is under its threshold
- **Operations ([plan.md](../plan.md) §6):** the "only Postgres needs backing up" statement still holds,
  but native data joins audit and run history as **irreplaceable** state. Backup verification must
  cover it, and the git mirror is a second copy.

## 5. Phasing

| Phase | Scope |
|---|---|
| 1 | Field-level `sources` declarations (replacing merge precedence); enforcement on all write paths; native store with revisions for `Group` metadata and business fields; UI lock/propose/edit affordances; `update_entity` MCP tool with routing |
| 2 | Approvals on native fields; changesets with preview, gates and revert; export/import |
| 3 | Write-back (GitHub custom properties first), drift detection; optional git mirror and `git:data-repo` direct mode; source-ownership migrations in `plan`; `native-data-hygiene` scorecard |

## 6. Decision summary

Keep config-as-code absolute for **behaviour and schema**. Allow **data** to be native **only where the
schema declares it, per field, in code**. Reject conflicting writes with an explanation, and route
edits automatically: inline for native fields, a PR for git-owned fields, a pointer to the source for
connector-owned fields. Add revisions, approvals, changesets, export and write-back.

This combines Port's flexibility, OpsLevel/Compass field locks and the rigour of ServiceNow's IRE,
without Port's silent overwrites or ServiceNow's unpreviewable rules.

## Sources

- Backstage: [descriptor format & FAQ](https://backstage.io/docs/features/software-catalog/descriptor-format) ·
  [well-known annotations (`backstage.io/edit-url`)](https://backstage.io/docs/features/software-catalog/well-known-annotations)
- Port:
  - [Manual entity management](https://docs.port.io/context-lake/ingestion/ingest-data-into-port/other/manual-entity-management/)
  - [Configure mapping](https://www.docs.port.io/build-your-software-catalog/customize-integrations/configure-mapping) ·
    [Sync interactions](https://docs.port.io/context-lake/ingestion/ingest-data-into-port/native-integrations/sync-mechanisms/sync-interactions/) ·
    [Ocean data flow](https://ocean.port.io/framework/architecture/data-flow)
  - [Sync Port properties to GitHub external custom properties](https://docs.port.io/guides/all/sync-port-properties-to-github-external-custom-properties/) ·
    [GitHub: sync external custom properties](https://docs.github.com/organizations/managing-organization-settings/sync-external-custom-properties)
- Cortex: [Using GitOps for Cortex](https://docs.cortex.io/configure/gitops/using-gitops-for-cortex) ·
  [UI editing, GitOps and API conflicts](https://support.getcortexapp.com/hc/en-us/articles/14233923381019-UI-Editing-GitOps-and-API-conflicts)
- OpsLevel: [Config as Code (opslevel.yml)](https://docs.opslevel.com/docs/opslevel-yml)
- Atlassian Compass: [What is config as code](https://developer.atlassian.com/cloud/compass/config-as-code/what-is-config-as-code/) ·
  [Config-as-code practicalities (community)](https://community.atlassian.com/forums/Compass-questions/config-as-code-practicalities/qaq-p/2970693)
- ServiceNow:
  [Identification and Reconciliation engine](https://www.servicenow.com/docs/r/Ir~NPf1dr8MGFXQE6QlBoQ/dspVVa0HIXNoVA5Kt7Vb3Q) ·
  [Understanding IRE reconciliation rules (community)](https://www.servicenow.com/community/cmdb-articles/understanding-ire-reconciliation-rules/ta-p/3289239)
- Kubernetes: [Server-Side Apply](https://kubernetes.io/docs/reference/using-api/server-side-apply/)
