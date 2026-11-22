---
id: D01M81ADN8C10EA1Z1VNRY3A5JZ
decided: 2026-11-22T09:55:00Z
---

# Observability: `toolbox-telemetry` owns OTel/Prometheus bootstrap and the metric catalog

**Status:** ✅

**Where:** [plan.md](../plan.md) §4 / §6.3, [v1-spec.md](../v1-spec.md) §10.5, [observability.md](../design/observability.md), D6

**Decision:** Ship `crates/toolbox-telemetry` as the sole OpenTelemetry and Prometheus
bootstrap (resource/`service.name`, exporters, metric catalog, `tracing` log bridge).
Other crates emit only via `tracing` and catalog helpers. Keep `GET /metrics` scrape;
honour `OTEL_*`. Canonical snapshot lag is `toolbox_snapshot_lag_seconds` with a
temporary `toolbox_snapshot_age_seconds` alias. T4T scores instrumentation via
`maintainer-observability` on kind `Observability` (portal/observability data) and
assesses `sdlc-stage:operator/monitor`. Analytics sinks stay stubbed and separate (D6).

**Why:** D6 and the v1 observability contract were design-only; a single crate prevents
Prom/OTLP name drift and lets the maintainer SDLC ratchet regress when the catalog or
`service.name` disappears.
