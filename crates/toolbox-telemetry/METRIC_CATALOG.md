# Metric catalog (SemVer)

Names, units and required labels are SemVer-covered. Adding optional labels is
minor; renaming or removing is major. Owned by `toolbox-telemetry` (maintainer
SDLC).

Pin: OpenTelemetry Rust **0.33.x**. MCP gen-ai semantic conventions are
Development; pin the revision when enabling MCP span attrs beyond the catalog
counter.

## v1 required set

| Name | Type | Unit | Labels | Notes |
|---|---|---|---|---|
| `toolbox_ingest_records_total` | counter | 1 | — | |
| `toolbox_ingest_batches_total` | counter | 1 | — | |
| `toolbox_recomputes_total` | counter | 1 | — | |
| `toolbox_recompute_seconds` | gauge | s | — | Last duration; histogram migration next minor |
| `toolbox_snapshot_lag_seconds` | gauge | s | — | Canonical (v1-spec §10.5) |
| `toolbox_snapshot_age_seconds` | gauge | s | — | Deprecated alias while `TELEMETRY_PROM_ALIASES` ≠ `0` |
| `toolbox_command_duration_seconds` | histogram | s | `status` | |
| `toolbox_run_transitions_total` | counter | 1 | `state` | Stub until M5 |
| `toolbox_authz_checks_total` | counter | 1 | `decision` | `allow` \| `deny` |
| `toolbox_mcp_tool_calls_total` | counter | 1 | `tool`,`outcome` | |
| `toolbox_connector_lag_seconds` | gauge | s | `connector`,`kind` | |
| `toolbox_scoring_backlog` | gauge | 1 | — | |
| `toolbox_action_runs_total` | counter | 1 | `action`,`state` | Stub until M5 |
| `toolbox_index_freshness_seconds` | gauge | s | `index` | |

`TOOLBOX_MCP_TRACE` NDJSON is **not** part of this catalog (contract coverage, F9).
