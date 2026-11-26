# seikika-telemetry

Sole OpenTelemetry / Prometheus bootstrap for Seikika (docs/v1-spec.md §10.5,
plan §6.3, D6). **Maintainer SDLC** crate: other crates emit via `tracing` and
[`catalog`](src/catalog.rs) helpers only.

## Init

```rust
let _guard = seikika_telemetry::init(seikika_telemetry::TelemetryOptions {
    mode: None, // detect from SEIKIKA_TELEMETRY / CI
    service_name: None, // or OTEL_SERVICE_NAME; default `seikika`
    prometheus: Some(true),
    log_format: None, // SEIKIKA_LOG_FORMAT=json|pretty
})?;
```

Call **before** binding listeners (`serve`) or accepting MCP stdio. Honours
standard `OTEL_*` env vars; does not override operator-set samplers.

| Variable | Behaviour |
|---|---|
| `OTEL_SERVICE_NAME` | `service.name` (required non-empty in CI/prod) |
| `OTEL_RESOURCE_ATTRIBUTES` | Merged into the resource (operator keys win) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Enables OTLP traces/metrics/logs |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `grpc` (default) or `http/protobuf` |
| `OTEL_METRIC_EXPORT_INTERVAL` | Periodic OTLP metric flush, milliseconds (default 1000) |
| `OTEL_TRACES_SAMPLER` (+ `_ARG`) | Honoured when set |
| `SEIKIKA_TELEMETRY` | `local` \| `ci` \| `prod` |
| `SEIKIKA_METRICS_PROMETHEUS` | Keep catalog scrapeable (default true for serve) |
| `SEIKIKA_LOG_FORMAT` | `json` \| `pretty` |
| `TELEMETRY_PROM_ALIASES` | Dual-export `seikika_snapshot_age_seconds` (default on) |

Local/CI completeness: [`experiments/otel-testbed`](../../experiments/otel-testbed/).

Scrape: `GET /metrics` → `seikika_telemetry::metrics_text()`.

See [`METRIC_CATALOG.md`](METRIC_CATALOG.md).
