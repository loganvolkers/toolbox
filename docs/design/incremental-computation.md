# Incremental Computation and Durable Execution

*Status: v1 design updated 2026-11-20 (D37, D39). Implementation detail: [v1-spec.md](../v1-spec.md) §6 and §8.1.*

## v1 design

**Derived state is recomputed in full from a consistent snapshot whenever inputs change.** Side effects
run as durable workflows on Postgres. Both sit behind Toolbox-owned contracts (`DerivedState`,
`Durable`) so an incremental engine or a different workflow runtime can replace them (G3).

| Concern | v1 | Evidence |
|---|---|---|
| Derived values (merge, ownership, scores, aggregates, closures, tuples, options) | One pipeline over a `REPEATABLE READ` snapshot, debounced after changes; result swapped atomically in memory | ~3 s at 100k entities / 2M relations ([technology-critique.md](../technology-critique.md) §7.1) |
| Consistency token | Snapshot id `<input version>.<manifest hash>`; reads accept `min_snapshot` | [v1-spec.md](../v1-spec.md) §6.1 |
| No duplicate writes | Inputs written once; derived tuples published as deltas to one table that OpenFGA reads through a view | critique §7.2 |
| Multiple instances | Each recomputes the same snapshot; one advisory-lock holder publishes snapshots and tuple deltas | v1-spec §8 |
| Durable execution | DBOS-model tables (column subset of DBOS's schema), step replay, idempotency keys, signals, cross-instance recovery | [experiments/durable-schema](../../experiments/durable-schema/README.md), v1-spec §8.1 |
| No batch or nightly jobs | Recompute is event-driven; freshness policies trigger connector refreshes, not recomputes | D21, [connector-control.md](./connector-control.md) |

**When the deferred incremental design comes back:** a real portal's full recompute exceeds 30 s on the
reference node, or webhook-to-visible misses 60 s at p95 after parallelising CEL. Then DBSP (or another
engine) implements `DerivedState`, and the reference model in
[examples/reference-portal](../../examples/reference-portal/README.md) stays the oracle (P2).

---

## Appendix: deferred incremental design (DBSP)

*The original design, kept for when its trigger is met. Where it conflicts with the v1 design above, v1 wins.*


*Status: research + decision. Research date 2026-11-19. Supersedes the hand-rolled incremental
evaluation in [plan.md](../plan.md) §4.1–4.2 and the "periodic full recomputation" suggestion in
[principal-review.md](../principal-review.md) §2.2. Decision: **no batch or nightly recomputation.
Every derived value is maintained incrementally by one rigorously-founded engine, and every side
effect runs as a durable workflow.***

---

### 1. Two different problems

They are easy to conflate, so we name them separately:

| | **Derived data** (incremental view maintenance, IVM) | **Side effects** (durable execution) |
|---|---|---|
| Examples | Raw records → entity fragments → merged entities → relations → authz tuples → rule outcomes → levels → rollups → search documents → history deltas | Run an action, wait for approval, call a GitHub workflow, send a notification, push write-back, fire a TTL expiry, run a connector sync, compute a remote plan |
| Nature | Pure, deterministic functions of inputs; must stay consistent as inputs change | Effects in other systems; must survive crashes, retries and restarts *exactly once* per intent |
| Correctness criterion | `incremental(inputs) ≡ batch(inputs)` | Every step either completes once or is compensated; no duplicates |
| Right tool | A dataflow/IVM engine | A durable-execution engine |

Port's strength (calculation, aggregation and mirror properties, scorecards that update as data
changes) is IVM. DBOS (which the user leaned towards) is durable execution. **Toolbox needs both, and
must not use either for the other's job.**

### 2. Incremental view maintenance: options

| Candidate | Model | Language / licence | Recursion (graph closures) | Embeddable? | Determinism | Verdict |
|---|---|---|---|---|---|---|
| **DBSP** (Feldera) | Streams of **Z-sets** (rows with ±weights). Any query is incrementalised automatically by algebra. **VLDB 2023 best paper**, SIGMOD 2024 research highlight, VLDB Journal 2025 | **Rust**, `dbsp` crate, **MIT** (open-source edition) | **Yes:** incremental fixed points; the repo ships `transitive_closure` and `orgchart` examples | **Yes**, a library circuit in-process | Synchronous circuit `step()`; single worker in simulation | **Choose** |
| **Differential dataflow** (McSherry; Materialize lineage) | Timely dataflow with partially-ordered timestamps; `iterate` for recursion | Rust, **MIT** | Yes (`iterate`) | Yes | Deterministic results, but progress tracking and workers are more complex to drive deterministically | Strong alternative; heavier API and operational model |
| **Salsa** (rust-analyzer) | Demand-driven memoised queries; early cutoff; durability levels | Rust, MIT/Apache | Cycles limited | Yes | Yes | **Use for the compiler/LSP** (manifest → typed IR, CEL type-checking), **not** for data volumes |
| Materialize / RisingWave / Feldera *as services* | Streaming SQL databases | Source-available / Apache | Varies | **No:** an extra service | n/a | Rejected: another service to run, and they duplicate our state |
| **pg_ivm** / **pg_trickle** (in-Postgres IVM) | Incremental materialized views inside Postgres | C / Rust (pre-1.0) | Limited | Inside Postgres | n/a | Rejected: limited query surface; pg_ivm's production readiness is disputed (sources conflict); pg_trickle is pre-1.0 |
| Jane Street Incremental, Adapton | Self-adjusting computation | OCaml / various | — | — | — | Influences only |

**Why DBSP:**
1. **Formal foundation.** The incrementalisation is *derived* from the query, with proofs, not
   hand-written, so `incremental ≡ batch` holds by construction. Our verification
   ([verification.md](./verification.md) E1) becomes a check of our operators, not of the theory.
2. **Recursion is first class.** The heart of Toolbox is graph closures: nested group membership,
   inherited ownership through bridge edges, `dependsOn` reachability, system and domain roll-ups,
   meta-catalog `reads` edges. DBSP maintains these incrementally.
3. **Rust, MIT, embeddable.** It runs in-process, adds no service, and is driven by our `Env`, so it
   fits deterministic simulation (single-threaded circuit steps).
4. **Port parity and beyond.** Mirror, calculation and aggregation properties, scorecards and
   roll-ups are just operators in the circuit.

**Risks:**
- `dbsp`'s API evolves with Feldera. Mitigation: pin it, wrap it in `toolbox-ivm`, contribute
  upstream.
- Memory for large state. Mitigation: DBSP supports spilling to storage; use single-node state sizes
  per the scale targets.

### 3. The design: one change log, one circuit, no duplicate writes

```
 WRITES (each fact written exactly once)            DERIVED (never written back as source data)
 ┌────────────────────────────────────────┐        ┌──────────────────────────────────────────────┐
 │ raw_records      (connector RECORDs)   │        │ DBSP circuit, compiled from the manifest:    │
 │ native_values    (Toolbox-owned fields)│  log   │  map (CEL) → fragments → merge by sources    │
 │ workflow state   (DBOS system tables)  │ ─────► │  → entities → relations → closures           │
 │ audit            (append-only)         │ (commit│  → authz tuples → rule outcomes → levels     │
 │ manifest         (immutable, per epoch)│ order) │  → rollups → search docs → history deltas    │
 └────────────────────────────────────────┘        └──────────────────────────────────────────────┘
                     ▲                                         │ output deltas
                     │ workflows write results                 ▼
                     │ (e.g. action fragments → raw_records)   indexes (graph, search, OpenFGA reads),
                     └──────────────────────────────────────── history archive, workflow triggers
```

- **The change log** is Postgres **logical decoding** of the input tables. It is commit-ordered,
  which fixes the outbox gap bug ([principal-review.md](../principal-review.md) §2.1). `NOTIFY` is
  only a wake-up hint.
- **Every input fact is written once.** Entities, relations, tuples, scores and search documents are
  *views*, not rows that something must keep in sync. There is no dual-write anywhere:
  - OpenFGA reads tuples *from the circuit's output index* ([authorization.md](../authorization.md)).
  - Search is fed by output deltas.
  - The API serves from the indexed views.
- **Consistency token = log position.** Every response carries the log LSN its views reflect.
  "At least LSN *n*" gives read-your-writes and Zanzibar-style freshness across replicas.
  Deterministic circuits over the same log prefix produce identical state on every replica.
- **Epochs:** a circuit is identified by `(engine_version, manifest_hash)`. A new deployment builds the
  new epoch's state from a checkpoint plus the log in the background, then cuts over atomically. This
  fixes the rolling-upgrade inconsistency (§2.5 of the review). Config changes that `plan` classifies
  as presentational don't create a new epoch.
- **Checkpoints:** circuit state is checkpointed to object storage, so cold start = checkpoint + log
  tail, not a full recompute.
- **History** (score trends, as-of queries) is an **archive of output deltas**: append-only and
  rebuildable by replaying the log through the circuit. That collapses the "six overlapping history
  mechanisms" into one log plus one archive.

**Rules and expressions inside the circuit:**
- CEL predicates and mappings compile to closures used by `filter`, `map` and `flat_map` operators.
- Graph predicates (`related`, `path_exists`) compile to bounded joins or recursive sub-circuits.
- Their **cost is visible at build time.** `toolbox plan` reports the circuit's size and estimated
  state, and rejects unbounded recursion that isn't declared.

**Information flow** ([authorization.md](../authorization.md) §4.2): every circuit column carries a
facet label. Operators propagate the *join* of input labels, and only explicit `declassify` operators,
declared in code and listed by `plan`, lower a label.

### 4. Durable execution: options

| Candidate | Model | Rust support (Nov 2026) | Infrastructure | Verdict |
|---|---|---|---|---|
| **DBOS Transact** | Library; workflows and steps checkpointed in **Postgres**; queues, scheduled workflows; one system-DB schema across SDKs | Official Rust SDK **0.6, "pre-production"**; Go 1.0 (Aug 2026), Java 1.0, Python, TypeScript | **Postgres only** (already ours) | **Choose the model.** Adopt the Rust SDK behind a `Durable` trait, contribute to it, and keep a minimal schema-compatible fallback if it isn't production-ready when we need it |
| Temporal | Server + workers; event-history replay | Rust SDK **GA (Sept 2026)** | A separate Temporal cluster | Excellent, but a second stateful system to operate |
| Restate | Single Rust server binary; SDKs | Rust SDK 0.10 (Apr 2026) | A separate server | Lighter than Temporal, still a service |
| Others (Obelisk, Flawless, Resonate, duroxide) | Various | Early | Varies | Watch |

**Why the DBOS model fits:**
- It needs **no new infrastructure**: workflow state lives in the same Postgres, and the "Postgres +
  object storage" promise holds.
- It is **cross-language by design.** The system-database schema is shared across DBOS SDKs, so a
  connector or action handler written in TypeScript or Python with a DBOS SDK can participate in the
  same durable-workflow tables. That fits the multi-language extension story.
- It is **testable**: a `Durable` trait lets deterministic simulation substitute an in-memory durable
  runtime with crash injection between steps.

**What runs as workflows:**
- action runs (approval waits, executor calls, callbacks, timeouts)
- connector sync orchestration (scheduling, cursors, snapshot bookkeeping; payloads still land in
  `raw_records`)
- notification delivery with dedup
- write-back to external systems
- TTL and lifecycle timers (scheduled workflows)
- remote plans
- preview-environment lifecycles
- re-verification campaigns (ownership attestations)

**Where IVM and workflows meet:** circuit output deltas that represent *events* (a score transition, a
TTL becoming due, an ownership gap) enqueue workflows idempotently. The key is
`(view, key, log position)`, so replays never double-fire. Workflow results that are *data* (an
action-created fragment, a write-back confirmation) are written once to the input tables and flow
back through the log.

### 5. What this removes from the plan

- Hand-rolled incremental scoring, `reads`-based invalidation, bitmap maintenance and per-subsystem
  incremental indexes. **The circuit is the single incremental mechanism.**
- The outbox + `NOTIFY` changefeed, replaced by logical decoding.
- Separate history mechanisms, replaced by the log plus the delta archive.
- Any batch, nightly or periodic recomputation.

### 6. Verification hooks

- **E1 (incremental ≡ batch)** is checked by running the same circuit in batch mode over a full
  snapshot and comparing. It's cheap, and runs continuously in simulation.
- **Replica determinism:** two replicas at the same LSN must produce byte-identical view checksums.
  This is checked in simulation and audited in production.
- **Workflow exactly-once:** simulation injects crashes between every pair of steps, and the history
  checker verifies there are no duplicate effects (S4).

## Sources

- DBSP:
  - [Best paper, VLDB 2023 (Feldera)](https://feldera.com/blog/Best-Research-Paper-VLDB-2023) ·
    [Feldera papers](https://docs.feldera.com/literature/papers/)
  - [feldera/feldera licence (MIT open-source edition)](https://github.com/feldera/feldera/blob/main/LICENSE)
  - `crates/dbsp` (examples include `transitive_closure_*`, `orgchart`)
- Differential dataflow: [crate](https://docs.rs/crate/differential-dataflow/latest) ·
  [repo (MIT)](https://github.com/TimelyDataflow/differential-dataflow)
- Salsa: [overview](https://docs.rs/crate/salsa/0.26.1/source/book/src/overview.md) ·
  [durable incrementality](https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html)
- In-Postgres IVM: [pg_trickle](https://pgxn.org/dist/pg_trickle/0.37.0/) ·
  [pg_ivm comparison](https://pgxn.org/dist/pg_trickle/0.9.0/plans/ecosystem/GAP_PG_IVM_COMPARISON.html) ·
  [RisingWave IVM guide](https://risingwave.com/blog/incremental-materialized-views-complete-guide/)
- DBOS:
  [Rust SDK (docs.rs)](https://docs.rs/crate/dbos/0.6.0) ·
  [dbos-transact-rust](https://github.com/dbos-inc/dbos-transact-rust) ·
  [What's new August 2026](https://dbos.dev/blog/new-in-dbos-august-2026) ·
  [Transactional execution](https://dbos.dev/blog/durable-code-execution-vs-transactional-execution)
- Temporal: [Rust SDK GA](https://temporal.io/blog/build-durable-applications-rust-temporal-rust-sdk-now-generally-available)
- Restate: [key concepts](https://docs.restate.dev/foundations/key-concepts) ·
  [Rust SDK changelog](https://docs.restate.dev/changelog/rust-sdk.md)
