# Verification: Test-Driven, Simulation-Tested, Differentially Checked

*Status: research + recommendation. Research date 2026-11-18. Related: every design doc, since each
contributes properties to check. The WarpStream post could not be fetched directly from this
environment; its details come from the post's indexed excerpts, the Antithesis case study and
secondary coverage.*

> **Goal.** Confidence that Toolbox behaves as specified across the space of portal configurations,
> deployment shapes, faults and time. "Absolute" confidence is not achievable by any technique;
> even AWS's lightweight formal methods "accept weaker guarantees… we can still miss bugs". The
> achievable goal is **layered, measurable assurance with explicit oracles**. Every layer has a release
> gate and a metric. The architecture is shaped *now* so that the strongest techniques
> (deterministic simulation, reference-model oracles) are possible.

---

## 1. State of the art (what the best teams do)

| Practice | Who | What it is | Evidence |
|---|---|---|---|
| **Deterministic simulation testing (DST)** | FoundationDB | Whole cluster simulated in one single-threaded process. All nondeterminism (network, disk, time, randomness) is simulated. **BUGGIFY** fault-injection sites are each enabled or disabled per run, then fire probabilistically (25% default). **Swizzle-clogging** network faults | "~a trillion CPU-hours" of simulation (FDB testing page, estimate) |
| DST for consensus and storage | TigerBeetle **VOPR** | Real code with simulated storage, network and time. Seeded faults (drop/reorder packets, partitions, read/write corruption up to ~8–9%, misdirected writes). **Safety mode, then liveness mode.** Reproducible from seed + commit | 3.3 s of simulation ≈ 39 min of real time (TigerBeetle) |
| DST for async Rust | **S2** (turmoil + "mad-turmoil" libc overrides for time and randomness), RisingWave (**madsim**), **Turso** (own DST + Antithesis) | Make tokio programs deterministic by controlling execution, entropy and time | Production Rust databases built "with DST from the ground up" |
| **Whole-system DST via a deterministic hypervisor** | **Antithesis** (used by WarpStream, Turso) | Unmodified containers in a deterministic hypervisor (built on bhyve). Fault injection (network disruption between components), parallel exploration, perfect replay. SDK assertions: **`always`** (must hold every time, and be reached), **`sometimes`** (must hold at least once, proving the test explored that state), **reachability** | WarpStream simulated its **entire SaaS** (agents + control plane + metadata store over object storage and a cloud KV store). A speculative-retry flush bug that CI missed was **found in the first 233 seconds**; 6 wall-clock hours ≈ 280 simulated hours |
| **Executable reference models + conformance** | AWS S3 **ShardStore** (SOSP'21 best paper) | A simple Rust model per component, with the same interface. Property-based tests compare implementation to model. Crash-refined models. **Shuttle** stateless model checking for concurrency/linearizability | Prevented 16 issues from reaching production; 18% of models maintained by non-specialists |
| **Verification-guided development** | AWS **Cedar** | Lean model + proofs of key properties. **Differential random testing** of Rust against the Lean model. PBT where there is no model. **Release gate:** no release unless model, proofs and DRT are current | Proofs found 4 bugs; DRT/PBT found 21 more |
| Fuzz + proof from one harness | **bolero + Kani** (s2n-quic) | One harness runs as a fuzz test *and* a bounded model-checking proof | Used in s2n-quic (Kani paper, 2026) |
| Concurrency checking | **loom** (exhaustive), **shuttle** (randomized/PCT, scales, not sound) | Controlled scheduling of threads and async | AWS ShardStore |
| History checkers | Jepsen **Elle** | Infers isolation anomalies (G0, G1c, G-single, G2…) from client-observed histories | Widely used on databases |
| Mutation testing | **cargo-mutants** | Measures whether tests would notice a broken function | Thoughtworks Radar (cost: build time) |
| **Combinatorial (t-way) testing** | NIST ACTS | Covering arrays over configuration parameters. Most failures involve 1–2 parameters; **4–6-way coverage found all faults found by exhaustive testing** in NIST studies; size grows like vᵗ·log n | NIST CSRC |

**Five lessons for Toolbox:**
1. DST must be **designed in from day one**. Every team that has it (FDB, TigerBeetle, S2, Turso)
   built the abstraction seams first. Retrofitting is the expensive path.
2. The power of DST comes from its **oracles** (invariants plus a model of the expected state), not
   from fault injection alone.
3. **Reference models** make oracles cheap. Toolbox's core is largely *pure derivation* (merge →
   entities → tuples → scores → shaped responses), so a slow, obviously-correct reference
   implementation of the whole pipeline is feasible. This is our biggest structural advantage.
4. **External oracles** exist for several of our semantics, and we should use them: real OpenFGA for
   authorization, the CEL spec conformance suite for expressions, Postgres vs. SQLite vs. the
   in-memory model for storage.
5. Configuration spaces are tamed by **generation** (random valid portals) plus **covering arrays**
   (deployment knobs) plus **corpora** (real portals), not by hand-picked matrices.

## 2. What must be true: Toolbox's properties

Every property gets an id and lives in `spec/properties.md`. Each one maps to checks: `always`
assertions in code, simulation checkers, property tests, differential tests.

**Safety (must never be violated, checked continuously):**

| Id | Property |
|---|---|
| S1 | **Non-interference:** a principal's view (API, GraphQL, MCP, CLI, UI, search, counts, views, changefeed, context packs, notifications, plan output) is identical across any two worlds that differ only in data that principal cannot read |
| S2 | **Authorization agreement:** every Check, ListObjects and ListUsers result equals OpenFGA's for the same model and tuples |
| S3 | **No unsafe deletes:** no entity is removed by a sync without `SNAPSHOT_COMPLETE`, with any `ERROR`, or above the deletion threshold |
| S4 | **At-most-once side effects per idempotency key.** No execution without the required approvals. Permission re-checked at execution |
| S5 | **Native data:** no lost updates (optimistic concurrency); a linear revision history per entity; writes to non-native fields are never applied |
| S6 | **Source precedence:** an entity's field value always comes from its highest-precedence available source |
| S7 | **Config immutability:** the running server never mutates configuration; the manifest hash is constant per deployment |
| S8 | **Plan gates:** no deploy proceeds past a tripped gate without acknowledgement; operator access is retained |
| S9 | **Raw-layer isolation:** raw records never reach context packs, embeddings, or non-operator principals |
| S10 | **Run state machine:** only legal transitions; terminal states are final |

**Liveness / convergence (must eventually hold once faults stop; checked in a TigerBeetle-style
liveness phase):**

| Id | Property |
|---|---|
| L1 | **Convergence to the oracle:** at quiescence, catalog, relations, tuples, scores, search index and graph replicas equal `Reference(world_truth, manifest, native_history)` |
| L2 | Replicas' in-memory indexes catch up with the changefeed within the bound; readiness is false until they do |
| L3 | Queues drain; no job is starved; expired leases are reclaimed |
| L4 | Timers (TTL, freshness, schedules) fire within tolerance under clock jumps and restarts |

**Equivalences (cheap, powerful oracles):**

| Id | Property |
|---|---|
| E1 | Incremental scoring ≡ full re-evaluation |
| E2 | Cached permission results and circuit permission views ≡ per-object Check by the embedded OpenFGA |
| E3 | Incremental search/graph index ≡ rebuild from the database |
| E4 | `plan` predicted diff ≡ observed diff after deploy (plan/apply equivalence) |
| E5 | Postgres store ≡ SQLite store ≡ in-memory model store (same operation histories, same results) |
| E6 | Surfaces are projections of the same data: API ≡ GraphQL ≡ MCP ≡ CLI for the same request and principal |
| E7 | Re-map from the raw layer ≡ re-fetch from sources |
| E8 | Export → import round-trip is the identity |

**Metamorphic relations:**
- Permutation invariance of connector record order.
- Idempotence of replayed syncs and duplicate webhooks.
- Rename/alias invariance of scores and permissions.
- Adding an unrelated entity changes no other entity's outcomes.
- Time-shift invariance of everything except time-based rules.

## 3. Architecture requirements (decide in Phase 0, enforce by lint)

1. **Functional core, imperative shell.** Merge, mapping, identity matching, derivation, scoring,
   authorization, response shaping, plan computation and view query evaluation are **pure functions**
   over explicit inputs. I/O lives only in the shell.
2. **One `Env` seam for all nondeterminism**, injected everywhere:
   - `Clock`, `Rng` (seeded)
   - `Net` (HTTP client and server)
   - `Store` (database), `Spawn`/timers
   - `Proc` (connector subprocesses), `Fs`
   - an `Ext` boundary for WASM/TCP extensions

   `clippy.toml` `disallowed-methods` bans `SystemTime::now`, `Instant::now`, `thread_rng`,
   `std::env::var` and raw `tokio::spawn` outside `toolbox-env`.
3. **Deterministic iteration** wherever order is observable: `BTreeMap`/`IndexMap`, or a seeded hasher
   under `cfg(sim)`.
4. **Three `Store` implementations behind one conformance suite:** Postgres (production), SQLite
   (small/dev), and **`MemStore`**, a simple reference model with a **fault-injection wrapper**
   (latency, errors, dropped connections, serialization failures, failover).
5. **A `buggify!()` macro** (FDB-style) at retry, timeout, partial-write, cache-eviction and
   lease-expiry sites. It is compiled out unless `cfg(sim)` or `cfg(antithesis)`.
6. **Assertions in code** (`always!`, `sometimes!`, `reachable!`) via a thin facade. It maps to the
   Antithesis SDK when present, to the simulation checker under `cfg(sim)`, and to debug assertions
   or metrics in production.
7. **WASM determinism:** wasmtime with fuel, NaN canonicalisation, no threads, and no WASI clock or
   randomness unless injected from `Env`.
8. **Language hosts stay at the edge.** Simulation consumes *manifests*. The TS/Python program →
   manifest path is tested separately (golden manifests + determinism checks).
9. **`MemStore` fidelity:** `MemStore` models the **isolation level Toolbox actually runs at**,
   including permitted anomalies, serialization failures and **commit reordering**, not an idealised
   serializable store. Critical transactions run at `SERIALIZABLE` with retry. Elle-style history
   checks run against **real Postgres** in container tests on every PR that touches the store, and
   continuously on `main` ([principal-review.md](../principal-review.md) §3.2).
10. **Dependency determinism audit:** every dependency with its own threads or clocks (tantivy,
    wasmtime compilation, sqlx pools, hyper/reqwest, rmcp, DBSP workers, the embedded Go runtime) is
    configured single-threaded or injected under `cfg(sim)`. A CI check replays each seed twice and
    diffs the traces.
11. **Engines under simulation:**
    - The DBSP circuit runs single-worker and steps on `Env` time.
    - The DBOS-style durable runtime is substituted by an in-memory durable store with **crash
      injection between every pair of steps**.
    - Embedded OpenFGA runs with `GOMAXPROCS=1` as a synchronous pure function of the circuit's
      tuple index.
12. **Structured history log** (`cfg(sim)` and opt-in in production) of every client operation and
   result, for history checkers (Elle-style) and trace validation against models.

## 4. The verification stack

| Layer | Technique | Targets | Oracle | Runs |
|---|---|---|---|---|
| L0 | Types, `forbid(unsafe)` in core, clippy bans (§3), `cargo-deny`, **`cargo-semver-checks`** on SDK/IR crates, JSON-Schema compatibility checks on the manifest IR and MCP tool schemas | All crates | Compiler/linters | Every commit |
| L1 | **TDD unit tests + snapshot tests** (insta), golden manifests, MCP tool-surface snapshots | All | Expected values / reviewed snapshots | Every commit |
| L2 | **Property-based tests against executable reference models** (ShardStore approach) using `proptest`/`bolero` | Merge, identity matching, derivation, scoring, response shaping, native store, plan diff | `toolbox-ref` crate: slow, obviously-correct models | Every commit (bounded); continuous campaign on `main` (deep) |
| L3 | **Differential and conformance testing vs. external oracles** | Embedded OpenFGA datastore adapter vs the stock OpenFGA server with its own storage (conformance corpus of ~1,200 checks + random model/tuple fuzzing); DBSP incremental vs batch evaluation of the same circuit (E1); CEL vs the **cel-spec conformance suite**; Backstage descriptor parsing vs Backstage; read-only GQL subset vs Neo4j (later); stores vs each other (E5) | The external implementation | Per PR (corpora + random) |
| L4 | **Fuzzing** (cargo-fuzz/bolero, structure-aware), enrolled in OSS-Fuzz | Parsers (OpenFGA DSL, YAML/catalog-info, CEL, manifest JSON, connector-protocol NDJSON, MCP JSON-RPC, AuthZEN), the WASM host boundary, the OpenFGA C ABI | Crash-freedom, sanitizers, round-trip properties | Continuous |
| L5 | **Bounded proofs** (Kani via bolero harnesses) | Small critical pure functions: ref/alias normalisation, C ABI buffer encode/decode, CEL cost accounting (no overflow, bounded), the facet-redaction function (totality, never emits a disallowed path) | Model checker | Per PR (changed harnesses) |
| L6 | **Concurrency checking** (shuttle randomized/PCT; loom for tiny primitives) | Index swap on circuit step, permission result cache, lease/job claiming logic, revision CAS | Linearizability against the reference model | Per PR (bounded) |
| L7 | **In-process DST** (`toolbox-sim`, turmoil + `Env`, S2-style) | **The whole server cluster:** N API replicas, M workers, scheduler leader, `MemStore`+faults, simulated connectors/executors/IdP/agents, simulated time | **Safety invariants continuously; liveness phase then L1–L4; full-state oracle `Reference(...)`** | Per PR: ~1k seeds. Continuously on `main`: millions of seeds. Every failing seed becomes a permanent regression test |
| L8 | **Whole-system DST** (Antithesis, optional but recommended) | Real binaries + **real Postgres** + connector containers + preview of the GitHub Action flow | Same `always`/`sometimes` assertions + an external checker container | Continuous on `main`; release gate |
| L9 | **Configuration-space testing** (§6) | Generated portals × covering-array deployments × the corpus | All oracles above | Per PR (sampled) + continuous on `main` |
| L10 | **Upgrade and migration simulation** | N−2 → N upgrades under load; mixed-version rolling clusters; model and source-ownership migrations | L1 + E8 + no data loss | Release |
| L11 | **Surface E2E** | UI (Playwright), MCP conformance, CLI, operator views, GitHub App (recorded API cassettes), Actions (`act`-style runner + a real GitHub sandbox org) | Snapshots + S1/E6 | Per PR |
| L12 | **Agent evals** | The ~50-task authoring and usage suite on real models | Success rate, retries, tokens (statistical) | Release |
| L13 | **Mutation testing** (cargo-mutants) | Core crates | Mutation score threshold | Per PR on changed files |
| L14 | **Lightweight formal models** | Distributed protocols: changefeed → replica readiness, leases, run state machine + approvals + idempotency, native-store concurrency, plan/apply | **Stateright** (Rust) or TLA+/P model checking at small scope, plus **trace validation**: simulation histories replayed against the model | Per change to a protocol; trace checks per PR |
| L15 | **Production verification** | Background "auditors": recompute a sample of scores from scratch (E1), compare cached permission results vs Check (E2), checksum replica indexes vs the database (E3), validate revision chains (S5) | The same oracles, online | Continuous; exported as metrics and alerts |

## 5. `toolbox-sim`: the deterministic world

**Actors in a simulation run** (all driven by one seed):
- **Toolbox:** 1–5 API replicas, 1–4 workers, a scheduler leader, and optionally a plan worker, all
  real code on simulated `Env`.
- **Store:** `MemStore` with a fault wrapper. Postgres-specific behaviour (SKIP LOCKED, LISTEN/NOTIFY,
  serialization failures) is modelled and kept honest by E5 conformance.
- **World sources:** simulated GitHub, IdP, PagerDuty, Kubernetes and Wiz, each with an evolving
  **ground truth**. Repos, teams and services are created, renamed, re-owned and deleted over time.
  Each simulated source speaks the real connector protocol over simulated HTTP/stdio, and can
  misbehave:
  - duplicate, drop, reorder or delay webhooks
  - fail mid-snapshot
  - omit `SNAPSHOT_COMPLETE`
  - return 403s (permission-probe paths)
  - emit malformed messages
- **Executors:** simulated GitHub workflows that succeed, fail, hang, call back twice, or call back
  after a timeout.
- **Clients:** simulated humans (API/UI-equivalent calls), **simulated agents** (random sequences of
  MCP task tools), CI (remote plans), and service-repo Actions (pushes, scorecard checks).
- **Faults:** process crash/restart at `buggify` sites, network partitions and swizzle-clogging
  between replicas, store and sources, store failover, clock jumps and skew, disk-full on the raw
  layer, WASM fuel exhaustion, slow language-host extensions.

**Phases per run** (TigerBeetle-style):
1. **Chaos phase:** workload + faults. Safety invariants (S1–S10) are checked on every step. `S1` is
   checked by **paired-world simulation**: the same seed runs twice with hidden-facet data perturbed,
   and principal-visible outputs are compared.
2. **Quiescence phase:** faults stop, workload drains.
3. **Liveness phase:** within a simulated time bound, L1–L4 must hold. The full state is compared with
   `toolbox-ref::expected(world_truth, manifest, native_history)`.

**Coverage proof.** `sometimes!` assertions must each be hit across the continuous seed campaign, for
example:
- a connector crashed mid-snapshot
- the deletion threshold tripped
- two native writers conflicted
- an approval arrived after a timeout
- a replica served while behind (and was not ready)
- a duplicate executor callback

A `sometimes!` that is never hit fails the campaign gate, because it means the simulation isn't
exploring that state.

**Reproduction:** `toolbox-sim --seed <n> --commit <sha>` replays exactly. Failing seeds are minimised
(shrinking the workload and fault schedule) and committed to `sim/regressions/`.

## 6. Testing across configurations

The configuration space has four axes, and each gets its own technique:

| Axis | Examples | Technique |
|---|---|---|
| **Portal configuration** (the customer's program) | Kinds, fields, facets, `sources` precedence, relations, authz model, derivation rules, scorecards and CEL rules, actions, canvases, notification policies, gates | **Generative:** `toolbox-gen` produces random *valid* manifests: random kinds and relation graphs, random OpenFGA models (seeded from OpenFGA's own corpus and mutated), random CEL rules type-checked against the generated schema, random facet assignments and source precedences. Each generated portal comes with a matching generated world and runs through L2/L3/L7. A small fraction goes through the TS/Python SDK path to cover language hosts |
| **Deployment configuration** | Store (SQLite / PG 15/16/17), topology (all-in-one / split roles), replicas (1/3), embeddings (on/off), auth mode (identity-aware proxy / OIDC), notification mode (shadow/pilot/on), telemetry sinks, preview data mode, arch (x86/arm) | **Covering arrays:** a 3-way array (≈ dozens of configurations) on every PR; 4-way before releases. NIST's evidence is that 4–6-way coverage matches exhaustive testing's fault detection |
| **Real portals** | Reference portals (startup, mid-size, regulated, multi-team, Backstage-migrated); public portal repos; opt-in anonymised customer manifests | **Corpus runs, "crater for portals":** every release candidate runs every corpus portal through `toolbox test`, remote-plan equivalence (E4) and DST |
| **Versions** | Toolbox N−2…N, SDK versions, connector protocol versions | Upgrade simulations (L10); protocol version-negotiation fuzzing |

**For customers: `toolbox simulate`.** The same harness runs *their* portal configuration with their
fixtures, `toolbox-synth` data and simulated sources. It reports:
- invariant violations
- non-converging derivations (e.g. source-precedence cycles)
- flapping rules
- notification storms
- gate trips
- performance budget breaches

This turns "simulation-tested" into a property of each customer's portal, not just of the engine. It
runs in their CI through the `toolbox-dev/test` Action.

## 7. Test-driven development workflow

1. **Spec first.** A feature PR starts by adding or adjusting the property ids it affects
   (`spec/properties.md`) and the **reference model** change (`toolbox-ref`).
2. **Red:** write property tests and simulation workload steps that fail against the current
   implementation (or `todo!()`).
3. **Green:** implement until L1–L3 pass and per-PR simulation seeds are clean.
4. **Prove exploration:** add `sometimes!` assertions for the new states, and show they are hit.
5. **Definition of done:**
   - properties are listed
   - the reference model is updated
   - property and differential tests exist
   - simulation workload and assertions exist
   - mutation score on changed files is ≥ threshold
   - snapshots are reviewed
6. **For AI-authored code** (likely much of it), strong oracles matter even more. Reference models,
   differential tests and simulation make "the agent says it works" checkable. Agents get
   `toolbox-sim --seed` as a debugging tool.

## 7b. Tiers for speed

Every layer runs at three intensities:
- **Inner loop:** affected components, tiny case counts, tens of seeds, cached results.
- **PR:** affected components, ~1k seeds, full conformance corpora.
- **Continuous on `main` / merge queue:** everything, millions of seeds, uncached.

The tiers use the same harnesses, so a failure found at any tier replays locally from its seed
([developer-velocity.md](./developer-velocity.md)).

## 8. Release gates and confidence metrics

A release ships only if (Cedar-style gate):

| Gate | Threshold (initial) |
|---|---|
| Unit/snapshot/property tests | 100% pass |
| OpenFGA conformance corpus + CEL conformance suite | 100% pass (documented, justified exceptions only) |
| Differential fuzzing (authz, CEL, stores) | ≥ 24 h with no divergences since the last change to the component |
| In-process DST | ≥ 10⁷ seeds since the last core change; **0 open failures**; all `sometimes!` hit |
| Whole-system DST (Antithesis) | ≥ 1,000 simulated hours on the release candidate; 0 open findings |
| Covering array | 4-way deployment array green |
| Corpus ("crater") | All corpus portals: tests green, plan ≡ apply |
| Upgrade simulations | N−2 → N green with no data loss |
| Mutation score (core crates) | ≥ 85% killed (raise over time) |
| Kani harnesses | All proofs pass |
| Agent evals | No regression > 2 pp in success rate |

**Published per release:** the property list, simulated hours, seeds run, mutation score and conformance
status. This makes the assurance claim auditable rather than rhetorical.

## 9. Tooling choices

| Need | Choice | Why / alternative |
|---|---|---|
| In-process simulation | **turmoil** + our `Env` seam (time and randomness controlled through `Env` rather than libc overrides) | Maintained tokio project; S2's production path. *Alt:* madsim (RisingWave), but it swaps tokio at compile time and its fault injection is reportedly lighter |
| Whole-system simulation | **Antithesis** (optional, commercial) | Proven on WarpStream's whole SaaS and on Turso; real Postgres and containers; replay. The Rust SDK assertions run locally as no-ops or file output, so the code is not coupled to it. *Fallback:* non-deterministic container chaos (Jepsen-style), run continuously on `main` |
| Property tests / fuzz / proofs | **bolero** (with proptest strategies), cargo-fuzz, **Kani** | One harness, three engines |
| Concurrency | **shuttle** (+ loom for primitives) | ShardStore precedent |
| Protocol models | **Stateright** (Rust-native, so engineers stay in one language) or TLA+ for the trickiest protocol | Model check plus simulation trace validation |
| History checking | Elle-style checker for native-store histories (port the relevant G0/G1c/G-single checks, or run Elle out of band) | Validates S5 |
| Snapshot tests | **insta** | Reviewable diffs |
| Mutation testing | **cargo-mutants** | Rust-native |
| Combinatorial | NIST **ACTS** (or a Rust covering-array generator) to emit the CI matrix | Free, supports constraints |

## 10. Phasing (verification is Phase 0 work, not Phase 6)

| Phase | Verification scope |
|---|---|
| **0** | `Env` seam + clippy bans; `buggify!` and assertion facade; `MemStore` + fault wrapper + store conformance suite; `toolbox-ref` skeleton (merge + scoring); `toolbox-sim` minimal (1 replica, 1 simulated source, chaos → liveness phases); OpenFGA corpus harness; CEL conformance harness. **Exit:** a planted bug (e.g. deleting without `SNAPSHOT_COMPLETE`) is found by simulation within 1,000 seeds |
| 1 | Reference models for merge, identity, derivation, shaping; S1 paired-world checks; multi-replica simulation with changefeed + readiness; covering array v1 (3-way); fuzzing of parsers; `toolbox simulate` preview |
| 2 | Scoring E1, plan/apply E4; simulated GitHub App + Actions flows; Kani harnesses; mutation testing on core crates |
| 3 | Action runs S4/S10 (Stateright model + trace validation); executor fault simulation; native store S5 + history checker |
| 4 | Knowledge/search E3 + S9; operator views S1; Antithesis whole-system runs |
| 5–6 | Corpus "crater" runs; 4-way arrays; upgrade simulations; published assurance report per release |

## 11. Honest limits

- **Simulated dependencies are only as faithful as their models.** E5 conformance keeps `MemStore`
  honest against Postgres, and recorded cassettes keep simulated GitHub honest. But a commentator
  notes that even Antithesis can't fully test integration with *external* systems you don't run. Real
  sandbox integration tests remain necessary (L11).
- **Differential testing against OpenFGA proves agreement, not correctness of OpenFGA.** If both are
  wrong the same way, we won't see it. Level-1 model tests written by humans catch intent errors.
- **Agent behaviour is statistical**, so evals measure rates, not guarantees.
- **Antithesis is commercial and closed source** (an emerging competitor, Brink, describes its own
  hypervisor as proprietary too). The in-process simulator must stand on its own, and does for
  everything except real-Postgres and multi-process effects.

## Sources

- WarpStream:
  [blog post (excerpts)](https://www.warpstream.com/blog/deterministic-simulation-testing-for-our-entire-saas) ·
  [Antithesis case study](https://antithesis.com/case_studies/warpstream/) ·
  [Aiven: DST in diskless Kafka](https://aiven.io/blog/deterministic-simulation-testing-in-diskless-apache-kafka)
- FoundationDB:
  [Simulation and Testing](https://apple.github.io/foundationdb/testing.html) ·
  [BUGGIFY (transactional.blog)](https://transactional.blog/simulation/buggify) ·
  [FDB book: correctness](https://pierrez.github.io/fdb-book/meet_fdb/correctess.html)
- TigerBeetle:
  [VOPR](https://docs.tigerbeetle.com/about/vopr) ·
  [Simulation testing for liveness](https://tigerbeetle.com/blog/2023-07-06-simulation-testing-for-liveness/) ·
  [Protocol-aware DST (2026)](https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst)
- Async Rust DST:
  [S2: DST for async Rust](https://s2.dev/blog/dst) ·
  [turmoil](https://docs.rs/crate/turmoil/0.7.2/source/src/lib.rs) ·
  [Phil Eaton: what's the big deal about DST](https://notes.eatonphil.com/2024-08-20-deterministic-simulation-testing.html) ·
  [Turso/Limbo](https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust)
- Antithesis:
  [Rust SDK tutorial](https://antithesis.com/docs/using_antithesis/sdk/rust/example.html) ·
  [Rust SDK docs](https://antithesis.com/docs/generated/sdk/rust/antithesis_sdk/index.html) ·
  [FreeBSD summit: deterministic hypervisor](https://forums.FreeBSD.org/threads/2024-freebsd-developer-summit-antithesis-deterministic-hypervisor.93970) ·
  [Brink (2026)](https://brink.dev/blog/2026/06/25/bedrock-deterministic-hypervisor/)
- AWS lightweight formal methods:
  [ShardStore paper (SOSP'21)](https://www.cs.utexas.edu/%7Ebornholt/papers/shardstore-sosp21.pdf) ·
  [shuttle](https://github.com/awslabs/shuttle)
- Cedar: [How we built Cedar (arXiv 2407.01688)](https://arxiv.org/pdf/2407.01688) ·
  [Lean into verified software development](https://aws.amazon.com/blogs/opensource/lean-into-verified-software-development)
- Kani/bolero: [Kani paper (2026)](https://arxiv.org/pdf/2607.01504) ·
  [From fuzzing to proof with bolero](https://model-checking.github.io/kani-verifier-blog/2022/10/27/using-kani-with-the-bolero-property-testing-framework.html)
- Mutation testing: [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) ·
  [Thoughtworks Radar](https://www.thoughtworks.com/radar/tools/cargo-mutants)
- Jepsen Elle: [VLDB paper](https://www.vldb.org/pvldb/volumes/14/paper/Elle%3A%20Inferring%20Isolation%20Anomalies%20from%20Experimental%20Observations)
- Combinatorial testing: [NIST ACTS FAQ](https://csrc.nist.gov/projects/automated-combinatorial-testing-for-software/faqs) ·
  [NIST IV&V 2012 slides](https://www.nasa.gov/sites/default/files/1-4b-ivv2012kuhn.pdf)
- OpenFGA conformance corpus:
  [openfga/openfga assets/tests](https://github.com/openfga/openfga/tree/main/assets/tests) ·
  [openfga/language test data](https://github.com/openfga/language)
