# toolbox-telemetry

Sole OpenTelemetry / Prometheus bootstrap for Toolbox (docs/v1-spec.md §10.5,
plan §6.3, D6). **Maintainer SDLC** crate: other crates emit via `tracing` and
[`catalog`](src/catalog.rs) helpers only.

## Init

```rust
let _guard = toolbox_telemetry::init(toolbox_telemetry::TelemetryOptions {
    mode: None, // detect from TOOLBOX_TELEMETRY / CI
    service_name: None, // or OTEL_SERVICE_NAME; default `toolbox`
    prometheus: Some(true),
    log_format: None, // TOOLBOX_LOG_FORMAT=json|pretty
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
| `OTEL_TRACES_SAMPLER` (+ `_ARG`) | Honoured when set |
| `TOOLBOX_TELEMETRY` | `local` \| `ci` \| `prod` |
| `TOOLBOX_METRICS_PROMETHEUS` | Keep catalog scrapeable (default true for serve) |
| `TOOLBOX_LOG_FORMAT` | `json` \| `pretty` |
| `TELEMETRY_PROM_ALIASES` | Dual-export `toolbox_snapshot_age_seconds` (default on) |

Scrape: `GET /metrics` → `toolbox_telemetry::metrics_text()`.

See [`METRIC_CATALOG.md`](METRIC_CATALOG.md).
