# Metric catalog (SemVer)

Names, units and required labels are SemVer-covered. Adding optional labels is
minor; renaming or removing is major. Owned by `seikika-telemetry` (maintainer
SDLC).

Pin: OpenTelemetry Rust **0.33.x**. MCP gen-ai semantic conventions are
Development; pin the revision when enabling MCP span attrs beyond the catalog
counter.

## v1 required set

| Name | Type | Unit | Labels | Notes |
|---|---|---|---|---|
| `seikika_ingest_records_total` | counter | 1 | — | |
| `seikika_ingest_batches_total` | counter | 1 | — | |
| `seikika_recomputes_total` | counter | 1 | — | |
| `seikika_recompute_seconds` | gauge | s | — | Last duration; histogram migration next minor |
| `seikika_snapshot_lag_seconds` | gauge | s | — | Canonical (v1-spec §10.5) |
| `seikika_snapshot_age_seconds` | gauge | s | — | Deprecated alias while `TELEMETRY_PROM_ALIASES` ≠ `0` |
| `seikika_command_duration_seconds` | histogram | s | `status` | |
| `seikika_run_transitions_total` | counter | 1 | `state` | Stub until M5 |
| `seikika_authz_checks_total` | counter | 1 | `decision` | `allow` \| `deny` |
| `seikika_mcp_tool_calls_total` | counter | 1 | `tool`,`outcome` | |
| `seikika_connector_lag_seconds` | gauge | s | `connector`,`kind` | |
| `seikika_scoring_backlog` | gauge | 1 | — | |
| `seikika_action_runs_total` | counter | 1 | `action`,`state` | Stub until M5 |
| `seikika_index_freshness_seconds` | gauge | s | `index` | |

`SEIKIKA_MCP_TRACE` NDJSON is **not** part of this catalog (contract coverage, F9).
