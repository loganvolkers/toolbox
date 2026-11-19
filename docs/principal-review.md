# Principal Engineer Review: Where the Design Breaks

*Written 2026-11-18 as a deliberately adversarial review of [plan.md](./plan.md) and every document
under [design/](./design/). Scope: architecture, security, consistency, operability, feasibility.
[red-team.md](./red-team.md) covered competitive and product gaps; this document does not repeat them.*

**Verdict:** the *thesis* is strong (portal as code, agent-first, ownership as the system of record,
simulation-tested). The *plan* is not buildable as written. It is ~4,200 lines of design for what
amounts to six products:
1. a catalog database with custom replication
2. a ReBAC engine
3. a multi-language SDK platform
4. a connector ecosystem
5. a dashboard runtime
6. a simulation framework

The plan's original schedule was unrealistic for its scope. Several components are internally inconsistent, and
three are security holes. The fixes are mostly *subtractions*.

Severity key: 🔴 must resolve before building · 🟠 resolve before the relevant phase · 🟡 track.

---

## 1. Security and correctness holes

### 1.1 🔴 "Summary public, details private" is declassification, and S1 forbids it
The headline authorization example (score visible to all, issue list visible to owners) **violates the
non-interference property S1** in [verification.md](./design/verification.md). The score is a function
of the hidden details. Change the details and the public score changes. The same leak occurs through:
- `mirror`/`aggregate`/`calc` properties reading restricted fields
- team rollups and canvas aggregates
- `explain` output
- search counts
- notification text

With one critical issue, "security: red" *is* the detail.

**Fix:**
- Adopt an explicit **information-flow model**. Every derived value (rule outcome, computed prop,
  rollup) takes the **join of its inputs' facets** by default (taint propagation).
- Make **declassification** an explicit, reviewed declaration in code: "this rule's pass/fail outcome
  is declassified to `can_view_score_summary`".
- Restate S1 as *non-interference modulo declared declassifications*.
- Have `toolbox plan` list every declassification a change introduces.
- Add a **minimum-cardinality threshold** to aggregates over restricted data (k-anonymity style) so
  rollups of one or two entities don't reveal details.

### 1.2 🔴 Remote plan is an inference oracle and a production attack surface
The planner principal "cannot read entity data", but it *can* submit arbitrary scorecards and then
read aggregate counts. A PR adding the rule `entity.facts['snyk.critical'] > 0`, scoped to
`metadata.name == 'payments-api'`, turns "1 entity drops a level" into a one-bit read of restricted
data. Binary search over thresholds extracts exact values. The same works through authz-diff counts,
notification-volume estimates and mapping-diff counts.

Separately, the plan worker runs **PR-authored WASM rules and canvases inside production**.

**Fix:**
- **Evaluate plans as the PR author's identity**, not as a system principal: aggregates are computed
  only over data the author could read via the API, so the oracle reveals nothing new.
- Apply k-anonymity floors to every count, and rate-limit plans per author.
- Run the plan worker as a **separate, resource-capped process** against a read replica, never the
  primary.
- Remove "planner principal" as a concept.

### 1.3 🔴 Self-asserted catalog data drives permissions, so claims can be squatted
ReBAC tuples are derived from `catalog-info.yaml`, which **any repo admin can write**. In
[identity-resolution.md](./design/identity-resolution.md), "the later claim is ignored until
resolved", so **first claim wins**. An attacker who creates a repo first, with
`pagerduty.com/service-id: <prod id>` or an alias matching a production service, captures links,
facts and possibly permissions derived from them. More generally, any authz-relevant field
(`owner`, `system`, `partOf`) is set by whoever controls the file.

**Fix:**
- Classify fields as **authz-relevant**.
- Only accept authz-relevant assertions from sources whose *provenance proves authority*. Examples:
  the claimed owner team must be in the repo's CODEOWNERS or have admin rights on it; an external id
  claim must be corroborated by the external system (e.g. the PagerDuty service's own link back, or
  an admin approval).
- Ownership **transfers require acceptance** by the receiving team.
- Conflicts **freeze both claims** rather than favouring the first.

### 1.4 🔴 MCP plus actions plus untrusted content is the "lethal trifecta"
Agents read PR titles, READMEs, incident notes and imported Wiz findings (attacker-influenceable), hold
private data (details facets) and can **act**: run actions, open PRs, send webhook notifications,
write native data. "Label as untrusted" does not stop prompt-injected exfiltration through an action's
inputs or a canvas draft.

**Fix:**
- **Session tainting:** once an MCP session has read untrusted content, side-effecting tools require
  out-of-band human confirmation (elicitation), unless the action is declared `injection_safe`
  (read-only or internal-only).
- **Egress-capable actions** (webhooks, PR bodies to public repos) are blocked from carrying
  details-facet data.
- This must be a first-class design, not a bullet in §5.

### 1.5 🟠 Canvases are a confused-deputy surface
A draft canvas shared by link runs **as the viewer**, with the viewer's data, and offers `ActionButton`.
A malicious colleague can share a convincing canvas that triggers actions as the victim. It can also
show the victim's own restricted data in a deceptive context, ready for screenshots.

**Fix:**
- Actions from canvases always go through a **host-rendered confirmation outside the iframe**, naming
  the canvas author.
- Shared *draft* canvases render with a persistent "authored by X, unreviewed" frame, and cannot
  invoke actions at all until promoted.

### 1.6 🟠 Revocation latency is undefined (the "new enemy" problem)
Permissions are evaluated on each replica's in-memory graph version. Removal from a team travels:
IdP → connector sync (minutes) → database → changefeed → replica. During that window a removed user
keeps access. Zanzibar's consistency tokens solve *content* staleness, not *revocation*.

**Fix:**
- A **revocation SLO**, e.g. ≤ 60 s from IdP event to enforcement.
- A **fast path**: IdP webhooks and SCIM deprovision events bypass normal ingest and invalidate
  principal contexts immediately.
- Token claims (contextual tuples) are treated as a *ceiling*, not only a grant.

---

## 2. Data and consistency design

### 2.1 🔴 The changefeed design has a known gap bug
"Outbox/changefeed" with a sequence plus `LISTEN/NOTIFY` fails as follows:
- **Sequence numbers are assigned at insert, not at commit.** Transaction A takes seq 100 and commits
  after transaction B takes seq 101 and commits. A consumer that read `seq > 99` and saw 101 moves its
  cursor past 100 forever.
- **`NOTIFY` is not durable:** it is lost if no listener is connected, and payloads are capped at 8 KB.

This is the classic outbox gap problem, and it would silently corrupt every replica's graph, bitmaps
and search index. Ironically, E3 would catch it in simulation *only if* `MemStore` models commit
reordering, which is §3.2's point.

**Fix:**
- Use **Postgres logical decoding** (commit-ordered) as the changefeed source, or a commit-ordered log
  (`pg_current_xact_id` + snapshot-visibility watermark).
- `NOTIFY` is only a wake-up hint.
- Write this down as an ADR now.

### 2.2 🔴 Hand-rolled incremental computation will be the main bug farm
Incremental scoring, `mirror`/`aggregate` props, graph predicates
(`entity.related('dependsOn', 2)`), bitmaps, search and rollups are all **incremental view
maintenance**, maintained by hand across five subsystems. Graph predicates make the dependency sets
dynamic and fan-out unbounded: one edge change can invalidate thousands of evaluations. E1 ("incremental ≡
full") will fail constantly, and every fix will be bespoke.

**Fix:**
- **Pick one incremental computation framework** for derived state and route all derivations through
  it. Options: DBSP/Feldera-style, differential dataflow, or at minimum a Salsa-style memoised query
  graph with explicit dependency tracking.
- Or deliberately choose **periodic full recomputation** with bounded staleness for v1 (100k entities
  × 100 rules in under 5 minutes is the stated target anyway) and do incremental work only for the
  hot paths later.
- The second option is cheaper and honest. I'd start there.

### 2.3 🟠 Five overlapping history mechanisms
We now have:
- the raw layer (timestamps)
- temporal validity on edges
- the daily history store
- native-data revisions
- the audit log
- the changefeed

Each has its own retention, permission and backup story.

**Fix:** one **append-only change log** (commit-ordered, from 2.1), with history, audit, changefeed,
as-of queries and revisions as **projections** of it. Retention and facets are defined once.

### 2.4 🟠 "Postgres only" is no longer true, and Postgres is now everything
The single database now holds:
- OLTP entities, the job queue and native writes
- the raw payload lake (likely the largest table)
- 10M-row rule outcomes (the meta-catalog doc says "columnar", which Postgres isn't)
- the time-series history
- pgvector HNSW over 1M chunks
- audit and analytics buffering

…while N replicas poll it.

**Fix:**
- Name the storage tiers honestly: Postgres for state; **object storage (Parquet/Iceberg-style
  files) for raw and history**, queryable with DuckDB/DataFusion in-process.
- Vectors in Postgres or a separate store, chosen by benchmark.
- Update the "only Postgres to back up" promise to "Postgres + one bucket".

### 2.5 🟠 Per-replica derived state breaks rolling upgrades
If replicas compute derived data in memory (graph, bitmaps, search), then during a rolling deploy of
a **config or engine change**, N and N+1 replicas compute *different* answers from the same database.
Users and agents flip between them, and `toolbox verify` can pass on one and fail on another.

**Fix:**
- **Derivation epochs:** derived state is tagged with `(engine_version, manifest_hash)`.
- Routing pins a session to one epoch.
- The new epoch builds in the background (blue/green for derived state), and traffic cuts over
  atomically.

### 2.6 🟠 Every config change is a full redeploy and a full recomputation
A typo fix in a scorecard description means a new image, rollout, full rescore, re-derived tuples,
bitmaps and reindex. At 100k entities that's minutes of degraded freshness, for every portal PR.

**Fix:** derive a **manifest diff** (which `plan` already computes) and recompute only affected
derivations. Purely presentational changes (descriptions, canvases) apply without recomputation.

### 2.7 🟡 Source precedence can silently flip values
`"spec.owner": ["git:catalog-info", "native"]` means a team's carefully curated native owner is
**silently overridden** the day someone adds `owner:` to `catalog-info.yaml`. That's the Compass slug
bug, by design.

**Fix:** flips between sources emit a **conflict event**: notify both parties, show a "source changed"
banner, and keep the losing value in history.

---

## 3. Verification plan reality check

### 3.1 🟠 The reference model may become a second implementation
`expected(world, manifest, native_history)` must reproduce merge, identity resolution with
confidence thresholds, OpenFGA semantics, CEL with graph predicates, time-based rules, facets and
declassification. That is a full second product. ShardStore kept models **per component and much
simpler** than the implementation. Shared misunderstandings mean both implementations agree and
both are wrong.

**Fix:** keep reference models **per component**, and have them be genuinely naive (no indexes, no
incrementality). Use the end-to-end oracle only for a restricted world (few kinds, no graph
predicates), and rely on invariants and equivalences, not a full model, for the rest.

### 3.2 🔴 `MemStore` will be more correct than Postgres, which gives false confidence
The dangerous bugs live in **isolation semantics**: READ COMMITTED anomalies, lost updates without
`FOR UPDATE`, outbox commit reordering (2.1), `SKIP LOCKED` interleavings. A natural `MemStore` is
effectively serializable, so simulation will *never* surface these bugs.

**Fix:**
- `MemStore` must model the **isolation level Toolbox actually runs at**, including permitted
  anomalies and commit reordering.
- Or Toolbox runs critical transactions at `SERIALIZABLE` with retry, and `MemStore` injects
  serialization failures.
- Either way, **Elle-style history checks run against real Postgres** in nightly container tests.
  This is where Antithesis earns its money.

### 3.3 🟠 Determinism leaks through dependencies
Several dependencies run their own threads or clocks:
- tantivy (indexing threads, background merges)
- wasmtime (parallel compilation)
- sqlx (pool timers)
- hyper/reqwest (connection pools)
- rmcp
- tracing exporters

turmoil controls tokio's network and time, not their internal threads. Seeds will stop being
reproducible.

**Fix:**
- A **dependency determinism audit** in Phase 0.
- Single-threaded configurations under `cfg(sim)` (tantivy writer threads = 1 with a synchronous
  merge policy; wasmtime with parallel compilation disabled).
- A CI check that replays the same seed twice and diffs the traces.

### 3.4 🟡 Release gates are expensive
10⁷ seeds, 24 h differential fuzzing and 1,000 Antithesis hours *per release*, plus mutation
testing on core crates, is significant compute and money for an OSS project.

**Fix:** budget it explicitly, and gate on *changed components* only.

---

## 4. Platform and extensibility

### 4.1 🔴 The Pulumi model is the biggest cost in the plan, and its type safety is mostly illusory
- **Costs:** per-language SDKs, codegen, language hosts, and determinism checks. That is Pulumi's
  main ongoing engineering investment, staffed by a company. We have 3–5 people.
- **The payoff is thin.** The logic that matters is **CEL strings**, which no TypeScript or Python
  type checker sees. The SDKs mostly wrap object literals the manifest JSON Schema already describes,
  and agents write schema-validated YAML/JSON as reliably as TS.
- **Risk:** customer logic in SDK-language *functions* (TCP `evaluate`) needs a runtime service the
  customer hosts. That contradicts "Toolbox is a dependency, not a platform you run".

**Fix:**
- **v1 = YAML/JSON manifest (with JSON Schema + LSP) + one TypeScript SDK.** Add Python when demand
  is proven. Go/C#/Java only with contributors.
- Generate **typed CEL**: compile-time checking of expressions against kind schemas, in the
  CLI/LSP, so the real logic is checked whatever the authoring language.
- Defer `evaluate`-over-HTTP extensions; WASM covers sandboxed logic.

### 4.2 🟠 CEL everywhere is wrong for mapping
CEL is excellent for predicates and weak for **reshaping nested JSON** (building objects, flattening
arrays, conditional fields), which is exactly what connector mappings do. Ocean uses JQ for a reason.

**Fix:** CEL for predicates (rules, filters, conditions, policies). **JQ (via jaq, Rust) or JSONata
for record → fragment mapping.** This also makes Port mapping import nearly mechanical.

### 4.3 🟠 Too many languages for portal authors
Portal authors face TS/Python SDKs, CEL, the OpenFGA DSL, TSX canvases, YAML catalog-info,
WIT/WASM, a GQL subset, possibly JQ, and SQL rules. Each is reasonable alone. Together they rebuild
Backstage's learning curve under a different name.

**Fix:** a stated **language budget for v1:** YAML/TS, CEL, OpenFGA DSL, TSX. Everything else waits.

### 4.4 🟠 Self-hosted reachability breaks the GitHub story
Most self-hosted portals are **internal-only**, which breaks:
- GitHub App webhooks *into* the portal
- GitHub-hosted runners calling remote `plan`, `scorecard-check` and `push`
- OIDC token exchange from runners

None of these reach a private network. Port solves this with outbound-only agents (Ocean over
Kafka/polling).

**Fix:** design an **outbound-only relay** now. A small `toolbox-relay` runs in the customer network
and long-polls a relay, either a customer-hosted public endpoint or an optional hosted one. Otherwise,
document self-hosted runners as required, and make the GitHub App work in polling mode.

### 4.5 🟡 We own the ReBAC engine, and its fast paths don't compose
- Conditional tuples (CEL with request context) and contextual tuples from tokens **cannot be
  precomputed into permission bitmaps**, so the "bitmaps make list filtering fast" story only holds
  for unconditional models.
- Bitmap memory: 10k principals × 5 hot permissions × up to 12.5 KB ≈ up to 625 MB per replica
  before eviction.

**Fix:**
- Document the fast path's preconditions.
- Make `plan` warn when a model change disables it.
- Use LRU-bounded bitmap caches.
- Keep the external-OpenFGA mode a real, tested option, not a footnote.

---

## 5. Scope and sequencing

### 5.1 🔴 The plan has no wedge
Every phase adds breadth. There is no point where a team gets **one painful problem solved end to
end**. Six products at once will produce six demos and zero adopters.

**A proposed wedge:** *"Ownership and standards for agents."*
- Ingest Backstage `catalog-info.yaml` + GitHub (repos, CODEOWNERS) + the IdP.
- Ownership resolution with authority checks (1.3), the meta catalog's `ownership` scorecard, and
  scorecards with CEL.
- Read-only MCP (`get`, `query`, `graph`, `explain`, `context`) and a CLI.
- A minimal read-only UI.
- Authorization: **one** model (portal member / owner / admin) with summary/detail facets and explicit
  declassification (1.1).
- Config: YAML manifest + TS SDK.
- Storage: Postgres, **with full recomputation instead of incrementality**.
- Simulation harness from day one.

Value test: an agent in Claude Code answers ownership and standards questions correctly, with
permissions, on a real org. Everything else (actions, canvases, notifications, native data, context
lake, previews) is sequenced *after* the wedge has users.

### 5.2 🟠 Buy or borrow before building

| Building | Consider instead (at least for v1) |
|---|---|
| Our own OpenFGA-compatible evaluator | Embed **OpenFGA as a sidecar** with tuples streamed from the changefeed. Write our own only when list-filtering limits bite (and they may not at wedge scale) |
| Bespoke incremental engine | Full recomputation (2.2) |
| Our own simulation framework from scratch | turmoil + the `Env` seam only, plus Antithesis for the system level. Defer the custom world simulators until the wedge ships |
| Canvas runtime + component library | Ship 6 built-in canvases written by us; open drafts to agents later |
| Five SDKs | YAML + TS (4.1) |

### 5.3 🟡 Success metrics are missing
The plan has SLOs for latency, not for **outcomes**. Add:
- time to first portal
- % of services with verified owners after 30 days
- agent task success with vs. without Toolbox (the Phase 4 benchmark, moved to the wedge)
- weekly active agents and humans

---

## 6. Decisions to record as ADRs now

1. Information-flow model: taint propagation, declassification, k-anonymity floors (1.1).
2. Plans are evaluated as the author; no planner principal (1.2).
3. Authority rules for authz-relevant fields and external-id claims (1.3).
4. Agent side-effect policy under untrusted input (1.4).
5. Commit-ordered change log via logical decoding as the single history source (2.1, 2.3).
6. Full recomputation for v1; incremental work only behind one framework later (2.2).
7. Derivation epochs for rolling upgrades and config deploys (2.5, 2.6).
8. Storage tiers: Postgres + object storage (2.4).
9. Isolation level and `MemStore` fidelity; real-Postgres history checks (3.2).
10. v1 authoring = YAML + TS; CEL for predicates, JQ for mappings (4.1, 4.2, 4.3).
11. Outbound-only relay for private deployments (4.4).
12. The wedge, and what is explicitly *not* in v1 (5.1, 5.2).

## 7. What I would keep exactly as designed

- Config as code, with `plan` and gates as the review surface.
- Field-level source-of-truth declarations.
- Ownership as the system of record, and the meta catalog's recursive ownership.
- OpenFGA *language and tests* as the authorization contract, whoever evaluates it.
- The `Env` seam, `buggify!`, always/sometimes assertions, and the property catalogue.
- A thin connector protocol that emits raw records, with engine-side mapping and deletion safety
  valves.
- Federating security graphs instead of copying them.

---

## 8. Resolution (2026-11-19)

| Finding | Decision | Where |
|---|---|---|
| 1.1 Declassification | **Accepted.** Facets with information-flow labels; explicit, reviewed declassification; minimum cardinality. Catalogs stay public by default | [authorization.md](./authorization.md) §4.2 |
| 1.2 Plan as inference oracle | **Rejected as over-engineering.** Git is the boundary: the portal repo is privileged, plan output goes to the PR in full, forks never plan | [design/sdlc.md](./design/sdlc.md) §4 |
| 1.3 Squatted claims | **Accepted, using CODEOWNERS** as the authority | [authorization.md](./authorization.md) §4.8 |
| 1.4 Lethal trifecta | **Rejected.** Protection is delegated to MCP clients, agent harnesses, egress controls and the identity-aware proxy. Toolbox provides annotations, markers, approvals and audit | [authorization.md](./authorization.md) §4.5 |
| 1.5 Canvases | **Canvases cancelled.** Dashboards are built via Claude Artifacts / Cursor Canvas + MCP; UI customisation is operator-only | [design/dashboards.md](./design/dashboards.md) |
| 1.6 Revocation | **Accepted:** revocation SLO, identity-aware-proxy authentication, Toolbox owns the permissions boundary | [authorization.md](./authorization.md) §4.9 |
| 2.1–2.6 Consistency | **Accepted and superseded** by one logical-decoding change log + a DBSP circuit (no batch or nightly recomputation) + DBOS-model workflows + epochs | [design/incremental-computation.md](./design/incremental-computation.md) |
| 3.x Verification realism | **Accepted:** `MemStore` isolation fidelity, dependency determinism audit, engines under simulation; continuous rather than nightly campaigns | [design/verification.md](./design/verification.md) |
| 4.1 Pulumi model | **Kept** (code-first extensibility), TS/Python/YAML first; typed CEL; WASM instead of HTTP `evaluate` services | [design/extensibility.md](./design/extensibility.md) |
| 4.2 CEL for mapping | **CEL kept, with object construction** (the Kubernetes MutatingAdmissionPolicy precedent); JSONata rejected; JQ not adopted; WASM functions for the rest | [design/extensibility.md](./design/extensibility.md) |
| 4.3 Too many languages | **Rejected:** the right language for each job, judged by agent developer experience (the Salesforce Lightning lesson) | [design/extensibility.md](./design/extensibility.md) |
| 4.4 Private networks | Accepted: outbound relay or self-hosted runners | [plan.md](./plan.md) Phase 2 |
| 4.5 Own ReBAC engine | **Superseded:** OpenFGA embedded as a Go module via C ABI; no duplicate tuple writes | [authorization.md](./authorization.md) §4.3 |
| 5.x Scope and estimates | Time estimates removed (built with agents). Phase 1 is the "ownership and standards for agents" first release | [plan.md](./plan.md) §8 |
