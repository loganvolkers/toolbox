# Developer Velocity: Fast Inner Loops for Maintainers and Operators

*Status: research + decisions, 2026-11-19. Applies to [plan.md](../plan.md),
[sdlc.md](./sdlc.md), [verification.md](./verification.md) and
[incremental-computation.md](./incremental-computation.md). Goal: speed is designed in **before**
the first line of code, because the naive versions of several components would force a
re-architecture later.*

---

## 1. Evidence

| Source | Finding | Implication |
|---|---|---|
| **Rust compiler performance survey 2025** (Rust project, Sept 2025; ~3–3.7k responses) | The most common complaint by far: **waiting for incremental rebuilds after small changes**. Build satisfaction averaged ~6/10 | Optimise the *edit → feedback* loop, not just clean builds |
| **Rust 2026 "Fast Builds" roadmap** (rust-project-goals) | Planned work: alternative debug backends (Cranelift), the **parallel frontend**, **Relink-Don't-Rebuild**, the Wild linker, end-to-end incremental compilation, shared check/build artifacts, crate slicing. "No one thing"; "end-to-end, not just the compiler" | Structure the workspace to benefit from these as they land. Don't wait for them |
| **Rust 1.90 (Sept 2025)** | **LLD is the default linker** on x86_64 Linux, with "significant" link-time wins | Free win; mold/wild optional for further gains |
| **Parallel frontend** | Being **enabled by default on nightly** (Nov 2026), ahead of stabilisation | An opt-in nightly "fast" profile for maintainers; stable stays supported |
| **Feldera** (DBSP's authors) | They compiled SQL → **Rust code → rustc**. An 8.5k-line SQL program became ~100k lines of Rust: ~102 s to generate, **~1,618 s to compile**. Fixes: dynamic dispatch (no monomorphization), splitting into ~1,000 crates, Merkle-hash naming to **reuse compiled crates**. Result: **30 min → 2 min**, after which their own compiler became the bottleneck and needed memoisation | **Toolbox must never compile Rust per portal.** Build circuits at runtime from precompiled, dynamically-typed operators. Content-addressing and memoisation are the right reuse tools |
| **cargo-nextest** | Process-per-test parallelism; sliced and hashed **partitioning** for CI sharding; **archives** (build once, run sharded); filtersets (`-E`). Speedups vary by suite (vendor claims 2–3×; measure) | The default maintainer test runner. CI builds once and shards |
| **sccache** | Incrementally compiled crates **can't be cached**. CI sets `CARGO_INCREMENTAL=0` with `RUSTC_WRAPPER=sccache` | Local: incremental on. CI: sccache, incremental off |
| **cargo-watch** | **Archived Jan 2025**; its maintainer recommends **Bacon** or **Watchexec** | Bacon for maintainers; the Watchexec library for `toolbox dev`'s watcher |
| **cargo-hakari** | A generated workspace-hack crate unifies dependency features so shared dependencies compile once, not once per feature set | Use from day one; CI checks it is current |
| **TypeScript 7** (Go-native `tsc`, GA July 2026) | Full type checks typically **8–12× faster**; watch mode reported for 7.1; the editor language service is still JS | Operators' TS programs are type-checked with the native compiler, *off* the evaluation hot path |
| **Javy** (JS → WASM) | **Dynamic linking**: 1–16 KB modules vs ≥ 869 KB static, with a shared QuickJS provider | TS WASM functions use dynamic mode; the provider is compiled once |
| **wasmtime** | Modules can be **precompiled and serialized** to skip compilation on startup. Artifacts are tied to the wasmtime version, engine config and CPU features | A content-addressed precompile cache keyed by (module hash, engine config, wasmtime version) |
| **sqlx `#[sqlx::test]`** | A **separate database per test**, with fixtures, for isolation under parallelism | Pattern for Postgres-backed tests (maintainers). Operators use the in-memory store |
| **Test selection at scale** (Google TAP; Meta *Predictive Test Selection*, 2019) | Large orgs don't run everything on every change. They select by dependency (and history), with full runs as a safety net | Dependency-based selection in the inner loop, full suites in CI and the merge queue |

## 2. The naive designs that would force a re-architecture (and what we do instead)

| # | Naive design | Why it fails | Decision |
|---|---|---|---|
| T1 | **Compile each portal's manifest to Rust** (circuits, rules, mappings as generated code) | Feldera's experience: tens of minutes per change; generated-code compile time dominates everything | **No per-portal native code generation, ever.** Toolbox ships a fixed, precompiled operator library over **one dynamic row type**. `toolbox-ivm` builds the DBSP circuit *at runtime* from the manifest. CEL runs as compiled expression programs (interpreted), and WASM covers custom logic. DBSP's generics are instantiated once, inside `toolbox-ivm` |
| T2 | The operator loop is `toolbox build` → image → deploy → look | Minutes per change | **`toolbox dev` daemon** with hot reload: program → manifest → circuit → tests → UI, all in process, no container (§4) |
| T3 | Re-run and type-check the whole TS/Python program on every save, spawning a new runtime each time | Node or Python cold start plus a full type check, per keystroke | A **warm, persistent language host**. **Transpile-only evaluation** (no type check on the hot path). **Native `tsc` 7 type checking in the background** and in the editor. **Salsa-memoised manifest compilation:** only changed definitions are re-validated and re-type-checked |
| T4 | Tests hit Postgres serially, or start a container per test | Seconds of overhead per test; no parallelism | **The in-memory store (`MemStore`) by default** for operator and most maintainer tests. Postgres only for store-conformance and isolation tests, with a **database per test** (cloned from a migrated template) in parallel |
| T5 | Run every test on every change | O(suite) feedback | **Selective tests from the dependency graph** (§3.4, §4.3), with **content-addressed test-result caching**. Full suites run in CI and the merge queue |
| T6 | Simulation and property testing only as long CI campaigns | Bugs found hours later | **Tiered seeds:** tens of seeds on affected modules in the loop, ~1k per PR, continuous millions on `main`. Seeds run in parallel processes |
| T7 | The embedded OpenFGA Go archive rebuilds on every Rust build | A Go toolchain build plus a large static link in every loop | **Content-hashed prebuilt `toolbox-fga` archives.** `build.rs` reuses the archive unless Go sources or `go.sum` change. The crate is isolated so only crates that need it link it |
| T8 | WASM functions recompiled and re-instantiated from scratch | Slow reloads; large JS modules | Javy dynamic linking; a wasmtime precompile cache; recompile only the changed function |
| T9 | A few large crates, generic-heavy APIs, proc macros everywhere | Every edit recompiles a lot; monomorphisation bloat | A **wide, shallow workspace**; interface crates separate from implementations; `dyn` at boundaries; heavy dependencies in leaf crates; hakari |
| T10 | Remote plan inside the inner loop, or a remote plan that recomputes all production state | Network and full-scale recompute per check | `toolbox plan --local` (fixtures/synthetic) in the loop; `--remote` before PRs. Remote plans build **only the affected sub-circuits** of the candidate epoch (§5) |

## 3. Maintainers: building Toolbox itself

### 3.1 Workspace shape (decided before the first crate)
- **Wide and shallow.** Most crates depend only on `toolbox-model` and small **interface crates**
  (`*-api`: traits and types).
  - Implementations depend on interfaces, never on each other.
  - Changing an implementation recompiles only that crate and the binary link, until
    Relink-Don't-Rebuild lands.
- **Heavy dependencies isolated in leaf crates:**
  - `dbsp` in `toolbox-ivm`
  - `wasmtime` in `toolbox-wasm`
  - `tantivy` in `toolbox-search`
  - the Go archive in `toolbox-fga`
  - `rmcp`, `sqlx`

  Tests for other crates use in-memory fakes of those interfaces and don't link them.
- **Reference models** (`toolbox-ref`) depend only on `toolbox-model`, so they build in seconds and
  property tests start immediately.
- **Generics policy:** no generic types crossing crate boundaries in hot APIs; `dyn` traits at
  boundaries. Proc-macro-heavy dependencies (e.g. derive-heavy serialisation in many crates) are
  confined; `cargo build --timings` in CI tracks the worst offenders.
- **`cargo hakari`** manages a workspace-hack crate; CI runs `hakari generate --diff` and
  `manage-deps --dry-run`.

### 3.2 Profiles and toolchain

```toml
# Cargo.toml (workspace)
[profile.dev]
debug = "line-tables-only"          # backtraces and line numbers, much less debuginfo to write and link
incremental = true

[profile.dev.package."*"]           # third-party deps: optimise a little, no debuginfo
opt-level = 1
debug = false

[profile.dev.package.dbsp]          # hot paths exercised heavily by tests; keep test runs fast
opt-level = 2

[profile.ci]                        # used by CI test builds: no incremental (sccache-friendly)
inherits = "dev"
incremental = false

[profile.release]
lto = "thin"
codegen-units = 16
```

- **Linker:** rustc's default LLD on x86_64 Linux (Rust ≥ 1.90). Mold or wild are documented as an
  opt-in; macOS uses the system linker.
- **Opt-in nightly `fast` toolchain profile:**
  - the parallel frontend (`-Zthreads`)
  - the Cranelift debug backend where it works

  Stable remains the supported baseline. Each nightly feature is adopted on stable as it stabilises.

### 3.3 Tools in the inner loop

| Need | Tool |
|---|---|
| Continuous check and test on save | **Bacon** (`bacon test -- -E 'rdeps(toolbox-score)'`). cargo-watch is archived |
| Test runner | **cargo-nextest**: process-per-test, retries for known-flaky integration tests (tracked, never hidden), JUnit output |
| Snapshots | **insta** (`cargo insta review`) |
| IDE | rust-analyzer; check-on-save limited to the current package in large workspaces |
| Property tests | proptest/bolero with **small case counts locally** (`PROPTEST_CASES=64`), deep runs in CI |
| Simulation | `toolbox-sim --component <crate> --seeds 50` locally; replay any seed; failing seeds become regression tests |
| Build profiling | `cargo build --timings`; tracked in CI |

### 3.4 Selective testing for maintainers
- **`xtask affected`** uses **guppy** (the package-graph library behind hakari and nextest):
  1. map changed files to packages
  2. take their reverse dependencies
  3. emit a nextest filterset
  4. run the matching property and simulation suites (by the property ids those crates declare)
- `bacon` and the PR pipeline both use it. **The merge queue always runs everything.**

### 3.5 CI for maintainers
1. **Build once:** `cargo nextest archive` with sccache (`CARGO_INCREMENTAL=0`) and a shared cache.
2. **Run sharded:** `--partition slice:k/n` across runners.
3. **Gate long campaigns on changed components** (simulation seeds, differential fuzzing, mutation
   testing).
4. **Track timings:** `--timings` artifacts and a **dev-loop benchmark** (§6) on every PR. Regressions
   fail like performance regressions.
5. **Build system:** Cargo. Bazel/Buck2 (remote execution, hermetic caching) are only reconsidered if
   the workspace outgrows Cargo with sccache and nextest. Their cost to contributors and agents is
   real.

## 4. Operators: building portals

### 4.1 `toolbox dev`: an incremental pipeline, not a rebuild

```
 file save (watchexec lib)
   → warm language host re-evaluates the program (TS transpile-only via esbuild-class transform, or Python)
   → manifest diff (Salsa-memoised: only changed definitions re-validated, CEL re-type-checked)
   → circuit reconfiguration (rebuild affected sub-circuits; replay fixture inputs from the in-memory log)
   → affected tests rerun (§4.3)          → results streamed to terminal, browser (live reload) and MCP
   → background: native tsc 7 type check, WASM function recompiles (Javy dynamic + precompile cache)
```

- **Data:** fixtures or a `toolbox-synth` organisation (default 2k entities, configurable) in
  `MemStore`. No Postgres or containers are needed. `--store sqlite` is available for persistence
  across restarts.
- **Real sources (optional):** `toolbox dev --connector github --readonly` records to cassettes, so
  the next run replays offline.
- **One daemon, many clients:** CLI watch output, the UI, and the authoring MCP server (`toolbox mcp
  --dev` attaches to the running daemon) all read the same incremental state. An agent's `validate` or
  `test` call returns in milliseconds when nothing relevant changed.

### 4.2 Fast feedback layers (cheapest first)

| Layer | Trigger | What runs |
|---|---|---|
| Editor | Keystroke | SDK types (generated), native `tsc`/Pyright, **CEL LSP** (typed against kind schemas) |
| `check` | Save / `PostToolUse` hook | Manifest diff validation, CEL type checks, authz model validation for changed definitions only |
| `test --affected` | Save (watch) | Tests whose inputs include changed definitions (§4.3), in parallel |
| `simulate --quick` | On demand / pre-commit | Tens of seeds on affected modules |
| `plan --local` | On demand | Full plan sections against fixtures or synthetic data |
| `plan --remote` | Before the PR | Against production state (§5) |

### 4.3 Selective, parallel, cached operator tests
- **The dependency graph already exists:** the meta catalog's edges
  ([meta-catalog.md](./meta-catalog.md)) such as `Rule.reads`, `View.reads`, `Action.scopedTo`,
  `AuthzModule.governs` and fixture references.
- `toolbox test --affected` selects every test whose **declared or observed inputs** (definitions,
  fixtures, authz modules) changed. Observed inputs are recorded on each run, the same idea as
  test-impact analysis.
- **Parallel by construction:** each test file gets its own `MemStore` and circuit instance, with no
  shared state, so files run across all cores (process isolation is available with `--isolate`).
  OpenFGA model tests run against the embedded engine in process, with no CLI spawn per file.
- **Content-addressed result cache:** given the same
  `(engine version, manifest-subset hash, fixture hash, test hash)`, a result is reused. This is safe
  because evaluation is deterministic, the same premise as Bazel/Turborepo caching. CI shares the
  cache.
- **`toolbox test --all`** ignores selection and the cache; CI runs it on merge.

### 4.4 Language and toolchain choices for operator speed
- **TypeScript:**
  - Evaluation uses a warm Node process with transpile-only loading.
  - Type checking uses the native TypeScript 7 compiler. Watch mode is used when available (reported
    for 7.1); otherwise a background rerun on change.
  - The generated SDK types are prebuilt per portal (`.d.ts`), never generated on each evaluation.
- **Python:** a warm interpreter; `uv`-managed environments created once and reused.
- **WASM functions:** Javy dynamic mode for TS (tiny modules, shared provider), native Rust or Go;
  wasmtime precompile cache keyed by module hash + engine config + wasmtime version.
- **Connectors:** cassette record and replay for development; `toolbox connector test --watch`.

## 5. Making remote `plan` and epochs fast at production scale

- **Content-addressed sub-circuits.** This applies Feldera's Merkle-naming idea to state, rather than
  to compiled code:
  - Each operator subgraph is identified by a hash of its definition and its inputs' hashes.
  - A candidate epoch (plan or deploy) **reuses the state of unchanged sub-circuits** from the current
    epoch's checkpoint, and computes only changed or new ones.
  - Adding one rule recomputes that rule's operators, not the whole catalog.
- **Plans compute deltas, not snapshots.** The plan reports differences between the old and new epochs
  of only the affected outputs.
- **Phase 0 spike:** verify how much DBSP state reuse across circuit rebuilds is possible through the
  `dbsp` API. Where it isn't, fall back to replaying only the inputs of the changed subgraphs.

## 6. Dev-loop SLOs (measured, enforced)

These are performance targets, measured by a benchmark harness that edits a reference portal and the
Toolbox workspace and times time-to-feedback. They run on every PR, and regressions block release.

| Loop | Target (p50, reference laptop class) |
|---|---|
| Maintainer: `cargo check` after editing a leaf crate | ≤ 2 s |
| Maintainer: affected tests after editing one implementation crate (incremental build + run) | ≤ 15 s |
| Operator: save a CEL rule → `check` diagnostics | ≤ 300 ms |
| Operator: save a definition → affected tests green/red in `toolbox dev` (2k-entity fixtures) | ≤ 2 s |
| Operator: add a WASM transform (TS) → reloaded | ≤ 3 s |
| Operator: `simulate --quick` on affected modules | ≤ 30 s |
| Operator: `plan --remote` for a single-rule change at 100k entities | ≤ 60 s |

## 7. Changes to the plan

1. **Architecture rules** added to plan §3:
   - no per-portal code generation
   - dynamic circuits over precompiled operators
   - a wide, shallow workspace with interface crates
   - heavy dependencies in leaf crates
2. **Phase 0 spikes:**
   - dynamic DBSP circuit construction from a manifest, measuring reconfiguration time at 2k and 100k
     entities
   - sub-circuit state reuse
   - warm language host evaluation latency
   - the Salsa manifest pipeline
   - the prebuilt `toolbox-fga` archive cache
   - the dev-loop benchmark harness and the §6 SLOs
3. **Phase 1:** `toolbox dev` daemon (watch, hot reload, attached MCP), `test --affected` with the
   result cache, `check`, `plan --local`, and maintainer `xtask affected` + Bacon/nextest/hakari
   setup.
4. **Phase 2:** content-addressed sub-circuits for remote plan and epochs.

## Sources

- Rust project:
  - [Compiler performance survey 2025 results](https://blog.rust-lang.org/2025/09/10/rust-compiler-performance-survey-2025-results)
  - [2026 goal: Fast Builds](https://rust-lang.github.io/rust-project-goals/2026/roadmap-fast-builds.html) ·
    [2026 goal: parallel front end](https://rust-lang.github.io/rust-project-goals/2026/parallel-front-end.html)
  - [LLD default on 1.90](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable) ·
    [Phoronix: nightly parallel frontend by default](https://www.phoronix.com/news/Rust-Nightly-Parallel-Frontend)
- Feldera:
  [Cutting compile times from 30 to 2 minutes with one thousand crates](https://feldera.com/blog/cutting-down-rust-compile-times-from-30-to-2-minutes-with-one-thousand-crates)
- Tooling:
  - [nextest partitioning](https://nexte.st/book/partitioning) · [how nextest works](https://nexte.st/book/how-it-works.html)
  - [sccache + GitHub Actions (Depot)](https://depot.dev/blog/sccache-in-github-actions) ·
    [Incremental Rust builds in CI (Earthly)](https://earthly.dev/blog/incremental-rust-builds/)
  - [cargo-watch (archived; recommends Bacon/Watchexec)](https://docs.rs/crate/cargo-watch/latest/source/README.md) ·
    [Bacon](https://dystroy.org/bacon/community/FAQ/)
  - [hakari](https://docs.rs/crate/hakari/latest/source/src/lib.rs) ·
    [RFC 3692 feature unification](https://rust-lang.github.io/rfcs/3692-feature-unification.html)
  - [matklad: Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html) ·
    [corrode: tips for faster compile times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/)
  - [Depot: faster Rust builds in CI](https://depot.dev/blog/guide-to-faster-rust-builds-in-ci)
- TypeScript 7:
  [announcement](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/) ·
  [InfoQ](https://infoq.com/news/2026/08/typescript-7-released/)
- WASM:
  [wasmtime serialize/deserialize](https://docs.wasmtime.dev/examples-rust-serialize.html) ·
  [Javy (dynamic linking)](https://github.com/bytecodealliance/javy)
- Databases: [sqlx `#[sqlx::test]`](https://docs.rs/sqlx/0.8.3/sqlx/attr.test.html)
- Test selection:
  [Google: assessing transition-based test selection in TAP (ICSE SEIP 2019)](https://mpapad.github.io/publications/pdfs/ICSE-SEIP2019.pdf) ·
  [Foundations of predictive test selection (DPE 2022)](https://dpe.org/files/sessions/2022/Foundations_of_Predictive_Test_Slides.pdf)
