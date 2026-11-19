# Technology Critique: Is the Toolbox Architecture Sprawl?

*Written 2026-11-19 as a skeptical principal engineer's **technology-only** review. Go-to-market,
demand, adoption and funding are out of scope (the maintainer set them aside). The question is whether
the architecture in [plan.md](./plan.md) and [design/](./design/) is justified, or whether it is
(a) technology sprawl and (b) replaceable by less vertical, general-purpose building blocks.*

*This builds on [principal-review.md](./principal-review.md) (architecture holes) and
[adversarial-critique.md](./adversarial-critique.md) (whether to build at all). Where they already made
a point, this document cites it and adds evidence instead of repeating it.*

*Evidence labels:* **[P]** primary source read directly or code I ran; **[S]** secondary (search excerpt,
vendor page summary); **[I]** my inference. Vendor docs sites are mostly blocked from this environment
([research-log.md](./research-log.md)); Backstage was read from a clone of `backstage/backstage`
at `184acad` (2026-11-19).

---

## 1. Verdict

**The differentiators are real; the stack built to deliver them is more than twice as large as they need** (sprawl score 77 vs 33, §2.2).
Three new measurements decide most of it. At the v1.0 scale target (100k entities, 2M relations, 10k
users, 100 rules), a **full recompute of every derived value takes 3.0 s in Rust and 2.6 s as plain
Postgres materialised views**, with identical results (§7.1). An **unmodified OpenFGA server reading
derived tuples through a Postgres view** agrees with the SQL permission view on 2,000/2,000 checks, at
2.4 ms p50 over HTTP, with no tuple writes and no FFI (§7.2). And the heaviest proposed dependency,
`dbsp` 0.384, **crashed the stable Rust 1.97 compiler** under the profile the velocity design prescribes
(§7.3). So the incremental circuit (DBSP), the logical-decoding change log, checkpoints in object
storage, content-addressed sub-circuits, the embedded Go c-archive and most of the deterministic
simulation exist to solve problems this scale does not have. Separately, the GraphQL layer, the
`@toolbox/ui` design system with persona-aware Storybook, the Pulumi-style polyglot SDKs, WASM
functions and the TCP-as-a-standard are generic web-app or platform concerns that no Port pain and no
hard requirement needs in v1. What survives is small and genuinely IDP-specific: a **manifest in git
with `plan` diffs**, **field-level source precedence**, **CODEOWNERS-backed ownership**, **facet
shaping with declared declassification**, **a curated MCP tool surface**, all over **one Rust binary,
Postgres, and an OpenFGA sidecar**. Backstage's compatibility obligations block **none** of the
maintainer's four claimed innovations (speed, MCP, config as code, DX); they are shipped or merely
unimplemented there. What Backstage's model genuinely constrains is **field-level source of truth** (one
source per entity) and **facet-level authorization** (whole entities to every plugin), which is where
Toolbox's real technical case lies (§4).

| # | Component (as designed) | Verdict | One-line reason |
|---|---|---|---|
| 1 | Rust core, single binary (`api\|worker\|all`) | **Keep** | Single static binary and predictable latency are cheap here; but performance is *not* the justification (Postgres matched it, §7.1) |
| 2 | Postgres as the only state store | **Keep** | Does every v1 job measured: inputs, derived views, jobs, audit |
| 3 | DBSP incremental circuit (D21) | **Cut / defer** | Full recompute is 3.0 s vs a 60 s freshness SLO and a 5 min rescore SLO; dbsp 0.384 ICEs stable rustc 1.97 |
| 4 | Logical-decoding change log + log positions | **Replace with generic** | Needed only by incremental consumers; a recompute over a `REPEATABLE READ` snapshot has no gap problem. Snapshot id = consistency token |
| 5 | Object storage (checkpoints, history archive) | **Cut / defer** | Checkpoints exist for DBSP; history fits a partitioned Postgres table at this scale |
| 6 | DBOS-model durable workflows (D22) | **Replace with generic** | Phase 1–2 have no multi-step side effects; a Postgres job table (`SKIP LOCKED`) plus executors that own durability (GitHub Actions) suffices until Phase 3 |
| 7 | OpenFGA embedded via Go c-archive (D26) | **Replace with generic** | OpenFGA as an unmodified sidecar over a Postgres tuple *view* keeps "no duplicate writes" and exact semantics (§7.2) |
| 8 | OpenFGA DSL + `.fga.yaml` as the authz contract (D7) | **Keep** | Standard, testable, and the sidecar runs it unchanged |
| 9 | Facets + information-flow labels + declassification (D15) | **Keep (simplified)** | Genuine differentiator; labels can be computed statically from CEL field references at build, not propagated through a circuit |
| 10 | CODEOWNERS authority (D17) | **Keep** | Cheap, IDP-specific, closes the squatting hole |
| 11 | Per-field `sources` precedence + native store (D13) | **Keep** | The merge measured at 1.6–4.6 ms for 100k entities; native data is one revisioned table |
| 12 | Pulumi model: language hosts, config monitor, polyglot codegen (D3) | **Cut / defer** | Replace with a YAML/JSON manifest + JSON Schema, and one TS typed builder that prints JSON (no host, no gRPC) |
| 13 | Manifest + `toolbox plan` (score, authz, mapping diffs) | **Keep** | The clearest differentiator; implemented as two full recomputes over one snapshot and a diff |
| 14 | CEL for predicates | **Keep** | Standard, shared with OpenFGA conditions; cel-rust runs it (2.7 s for 10M evaluations on 4 threads) |
| 15 | CEL for mappings + own CEL type checker | **Defer** | Validate mapping outputs against kind JSON Schemas on fixtures; write a type checker only if fixtures miss real errors |
| 16 | WASM functions (wasmtime, WIT, Javy) | **Cut / defer** | No v1 rule needs Turing-complete in-engine logic; connectors can compute facts |
| 17 | TCP as an open, governed protocol (D4) | **Replace with generic** | Keep the semantics (raw records, snapshot markers, deletion valves) as an OpenAPI ingest endpoint; no spec governance |
| 18 | REST + OpenAPI | **Keep** | The base projection everything else should call |
| 19 | MCP server (~12 task tools) | **Keep** | Differentiator; generic OpenAPI→MCP generators produce CRUD-per-endpoint tools, the opposite of the design |
| 20 | CLI | **Keep** | Thin client; local `plan`/`test` |
| 21 | GraphQL, read-only, trusted documents (D30) | **Cut / defer** | A first-party UI needs a handful of view endpoints, not a query language, cost analysis and a document registry |
| 22 | React SPA + `@toolbox/ui` + Storybook 10 page/persona stories + fixture recording (D31) | **Replace with generic / defer** | A web-framework concern. Ship a minimal read-only UI; let operators build rich UIs on the public API (§6) |
| 23 | `toolbox dev` daemon with Salsa-memoised pipeline, warm language hosts | **Replace with generic** | A watcher plus full recompute over 2k fixture entities is tens of milliseconds [I, from §7.1] |
| 24 | tantivy + vectors (search) | **Defer** | Phase 4; Postgres full-text search first, pgvector if needed |
| 25 | SQLite mode | **Cut** | A second store doubles conformance work; dev uses in-memory, prod Postgres |
| 26 | Deterministic simulation (turmoil), Antithesis, Kani, shuttle, covering arrays | **Cut / defer** | Most target the replicated incremental design that §7.1 removes; the full recompute *is* the reference model |
| 27 | Velocity tooling (hakari, guppy `xtask affected`, content-addressed test cache, sub-circuits) | **Defer** | Sized for a 31-crate workspace; a small workspace with nextest needs none of it |
| 28 | Observability: OTel + Prometheus | **Keep** | Generic and cheap |
| 29 | Grafana pack, PostHog sink, versioned event catalog | **Defer** | Useful, not differentiating |
| 30 | Agent plugins: operator plugin (exists) / generated user plugin | **Keep / defer** | The operator plugin exists and validates; the generated one waits for a stable MCP surface |
| 31 | AuthZEN endpoint, GQL subset, context lake inventory tier | **Defer** | No v1 requirement |

---

## 2. Sprawl inventory and score

### 2.1 Inventory

Every language, runtime, DSL, protocol, store, toolchain and major dependency named in the design.
"Needed by" names the feature or interface that pulls it in; "Generic substitute" is the boring
alternative.

| Item | Problem it solves | Needed by | Cost (build, CI, knowledge, ops, failure modes) | Generic substitute |
|---|---|---|---|---|
| **Rust** | Core engine, single binary | Everything server-side | Slow clean builds (§7.3); fewer agent/contributor hours of training data than TS [I] | TypeScript/Node, Go |
| **Go via C ABI** (cgo c-archive) | Embed OpenFGA in-process | D26 | Second runtime + GC + signal handlers in one process; 78 MB archive, 45 s cold build (§7.4); determinism under simulation needs `GOMAXPROCS=1` or a wasip1 build | OpenFGA sidecar (§7.2) |
| **TypeScript** | UI, SDK runtime, Storybook, Javy functions | D31, D3, D23 | Node, Vite 8, TS 7, Vitest, Playwright in CI | Required anyway for any UI |
| **Python** | Second SDK; Ocean runner | D3, ocean.md §4 | Language host, `uv`, PyPI publishing, codegen target | YAML manifest |
| **YAML/JSON manifest + JSON Schema** | The config contract | D2, D3 | Low | — (keep) |
| **CEL** (cel-rust + own type checker) | Predicates and mappings | D24 | cel-rust skips ~25% of conformance tests and has no checker ([adversarial-critique.md](./adversarial-critique.md) H5) | CEL without static typing + fixture tests; JQ/jaq for mappings |
| **WASM** (wasmtime, WIT, Javy/StarlingMonkey, QuickJS) | Logic beyond CEL | D23, D24 | wasmtime (Cranelift) is a large build dependency (§7.3); three toolchains for function authors; opaque to info-flow labels | None in v1; connectors compute facts |
| **OpenFGA DSL + `.fga.yaml`** | Authorization model and tests | D7 | One more author-facing language, well documented | — (keep) |
| **GraphQL** (async-graphql dynamic schema, trusted documents, IBM cost directives) | Rich entity pages | D30 | Schema generation, document registry, cost analysis, batched authz loaders, plan diffs for schema | View endpoints in REST |
| **REST / OpenAPI 3.1** (axum, utoipa) | Base API | Plan §4.5 | Low | — (keep) |
| **MCP** (rmcp, Streamable HTTP, OAuth 2.1) | Agent interface | D1 | MCP spec churn and deprecations (plan §9 risk table) | — (keep) |
| **TCP** (own protocol: HTTP+JSON, NDJSON, stdio, push; conformance kit; SDKs in 4 languages) | Connectors in any language | D4 | A spec, its governance (Q2), versioning, 4 SDKs | An OpenAPI ingest endpoint |
| **Config monitor** (tonic/gRPC or HTTP) + language hosts | Pulumi-style `Register` calls | D3 | Process management, per-language hosts | Program prints JSON to stdout |
| **AuthZEN**, **SCIM**, **CloudEvents**, **SSE** | Interop, revocation, events | Plan §5, notifications | Each small; together a surface | SSE only in v1 |
| **IAP JWT / OIDC / OAuth 2.1 / PAT / GitHub OIDC** | Authentication | D20 | Five auth paths | IAP JWT + PAT in v1 |
| **DBSP** (`dbsp` 0.384, 323 published versions) | Incremental view maintenance | D21 | Pre-1.0 churn; checkpoints and state-preserving changes are Feldera-Enterprise features Toolbox must rebuild ([adversarial-critique.md](./adversarial-critique.md) H5); **ICE on rustc 1.97** (§7.3); Feldera pins rustc 1.99.0 | Full recompute; Postgres materialised views |
| **DBOS** (Rust SDK 0.6, "pre-production") | Durable side effects | D22 | Pre-production; ties workflow tables to DBOS schema | Postgres job table; Temporal/Restate later |
| **Postgres logical decoding** (`wal_level=logical`, replication slots) | Commit-ordered change log | D21 | Slot management, WAL retention incidents if a consumer stalls [I]; managed-Postgres permissions | Snapshot reads + version counter |
| **Postgres** | State | Plan §6.2 | — | — (keep) |
| **SQLite** | Dev and small installs | Plan §6.2 | A second store for conformance | In-memory store |
| **Object storage** | Checkpoints, history, index snapshots | D21, plan §6.2 | A second backup target | Postgres tables |
| **tantivy** + **pgvector/usearch** + **ONNX/OpenAI embeddings** | Search | Plan §4.4 | Index lifecycle per replica; determinism issues ([principal-review.md](./principal-review.md) §3.3) | Postgres FTS, pgvector later |
| **Salsa** | Memoised manifest/CEL compilation | D29 | Learning curve; only pays off for large manifests | Recompile the manifest (milliseconds) |
| **comrak / pulldown-cmark / Mermaid / MiniJinja** | Docs and templates | Plan §4.4, §4.3 | Small | Same (fine) |
| **React 19 + React Aria + design tokens** | First-party and operator UI | D31 | A design system to maintain | Any React component library; or no first-party rich UI |
| **Storybook 10 + Vitest browser + Playwright + MSW-style recorded fixtures** | Stories as tests, persona views | D31 | A second test stack; fixture recording through the real engine | Plain component tests; API contract tests |
| **Vite 8 (Rolldown) in `toolbox dev`, TS 7 native `tsc`** | UI dev loop | D29, D31 | A Node toolchain inside a Rust CLI | Standard Vite in a separate UI repo |
| **turmoil / madsim, Antithesis, Kani, shuttle, loom, bolero/proptest, cargo-mutants, Elle, NIST covering arrays** | Verification | D14 | Each is a specialism; release gates cost compute ([principal-review.md](./principal-review.md) §3.4) | proptest + differential tests |
| **cargo-nextest, Bacon, watchexec, hakari, guppy, sccache, LLD/mold/wild, Cranelift, nightly parallel frontend** | Build speed | D29 | Tooling to maintain; nightly opt-in | Cargo + nextest |
| **OpenTelemetry, Prometheus, Grafana Foundation SDK, Perses, PostHog** | Observability | D6 | Moderate | OTel + Prometheus only |
| **Helm, docker-compose, Terraform modules, OCI + signing/attestation, GitHub Actions** | Delivery | D9 | Moderate; mostly unavoidable | Helm + compose |
| **Agent Plugins 1.0.0, Agent Skills, `skills-ref`** | Agent distribution | D28 | Low (exists and validates) | — (keep) |

### 2.2 Score

To keep the score auditable, each category is counted and weighted by how much it costs a maintainer
or an operator: things you must *build and keep correct* weigh more than libraries you call.

| Category (weight) | Toolbox as designed | Toolbox, minimal (§8) | Backstage | Next.js + Postgres app | Port (adopter-facing only) |
|---|---|---|---|---|---|
| Implementation languages (×3) | 4: Rust, Go, TS, Python | 2: Rust, TS | 1: TS | 1: TS | n/a (SaaS) |
| Author-facing languages (×2) | 8: TS/Py program, YAML, CEL, OpenFGA DSL, TSX, GraphQL docs, MiniJinja, WASM source | 3: YAML/TS manifest, CEL, OpenFGA DSL | 3: TS, YAML (app-config, descriptors), Nunjucks templates | 2: TS, SQL | 3: JQ, YAML/JSON mappings, HCL (Terraform) |
| Required runtime services besides the app (×3) | 3: Postgres with logical replication, object storage, connector processes | 2: Postgres, OpenFGA sidecar | 1: Postgres (search engine, TechDocs storage optional) | 1: Postgres | 1–2: Ocean containers, execution agent (Kafka/polling) |
| Project-defined protocols and specs (×3) | 4: TCP, manifest IR, metric catalog, event catalog | 2: manifest schema, ingest API | 1: descriptor format (plus the plugin APIs) | 0 | 0 for adopters |
| Build toolchains (×2) | 5: Cargo, Go/cgo, Node/Vite/Storybook, Python/uv, WASM (Javy) | 2: Cargo, Node | 1–2: Node/Yarn (+ Python for TechDocs) | 1: Node | 0 |
| Major engines/frameworks embedded (×1) | 18 | 6: axum, sqlx, cel-rust, rmcp, React, OTel | ~8 | ~4 | n/a |
| **Score** | **77** | **33** | **≈ 27** | **16** | **≈ 12 (adopter side)** |

**Reading [I]:** the designed Toolbox is roughly **3× Backstage and 5× a Next.js app** by this measure,
while its product scope (Phase 1) is narrower than Backstage's. The minimal architecture lands close to
Backstage, which is the right neighbourhood for a framework that people deploy and extend. The
Port column is low because Port keeps its stack behind a SaaS boundary; the adopter's surface (JQ,
mapping YAML, Terraform drift, §5) is where Port's pains live.

---

## 3. Layer-by-layer replaceability

For each layer: what is genuinely IDP-specific, what a general-purpose component already covers, and
how well.

| Layer | IDP-specific part | Generic part | Off-the-shelf candidates and fit | Verdict |
|---|---|---|---|---|
| **Catalog store** | Entity envelope, refs, relation types, Backstage-compatible descriptors | Storage, indexes, transactions | **Postgres** (JSONB + relations table): full fit. Backstage's own catalog is Postgres via Knex [P] | Keep Postgres; no custom store |
| **Ingestion / connectors** | Snapshot completeness, deletion safety valves, identity, provenance ([ocean.md](./ocean.md) §1) | HTTP endpoints, pagination, retries, scheduling | Airbyte/CloudQuery/Steampipe cover *extraction* but not catalog identity or deletion semantics [I]; Ocean integrations are reusable as code ([ocean.md](./ocean.md) §4) | Keep the semantics as an ingest API; **replace** "TCP as a standard" |
| **Mapping** (record → fragment) | Mapping to *kinds* with lookups and explode | Expression evaluation | **JQ/jaq** (what Port and Ocean use) or **CEL** both fit; neither needs a custom type checker to start | Keep CEL; **defer** typed CEL and object construction |
| **Merge / source of truth** | Per-field precedence, CODEOWNERS authority, conflict events | `COALESCE` | SQL covers the merge exactly (§7.1); Kubernetes SSA is the conceptual model ([source-of-truth.md](./design/source-of-truth.md) §2) | **Keep** (cheap, differentiating) |
| **Scorecards / rules** | Levels, `skip`/`unknown`, waivers, explain, initiatives | Predicate evaluation over rows | CEL in Rust (2.7 s for 10M evaluations) or the same rules as SQL (1.1 s) [P]; OPA/Rego, Cerbos also evaluate predicates but add a language | Keep CEL rules; logic is IDP-specific, the engine is not |
| **Derived data / incrementality** | Which derivations exist (closures, roll-ups, tuples) | Keeping them fresh | **Full recompute** (3.0 s) or **Postgres materialised views** (2.6 s) [P]; `pg_ivm`/RisingWave/Materialize/DBSP only if that stops fitting ([incremental-computation.md](./design/incremental-computation.md) §2) | **Replace with generic** (recompute) |
| **Authorization** | Tuples derived from ownership; facets; declassification; CODEOWNERS authority | ReBAC evaluation | **OpenFGA server** (sidecar) fits exactly (§7.2); SpiceDB also fits but needs tuple writes; Cerbos is ABAC (no graph) [I] | Keep the model; **replace** the c-archive with a sidecar |
| **Durable actions / workflows** | Approvals, dry-run, run states, ownership-routed approvers | Durable steps, retries, timers | **Postgres job table** (pg-boss pattern), **Temporal**, **Restate**, **Inngest**, **DBOS** all fit; GitHub Actions already gives durable execution for most executors [I] | **Replace with generic**; choose at Phase 3 |
| **API** | Facet shaping, log/snapshot positions | HTTP, pagination, ETags | axum + OpenAPI: full fit | Keep |
| **MCP** | Task-level tools, token-budgeted responses, annotations from action metadata | Transport, OAuth | rmcp for transport. OpenAPI-to-MCP generators produce one tool per endpoint (Harness cut 130+ tools to 11, [adversarial-critique.md](./adversarial-critique.md) H1) | Keep hand-designed tools over the service layer |
| **CLI** | `plan`, `test`, explain output | Arg parsing, output formats | clap: full fit | Keep, thin |
| **GraphQL** | Nothing specific: facets are enforced below it ([graphql.md](./design/graphql.md) §5) | All of it | Hasura/PostGraphile would generate it from Postgres but bypass facet shaping [I]; a few REST view endpoints cover a first-party UI | **Cut / defer** |
| **UI** | Entity pages generated from kinds; action forms from JSON Schema; redaction markers | Shell, routing, auth session, forms, design system, dev server | Next.js/Remix/Vite + any component library; RJSF for schema forms; or a Backstage frontend plugin | **Replace with generic** (§6) |
| **Config SDK / manifest** | The resource types (kinds, sources, scorecards, actions, policies) and their validation | Language hosts, codegen, package management | JSON Schema + any language that can print JSON; Backstage's TS `createCatalogModelLayer` shows the same idea inside Backstage [P] | Keep manifest; **cut** the Pulumi machinery |
| **Plan / diff** | Score, authz, ownership, MCP-surface diffs | Diffing two outputs | Terraform/Pulumi providers give resource diffs but **not data-dependent effects** (who drops a level), which is the differentiator; and in Port the provider is an API client of a DB that also takes UI edits, which causes drift (§5) | **Keep**, implemented as recompute(old) vs recompute(new) |
| **Search** | Permission filtering, graph-proximity boosting | Indexing, ranking | Postgres FTS, Meilisearch, Typesense, pgvector: good fit; permission filtering by intersecting with the visible set | **Defer**; Postgres FTS first |
| **Verification** | The property catalogue (S1 non-interference, S3 safe deletes, S6 precedence) | Test harnesses | proptest, differential testing (SQL vs Rust, OpenFGA vs SQL, §7.1–7.2); the naive recompute is the reference model | Keep properties; **cut** most of the machinery |
| **Kubernetes CRDs + controllers / Crossplane as the platform** | — | Config as code with reconciliation | Fit for *infrastructure* resources; poor for a 100k-entity read-heavy catalog in etcd (object count and list costs) [I] | Not recommended |

**Summary:** the IDP-specific parts are the *semantics* (merge precedence, authority, facets, plan
effects, task-level MCP tools, deletion safety). Almost every *mechanism* under them has a
general-purpose implementation that measured fast enough.

---

## 4. The Backstage compatibility claim

**Claim tested:** Backstage's backwards-compatibility obligations will prevent it from adopting
Toolbox's innovations in speed (dev loop and data freshness), MCP/agent-first interfaces, config as
code, and developer experience.

**Finding: mostly false.** Backstage's compatibility policy is *weak*, not strong: monthly releases may
contain breaking changes, with a deprecation path of at least one release where possible [P]. What slows
Backstage is ecosystem size and migration latency (about three years per core rewrite), not a
compatibility freeze. One Toolbox idea is genuinely blocked by Backstage's data model; the rest are
unimplemented or already shipping.

### 4.1 The policy

| Fact | Source |
|---|---|
| Main line: **monthly** releases since March 2022; versions do **not** follow semver; "each new minor version can contain new functionality, breaking changes, and bug fixes" | `docs/overview/versioning-policy.md` [P] |
| Packages ≥ 1.0: breaking changes to stable exports "include a deprecation phase if possible", released "for at least one mainline release before it can be removed"; `@alpha`/`@beta` exports may break without deprecation | same [P] |
| Producer-side breaks of built-in services are explicitly *not* breaking (`**BREAKING PRODUCERS**`) | same [P] |
| Scale of the surface: 161 plugin directories and 78 packages in the main repo; Node `^24.15.0 \|\| ^26.13.0`, Yarn 4 | repository tree at `184acad` [P] |

### 4.2 Migration history (how long a core change takes)

| Change | Start | End | Elapsed | Source |
|---|---|---|---|---|
| New backend system | First highlighted in v1.9 | Legacy backend support removed in **v1.42** ("we say goodbye to the last occurrence of the old `@backstage/backend-common`") | 33 monthly releases ≈ 2.75 years [I, from the cadence] | `docs/releases/v1.9.0.md`, `v1.32.0.md`, `v1.42.0.md` [P] |
| New frontend system | Experimental in v1.18 | **1.0 in v1.57** (2026-11); old system still supported | 39 releases ≈ 3.25 years | `docs/releases/v1.18.0.md`, `v1.57.0.md` [P] |
| Rspack instead of webpack | Experimental flag in v1.32 | "In the near future, we hope to make rspack the default" (v1.40) | ongoing | `v1.32.0.md`, `v1.40.0.md` [P] |
| Catalog event stream (changefeed) | RFC #8219, 2021-11 | **Still open** (56 comments) | 5 years | GitHub search API [P] |

### 4.3 Can Backstage adopt each innovation?

| Innovation | Backstage today | Blocked by compatibility, or unimplemented? | Could it ship as a Backstage module? |
|---|---|---|---|
| **Sub-minute freshness of catalog entities** | **Already there for event-driven providers.** The GitHub provider subscribes to `github.push` and `github.repository` events; provider deltas set `next_update_at = now`; the processing engine polls every **1 s** and stitching polls every **1 s** by default. Pull-based locations re-process every **100–150 s** by default (`createRandomProcessingInterval({ minSeconds: 100, maxSeconds: 150 })`) | Neither: implemented | n/a |
| **Fresh derived data** (scorecards, roll-ups, aggregates) | No general derivation engine in core; Tech Insights (now a community plugin) runs fact retrievers on schedules [I, not re-verified here]. (Port, for comparison, recalculates aggregation properties **every 15 minutes** [S]) | **Unimplemented.** A backend plugin that recomputes rules on catalog events is additive | **Yes**: a scorecard backend plugin doing full recompute on catalog events would meet a sub-minute SLO at 100k entities (§7.1) [I] |
| **Changefeed / consistency tokens** | No event stream (RFC #8219 open since 2021) | Unimplemented; hard because processing and stitching are asynchronous per entity, with no commit-ordered log [I] | Partly: a module can emit events after stitching; not commit-ordered |
| **MCP / agent-first interfaces** | `@backstage/plugin-mcp-actions-backend` **0.2.4**: Streamable HTTP, OAuth via CIMD, audit events, action filtering by glob; permissions are enforced per action, not by the MCP layer [P] | Implemented; the remaining gap (one enforcement point, task-level curated tools) is unimplemented, not blocked | **Yes**: curated actions registered through the actions registry |
| **Config as code in real languages** | Backends and frontends are composed in TypeScript; `app-config` supports pluggable `ConfigSource`s (file, env, remote, static, mutable) [P]. **Catalog model layers** (alpha, v1.50) declare kinds, JSON Schemas and relation fields in TS via `createCatalogModelLayer` and `provideStaticCatalogModel` [P]. Locations can still be registered at runtime through the catalog API (config-like state in the DB) | Mostly implemented; "no runtime config" is a policy choice an adopter can enforce | **Yes** |
| **`plan` diffs on the PR** (score, ownership, authz effects) | None | Unimplemented; nothing in the policy prevents a CI tool that runs processors and rules against a snapshot | **Yes**: a CLI module plus a GitHub check |
| **Field-level source of truth** | One entity per ref. Conflicts resolve by `locationKey`: "If the existing entity has a location key, the new entity only wins when the location keys match" — whole-entity, first owner wins [P]. Enrichment from other sources happens in processors, per entity, in the processing loop | **Blocked by the data model.** Every provider relies on entity-level location keys, orphaning and the `managed-by-origin-location` contract. Changing to per-field ownership is a core catalog change of the size of §4.2's migrations [I] | **Partly**: a "merging provider" can keep fragments in its own tables and emit one merged entity under its own location key. That works, but it is a second catalog beside the catalog |
| **Facet-level (field-level) authorization** | The permission framework decides per resource (entity) with conditional rules [P]. Relations still leak names of hidden entities (issue #22505, open since 2024) | Unimplemented, **constrained**: the catalog API returns whole entities to every plugin, and many plugins read arbitrary fields. Shaping fields per principal risks breaking plugins [I] | **Partly**: split restricted facets into separate entities or a separate backend with its own API |
| **Fast dev loop** | `yarn start` with webpack, Rspack behind a flag; backend restarts on change | Unimplemented, not blocked (Rspack migration in progress) | n/a |

**Conclusion.** Of the maintainer's four innovations: **MCP/agent-first** and **config as code** are
already partly in Backstage; **speed** is implemented for entity ingestion and unimplemented (not
blocked) for derived data and the dev loop; **DX** is not a compatibility question. The genuinely
compatibility-constrained items are **field-level source of truth** and **facet-level authorization**.
Those two are real, IDP-specific and Toolbox's strongest technical case. They do not require DBSP,
GraphQL, WASM or polyglot SDKs.

**Could Toolbox ship as Backstage modules?** For a Backstage shop, yes for most of it: a
scorecard/derivation backend plugin, a CODEOWNERS-authority processor, a `plan` CLI and GitHub check,
curated MCP actions, and an OpenFGA-backed permission policy. The two constrained items would live in
a sidecar service (the merging provider plus a facet-shaping API) that Backstage reads. That is close to
the headless engine of §6 with a Backstage frontend.

---

## 5. Port pains → Toolbox components → smallest sufficient stack

The maintainer did not list Port's problems. These are inferred from the design docs (config in a DB as
a non-goal, JQ, source of truth, write-back, Ocean, actions) and checked against public evidence.

| # | Inferred Port pain | Evidence | Toolbox component that addresses it | Smallest stack that also addresses it |
|---|---|---|---|---|
| P1 | **Config lives in Port's database; Terraform is only an API client, so UI edits and code drift apart** | Provider issues: #280 "port_blueprint drift not detected after manual UI edits"; #290 "Constant drift in the entity resource"; #273 "Persistent drift due to sorting of entity owning teams"; #108 "I had to do ClickOps … then run TF to identify the drift" to learn the syntax [P, via GitHub search API] | Manifest in git; server refuses runtime config mutation (D2) | A manifest (YAML/JSON + JSON Schema) applied by CI to a server with **no config write API**. No SDK or language host needed |
| P2 | **JQ mappings are hard to check** (nulls, silent failures, type errors; version lag) | Port's docs list "null returns, silent failures, and type errors" as common JQ problems; Port runs jq 1.7.1 and lacks 1.8 functions (roadmap) [S]; Ocean evaluates JQ per field per record in Python ([ocean.md](./ocean.md) §2) [P] | Typed CEL mappings with object construction; raw-record store so remaps don't re-fetch | Keep JQ (jaq in Rust) or untyped CEL; **validate mapping outputs against kind JSON Schemas on fixtures** at build; store raw records in one Postgres table |
| P3 | **Last sync wins**: integrations overwrite mapped fields | Port docs as summarised in [source-of-truth.md](./design/source-of-truth.md) §2 [S] | Per-field `sources` precedence; conflict events | Fragments per source + `COALESCE` in precedence order: **1.6 ms (Rust) per 100k entities** [P] |
| P4 | **Derived values are stale** | Aggregation properties recalculated every 15 min [S] | DBSP circuit | Full recompute on change: **3.0 s** (Rust) or **2.6 s** (Postgres) at 100k/2M [P] |
| P5 | **Action validation is UI-side**; custom logic means running your own services (execution agent via Kafka) | Roadmap request to enforce UI validations in the backend ([frontend.md](./design/frontend.md) sources) [S]; Port execution agent ([extensibility.md](./design/extensibility.md) §1) [S] | Server-side validation, dry-run, durable runs (DBOS), executors | JSON Schema validation server-side + executors that own durability (GitHub Actions `workflow_dispatch`, webhooks) + a Postgres job table for approval waits |
| P6 | **Integrations are thick and coupled to Port's SaaS** | 39 of 231 Ocean framework modules reference `port_client`; config change triggers a full re-fetch ([ocean.md](./ocean.md)) [P] | TCP, raw records, engine-side reconciliation | An HTTP ingest endpoint (records, snapshot markers, deletion valves) + the Ocean compatibility runner. No new protocol standard |
| P7 | **SaaS API ceilings for agents** | 35,000 Entities API requests per 5 min per org (older page: 5,000 per 5 min per IP) [S] | Self-hosted engine | Any self-hosted server |
| P8 | **Vendor risk** | Compass end of sale ([adversarial-critique.md](./adversarial-critique.md) H10) [S] | Open source, self-hosted | Any open-source option, including Backstage |

**Components that address no Port pain and no hard requirement in v1 (cut candidates):** GraphQL;
WASM functions; the Python SDK and the language-host/codegen machinery; the embedded c-archive (the
sidecar gives the same semantics); DBSP, the logical-decoding change log and object-storage
checkpoints (P4 is met by recompute); DBOS in Phases 1–2; tantivy; SQLite mode; content-addressed
sub-circuits and Salsa; deterministic simulation of replicated incremental state; the persona-aware
Storybook pipeline; TCP governance; AuthZEN.

---

## 6. "Rebuilding Next.js": overlap and the headless-engine alternative

### 6.1 What the plan rebuilds that a web framework already provides

| Planned piece | Where | What a web framework (Next.js, Remix, plain Vite + React Router) already gives |
|---|---|---|
| SPA shell, routing, code splitting per action/view | [frontend.md](./design/frontend.md) §3.2 | File-based routing, code splitting |
| Building operator TSX into the server image (`toolbox build` runs Vite, `rust-embed`) | plan §4.5, frontend §3.2 | A normal frontend build and deploy |
| Session handling behind IAP/OIDC, CSRF rules | [authorization.md](./authorization.md) §4.9, graphql §5 | Auth middleware and libraries |
| Generated forms from JSON Schema, field widgets, custom components | frontend §3 | RJSF or similar; any component library |
| A read-only GraphQL data layer with trusted documents, cost analysis, ETags | [graphql.md](./design/graphql.md) | Server-side data fetching (server components or loaders) against REST; persisted queries if GraphQL is used at all |
| `@toolbox/ui` design system (React Aria, tokens, primitives) | frontend §3.1 | React Aria Components, shadcn-style kits |
| Storybook 10 page-level persona stories with fixtures recorded by running the engine | frontend §5 | Component tests; API contract tests against fixtures |
| Vite dev server inside the `toolbox dev` daemon | developer-velocity §4 | `vite dev` next to the API |

None of these is IDP-specific. They are where "rebuilding Next.js" is literally true. The
IDP-specific parts of the UI are thin: entity pages generated from kind schemas, redaction markers, and
the dry-run/approval presentation, which the server already computes.

### 6.2 The headless-engine alternative

**Toolbox becomes a headless engine: REST + MCP + CLI over one service layer, plus a minimal built-in
read-only UI. Rich UIs are clients of the public API.**

| Concern | Headless engine | Lost or deferred |
|---|---|---|
| Speed (freshness, dev loop) | All in the engine | — |
| MCP / agent-first | All in the engine | — |
| Config as code + `plan` | All in the engine and CLI | — |
| Facets, source of truth, CODEOWNERS | Enforced in the engine. [frontend.md](./design/frontend.md) §3.2 already says "the UI is never a security boundary" | — |
| Generated entity pages and action forms | Minimal built-in UI (server-rendered, or a small SPA with no design-system package) | Pixel-level polish |
| Flagship action UX ([frontend.md](./design/frontend.md) §1) | Operators build it in their own Next.js/Vite app on the typed OpenAPI client; or a Backstage frontend plugin for Backstage shops | First-party tier-2/3 action UIs and persona stories (deferred to Phase 3, if ever) |

**Risk [I]:** handing UI to operators recreates Backstage's "own a scaffolded app" pain
([extensibility.md](./design/extensibility.md) §1). Mitigation: keep the built-in UI good enough for
reading and generated forms; make the rich UI optional, not a template every portal must own. The rule
that makes this safe: **the built-in UI uses only public API endpoints**, so any external UI has the
same power.

---

## 7. Experiments and results

All runs on the session machine: 4 vCPU Intel Xeon @ 2.1 GHz, 15 GB RAM. Code and pinned versions are
in [`experiments/`](../experiments/README.md); results are also recorded in
[research-log.md](./research-log.md).

### 7.1 Full-pipeline recompute at the v1.0 scale target (E4)

[`experiments/derived-state-scale`](../experiments/derived-state-scale/README.md) extends the earlier
[full-recompute](../experiments/full-recompute) experiment with everything
[adversarial-critique.md](./adversarial-critique.md) §7 left out: two-source merge, 2M skewed
`dependsOn` relations, a rule that reads other entities' rule outcomes, team closure and inherited
roll-ups, domain roll-ups, OpenFGA-style tuple derivation, and a materialised permission view
(ListObjects for all 10k users, 12.6M pairs). The same pipeline was written as Postgres views.

| Stage (100k entities, 2M relations) | Rust, 1 thread | Rust, 4 threads | Postgres 16 `REFRESH` |
|---|---|---|---|
| Merge two fragments per entity | 4.6 ms | 1.6 ms | (in the scores view) |
| Build CEL values | 711 ms | 242 ms | — |
| 100 rules × 100k entities | 10,830 ms (CEL) | 2,661 ms (CEL) | 1,132 ms (SQL) |
| Dependents count over 2M edges | 4.5 ms | 3.6 ms | 327 ms |
| Minimum level of dependencies (2M-edge join on rule outcomes) | 3.6 ms | 0.9 ms | 856 ms |
| Team closure + inherited roll-ups; domain roll-ups | 2.7 ms | 2.7 ms | 119 ms |
| Tuple derivation (217k tuples) | 1.9 ms | 1.5 ms | 178 ms |
| **Total, without the permission view** | **11.6 s** | **2.9 s** | **2.6 s** |
| Permission view, ListObjects × 10k users (12.6M pairs) | 246 ms | 58 ms | 18,517 ms |
| Peak memory | 575 MB | 576 MB | — |

| Single-entity change (owner and tier) | Time |
|---|---|
| Rust, hand-written invalidation, median entity (fan-in 20) | 0.33–0.38 ms |
| Rust, hottest entity (fan-in 6,337, level changed, 6,337 dependents rechecked) | 1.50–1.55 ms |
| Postgres, computed on read from plain views: rules + level / dependency check / dependents / `can_view_details` | 1.5 / 2.0 / 0.4 / 0.5 ms |

**Differential check:** Rust (CEL) and Postgres (SQL) produced identical outputs on every summary
statistic, including 12,640,370 permission pairs [P].

**Reading:**
- Against the SLOs in [plan.md](./plan.md) §6.6, a full recompute is **~100× inside** the 5-minute
  rescore target and **~20× inside** the 60-second webhook-to-visible target. E4's threshold ("full
  recompute < 30 s → drop DBSP from v1") is met by a factor of ten.
- About 98% of the Rust time is the CEL interpreter building and evaluating values. The graph work that
  motivated DBSP (closures, roll-ups, 2M-edge joins) takes **milliseconds**.
- Materialising ListObjects for every user is the one expensive derivation (18.5 s in SQL). It is also
  unnecessary: OpenFGA answers checks at read time (§7.2).

### 7.2 OpenFGA as a sidecar over a Postgres view (replaces D26's c-archive)

[`experiments/derived-state-scale/openfga`](../experiments/derived-state-scale/openfga/run.sh): an
**unmodified OpenFGA v1.22.1 server** with its Postgres datastore. After `openfga migrate`, the `tuple`
table is renamed and replaced by a **read-only view** over the derived tuples (a materialised view of
`mv_tuples` with OpenFGA's columns). Nothing writes tuples through OpenFGA.

| Measure | Result |
|---|---|
| Agreement with the SQL permission view, 2,000 sampled (user, service) checks (1,008 positive) | **2,000 / 2,000** |
| ListObjects for a user with 667 visible services | 667 = 667, 10.6 ms |
| Check latency over HTTP, sequential, keep-alive | p50 **2.4 ms**, p95 3.6 ms, p99 4.3 ms |
| Propagation of an ownership change (update source, refresh scores, tuples and the view) | 2.6 s, then `allowed: true` |

**Reading:** D26's goals (exact OpenFGA semantics, the same engine in `fga model test` and production,
**no duplicate writes**) are met without cgo, a second runtime in the process, or a custom datastore
adapter. The cost is one more container and a few milliseconds per check, which list filtering would
batch. *Not tested:* OpenFGA's write, `ReadChanges` and changelog paths (deliberately unsupported),
conditions and wildcards.

### 7.3 Build cost of the proposed heavy dependencies

[`experiments/build-cost`](../experiments/build-cost/README.md): a minimal workspace with one leaf
crate per heavy dependency (each actually used), built with the profile from
[developer-velocity.md](./design/developer-velocity.md) §3.2. "Boring" = axum, sqlx, rmcp, cel-rust.
"Full" = boring + dbsp, wasmtime, tantivy, async-graphql.

**First finding: the full set did not build on stable Rust 1.97.0.** `dbsp` 0.384.0 triggered an
internal compiler error (`instantiate_and_check_impossible_predicates` in
`operator::dynamic::multijoin::star_join`) at `opt-level = 2`, the setting the velocity design gives
`dbsp`. Feldera pins **rustc 1.99.0** in `rust-toolchain.toml`, although `dbsp` declares
`rust-version = 1.96.1` [P]. Toolbox would inherit Feldera's toolchain cadence.

With rustc 1.99.0 the full set builds. Results:

| Measure | Boring | Full | Ratio |
|---|---|---|---|
| Crates compiled | 167 | 494 | 3.0× |
| Clean dev build | 96 s | 496 s | 5.1× |
| Clean release build (thin LTO) | 64 s | 363 s | 5.7× |
| Release binary | 10.1 MB | 52.9 MB | 5.2× |
| Rebuild after editing the binary crate / a leaf crate | 0.45–0.50 s | 1.21–1.29 s | ~2.5× |
| First `cargo check` after an edit / warm `cargo check` | 16.5 s / 0.30 s | 67.4 s / 0.50 s | 4.1× / 1.7× |

**Reading:**
- The wide-and-shallow layout in [developer-velocity.md](./design/developer-velocity.md) works: edit
  loops stay near one second even with every heavy dependency linked. The **dev loop is not the
  cost.**
- The cost is in **clean builds** (CI without a warm cache, new contributors, every dependency or
  toolchain bump): 8 minutes instead of 1.6 on this machine, five times the binary, three times the
  dependency tree to audit, and a toolchain pinned to Feldera's release cadence.
- None of the four heavy dependencies is needed by the minimal architecture (§8).

### 7.4 The OpenFGA c-archive

[`experiments/build-cost/fga-carchive`](../experiments/build-cost/fga-carchive/main.go) builds the
proposed embedding (OpenFGA v1.22.1 `pkg/server` + memory datastore, `-buildmode=c-archive`, Go
1.26.11): **45 s** with a cold build cache (modules already downloaded), **2.0 s** warm, and a **78 MB**
static archive [P]. The build cost is modest; the case against the c-archive is runtime complexity
(two runtimes and GCs, signals, determinism) for semantics the sidecar already delivers (§7.2).

### 7.5 What these do not show

- No real organisation's data; skew is synthetic (fan-in up to 6,337).
- Search, embeddings and docs rendering are not measured (Phase 4).
- The recompute runs on one node; replicas would each recompute from the same snapshot (or one worker
  publishes the snapshot) [I].
- The Next.js comparison in §2.2 is counted, not built.

---

## 8. Recommended minimal architecture

**One Rust binary, Postgres, and an OpenFGA sidecar. Derived state is recomputed, not maintained
incrementally. The UI is a client of the public API.**

```
 git (manifest, catalog-info, CODEOWNERS)   connectors (any language, POST /v1/ingest)   IdP/SCIM
            │                                         │                                   │
            ▼                                         ▼                                   ▼
 ┌──────────────────────────── Postgres (the only state) ─────────────────────────────────────┐
 │ raw_records · fragments per source · native_values (revisioned) · audit · jobs · snapshots │
 └───────────────▲───────────────────────────────┬────────────────────────────────────────────┘
                 │ writes                         │ REPEATABLE READ snapshot + input version
 ┌───────────────┴───────────────────────────────▼────────────────────────────────────────────┐
 │ toolbox (Rust, one binary: api | worker)                                                    │
 │  recompute on change (debounced): map (CEL) → merge by sources → CODEOWNERS authority       │
 │   → rules (CEL) → levels → aggregates/closures → tuples (written as a derived table)        │
 │  immutable in-memory snapshot, swapped atomically; snapshot id = consistency token          │
 │  one service layer with facet shaping → REST/OpenAPI · MCP (~12 task tools) · CLI · min UI  │
 │  plan = recompute(old manifest) vs recompute(new manifest) on one snapshot → diff           │
 └───────────────────────────────┬───────────────────────────────────────────────────────────┘
                                 │ Check / ListObjects
                       OpenFGA server (sidecar) reading the derived tuple table via a view
```

| Differentiator | How the minimal architecture keeps it | Evidence |
|---|---|---|
| **Speed: freshness** | Full recompute in ~3 s at 100k/2M; single-entity paths in milliseconds if needed | §7.1 |
| **Speed: dev loop** | `toolbox dev` = watcher + full recompute over fixtures; no daemon pipeline, no Salsa; small workspace | §7.1 (2k entities ≈ 1/50 of 3 s) [I], §7.3 |
| **MCP / agent-first** | Hand-designed task tools over the same service layer as REST | §3 |
| **Config as code + `plan`** | Manifest (YAML/JSON + JSON Schema; optional TS typed builder that prints JSON); plan diffs scores, ownership and tuples between two recomputes | §7.1 |
| **DX** | One binary, one store, one auth sidecar; three author-facing languages (manifest, CEL, OpenFGA DSL) | §2.2 |
| **Source of truth** | Fragments per source, per-field precedence, CODEOWNERS authority, a revisioned native table | §7.1 (merge 1.6 ms) |
| **Facet authz** | OpenFGA for relationships; facet shaping in the service layer; information-flow labels computed at build from the fields each CEL rule reads; `declassify` declared in the manifest and listed by `plan` | §7.2 |

### 8.1 What is deferred, and the evidence that would bring each piece back

| Deferred piece | Bring it back when… (measure with the experiments in §7) |
|---|---|
| **DBSP / incremental circuit**, logical-decoding change log, object-storage checkpoints | A full recompute on a real portal exceeds **30 s** on the reference node, or p95 webhook-to-visible misses 60 s, *after* parallelising CEL |
| **Durable-execution engine** (DBOS, Temporal or Restate) | Phase 3 has an action with **≥ 3 side-effecting steps needing compensation** that the executor cannot hold durably |
| **Embedded OpenFGA (c-archive)** | Sidecar Check p95 exceeds the read budget after batching, or adopters reject a second container |
| **Pulumi-model SDKs** (language hosts, Python, codegen) | Real portal manifests need abstraction a TS builder cannot give (shared packages, generated kinds), shown by examples |
| **Typed CEL checker, CEL object construction for mappings** | Fixture validation misses mapping or rule errors that reach production more than occasionally |
| **WASM functions** | **≥ 5 real rules** in a real portal cannot be expressed in CEL or as connector-computed facts |
| **GraphQL** | A non-first-party UI client exists, or entity pages need more than three round trips at the p95 budget |
| **`@toolbox/ui`, persona Storybook, tier-2/3 action UIs** | Phase 3 actions ship and generated forms measurably fail users (task completion, support load) |
| **tantivy / vector search** | Postgres full-text search misses the relevance or latency bar on the Phase 4 benchmark |
| **TCP as a governed standard** | A third party asks to implement it |
| **Deterministic simulation (turmoil), Antithesis** | Multi-replica concurrency exists (workflows, leases), i.e. Phase 3 |
| **Velocity tooling** (hakari, guppy selection, result cache) | Workspace exceeds ~15 crates or the dev-loop SLOs regress |
| **SQLite mode** | A single-node install cannot run Postgres (not seen so far) |

---

## 9. Proposed decision changes (candidates for [decisions.md](./decisions.md))

Not recorded here; each needs the maintainer's call.

| # | Candidate | Supersedes / amends | Evidence |
|---|---|---|---|
| C1 | **v1 derives all state by full recompute from a consistent snapshot**; incremental maintenance is deferred behind the §8.1 threshold | D21 | §7.1 |
| C2 | **OpenFGA runs as an unmodified sidecar** reading derived tuples through a Postgres view; the c-archive is deferred | D26 (keeps D7) | §7.2, §7.4 |
| C3 | **No durable-execution engine before Phase 3**: a Postgres job table; Q4 is decided at Phase 3 with Temporal/Restate/DBOS as candidates | D22, Q4 | §3, §5 P5 |
| C4 | **v1 authoring = YAML/JSON manifest + one TS typed builder** that prints JSON; no language hosts or config monitor; Python and WASM deferred | D3, D23, D24 (WASM part) | §2, §5; [principal-review.md](./principal-review.md) §4.1 |
| C5 | **Ingestion is an OpenAPI endpoint, not a published protocol**; TCP governance (Q2) resolved as "not now" | D4, Q2 | §3, §5 P6 |
| C6 | **Defer GraphQL** | D30 | §3, §6 |
| C7 | **Headless engine**: the built-in UI is minimal, read-only plus generated forms, and uses only public endpoints; `@toolbox/ui`/Storybook personas deferred to Phase 3 | D31 | §6 |
| C8 | **Verification and velocity scope cut**: the full recompute is the reference model; differential tests (Rust vs SQL, OpenFGA vs SQL) and property tests; DST and the velocity tooling deferred | D14, D29 | §7.1, §7.2 |
| C9 | **A sprawl budget for v1**: ≤ 2 implementation languages, ≤ 1 required runtime service besides Postgres, ≤ 3 author-facing languages; any addition needs an ADR with a measurement | new | §2.2 |
| C10 | **Toolchain risk**: any dependency that pins a Rust toolchain newer than the workspace's must be isolated or rejected | new | §7.3 (`dbsp` ICE on 1.97, Feldera pins 1.99) |
| C11 | **Run the Backstage parity audit (E6) against §4.3**, and decide whether field-level source of truth and facet authz ship as a standalone engine or as a sidecar that Backstage reads | new | §4 |

---

## 10. Sources

**Run locally [P]:**
- [`experiments/derived-state-scale`](../experiments/derived-state-scale/README.md) (Rust + cel-rust `004a4b8`;
  PostgreSQL 16.15; OpenFGA v1.22.1, fga CLI v0.9.0)
- [`experiments/build-cost`](../experiments/build-cost/README.md) (rustc 1.97.0 and 1.99.0; dbsp 0.384.0,
  wasmtime 50.0.3, tantivy 0.27.0, async-graphql 7.2.1, rmcp 3.11.0, sqlx 0.9.1, axum 0.8.9; Go 1.26.11)
- [`experiments/full-recompute`](../experiments/full-recompute) (earlier baseline)

**Backstage, read from a clone at `184acad` [P]:**
- [docs/overview/versioning-policy.md](https://github.com/backstage/backstage/blob/master/docs/overview/versioning-policy.md)
- [docs/features/software-catalog/configuration.md](https://github.com/backstage/backstage/blob/master/docs/features/software-catalog/configuration.md) (processing interval, stitching)
- [docs/features/software-catalog/life-of-an-entity.md](https://github.com/backstage/backstage/blob/master/docs/features/software-catalog/life-of-an-entity.md)
- [docs/features/software-catalog/external-integrations/entity-providers.md](https://github.com/backstage/backstage/blob/master/docs/features/software-catalog/external-integrations/entity-providers.md) (location keys)
- [docs/features/software-catalog/catalog-model-layers.md](https://github.com/backstage/backstage/blob/master/docs/features/software-catalog/catalog-model-layers.md)
- [docs/permissions/concepts.md](https://github.com/backstage/backstage/blob/master/docs/permissions/concepts.md) ·
  [docs/ai/mcp-actions.md](https://github.com/backstage/backstage/blob/master/docs/ai/mcp-actions.md)
- [plugins/catalog-backend/src/service/CatalogBuilder.ts](https://github.com/backstage/backstage/blob/master/plugins/catalog-backend/src/service/CatalogBuilder.ts) (100–150 s default) ·
  [DefaultCatalogProcessingEngine.ts](https://github.com/backstage/backstage/blob/master/plugins/catalog-backend/src/processing/DefaultCatalogProcessingEngine.ts) (1 s poll) ·
  [DefaultProviderDatabase.ts](https://github.com/backstage/backstage/blob/master/plugins/catalog-backend/src/database/DefaultProviderDatabase.ts) ·
  [GithubEntityProvider.ts](https://github.com/backstage/backstage/blob/master/plugins/catalog-backend-module-github/src/providers/GithubEntityProvider.ts)
- [packages/config-loader/src/sources](https://github.com/backstage/backstage/tree/master/packages/config-loader/src/sources) ·
  [plugins/mcp-actions-backend/package.json](https://github.com/backstage/backstage/blob/master/plugins/mcp-actions-backend/package.json)
- Release notes: [v1.9.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.9.0.md) ·
  [v1.18.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.18.0.md) ·
  [v1.32.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.32.0.md) ·
  [v1.40.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.40.0.md) ·
  [v1.42.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.42.0.md) ·
  [v1.50.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.50.0.md) ·
  [v1.57.0](https://github.com/backstage/backstage/blob/master/docs/releases/v1.57.0.md)
- Issues (via `gh api search/issues`): [#8219 Catalog Event Stream Endpoint](https://github.com/backstage/backstage/issues/8219) ·
  [#22505 relations in the permission framework](https://github.com/backstage/backstage/issues/22505) ·
  [#27907 facets slow with many entities](https://github.com/backstage/backstage/issues/27907)

**Port [P] via GitHub search API, [S] otherwise:**
- terraform-provider-port-labs issues [#280](https://github.com/port-labs/terraform-provider-port-labs/issues/280) ·
  [#290](https://github.com/port-labs/terraform-provider-port-labs/issues/290) ·
  [#273](https://github.com/port-labs/terraform-provider-port-labs/issues/273) ·
  [#108](https://github.com/port-labs/terraform-provider-port-labs/issues/108) ·
  [#80](https://github.com/port-labs/terraform-provider-port-labs/issues/80) [P]
- [Aggregation property (recalculated every 15 minutes)](https://docs.getport.io/build-your-software-catalog/customize-integrations/configure-data-model/setup-blueprint/properties/aggregation-property) [S] ·
  [Common JQ use cases (null returns, silent failures, type errors)](https://docs.port.io/context-lake/ingestion/configure-mapping/common-jq-use-cases/) [S] ·
  [Roadmap: support jq 1.8](https://roadmap.port.io/ideas/p/support-jq-version-18-in-port) [S] ·
  [Rate limits](https://docs.port.io/api-reference/rate-limits/) and [older rate limits page](https://docs.getport.io/api-reference/rate-limits) [S]
- `port-labs/ocean` source: see [ocean.md](./ocean.md) [P]

**Feldera / OpenFGA [P]:**
- [feldera/feldera rust-toolchain.toml](https://github.com/feldera/feldera/blob/main/rust-toolchain.toml) (1.99.0) ·
  [crates/dbsp](https://github.com/feldera/feldera/tree/main/crates/dbsp)
- [openfga/openfga assets/migrations/postgres](https://github.com/openfga/openfga/tree/main/assets/migrations/postgres) ·
  [pkg/storage/sqlcommon/sqlcommon.go](https://github.com/openfga/openfga/blob/main/pkg/storage/sqlcommon/sqlcommon.go)

**Earlier Toolbox reviews:** [principal-review.md](./principal-review.md) ·
[adversarial-critique.md](./adversarial-critique.md) · [red-team.md](./red-team.md)
