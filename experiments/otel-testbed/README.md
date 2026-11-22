# OTel local testbed (step 5)

Prove Toolbox OTLP + Prometheus instrumentation **without** Grafana Cloud or Datadog.
A pinned `otelcol-contrib` receives traces/metrics/logs (and scrapes `/metrics`), writes
JSON files under `out/`, and [`scripts/score.sh`](./scripts/score.sh) applies
[`rubric.yaml`](./rubric.yaml).

Cited by [docs/design/observability.md](../../docs/design/observability.md) and the
store design §13.

## Quick start (CI / no Docker)

From the repository root:

```bash
cargo build --locked -p toolbox-cli
experiments/otel-testbed/scripts/score.sh
```

The script downloads `otelcol-contrib` **0.142.0** (SHA-256 in [`pin.env`](./pin.env))
into `.bin/`, starts it, runs `toolbox serve examples/reference-portal --fixtures`
with `OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317`, exercises HTTP + `/metrics`,
then asserts the rubric.

## Optional Docker Collector

```bash
mkdir -p experiments/otel-testbed/out
docker compose -f experiments/otel-testbed/docker-compose.yaml up -d
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 \
  target/debug/toolbox serve examples/reference-portal --fixtures --listen 127.0.0.1:18180
# then curl healthz / metrics; inspect experiments/otel-testbed/out/
```

## Rubric (design §13.2)

| Check | What passes |
|---|---|
| `service.name` | Resource attrs in exports include `toolbox` |
| HTTP span | `http.server.request` in `out/traces.json` |
| Catalog metrics (file) | `toolbox_snapshot_lag_seconds` + `toolbox_ingest_records_total` in `out/metrics.json` |
| Log ↔ trace | JSON stderr line with `trace_id` matching an exported span |
| `/metrics` | Scrape still exposes catalog `# TYPE` lines |
| Testbed present | This directory + `score.sh` |

## Layout

```
experiments/otel-testbed/
  docker-compose.yaml      # optional otelcol container
  otelcol.yaml             # host score.sh template (placeholders)
  otelcol.compose.yaml     # compose-mounted config
  pin.env                  # version + SHA-256
  rubric.yaml              # expectations
  scripts/score.sh         # exercise + assert (CI gate)
  README.md
```

`.bin/` and `out/` are gitignored.
