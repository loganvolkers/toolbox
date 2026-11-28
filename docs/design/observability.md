# Observability (OTel + Prometheus)

*Status: built — bootstrap + catalog (step 4) and local collector testbed (step 5).
Maintainer SDLC.*

## Conclusion

Seikika is an **OTLP producer** with **Prometheus `/metrics` retained**. All bootstrap
lives in [`crates/seikika-telemetry`](../../crates/seikika-telemetry/): `init`, resource
(`service.name`), exporters, SemVer metric catalog, and the `tracing` → OTLP log bridge.
Operators own the Collector and backends (Grafana LGTM as the D6 reference).

## Signals

| Signal | Contract |
|---|---|
| Traces | `tracing` spans (`http.server.request`); OTLP when `OTEL_EXPORTER_OTLP_ENDPOINT` is set |
| Metrics | Catalog instruments; scrape via `/metrics`; OTLP meters dual-write catalog names |
| Logs | JSON (CI/prod) or pretty (local); correlated trace and span IDs in OTLP records |

`SEIKIKA_MCP_TRACE` NDJSON stays a contract-coverage artifact (F9), not distributed tracing.

## Metric catalog

See [`METRIC_CATALOG.md`](../../crates/seikika-telemetry/METRIC_CATALOG.md). Canonical
snapshot lag is `seikika_snapshot_lag_seconds`; `seikika_snapshot_age_seconds` is a
deprecated alias while `TELEMETRY_PROM_ALIASES` is on (default).

## Local testbed

[`experiments/otel-testbed`](../../experiments/otel-testbed/) runs a pinned
`otelcol-contrib` with a file exporter and [`scripts/score.sh`](../../experiments/otel-testbed/scripts/score.sh)
applies the design §13 rubric. CI job `otel-testbed` must pass.

The hosted demo uses the [Cloudflare observability example](../../examples/cloudflare-containers/observability/)
to export over HTTPS and publish a guest Grafana dashboard. The Collector filters the dedicated demo
data set before storage. See the [operator playbook](../../site/src/content/docs/operators/observability.md).

## S4S

Scorecard `maintainer-observability` over kind `Observability` (declared in
`.seikika/observability/`). Rules fail when the crate, catalog, scrape test, alert stubs,
operator/monitor assessment or OTLP testbed are missing. Gold includes
`otlp-export-tested`.

## Env

Documented in the crate README: `OTEL_*`, `SEIKIKA_TELEMETRY`, `SEIKIKA_METRICS_PROMETHEUS`,
`SEIKIKA_LOG_FORMAT`, `TELEMETRY_PROM_ALIASES`, `OTEL_METRIC_EXPORT_INTERVAL`.

## Sources

- D6 (`docs/decisions/D01M7TCJ31X0000000000000000.md`)
- D01M81ADN8C10EA1Z1VNRY3A5JZ
- plan.md §6.3–6.5
- v1-spec.md §10.5
- experiments/otel-testbed
