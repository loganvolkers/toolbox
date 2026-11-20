# Decision Log

The single, chronological record of design decisions made with the maintainer. Each entry links to
the document holding the rationale. **Agents: when a decision changes, add a new entry (don't rewrite
history) and update the linked document.**

Status key: ✅ decided · ❓ open · ⏸ deferred.

## 2026-11-18

| # | Decision | Status | Where |
|---|---|---|---|
| D1 | Build an open-source, agent-first IDP ("Toolbox") with a Rust core and four interfaces (API, MCP, CLI, UI); humans are a secondary audience | ✅ | [plan.md](./plan.md) §1 |
| D2 | **Configuration lives in code, never in a database.** Toolbox is a dependency customers deploy themselves | ✅ | [plan.md](./plan.md) §1 |
| D3 | Config authoring follows the **Pulumi model** (real languages emit a manifest), replacing the earlier Starlark proposal | ✅ | [plan.md](./plan.md) §2 |
| D4 | Define an open, MCP-style **Toolbox Connector Protocol (TCP)** over HTTP (plus stdio and push) so connectors can be written in any language | ✅ | [plan.md](./plan.md) §4.6 |
| D5 | **Don't adopt Port Ocean as the framework.** Reuse its integrations (compatibility-runner spike), its API knowledge and its patterns | ✅ | [ocean.md](./ocean.md) |
| D6 | **Observability:** OpenTelemetry/Prometheus as the contract, **Grafana** as the reference backend; product analytics through a versioned event catalog, **PostHog** as the reference sink; both replaceable | ✅ | [plan.md](./plan.md) §6 |
| D7 | **Authorization:** ReBAC using the OpenFGA language and `.fga.yaml` test format, derived from the catalog graph; field-level facets (e.g. score summary public, details owner-only) | ✅ (refined by D27) | [authorization.md](./authorization.md) |
| D8 | Red-team gaps A2, A3, B1, B4 and C1 to be designed; notifications are GitHub-native, email and webhook, **Slack out of scope**; **demos deferred** | ✅ / ⏸ | [red-team.md](./red-team.md) §G |
| D9 | Ship GitHub Actions for the portal SDLC (local → CI → PR → ephemeral preview → prod) and for customers' service repos | ✅ | [design/sdlc.md](./design/sdlc.md), [design/notifications.md](./design/notifications.md) |
| D10 | Every portal ships a **meta catalog** that owns and scores itself (recursive ownership) | ✅ | [design/meta-catalog.md](./design/meta-catalog.md) |
| D11 | Meta-catalog proposals accepted: owners must be teams with an optional accountable individual (org-size override); ownership re-verified every 180 days *and* on IdP reorganisation events; meta entities visible to all, operational facts operator-only | ✅ | [design/meta-catalog.md](./design/meta-catalog.md) §11 |
| D12 | **Context lake:** layered (raw → two-tier graph + catalog-anchored knowledge → derived); federate security graphs (Wiz, Cartography) rather than copy them; Toolbox as the ownership system of record | ✅ | [design/context-lake.md](./design/context-lake.md) |
| D13 | **Source of truth per field**, declared in code; native (Toolbox-owned) *data* allowed where declared; routed writes; write-back | ✅ | [design/source-of-truth.md](./design/source-of-truth.md) |
| D14 | **Verification:** TDD against named properties, reference models, deterministic simulation, differential/conformance testing, configuration-space testing, release gates | ✅ | [design/verification.md](./design/verification.md) |

## 2026-11-19

| # | Decision | Status | Where |
|---|---|---|---|
| D15 | Information flow: derived values inherit facet labels; **explicit, reviewed declassification**; minimum cardinality for aggregates. **Catalogs are public by default** | ✅ | [authorization.md](./authorization.md) §4.2 |
| D16 | **Remote plan:** Git is the access boundary (Terraform model). Full plan output goes on the PR; the portal repo is privileged; no per-viewer redaction or k-anonymity for plans | ✅ | [design/sdlc.md](./design/sdlc.md) §4 |
| D17 | Permission-relevant claims (ownership, external ids) are accepted only when backed by **CODEOWNERS**; conflicting claims freeze | ✅ | [authorization.md](./authorization.md) §4.8 |
| D18 | Prompt-injection and "lethal trifecta" defence is **delegated to other layers** (MCP clients, harnesses, egress controls, proxies). Toolbox provides annotations, markers, approvals and audit; no session tainting | ✅ | [authorization.md](./authorization.md) §4.5 |
| D19 | **No user-authored dashboards/canvases** inside Toolbox. Users build them in Claude Artifacts / Cursor Canvas over MCP; UI customisation is operator-only, in code | ✅ | [design/dashboards.md](./design/dashboards.md) |
| D20 | Authentication via an **identity-aware proxy** (Google IAP, Cloudflare Access, …) or OIDC; Toolbox owns the permissions boundary; **revocation SLO** (default 60 s) | ✅ | [authorization.md](./authorization.md) §4.9 |
| D21 | **Incremental computation:** one commit-ordered change log feeds one **DBSP** circuit for all derived data; **no batch or nightly recomputation** | ✅ | [design/incremental-computation.md](./design/incremental-computation.md) |
| D22 | **Durable execution:** DBOS model (workflows checkpointed in Postgres) for every side effect | ✅ | [design/incremental-computation.md](./design/incremental-computation.md) §4 |
| D23 | Keep **code-first extensibility**; the right language for each job, judged by agent developer experience (the Salesforce Lightning lesson); many languages are fine if each is the right tool | ✅ | [design/extensibility.md](./design/extensibility.md) |
| D24 | **JSONata rejected** (under-specified). CEL for predicates *and* mappings (with object construction); JQ not adopted; WASM functions for logic beyond CEL | ✅ | [design/extensibility.md](./design/extensibility.md) |
| D25 | **Remove all time estimates** (built with agents); phases defined by exit criteria | ✅ | [plan.md](./plan.md) §8 |
| D26 | **Embed OpenFGA as a Go module via a C ABI**, with a read-only datastore over the circuit's tuple index; **no duplicate writes anywhere** | ✅ | [authorization.md](./authorization.md) §4.3 |
| D27 | Worked example: self-service installation of approved GitHub Apps (GitHub's 13 Oct 2026 App changes) as the reference action | ✅ | [examples/approved-github-apps.md](./examples/approved-github-apps.md) |
| D28 | Ship **two agent plugins** (Agent Plugins 1.0.0): `toolbox-operator` in this repo, and a user plugin generated from each portal's config pointing at the production MCP URL | ✅ | [design/agent-plugins.md](./design/agent-plugins.md) |
| D29 | **Developer velocity is a measured requirement:** no per-portal code generation; wide, shallow workspace; `toolbox dev` incremental pipeline; selective, parallel, cached tests; dev-loop SLOs | ✅ | [design/developer-velocity.md](./design/developer-velocity.md) |
| D30 | **GraphQL**, read-only, trusted documents only in production, for rich catalog pages and operator views; not for agents | ✅ | [design/graphql.md](./design/graphql.md) |
| D31 | **Frontend:** React + TypeScript on `@toolbox/ui` (React Aria); three action-UI tiers (generated → field widgets → custom); not framework-agnostic (Astro islands rejected); Storybook 10 for primitives *and* page-level, persona-aware stories | ✅ | [design/frontend.md](./design/frontend.md) |
| D32 | During the design phase, commit directly to `main` (no PRs) so parallel agents and harnesses share one source of truth | ✅ | [../AGENTS.md](../AGENTS.md) |

## 2026-11-20

Charter goals added by the maintainer (G1–G3), and the [technology-critique.md](./technology-critique.md)
minimal architecture adopted with amendments where it conflicted with them.

| # | Decision | Status | Where |
|---|---|---|---|
| D33 | **Charter goal G1, simple deployment.** The default deployment is **one container image plus Postgres**, runnable on Google Cloud Run or an equivalent. Custom UIs and self-service action code ship *inside* that image, not as separate pods or microservices. Modular deployment (split roles, external services) is optional, never required | ✅ | [plan.md](./plan.md) §1, §6.2 |
| D34 | **Charter goal G2, agent SDLC.** Operators (and their agents) can plan, build, test, deploy and monitor a portal with a full SDLC, at the maturity of regular programming or force.com (sandboxes, tests, deploys, rollbacks). This is the main counterpoint to Port and Backstage | ✅ | [plan.md](./plan.md) §1, [design/sdlc.md](./design/sdlc.md) |
| D35 | **Charter goal G3, replaceable components.** React, DBOS, OpenFGA's packaging, the derived-state engine and monitoring vendors are *defaults*, not lock-ins. Each sits behind a Toolbox-owned contract with conformance tests, so a better technology can replace it | ✅ | [plan.md](./plan.md) §1, §3 |
| D36 | **Adopt the minimal architecture** ([technology-critique.md](./technology-critique.md) §8) as the v1 baseline: one Rust binary, Postgres, derived state by full recompute, curated MCP, `plan` diffs, field-level sources, CODEOWNERS authority, facets. Amended by D37–D40 | ✅ | [plan.md](./plan.md) §3 |
| D37 | **Derived state: full recompute from a consistent Postgres snapshot** (critique C1). The snapshot id is the consistency token; the logical-decoding change log and object-storage checkpoints are deferred. Recompute is triggered by changes, so "no batch or nightly recomputation" (D21) still holds. DBSP comes back, behind the same derived-state contract, only if a real portal's full recompute exceeds 30 s or freshness misses 60 s p95 | ✅ supersedes D21's engine choice | [design/incremental-computation.md](./design/incremental-computation.md) |
| D38 | **OpenFGA runs inside the Toolbox container** (G1), unmodified upstream, reading derived tuples through a Postgres view (critique §7.2), behind an `Authorizer` contract (G3). How it's packaged inside the container is Q11. An external OpenFGA service is a modular option. Amends critique C2 (sidecar) and D26 (c-archive no longer assumed) | ✅ | [authorization.md](./authorization.md) §4.3 |
| D39 | **Durable execution stays in-process on Postgres (DBOS model)** (G1), behind a `Durable` contract (G3); no Temporal or other external engine by default. Q4 (adopt the DBOS Rust SDK vs own a minimal runtime) stays open. Amends critique C3 | ✅ refines D22 | [design/incremental-computation.md](./design/incremental-computation.md) §4 |
| D40 | **Custom UIs and actions deploy in the image.** UI: a framework-agnostic **UI extension contract** (static bundles built by `toolbox build`, mounted as ES modules or Web Components, calling the public API through a generated typed client); React + `@toolbox/ui` + Storybook is the default kit (G3). Actions: **in-image executors** run as child processes over the stdio transport (any language, packaged into the image), alongside external executors (GitHub Actions, webhooks). Amends critique C7 and D31's "not framework-agnostic" | ✅ | [design/frontend.md](./design/frontend.md), [design/action-entities.md](./design/action-entities.md) |
| D41 | **Authoring: YAML/JSON manifest + JSON Schema, plus one TypeScript typed builder** that prints the manifest (critique C4). Pulumi-style language hosts, the Python SDK and WASM functions are deferred | ✅ supersedes D3; amends D23, D24 | [design/extensibility.md](./design/extensibility.md) |
| D42 | **Ingestion is an OpenAPI endpoint plus the stdio transport, not a governed protocol** (critique C5). Q2 resolved: not now | ✅ amends D4 | [plan.md](./plan.md) §4.6 |
| D43 | **GraphQL deferred** (critique C6) | ⏸ supersedes D30 | [design/graphql.md](./design/graphql.md) |
| D44 | **Verification and velocity scope cut** (critique C8): the full recompute is the reference model; differential tests (Rust vs SQL, OpenFGA vs SQL) and property tests first. Deterministic simulation and the velocity tooling (hakari, `xtask affected`, result cache) are deferred until multi-replica concurrency or workspace size needs them. The operator SDLC (G2) is not cut | ✅ amends D14, D29 | [design/verification.md](./design/verification.md), [design/developer-velocity.md](./design/developer-velocity.md) |
| D45 | **Sprawl budget for v1** (critique C9, tightened by G1): ≤ 2 implementation languages in the default image (Rust; Go only inside the OpenFGA component), **no required runtime service besides Postgres**, ≤ 3 author-facing languages (manifest, CEL, OpenFGA DSL; TypeScript optional for the builder and UIs). Any addition needs an ADR with a measurement | ✅ | [plan.md](./plan.md) §3 |
| D46 | **Toolchain risk** (critique C10): a dependency that pins a newer Rust toolchain than the workspace is isolated or rejected | ✅ | [design/developer-velocity.md](./design/developer-velocity.md) |
| D47 | **Run the Backstage parity audit** (critique C11, adversarial E6) as a Phase 0 task | ✅ | [technology-critique.md](./technology-critique.md) §4 |

| D48 | **Connector control channel.** Besides the ingest endpoint, Toolbox sends connectors commands (scoped `refresh`, `resync`, `cancel`) with acknowledgements, completion and the snapshot id that contains the results. Delivered over stdio for in-image connectors, long-poll pull for remote ones (no inbound ports), optional push. Still a versioned Toolbox API, not a governed standard | ✅ amends D42 | [design/connector-control.md](./design/connector-control.md) |
| D49 | **Action runs: preview → confirm → dispatch → verify.** A run succeeds only when declared CEL postconditions hold at a snapshot taken after a scoped connector refresh ("done means observed"). Confirmation is bound to the preview by a token. Executors implement `preview`, `run` with progress, `cancel` and optional `verify`. In-image and HTTP executors are the high-fidelity default; GitHub Actions is supported at a coarser preview tier; fire-and-forget webhooks or queues never count as completion and end `unverified` without a postcondition | ✅ replaces the run states in plan §4.3 | [design/action-runs.md](./design/action-runs.md) |

## Open questions (consolidated)

| # | Question | Recommendation | Where |
|---|---|---|---|
| Q1 | Licence | Apache-2.0 | [plan.md](./plan.md) §10 |
| Q2 | TCP governance: separate neutral spec repo now, or later? | **Resolved by D42: not now** | [plan.md](./plan.md) §10 |
| Q3 | First-release connectors: GitHub only, or GitHub + GitLab? | GitHub first | [plan.md](./plan.md) §10 |
| Q4 | Durable runtime behind the `Durable` contract (D39): adopt the DBOS Rust SDK, or own a schema-compatible minimal runtime? | Decide after the Phase 0 spike | [design/incremental-computation.md](./design/incremental-computation.md) |
| Q5 | Temporary/just-in-time grants as approved, time-bound state? | Open | [authorization.md](./authorization.md) §8 |
| Q6 | Redaction markers on or off by default | On (helps agents ask for access) | [authorization.md](./authorization.md) §8 |
| Q7 | Agent ceilings in the authorization model, or a simpler allow-list? | Open | [authorization.md](./authorization.md) §8 |
| Q8 | GitHub spike: the exact "add repository to installation" endpoint path, and whether the owner-only install restriction applies to Apps | Verify in the spike | [examples/approved-github-apps.md](./examples/approved-github-apps.md) §2.6 |
| Q9 | How much DBSP state survives a circuit rebuild (sub-circuit reuse)? | **Deferred with DBSP (D37)** | [design/developer-velocity.md](./design/developer-velocity.md) §5 |
| Q10 | cel-rust conformance level (688 of ~2,771 conformance tests skipped; no type checker) | Measure in Phase 0; validate rule outputs against JSON Schemas on fixtures | [design/extensibility.md](./design/extensibility.md) |
| Q11 | How OpenFGA is packaged inside the one container (D38): **a supervised co-process** (the unmodified upstream binary, talking over localhost, reading the tuple view) or **linked in-process** via the c-archive (78 MB, 45 s build, Go runtime inside the Rust process) | **Co-process**: one container either way, no FFI, upstream unmodified. Phase 0 measures cold start, memory and crash recovery on Cloud Run for both | [authorization.md](./authorization.md) §4.3 |
| Q12 | Isolation for in-image executors (D40): separate Unix user, scrubbed environment, per-action secrets, resource limits; is that enough, or is a sandbox (gVisor, WASM) needed? | Unix user + scoped secrets for v1; revisit in Phase 3 | [design/action-entities.md](./design/action-entities.md) |
| Q13 | Cloud Run fit (D33): background recompute and workflows need instance-based billing and ≥ 1 minimum instance; autoscaling can stop an instance mid-workflow, so another instance must recover it from Postgres. Split worker role on Cloud Run worker pools (GA 2026, secondary source) or keep one service? | One service with min instances ≥ 1 by default; verify recovery in the Phase 0 deployment spike | [plan.md](./plan.md) §6.2 |
