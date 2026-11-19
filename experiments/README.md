# Experiments

Small, reproducible experiments cited by the design docs. Each one records its result in
[docs/research-log.md](../docs/research-log.md).

| Experiment | Question | Cited in |
|---|---|---|
| [full-recompute](./full-recompute) | Is a naive full recompute of CEL scorecard rules fast enough at 100k entities? Run: `cargo run --release -- <entities> <rules>` | [adversarial-critique.md](../docs/adversarial-critique.md) §7 |
| [derived-state-scale](./derived-state-scale) | At 100k entities and 2M relations, how long does a full recompute of every derived value (merge, 100 CEL rules, cross-entity aggregates, closures, roll-ups, authz tuples, permission view) take in Rust, and as Postgres materialised views? Can an unmodified OpenFGA read derived tuples through a Postgres view? Run: `cargo run --release -- 100000 4`, `./run-postgres.sh`, `./openfga/run.sh` | [technology-critique.md](../docs/technology-critique.md) §7.1–7.2 |
| [build-cost](./build-cost) | What do the proposed heavy Rust dependencies (dbsp, wasmtime, tantivy, async-graphql) cost in clean and incremental builds, versus a boring set (axum, sqlx, rmcp, cel)? And the OpenFGA c-archive? Run: `TARGET_ROOT=/scratch ./run.sh` | [technology-critique.md](../docs/technology-critique.md) §7.3–7.4 |
