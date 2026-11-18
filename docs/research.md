# Toolbox — Landscape Research & Validation

*Research date: 2026-11-18. Primary sources were read where the network allowed (Backstage docs from the
`backstage/backstage` repo, Port's Terraform provider schemas, Port Ocean and MCP READMEs, competitor
READMEs on GitHub, Datadog docs). Vendor docs sites for Port, Cortex, OpsLevel, DX and Compass were not
reachable from the research environment, so those sections rely on search-result excerpts of those docs
and are marked accordingly. Treat vendor-blog statistics as directional.*

---

## 1. TL;DR

1. **The market has moved to agents.** All the major IDPs added an agent surface in 2025–2026. Port
   rebranded around an "Agentic Engineering Platform" with a *Context Lake*. Cortex ships an MCP server
   and a "Context Graph". Backstage added an Actions Registry exposed over MCP, plus an `AiResource`
   catalog kind. DX was bought by Atlassian (~$1B) to measure AI impact. In every case, though, the agent
   interface is **bolted onto a human-first, DB-configured product**.
2. **Backstage's pain is structural.** Adopters own a scaffolded TypeScript app, so upgrades are the #1
   pain (56% of adopters, per a Roadie survey quoted secondhand). It typically needs 3+ FTE, and the
   frontend imposes a "TypeScript tax" on platform teams that mostly write Go, Python and YAML. Being *a
   dependency you pin, configured in code* directly targets this.
3. **The commercial tools store configuration in a database.** Port, Cortex, OpsLevel, DX and Datadog
   all treat Terraform/GitOps as a *client* of DB-held config. Cortex even has a toggle that locks UI
   editing once GitOps is enabled. No tool treats the portal itself as a build artifact of a repo.
4. **No existing open-source project fits the target.** The target is: agent-first, config as code, Rust
   core, API+MCP+CLI+UI, and catalog+scorecards+actions+knowledge+search. The closest projects each cover
   a slice (§5). The gap is real, but small projects in the same area are appearing monthly, so speed
   and a clear thesis matter.
5. **The approach holds up, with three amendments** (§7):
   - (a) Be Backstage-catalog-compatible on day one.
   - (b) Make "edit in UI" produce a pull request instead of mutating state.
   - (c) Offer more than one extension mechanism: WASM for pure logic, a process protocol for
     connectors, executors for actions, and web components for UI.

---

## 2. Backstage (Spotify / CNCF)

**What it is.** A TypeScript framework for building portals. Adopters scaffold an app (`packages/app`,
`packages/backend`) and compose plugins. It has about 35k GitHub stars and 250+ community plugins. The
new backend system is now the default, and legacy backend support was removed in v1.37.

### Catalog model (from `docs/features/software-catalog/*`)
- Entities use a Kubernetes-like envelope: `apiVersion`, `kind`, `metadata` (`name`, `namespace`,
  `title`, `description`, `labels`, `annotations`, `tags`, `links`), `spec`, plus the derived
  `relations` and `status`. Entity refs look like `kind:namespace/name`. Descriptors are usually
  `catalog-info.yaml` files next to the code.
- Built-in kinds: Component, API, Resource, System, Domain, Group, User, Location, Template. The new
  `@backstage/plugin-catalog-backend-module-ai-model` adds an `AiResource` kind (`spec.type: skill | rule`,
  with `agents`, `disciplines`, `dependsOn`). It models MCP servers as `kind: API`, `spec.type: mcp-server`.
- **Pipeline:**
  1. *Entity providers* ingest unprocessed entities. Each provider owns its set, and **no two providers
     may emit the same entity**.
  2. *Processors* validate and mutate entities and emit relations and child entities. Processors run in
     a distributed loop scheduled by timestamp.
  3. *Stitching* assembles the final entity and refreshes the filter "search table".
- Stitching is skipped when a hash of body, relations, errors and parents is unchanged. Array reordering
  (e.g. `metadata.tags`) changes the hash and causes re-stitching churn. Stitching is "a fixed process,
  that cannot be modified or extended". Deferred stitching via a worker queue is now the only mode (v1.52).
- **Orphans:** when a parent stops emitting a child, the child gets a `backstage.io/orphan: 'true'`
  annotation and is kept unless `orphanStrategy: delete` is set.
- **Stated pitfalls (FAQ):**
  - Never call external systems, or the catalog itself, from processors. The loop "races through all
    entities" and work piles up.
  - Don't hard-fail on soft errors such as dangling relations, because the catalog is *eventually
    consistent*.
  - Don't model fine-grained versions in the catalog. It holds "rarely changing, human curated" data.
  - Recent releases fixed query planning that made filtered lists 10–40× slower, and entities that
    disappeared after errors during stitching.

### Other subsystems
- **Scaffolder (self-service):** Templates are entities with JSON-Schema `parameters` (form steps) and
  `steps` of named actions (`fetch:template`, `publish:github`, `catalog:register`). The actions run
  in-process in Node.
- **TechDocs:** "Docs like code". MkDocs, which is Python, builds static HTML, ideally in CI, and the
  output is published to object storage. The reader strips JS and applies transformers.
- **Search:** Pluggable engines (Lunr, Postgres, Elasticsearch/OpenSearch) behind a collator/indexer
  architecture. This is *separate* from the catalog's filter table, so "search" means two different
  things in Backstage.
- **MCP (`@backstage/plugin-mcp-actions-backend`):**
  - Exposes Actions-Registry actions as MCP tools over Streamable HTTP at `/api/mcp-actions/v1`.
  - Tool names are namespaced as `plugin-id.action`. Named sub-servers filter actions by id glob or by
    attribute (e.g. exclude `destructive: true`).
  - Attributes `readOnly`, `idempotent` and `destructive` have conservative defaults.
  - Server `instructions` are configurable.
  - External auth relies on static tokens, which the docs call "a temporary workaround". Spotify's
    docs say **the registry does not enforce permissions automatically**: each action must check them
    itself.

### Adoption economics (secondary sources, mostly vendors selling alternatives)
- Upgrades are the top pain point (56%). 70% of "very happy" adopters still dedicate ≥3 FTE. Typical
  production readiness takes 3–12 engineers and 6–12 months, and estimated all-in cost is
  $200k–$1M+/year. Internal adoption is often low (~10% is cited).
- Port's "Backstage is dead" essay argues that AI now builds portal UI in minutes, so Backstage's main
  2020 value (a head start on UI) has evaporated. This bears directly on our thesis: **agents should
  generate the portal, and the framework should make that safe and upgradable.**

### Lessons for Toolbox
| Keep | Fix |
|---|---|
| `catalog-info.yaml` envelope and entity refs (a de facto standard; import it unchanged) | Allow **multiple sources per entity** (fragments + precedence), not one provider per entity |
| Eventual consistency; soft errors become status, not rejection | Make the merge/stitch step **extensible** (pure WASM transforms) and canonicalize arrays to avoid hash churn |
| JSON Schema for action inputs | Run side effects in **external executors**, not in the portal process |
| MCP tool attributes (readOnly/destructive/idempotent) and namespacing | Enforce authorization **centrally** for every surface |
| Docs-like-code, built in CI | Render Markdown natively; no Python/MkDocs dependency (still read `mkdocs.yml` nav) |
| — | One search system for entities, docs, actions and APIs |
| — | Ship as a **versioned dependency**, not a scaffold the adopter owns |

---

## 3. Commercial portals

### Port (getport.io / port.io)
- **Data model:** User-defined *blueprints* (kinds), held in Port's DB. Each blueprint has typed
  `properties`, `relations`, `mirror_properties` (fields read through relations), `calculation_properties`
  (computed), aggregation properties and `ownership` (Direct or Inherited via a relation path). Entities
  are instances. (Source: `port-labs/terraform-provider-port-labs` resource schemas.)
- **Scorecards:** Defined per blueprint. Rules carry a `level` (e.g. Bronze/Silver/Gold) and a `query` of
  `{combinator, conditions[{property, operator, value}]}`.
- **Actions:** One `action` resource with exactly one invocation method: `webhook_method`,
  `github_method`, `gitlab_method`, `azure_method`, `kafka_method`, `upsert_entity_method` or
  `integration_method`.
  - *Self-service* triggers have user inputs, and *automation* triggers react to entity or run events,
    with JQ conditions and timer expiry.
  - Approvals use `required_approval` = `ANY` or `ALL`, with notifications.
- **Integrations:** *Ocean* is an open-source Python framework. It maps third-party API objects to
  entities with **JQ mappings**, and supports scheduled resync plus real-time webhooks. (`port-labs/ocean`,
  about 200 stars.)
- **AI:** Positioned as an "Agentic Engineering Platform". The *Context Lake* is a machine-readable,
  searchable store of services, teams, ownership, dependencies and policies, exposed over MCP. There is
  also *AI Builder* ("vibe-coding" the portal), and Port raised a $100M Series C (Dec 2025).
  - The local `port-mcp-server` repo is **deprecated** in favour of a remote MCP server.
  - Its tools are mostly admin CRUD (`get_blueprints`, `create_blueprint`, `create_entity`, scorecard
    and action CRUD) plus `run_action`. They run with the caller's permissions.
- **Config as code:** Terraform and Pulumi providers exist, but they are *clients* of DB config, so
  drift between code and DB is possible.
- **Lesson:**
  - Port's blueprint model (mirror, calculated and aggregated properties) is the most expressive
    commercial schema, and worth matching *in code*.
  - JQ mappings are a proven way to customize ingestion without writing a connector.
  - An agent surface made of CRUD tools is a weak design for agents.

### Cortex (cortex.io)
- **Scorecards:** Every rule is a **CQL** query, e.g. `ownership != null`,
  `links("runbook").length > 0` or `git.numOfRequiredApprovals() > 0`. Levels are progressive: an entity
  must pass all rules of a level to advance. Point-based scoring is an alternative.
- **Scorecards as code:** YAML files under `.cortex/scorecards/` in a dedicated repo. Once GitOps is
  enabled, **UI editing is locked**. Entities are defined in `cortex.yaml` files. There is a Terraform
  provider.
- **AI:**
  - The MCP server (July 2025) is mostly read access plus CQL queries, with limited writes.
  - "Context Graph" is pitched as the map coding agents consult.
  - Cortex open-sourced **engrams**, a *Rust* self-hosted agent "software factory" that runs Firecracker
    microVMs. It signals where the vendor sees value: running agents, not just informing them.
- **Lesson:**
  - A dedicated rule language is a strength for scorecards, but proprietary syntax is a barrier for
    agents. Prefer a standard expression language (CEL).
  - The "lock UI once GitOps is on" behaviour validates config-as-code-only for serious users.

### DX (getdx.com, now Atlassian)
- Atlassian agreed to acquire DX for ~$1B (Sept 2025), to sit alongside Compass, Jira, Bitbucket and
  Rovo Dev. Its core strength is developer-experience measurement, with about 60 connectors.
- **Fabric / Service Cloud** bundles a catalog, scorecards and self-service workflows. **Scorecard checks
  are written in SQL** against catalog tables (e.g. `dx_catalog_entities`) in its data cloud. Checks can
  be managed in the UI, CLI or API, and can import from Backstage or Compass.
- **Lesson:**
  - SQL is the most agent-familiar language for *aggregate, cross-entity* checks, e.g. "% of services
    on the latest base image". Toolbox should support SQL rules after CEL.
  - Metrics and DORA analytics are DX's territory. Toolbox should *ingest* such facts, not compete on
    analytics.

### OpsLevel
- **Model:** A **rubric** of levels × categories, with checks placed in cells. Levels are global and
  shared by rubrics and scorecards. **Campaigns** are time-boxed pushes to get checks passing, and can be
  converted to and from checks. **Custom event checks** accept arbitrary JSON pushed to a per-integration
  endpoint and evaluate it. `opslevel.yml` defines components and ownership as code.
- **Lesson:**
  - Campaigns (initiatives with deadlines) and push-based facts are both needed.
  - Shared global levels keep scorecards comparable.

### Datadog Software Catalog / IDP (docs read directly)
- **Entity schema v3:** Kubernetes-style YAML (`apiVersion: v3`, `kind` = service | queue | datastore |
  system | api | custom, plus `metadata`, `spec`, `integrations` and `extensions`). Components inherit
  system metadata via `inheritFrom`. A `datadog.codeLocations` block maps path globs to entities. Most
  entities and dependencies are **auto-discovered from telemetry** (APM, USM, RUM), and declared ones
  augment them.
- **Scorecards:** Outcomes per `{rule, entity}` are `pass | fail | skip` with a `remarks` field. Default
  scorecards are evaluated every 24h. Custom rules are evaluated by Workflow Automation (JS transform
  steps) and posted via `/scorecard/outcomes/batch`.
- **Lesson:**
  - `skip` (not applicable) as a first-class outcome, and a remark explaining *why*, are both important
    for agent explanations.
  - Telemetry-derived dependencies beat hand-maintained ones. Toolbox connectors should emit relations
    with provenance and confidence.

### Others
- **Atlassian Compass** offers `compass.yml` config as code and Forge-based extensions. It now overlaps
  with DX inside Atlassian. (Docs not reachable; no further detail claimed.)
- **Backstage distributions:**
  - Roadie (hosted Backstage, with Tech Insights scorecards).
  - Spotify Portal for Backstage (commercial, MCP).
  - Red Hat Developer Hub (dynamic plugins).
  - Harness IDP (Backstage-based).
  All of them inherit Backstage's model and its upgrade economics, managed for you.
- **Platform orchestrators** (not portals, but adjacent): Humanitec, Kratix (Promises on Kubernetes),
  OpenChoreo (WSO2, CNCF Sandbox, Go, Backstage UI, MCP and agents), Radius, KusionStack, Score. Toolbox
  should *integrate* with these as action executors and connectors, not replace them.

---

## 4. Cross-vendor feature matrix

| Capability | Backstage | Port | Cortex | DX | OpsLevel | Datadog |
|---|---|---|---|---|---|---|
| Custom kinds/schema | Code (TS kinds, validators) | Blueprints (DB, TF client) | Custom entity types | Catalog types | Component types | v3 `kind` + custom |
| Entity definitions in repo | `catalog-info.yaml` | via Ocean/GitOps | `cortex.yaml` | yes | `opslevel.yml` | `entity.datadog.yaml` |
| Multi-source merge per entity | ✗ (one provider) | ✓ | ✓ | ✓ | ✓ | ✓ (telemetry + YAML) |
| Scorecard language | plugins (Tech Insights) | JSON conditions | CQL | SQL | check types | JS in workflows |
| Scorecards in code | partly | TF client | GitOps YAML | CLI/API | TF/API | API |
| Actions | Scaffolder (in-process) | webhook/GH/GL/ADO/Kafka | Workflows | Workflows | Actions | Workflow Automation |
| Docs | TechDocs (MkDocs) | via integrations | via links | — | TechDocs-like | — |
| MCP | Actions registry (perm checks per action) | Remote MCP (CRUD-ish) | Read + CQL | — | MCP (OpsLevel MCP) | — (not found) |
| Portal config lives in | code + app-config | **DB** | **DB** (GitOps optional) | **DB** | **DB** | **DB** |
| Open source | ✓ (Apache-2.0) | Ocean only | ✗ (engrams OSS) | ✗ | ✗ | ✗ |

---

## 5. Open-source alternatives (searched GitHub topics and search, 2026-11)

| Project | Stack / license | What it covers | Gap vs. Toolbox thesis |
|---|---|---|---|
| **Frontkeep** (`glemmestad/frontkeep`) | Rust single binary, SQLite/Postgres, Cedar, MCP/CLI/REST/GraphQL/UI; open core | "Agent control plane": manifest-driven *provisionable-service* catalog, project registry gate, LLM gateway + budgets, cost, knowledge (standards/guidance/recipes) | Closest in spirit (Rust, agents-first, Cedar). Focused on provisioning, LLM spend and governance, not an ownership/dependency catalog, scorecards or a config-as-code portal. Tiny community (single digits of stars). |
| **OpsCat** (`sumant1122/opscat`) | Go single binary, SQLite, Apache-2.0 | Auto-discovered catalog from code scan, dependency graph, fixed health scorecards, MCP server | Zero-config discovery rather than config as code; fixed scorecards; early. |
| **Lekton** (`dghilardi/lekton`) | Rust (Axum + Leptos), MongoDB, Meilisearch, S3; AGPL-3.0 | Docs/schema hub pushed from CI, RAG, MCP, prompt library | Docs only; AGPL; several external dependencies. |
| **OpenChoreo** (WSO2) | Go, Kubernetes CRDs, Backstage UI; Apache-2.0; CNCF Sandbox | Platform orchestration + portal, agents/MCP | Kubernetes-centric orchestrator; Backstage for portal. |
| **idp-core** (`Decathlon/internal-developer-platform`) | Java/Spring, Postgres | Catalog, scorecards, self-service, "IDP backend for the AI era" | Dynamic (DB) data modelling, JVM, early. |
| **ForgePortal** | TypeScript (Fastify, React, PG), MIT | Catalog, Bronze/Silver/Gold scorecards with auto-fix PRs, templates, docs, PG FTS | No MCP; early. |
| **Backline** | Go + htmx | Reads Backstage entities; typed Go plugins | Small; catalog only. |
| **FrontStage**, **Portway**, **Junction** (Rails), **Torii** (Qovery, stale since 2024) | various | Lightweight catalogs | Small, no agent focus. |
| **agentregistry**, **open-mcp-portal**, **STOA** | Go / Go / Rust gateway + Python | Registries and gateways for MCP servers, agents and skills | Not service catalogs, but useful models for the `MCPServer` and `Skill` kinds. |
| **Lyft Clutch** | Go + TS | Infra-management console | **Archived Feb 2026.** |

**Verdict:** No open-source project combines an agent-first design, config as code, a Rust core, the four
interfaces, and the five capability pillars. The Rust-core, agent-first niche is beginning to fill
(Frontkeep, Lekton, Cortex's engrams), which validates both the language choice and the timing. There
is not yet an incumbent in it.

---

## 6. Prior art for "configuration as code" and polyglot extension

| Tool | Pattern | Take for Toolbox |
|---|---|---|
| **Terraform** | HCL config → `plan` (diff vs. state) → `apply`. Providers are separate binaries speaking gRPC via `hashicorp/go-plugin`, pinned in `.terraform.lock.hcl`. Modules come from registries or git. | `toolbox plan` previewing the *effect* of config changes on live data; provider-style connectors pinned by digest in `toolbox.lock`; reusable modules. |
| **Pulumi** | Programs in TS/Python/Go/C#/Java/YAML run by per-language **language hosts**. Thin SDKs send `RegisterResource` gRPC calls to the Go engine's **resource monitor**; the engine diffs against state (`preview`/`up`). Providers and multi-language components are plugins behind a language-neutral protocol; per-language SDKs are **generated from one package schema**. | **Chosen config model.** Language hosts + config monitor → manifest IR; schema-generated SDKs; portal packages shareable across languages; YAML for zero-runtime setups. |
| **Tilt** | Tiltfile in **Starlark**, evaluated into objects registered in an embedded **Kubernetes-style API server**. Query with `tilt get` / `tilt describe` / `tilt explain`. `tilt up` live-reloads. Extensions via `load('ext://…')` from git repos. | Starlark entrypoint; evaluated config becomes typed objects inspectable through the same API; `toolbox dev` live reload; `load()` of shared modules from git. |
| **starlark-rust** (Meta) | Deterministic, hermetic Python dialect used by Buck2; Rust-native; LSP and DAP; optional types; **no API-stability promise**. | Considered as the single config DSL, then superseded by the Pulumi model (real languages). Could return later as another "language" emitting the same manifest. |
| **Pkl / CUE / KCL / Nickel** | Typed configuration languages. KCL and Nickel are Rust-implemented; Pkl's Rust support is community IPC wrappers; CUE is Go. | Strong typing is attractive, but agents know Python syntax far better than any of these. Starlark + JSON Schema validation gives most of the benefit. Revisit KCL/Nickel if typing pain appears. |
| **hashicorp/go-plugin** | Subprocess + gRPC, crash isolation, protocol versioning, local-only. | The model for out-of-process connectors (we use JSON-lines over stdio; see plan). |
| **Airbyte / Singer** | Connectors as containers emitting JSON messages (`spec`/`check`/`discover`/`read`, `RECORD`/`STATE`) on stdout. | The connector message shape; TCP keeps a compatible stdio transport. |
| **MCP** | Capability negotiation, JSON-RPC messages, Streamable HTTP + stdio transports, OAuth for remote servers. | The protocol *style* for TCP over HTTP. Not the semantics: MCP has no sync cursors, deletion/snapshot rules or entity identity. |
| **WASM Component Model** | WIT interfaces, wasmtime host. **WASI 0.3 shipped June 2026** with native async. Guests in Rust, Python (componentize-py), JS (jco; `componentize` still marked experimental), Go (TinyGo / componentize-go). | Sandboxed, deterministic, in-process extension for *pure* logic (rules, transforms, computed fields). Not for connectors. |
| **Extism** | Simpler cross-language plugin ABI (BSD-3). | Fallback if component-model tooling disappoints for a language. |
| **Grafana as code / Foundation SDK** | Dashboards generated from typed SDKs in several languages, provisioned from files. | Optional future TypeScript/Python SDKs that emit the same manifest IR. |
| **Cedar** (AWS) | Authorization policy language and engine in Rust. | Considered first; superseded by ReBAC: relationships must be supplied per request and there are no list primitives. See [authorization.md](./authorization.md). |
| **OpenFGA** (CNCF) | Zanzibar-style ReBAC: DSL, CEL conditions, modular models, `.fga.yaml` model tests, public conformance corpus. | **Chosen authorization language and test format**, evaluated natively over the catalog graph. See [authorization.md](./authorization.md). |
| **CEL** | Non-Turing-complete expression language (Kubernetes admission policies). Rust crate `cel` (cel-rust) is maintained, but conformance should be verified. | One expression language for filters, rules, mappings and conditions. |

---

## 7. Validation of the proposed approach

| Proposal element | Evidence for | Risk / amendment |
|---|---|---|
| **Agents are the primary audience** | Every vendor added MCP in 2025–26; Port's Context Lake and Cortex's Context Graph target coding agents; Backstage added `AiResource` and MCP actions. The current offerings are thin (CRUD tools, per-action permission checks, static tokens). | Agents still need humans for approvals and trust. Keep a solid read-mostly UI. |
| **Config only in code, never in DB** | Cortex locks the UI under GitOps; Terraform/Pulumi providers exist for Port, Cortex and Datadog because customers want code. Backstage's code-centric design is its most loved trait, and its *scaffold ownership* is its most hated. | Non-engineers lose UI editing. **Amend: "Edit" in UI/MCP opens a pull request** against the portal repo or the entity's repo. Agents prefer PRs anyway. |
| **Toolbox as a dependency** | Upgrades are Backstage's #1 pain. Backline's pitch ("just update go.mod") and Frontkeep's single binary target the same thing. | We need a stable, versioned config IR and codemods (`toolbox upgrade`). |
| **Rust core** | Scale (Backstage's catalog loop, stitching and query-planner issues), single-binary ops, a memory-safe sandbox host (wasmtime), and a mature ecosystem (axum, sqlx, tantivy, rmcp, cedar, starlark-rust). New entrants (Frontkeep, Lekton, engrams) chose Rust. | Fewer contributors know Rust than TS. Mitigated because *customers never write Rust*. |
| **Polyglot extensions** | Platform teams write Go, Python and YAML ("TypeScript tax"). Port's Ocean (Python) and Airbyte prove process-protocol connectors. | WASM tooling maturity varies by language. **Amend: tiered extension model**, with the process protocol as the universal fallback. |
| **Scale to thousands of services** | Roadie runs tens of thousands of entities on Backstage only after tuning. | Needs explicit SLOs and a synthetic-org benchmark from week 1. |
| **Four interfaces** | Frontkeep, OpsCat and Lekton all converge on CLI+MCP+REST+UI from one binary. | Generate all four from one definition to avoid drift. |

**Conclusion: proceed.** The differentiator is not "another catalog". It is *a portal that agents can
author, test, plan and safely operate*: `toolbox plan` shows score and permission deltas before merge,
every capability is an MCP tool with consistent authorization, and upgrades are a version bump.

---

## 8. Sources

Backstage (read from `github.com/backstage/backstage/tree/master/docs`):
`features/software-catalog/life-of-an-entity.md`, `descriptor-format.md`, `faq.md`,
`well-known-relations.md`; `ai/mcp-actions.md`; `ai/ai-in-the-catalog.md`;
`features/software-templates/writing-templates.md`; `features/techdocs/architecture.md`;
`features/search/architecture.md`.
- [Backstage new backend system](https://backstage.io/docs/plugins/new-backend-system) ·
  [v1.56 release notes](https://backstage.io/docs/releases/v1.56.0) ·
  [Spotify Portal MCP tools](https://backstage.spotify.com/docs/portal/core-features-and-plugins/mcp/registering-tools)
- [Roadie – Scaling Backstage](https://roadie.io/blog/scaling-backstage-intro) ·
  [Roadie – True cost of self-hosting Backstage](https://roadie.io/blog/the-true-cost-of-self-hosting-backstage/) ·
  [Port – Backstage is dead](https://newsletter.port.io/p/backstage-is-dead)

Port:
- [port-labs/terraform-provider-port-labs](https://github.com/port-labs/terraform-provider-port-labs)
  (`docs/resources/blueprint.md`, `scorecard.md`, `action.md`) · [port-labs/ocean](https://github.com/port-labs/ocean) ·
  [port-labs/port-mcp-server](https://github.com/port-labs/port-mcp-server) (deprecated) ·
  [Port MCP server docs](https://docs.port.io/ai-interfaces/port-mcp-server/overview) ·
  [Port $100M Series C](https://www.port.io/blog/platform-engineering-ai) ·
  [Port agentic engineering platform](https://port.io/blog/port-agentic-engineering-platform)

Cortex:
- [Scorecards as code](https://docs.cortex.io/standardize/scorecards/scorecards-as-code) ·
  [CQL](https://docs.cortex.io/cql) · [Using Cortex MCP](https://docs.cortex.io/get-started/cortex-ai-assistant/mcp/using-cortex-mcp) ·
  [Cortex MCP announcement](https://www.cortex.io/post/mcp-server) · [Context Graph](https://www.cortex.io/products/context-graph) ·
  [cortexapps/engrams](https://github.com/cortexapps/engrams)

DX:
- [DX joining Atlassian](https://getdx.com/blog/dx-is-joining-atlassian/) ·
  [SD Times on the acquisition](https://sdtimes.com/atlassian-enters-into-definitive-agreement-to-acquire-dx/) ·
  [Writing scorecard checks (SQL)](https://docs.getdx.com/scorecards/writing-checks/) · [Fabric overview](https://docs.getdx.com/fabric/overview/) ·
  [Catalog, scorecards & self-service launch](https://getdx.com/blog/introducing-software-catalogs-scorecards-and-self-service/)

OpsLevel:
- [Rubrics](https://docs.opslevel.com/docs/getting-started-with-rubrics) · [Campaigns](https://docs.opslevel.com/docs/getting-started-with-campaigns) ·
  [Custom event checks](https://docs.opslevel.com/docs/custom-event-checks) · [Scorecards](https://docs.opslevel.com/docs/scorecards)

Datadog:
- [Entity model](https://docs.datadoghq.com/internal_developer_portal/catalog/entity_model) ·
  [Scorecards](https://docs.datadoghq.com/internal_developer_portal/scorecards/) ·
  [Custom rules](https://docs.datadoghq.com/internal_developer_portal/scorecards/custom_rules) ·
  [Workflows](https://docs.datadoghq.com/actions/workflows)

Open source:
- [Frontkeep](https://github.com/glemmestad/frontkeep) · [OpsCat](https://github.com/sumant1122/opscat) · [Lekton](https://github.com/dghilardi/lekton) ·
  [OpenChoreo](https://github.com/openchoreo/openchoreo) · [Decathlon idp-core](https://github.com/Decathlon/internal-developer-platform) ·
  [ForgePortal](https://github.com/forgeportal/forgeportal) · [Backline](https://github.com/iamgoroot/backline) ·
  [agentregistry](https://github.com/agentregistry-dev/agentregistry) · [open-mcp-portal](https://github.com/ncecere/open-mcp-portal) ·
  [STOA](https://github.com/stoa-platform/stoa) · [Lyft Clutch](https://github.com/lyft/clutch) (archived) · [Kratix](https://github.com/syntasso/kratix)

Configuration and extension prior art:
- [Tilt API server reference](https://api.tilt.dev/) · [tilt-apiserver](https://github.com/tilt-dev/tilt-apiserver) · [Tilt extensions](https://docs.tilt.dev/extensions.html) ·
  [hashicorp/go-plugin](https://github.com/hashicorp/go-plugin) · [starlark-rust](https://github.com/facebook/starlark-rust) ·
  [WASI 0.3 release](https://wasi.dev/releases/wasi-p3) · [Component tooling compatibility](https://bytecodealliance.org/articles/component-model-tooling-compatibility) ·
  [componentize-py](https://pypi.org/project/componentize-py) · [Extism](https://extism.org/) · [Cedar](https://github.com/cedar-policy/cedar) ·
  [cel-rust](https://docs.rs/crate/cel/0.14.4) · [KCL vs Pkl](https://www.kcl-lang.io/blog/2024-03-22-pkl-kcl-comparison) · [Nickel](https://github.com/nickel-lang/nickel)

Rust building blocks:
- [tantivy](https://github.com/quickwit-oss/tantivy) · [rmcp](https://github.com/modelcontextprotocol/rust-sdk) · [axum](https://github.com/tokio-rs/axum) ·
  [sqlx](https://github.com/transact-rs/sqlx) · [utoipa](https://github.com/juhaku/utoipa) · [wasmtime](https://github.com/bytecodealliance/wasmtime) ·
  [pgvector](https://github.com/pgvector/pgvector) · [LanceDB](https://github.com/lancedb/lancedb) · [Kuzu (archived Oct 2025)](https://github.com/kuzudb/kuzu)

Agent tool design:
- [Too many MCP tools (EclipseSource)](https://eclipsesource.com/blogs/2026/01/22/mcp-context-overload/) ·
  [Amplitude: consolidating our MCP server](https://amplitude.com/blog/lessons-from-consolidating-our-mcp-server) ·
  [Roadie: IDP as context engineering](https://roadie.io/blog/idp-ai-goldmine-context-engineering/)

Pulumi (added after a follow-up review):
- [Pulumi architecture (developer docs)](https://pulumi-developer-docs.readthedocs.io/latest/docs/architecture/README.html) ·
  [Resource registration](https://pulumi-developer-docs.readthedocs.io/latest/docs/architecture/deployment-execution/resource-registration.html) ·
  [Deployment execution](https://pulumi-developer-docs.readthedocs.io/latest/docs/architecture/deployment-execution/README.html)
