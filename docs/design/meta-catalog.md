# The Meta Catalog: Toolbox Catalogs, Owns and Scores Itself

*Status: design. A cornerstone of the default model. Related: [plan.md](../plan.md),
[authorization.md](../authorization.md), [identity-resolution.md](./identity-resolution.md),
[notifications.md](./notifications.md), [sdlc.md](./sdlc.md), [dashboards.md](./dashboards.md).*

> **Thesis.** A portal that asks every team to own, document and score its software must hold itself
> to the same standard. Every Toolbox portal ships a **meta catalog**. Its entity types, connectors,
> scorecards, rules, actions, views, authorization modules and packages are themselves catalog
> entities, with **owners**, **relations** and **default scorecards**. This includes the scorecards
> that score scorecards.
>
> Different teams own different parts of the portal. The meta catalog makes that ownership explicit,
> verifiable and *operational*: it drives review routing, notifications, permissions and agent
> answers.

---

## 1. Why this matters (beyond dogfooding)

1. **Distributed ownership needs a map.** Once security owns the security scorecard and the Snyk
   connector, SRE owns production readiness and PagerDuty, and FinOps owns cost views, *who owns
   which part of the portal* is a first-class question. Without a map, the portal team becomes the
   bottleneck: the Backstage failure mode, recreated.
2. **Correct routing of problems.** When a rule's outcome is `unknown` because the Snyk connector is
   stale, the *connector owner* should hear about it, not 400 service teams. When a team disputes a
   rule, the *rule owner* should hear about it. That requires rules and connectors to be entities with
   owners and dependency edges.
3. **Agents need provenance.** "Why is payments-api red?" becomes a graph walk:
   - outcome → rule (owner: security)
   - → fact `snyk.critical` → connector `snyk` (owner: security, last sync 3 h ago, healthy)
   - → fix text + remediation action

   The same `graph`, `impact` and `explain` machinery answers it, because these are ordinary entities.
4. **Portal quality is measurable.** Dead kinds, flapping or always-passing rules, actions nobody
   uses, connectors with poor identity coverage and views with broken queries all become visible
   and scored.
5. **Walk the talk.** The portal's own deployment is a `Component` in its own catalog, scored by the
   same production-readiness scorecard as everyone else's services.

## 2. What competitors do

| Product | Meta / self-modelling | Ownership of portal objects | Gap Toolbox fills |
|---|---|---|---|
| **Backstage** | `Template` and `Location` are catalog entities. A Template has `spec.owner` ("the singular entity (commonly a team) that bears ultimate responsibility"); entities record `backstage.io/source-template`. Plugins, processors, config and Tech Insights checks are code, **not** entities | Templates only | No meta scorecards; checks have no owners; no dependency graph of the portal itself |
| **Port** | **"Scorecards as blueprints"**: `_scorecard`, `_rule` and `_rule_result` system blueprints (protected, extensible; up to 5M rule results), enabling relations, permissions, history and automations on scorecard data | Possible via extension properties | Rules and results as entities, but configuration lives in Port's DB; no owner-routing of data-quality problems documented; integrations/actions as entities not verified |
| **OpsLevel** | Checks are first-class | **Check owner (team)**: "Everything in OpsLevel should have ownership, including checks"; tells service owners whom to contact about a failing check | Ownership per check, but no meta scorecards or dependency graph |
| **Cortex** | Ownership on entities; owners verify entity data and receive scorecard and initiative notifications | No scorecard-owner field found | No self-modelling |
| **Datadog** | Custom scorecard rules have an `owner` | Rule owner | No meta catalog beyond rules |

**Takeaway:** each competitor has *one piece*. Backstage has template ownership, Port models scorecards
as entities, OpsLevel has check owners, and Cortex verifies ownership. Nobody treats **the whole portal
as an owned, scored system with its own dependency graph.** That is the differentiator, and it is
cheap for us because config is code (the manifest already contains every object) and the graph engine
already exists.

## 3. Industry practice on ownership, and the principles we adopt

| Source | Finding / practice | Principle for Toolbox |
|---|---|---|
| Bird et al., *Don't Touch My Code!* (ESEC/FSE 2011, Windows); replicated by Greiler et al. (MSR 2013, four Microsoft products: median precision 0.74 for flagging buggy files); weaker in open source (Foucault et al.; Eclipse/Firefox follow-up) | Strong ownership and few low-expertise contributors correlate with fewer defects in closed, team-based processes | **P2:** measure *observed* ownership (contributions), not only declared ownership. Treat it as a signal, not dogma |
| Backstage descriptor format | Owner = "singular entity (commonly a team)" with authority and capability to maintain | **P1:** exactly one owner, and it should be a team |
| Team Topologies / microservices.io *service per team* | Teams are the unit of ownership; shared ownership raises quality risk; the platform stays thin and is run as a product | **P1** + the portal itself is owned as a product (meta catalog) |
| ITSM practice (IF4IT, Xurrent) | Warns that team-only ownership blurs accountability; recommends a *named accountable individual* and "never orphaned" ownership | **P1b:** optional **accountable individual** role (derived from the owner team's lead) alongside the owning team |
| Chromium OWNERS → Kubernetes OWNERS → GitHub CODEOWNERS | Path-based, hierarchical ownership drives review; Kubernetes prunes inactive reviewers to an *emeritus* list; Chromium restricts `Owners-Override` to specific groups | **P3:** ownership decays, so verify on a cadence and prune. **P5:** overrides are restricted and audited |
| Cortex (owners verify entity data) | Periodic human verification | **P3:** a verification cadence with a `verified_at` fact |
| Industry generally | Ownership drives on-call, reviews, notifications | **P4:** one ownership source of truth drives review routing, notifications, approvals *and* permissions (ReBAC derives from it) |

**Toolbox ownership principles:**
- **P1 – One accountable team.** Every entity, including every config object, has exactly one owning
  team. An optional **accountable individual** is derived (the owner team's lead) or set explicitly.
- **P2 – Declared, then verified.** Ownership is declared in code, next to the thing, and continuously
  compared with *observed* signals: CODEOWNERS, contribution share, on-call, IdP team validity.
- **P3 – Ownership decays.** It has a verification cadence, orphan detection when teams dissolve, and
  pruning of inactive members.
- **P4 – Ownership is operational.** The same owner data routes reviews, notifications, approvals and
  permissions.
- **P5 – Overrides are rare and loud.** Ownership overrides and break-glass access are restricted
  relations, audited and notified.
- **P6 – Recursion.** The portal's own objects follow P1–P5, and the meta scorecards are themselves
  owned and scored.

## 4. The meta model

Built-in kinds in the `toolbox` namespace, **derived from the manifest at build time and from
runtime state**. They are never hand-authored as YAML: the definition in code *is* the entity.

| Kind | Source of truth | Key relations | Key facts |
|---|---|---|---|
| `Portal` | Portal program | `hasPart` → everything below | version, deployed digest, environments |
| `Module` | Portal program (folder/package grouping) | `contains` → definitions; `ownedBy` | path, CODEOWNERS teams |
| `Package` | `toolbox.lock` | `provides` → definitions; `publishedBy`; `stewardedBy` (local team) | version, digest, signature status, license |
| `Kind` | Manifest | `governedBy` → AuthzModule; `fedBy` ← Connector; `scoredBy` ← Scorecard | entity count, field doc coverage, facets |
| `Connector` | Manifest + runtime | `feeds` → Kind; `provides` → Fact types; `uses` → credential scope | lag, error rate, link coverage, permission probe, last sync |
| `Scorecard` | Manifest | `appliesTo` → Kind; `hasRule` → Rule; `hasInitiative` | entities in scope, level distribution |
| `Rule` | Manifest | `reads` → Fact types / fields (from **CEL static analysis**); `remediatedBy` → Action | pass/fail/skip/unknown rates, flapping index, waivers |
| `RuleResult` (virtual) | Outcome store | `of` → Rule; `for` → entity | outcome, remark, since. *Not stored as entities* (see below) |
| `Action` / `Automation` | Manifest | `scopedTo` → Kind; `executesVia` → Connector/executor | runs, success rate, p95 duration, last used |
| `View` | Manifest (operator-authored UI views) | `reads` → Kind/Scorecard/Fact (from its declared queries); `shownIn` → slots | page views (from analytics), query errors |
| `AuthzModule` | `fga.mod` | `governs` → Kind; `providedBy` → Package | test coverage %, relations without assertions |
| `NotificationPolicy`, `Matcher`, `PlanGates` | Manifest | `ownedBy`; `appliesTo` | volumes, gate triggers |
| `MCPTool` | Generated agent surface | `exposes` → Action/Kind | calls, error rate, response tokens (from analytics) |
| `AgentClient` | Registered OAuth clients | `actsFor` → users; `ceiling` | activity, failures |

The portal's own runtime is cataloged as ordinary software:
- `system:toolbox/portal`
- the server `Component`
- its `Resource`s (Postgres, object storage)
- its `API`s (REST, MCP)

These are scored by the same production-readiness scorecard as every other service.

**Why rule results aren't entities (vs. Port's `_rule_result`):** 100 rules × 100k entities = 10M
results. Port documents a 5M rule-result limit. Toolbox keeps outcomes in a columnar outcome store,
with the history archive from [incremental-computation.md](./incremental-computation.md) §3. It exposes them as a **virtual
`RuleResult` kind**: queryable, relatable and permission-faceted like entities, without paying entity
overhead.

## 5. Ownership resolution for meta entities

Each definition gets an owner from the first source that applies:

1. Explicit `owner:` on the definition (the SDKs require it for new definitions unless a module
   default exists).
2. **Module default:** `new tb.Module("security", { owner: "group:default/security" })`.
3. **Package steward:** definitions from third-party packages have `publishedBy` (upstream, e.g.
   `toolbox-dev`) and a required **local steward** team (the team accountable for adopting and
   upgrading it).
4. **CODEOWNERS** of the defining file, mapped from GitHub team to Group via external ids.
5. The `Portal` owner, as a fallback. The entity is then flagged `owner: inherited-fallback`, which
   fails the bronze rule below.

**Consistency check (P2):** the declared owner must match the CODEOWNERS team for the definition's
file. A mismatch means review routing and accountability have diverged. This is caught by a default
rule and by `plan`.

## 6. How the meta catalog is used

| Use | Mechanism |
|---|---|
| **Route data problems to data owners** | An outcome is `unknown` because a fact is stale → follow `Rule.reads` → fact → `Connector` → notify the connector owner. Service owners see "unknown (data stale, owned by @security)" rather than a false failure |
| **Contest a rule** | Day-2 actions on `Rule` entities: *request waiver* (PR to the waivers file, reviewed by the rule owner), *report false positive* (issue assigned to the rule owner), *propose change* (agent drafts a PR) |
| **Ownership-aware review** | `plan` computes the touched meta entities and their owners, and requires approval from each affected owner team ([sdlc.md](./sdlc.md) §6) |
| **Impact analysis** | "What breaks if we remove `facts.snyk.critical`?" → `impact(fact)` → rules, views, initiatives, actions and their owners |
| **Agent answers** | MCP `get`, `graph`, `explain` and `impact` work on meta entities unchanged, e.g. "who owns the PCI scorecard and which connectors does it depend on?" |
| **Permissions** | Meta entities participate in ReBAC. Rule *definitions* are visible to all, but a rule's per-entity results follow the details facet. Owners of a `Kind` can approve its schema changes |
| **Portal health view** | Built-in view over meta entities: ownership coverage of portal parts, connector freshness, rule quality, unused actions, dead kinds |

## 7. Default scorecards shipped with every portal

All of these are owned by the portal's platform team by default (reassignable), and are overridable
or extendable in code. CEL shown is illustrative. "Facts" are maintained by Toolbox from the manifest,
analytics and runtime health.

### 7.1 `ownership` (applies to **every** kind, meta included)

| Level | Rule | Check |
|---|---|---|
| bronze | Has an owner | `has(entity.spec.owner)` and the owner is not the fallback |
| bronze | Owner exists and is a team | `entity.owner.kind == 'Group'` (a User owner yields a remark, not a pass) |
| silver | Owner team is active | `entity.owner.facts['members.active'] >= 2` (bus factor) and the team exists in the IdP |
| silver | Owner team is reachable | `has(entity.owner.spec.notify)` |
| silver | Matches CODEOWNERS | `entity.facts['codeowners.team'] == entity.spec.owner` (repos and meta entities) |
| gold | Verified recently | `now - entity.facts['ownership.verified_at'] < duration('180d')` (verification is a day-2 action) |
| gold | Observed ownership agrees | `entity.facts['contrib.top_team'] == entity.spec.owner` over 90 days (Bird et al.-style contribution share ≥ 50%) |

### 7.2 `connector-health` (Connector)

| Level | Rule |
|---|---|
| bronze | Owner + description; digest-pinned; signature verified |
| bronze | Last successful sync within the freshness SLO (`lag < 2 × schedule`) |
| silver | Permission probe green **and not over-privileged** |
| silver | Identity link coverage ≥ 90% (no unresolved conflicts) |
| gold | Error rate < 1% over 7 days; runbook linked; contract tests (`toolbox connector test`) pass in CI |

### 7.3 `scorecard-quality` (Scorecard, Rule)

| Level | Rule |
|---|---|
| bronze | Scorecard and every rule have an owner and description; every rule has `fix` text |
| bronze | Every rule has fixture tests covering at least one pass and one fail |
| silver | Not **always-passing** (> 99% pass for 90 days → retire or raise the bar) and not **never-passing** (0% for 30 days → broken or unrealistic) |
| silver | Not **flapping** (outcome changes > N times in 7 days for > 5% of entities) |
| silver | `unknown` rate < 5% (data dependencies healthy) |
| gold | A remediation action exists for each failing-prone rule; no expired waivers; part of an initiative or a level policy |

**Recursion:** `scorecard-quality` applies to itself and to every default scorecard here. The
evaluation is well-founded because rules read *definitions* and *outcome statistics from previous
cycles* (≤ t−1), never the outcome being computed.

### 7.4 `kind-quality` (Kind)

| Level | Rule |
|---|---|
| bronze | Owner, description; ≥ 1 entity, or marked `experimental` |
| silver | ≥ 90% of fields documented (agent readability); sensitive fields assigned to facets |
| silver | Has fixtures and derivation tests ([authorization.md](../authorization.md) Level 2) |
| gold | ≥ 95% of its entities pass `ownership` bronze |

### 7.5 `action-quality` (Action)

| Level | Rule |
|---|---|
| bronze | Owner, agent-grade description, documented input schema, explicit annotations (`destructive`/`idempotent`) |
| silver | Dry-run supported; tests; destructive → approval policy |
| gold | Success rate ≥ 95% over 30 days; used in the last 90 days (otherwise a retirement candidate) |

### 7.6 `view-quality`, `authz-quality`, `package-hygiene`

| Scorecard | Rules |
|---|---|
| View | Owner; valid queries; snapshot tests; viewed in the last 60 days |
| AuthzModule | Owner; Level 1 test coverage ≥ 80% of relations; no `user:*` on details facets without an explicit acknowledgement |
| Package | Local steward; ≤ 1 minor version behind; signed; license on allowlist |

### 7.7 `production-readiness` applied to the portal itself

The same default scorecard every service gets: on-call, runbook, SLOs, backups verified, alerts
linked. It is applied to `system:toolbox/portal`. The portal's operators get the same dashboard issue
and notifications as any team.

## 8. Distributed ownership: a reference layout

```
acme-portal/
├── Toolbox.yaml
├── CODEOWNERS
│     /modules/core/        @acme/platform
│     /modules/security/    @acme/security
│     /modules/sre/         @acme/sre
│     /modules/finops/      @acme/finops
├── modules/
│   ├── core/       (portal, kinds, ownership policy, default scorecards, authz core)   owner: platform
│   ├── security/   (security scorecard, snyk connector, security authz module)          owner: security
│   ├── sre/        (prod-readiness, pagerduty connector, incident kinds, on-call views)    owner: sre
│   └── finops/     (cost connector, cost facets, cost views)                               owner: finops
└── authz/fga.mod   (lists modules' .fga files)
```

- Each module is a `Module` entity whose owner defaults to its definitions' owners.
- **Cross-module changes require cross-team approval**, computed from the meta graph rather than file
  paths alone. For example, security extending `type service` in its authz module touches `Kind:Service`
  (owned by platform), so `plan` requires platform's approval.
- Teams can ship modules as **packages** reused across portals (e.g. an internal
  `@acme/security-standards` package consumed by several business-unit portals).

## 9. API, MCP and CLI

- Meta entities are ordinary entities under `kind:toolbox/...`. There is no special API, so every
  existing tool works on them.
- Shortcuts:
  - `toolbox meta` (CLI)
  - the MCP prompt `portal_overview`
  - `GET /v1/meta/summary` (counts, ownership coverage, health)
- `explain` on any outcome includes the meta chain: rule owner, data sources, connector health.

## 10. Phasing

| Phase | Scope |
|---|---|
| 1 | Meta kinds derived from the manifest (`Portal`, `Module`, `Package`, `Kind`, `Connector`, `Scorecard`, `Rule`, `Action`); required owners with resolution order; `ownership` scorecard bronze/silver; the portal cataloged as a system |
| 2 | `scorecard-quality`, `connector-health`, `kind-quality`; `RuleResult` virtual kind; CEL `reads` edges; routing of `unknown` outcomes to connector owners; portal-health canvas |
| 3 | `action-quality`; Rule day-2 actions (waiver, false positive); ownership-aware review in `plan` |
| 4 | `view-quality`, `authz-quality`, `package-hygiene`; contribution-based ownership facts; verification cadence |

## 11. Open questions

1. Should individuals ever be valid owners (small orgs), or always teams with an accountable lead?
   *Proposal:* teams required, with an org-size override.
2. Verification cadence default: 180 days (proposed), or tied to reorganisation events from the IdP?
3. Should meta entities be hidden from non-admins by default? *Proposal:* visible to all (transparency
   about standards and who owns them), with operational facts (credentials scope, error details) in an
   `ops` facet.

## Sources

- Backstage descriptor format (Template/Component `spec.owner`):
  [backstage.io](https://backstage.io/docs/features/software-catalog/descriptor-format)
- Port, scorecards as blueprints: [docs.port.io/scorecards/scorecards-as-blueprints](https://docs.port.io/scorecards/scorecards-as-blueprints)
- OpsLevel check ownership: [Checks and filters](https://docs.opslevel.com/docs/checks-and-filters) ·
  [Ownership framework](https://www.opslevel.com/resources/the-opslevel-ownership-framework)
- Cortex ownership: [docs.cortex.io ownership](https://docs.cortex.io/ingesting-data-into-cortex/entities/ownership)
- Datadog custom rules (owner field):
  [docs.datadoghq.com](https://docs.datadoghq.com/internal_developer_portal/scorecards/custom_rules)
- Research:
  - Bird et al. 2011, [Don't Touch My Code!](https://www.microsoft.com/en-us/research/?p=161465)
  - Greiler, Herzig, Czerwonka 2013, [Code Ownership and Software Quality: A Replication Study](https://www.microsoft.com/en-us/research/publication/code-ownership-and-software-quality-a-replication-study/)
  - [Ownership across Windows, Eclipse, Firefox](https://www.microsoft.com/en-us/research/?p=160365)
- OWNERS practice: [Kubernetes OWNERS files](https://www.kubernetes.dev/docs/guide/owners/) ·
  [Chromium code reviews / OWNERS](https://chromium.googlesource.com/chromium/src/+show/47cd4f581fdc93cbca03bc94de084cd418c8c59e/docs/code_reviews.md)
- Org design: [Team Topologies key concepts](https://teamtopologies.com/key-concepts) ·
  [Service per team](https://microservices.io/patterns/decomposition/service-per-team.html)
- ITSM view:
  [IF4IT: named service owner](https://if4it.org/best-practices/service-catalog/assign-a-named-service-owner-to-every-service-in-the-catalog/) ·
  [IF4IT: never orphaned](https://if4it.org/best-practices/service-catalog/ensure-service-ownership-is-always-current-and-never-orphaned/) ·
  [Xurrent: service owner](https://www.xurrent.com/glossary/service-owner)
