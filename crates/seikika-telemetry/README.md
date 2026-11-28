# seikika-telemetry

This crate sets up OpenTelemetry and Prometheus for Seikika. It is part of
the framework. Other crates emit through `tracing` and the
[shared metric catalog](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-telemetry/src/catalog.rs).

## Initialize telemetry

```rust
let _guard = seikika_telemetry::init(seikika_telemetry::TelemetryOptions {
    mode: None, // detect from SEIKIKA_TELEMETRY / CI
    service_name: None, // or OTEL_SERVICE_NAME; default `seikika`
    prometheus: Some(true),
    log_format: None, // SEIKIKA_LOG_FORMAT=json|pretty
})?;
```

Call **before** binding listeners (`serve`) or accepting MCP stdio. Honors
standard `OTEL_*` env vars; does not override operator-set samplers.

| Variable | Behavior |
|---|---|
| `OTEL_SERVICE_NAME` | `service.name` (required non-empty in CI/prod) |
| `OTEL_RESOURCE_ATTRIBUTES` | Merged into the resource (operator keys win) |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Enables OTLP traces, metrics, and logs. For HTTP, use the base URL. The SDK appends the signal path. |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `grpc` (default) or `http/protobuf` |
| `OTEL_EXPORTER_OTLP_HEADERS` | Comma-separated exporter headers, such as `authorization=Bearer%20<TOKEN>` |
| `OTEL_METRIC_EXPORT_INTERVAL` | Periodic OTLP metric flush, milliseconds (default 1000) |
| `OTEL_TRACES_SAMPLER` (+ `_ARG`) | Honored when set |
| `SEIKIKA_TELEMETRY` | `local` \| `ci` \| `prod` |
| `SEIKIKA_LOG_FORMAT` | `json` \| `pretty` |
| `TELEMETRY_PROM_ALIASES` | Dual-export `seikika_snapshot_age_seconds` (default on) |

Local/CI completeness: [Telemetry testbed](https://github.com/seikika/seikika/tree/25a208a3bc1a97f184588c512afcb12d636ce7be/experiments/otel-testbed).

The server exposes the catalog at `GET /metrics`. The crate records metrics in
process even when it has no OpenTelemetry export endpoint.

See [metric catalog](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-telemetry/METRIC_CATALOG.md).
