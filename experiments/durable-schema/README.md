# durable-schema

**Question** (Q4, D39, [v1-spec.md](../../docs/v1-spec.md) §8.1): can v1's `Durable` contract run on a
minimal Postgres schema that is a **column subset of DBOS's system tables**, so that adopting the DBOS
Rust SDK later is a migration rather than a redesign?

**Answer (2026-11-20, Postgres 16.15): yes for the behaviours v1 needs.** [`schema.sql`](./schema.sql)
uses DBOS's table and column names (`workflow_status`, `operation_outputs`, `notifications`, from
`dbos-transact-rust` at `bad825f`, `migrations/1_initial_dbos_schema.sql`) plus one Seikika table
(`executors`, for instance heartbeats). [`test.sql`](./test.sql) shows:

| Behaviour | Result |
|---|---|
| A second workflow with the same idempotency key (`deduplication_id`) | Rejected (unique violation) |
| Replaying a completed step | Returns the stored output; nothing re-runs |
| Instance A dies (stale heartbeat) with a `PENDING` workflow | Instance B claims it, `recovery_attempts` = 1 |
| An approval delivered as a notification | Consumed exactly once (`FOR UPDATE SKIP LOCKED`) |

Not shown: the Rust `Durable` implementation, timers (`workflow_deadline_epoch_ms`), or a crash
*inside* a step (the step's external call must carry the run's idempotency key, P12).

Run: `./run.sh` (needs Postgres 16 binaries).
