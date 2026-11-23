# Experiments

Small, reproducible experiments cited by the design docs. Each one records its result in
[docs/research-log.md](../docs/research-log.md).

**DX Benchmarks.** The agent A/Bs below are DX Benchmarks ([docs/design/dx-benchmarks.md](../docs/design/dx-benchmarks.md)): pre-registered, on open-source forks, with Toolbox against its strongest alternative. Each is catalogued in [portal/benchmarks/](../portal/benchmarks), with its status (complete, blocked or archived) and what it changed.

| Experiment | Question | Cited in |
|---|---|---|
| [full-recompute](./full-recompute) | Is a naive full recompute of CEL scorecard rules fast enough at 100k entities? Run: `cargo run --release -- <entities> <rules>` | [adversarial-critique.md](../docs/adversarial-critique.md) §7 |
| [derived-state-scale](./derived-state-scale) | At 100k entities and 2M relations, how long does a full recompute of every derived value (merge, 100 CEL rules, cross-entity aggregates, closures, roll-ups, authz tuples, permission view) take in Rust, and as Postgres materialised views? Can an unmodified OpenFGA read derived tuples through a Postgres view? Run: `cargo run --release -- 100000 4`, `./run-postgres.sh`, `./openfga/run.sh` | [technology-critique.md](../docs/technology-critique.md) §7.1–7.2 |
| [build-cost](./build-cost) | What do the proposed heavy Rust dependencies (dbsp, wasmtime, tantivy, async-graphql) cost in clean and incremental builds, versus a boring set (axum, sqlx, rmcp, cel)? And the OpenFGA c-archive? Run: `TARGET_ROOT=/scratch ./run.sh` | [technology-critique.md](../docs/technology-critique.md) §7.3–7.4 |
| [typescript-builder](./typescript-builder) | Can a typed TS builder emit the v1 manifest exactly and catch mistakes at compile time? Run: `npm install && npm run check` | [design/typescript-builder.md](../docs/design/typescript-builder.md) |
| [durable-schema](./durable-schema) | Can `Durable` run on a minimal, DBOS-compatible Postgres schema (idempotency, step replay, cross-instance recovery, signals)? Run: `./run.sh` | [v1-spec.md](../docs/v1-spec.md) §8.1 |
| [otel-testbed](./otel-testbed) | Does Toolbox export OTLP traces/metrics/logs (and keep `/metrics`) that a local Collector can score without SaaS? Run: `scripts/score.sh` | [design/observability.md](../docs/design/observability.md) |
| [security-ab](./security-ab) | DX Benchmark: is a reusable standard (OpenSSF Scorecard) better than a security scorecard an agent builds from scratch? Build, reuse and a second build on OpenFGA and Codex, against Scorecard 5.7.0. Results: [RESULTS.md](./security-ab/RESULTS.md) | [design/openssf-template.md](../docs/design/openssf-template.md) §5 |
| [out-of-box](./out-of-box) | DX Benchmark (R2, R3, R4): do the starter and packages, untuned, give true findings on four forks; does a checklist or `toolbox report` raise standards further; does the ratchet catch real regressions in upstream history? | [design/repo-value.md](../docs/design/repo-value.md) |
| [standards-ab](./standards-ab) | DX Benchmark (R3, archived): over a standard's lifecycle, does Toolbox cost an agent less than bespoke scripts? Round 2 and the rerun B′ | [design/repo-value.md](../docs/design/repo-value.md) |
| [box-ab](./box-ab) | DX Benchmark (R6): several tools in one box, or hand-written CI glue? | [design/repo-value.md](../docs/design/repo-value.md) |
| [spec-ab](./spec-ab) | DX Benchmark (R7, blocked): results by hand from a spec, with Toolbox, or on Backstage | [design/results-spec.md](../docs/design/results-spec.md) |
| [connector-ab](./connector-ab) | DX Benchmark (C1–C3): authoring a cloud/SaaS connector in Toolbox vs Port Ocean vs Backstage | [design/package-author-sdlc.md](../docs/design/package-author-sdlc.md), [design/dx-benchmarks.md](../docs/design/dx-benchmarks.md) §7 B4 |
| [deploy-ab](./deploy-ab) | DX Benchmark (blocked): deploy a usable portal on a laptop, Toolbox against Backstage, with no cloud account. Not run | [design/local-simulation.md](../docs/design/local-simulation.md) |
