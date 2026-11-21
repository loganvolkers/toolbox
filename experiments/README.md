# Experiments

Small, reproducible experiments cited by the design docs. Each one records its result in
[docs/research-log.md](../docs/research-log.md).

| Experiment | Question | Cited in |
|---|---|---|
| [full-recompute](./full-recompute) | Is a naive full recompute of CEL scorecard rules fast enough at 100k entities? Run: `cargo run --release -- <entities> <rules>` | [adversarial-critique.md](../docs/adversarial-critique.md) §7 |
| [derived-state-scale](./derived-state-scale) | At 100k entities and 2M relations, how long does a full recompute of every derived value (merge, 100 CEL rules, cross-entity aggregates, closures, roll-ups, authz tuples, permission view) take in Rust, and as Postgres materialised views? Can an unmodified OpenFGA read derived tuples through a Postgres view? Run: `cargo run --release -- 100000 4`, `./run-postgres.sh`, `./openfga/run.sh` | [technology-critique.md](../docs/technology-critique.md) §7.1–7.2 |
| [build-cost](./build-cost) | What do the proposed heavy Rust dependencies (dbsp, wasmtime, tantivy, async-graphql) cost in clean and incremental builds, versus a boring set (axum, sqlx, rmcp, cel)? And the OpenFGA c-archive? Run: `TARGET_ROOT=/scratch ./run.sh` | [technology-critique.md](../docs/technology-critique.md) §7.3–7.4 |
| [typescript-builder](./typescript-builder) | Can a typed TS builder emit the v1 manifest exactly and catch mistakes at compile time? Run: `npm install && npm run check` | [design/typescript-builder.md](../docs/design/typescript-builder.md) |
| [durable-schema](./durable-schema) | Can `Durable` run on a minimal, DBOS-compatible Postgres schema (idempotency, step replay, cross-instance recovery, signals)? Run: `./run.sh` | [v1-spec.md](../docs/v1-spec.md) §8.1 |
| [oss-monorepo-federation](./oss-monorepo-federation) | Can federation push/extend/report over real Babel, Jest and pnpm conventions (mined, not invented) with a readable aggregated portal? Run: `python3 experiments/oss-monorepo-federation/tests/test_compose.py` | [design/federation.md](../docs/design/federation.md) |
