# OTel local testbed (step 5)

Exercise Seikika instrumentation with a pinned local Collector. It writes separate
OTLP traces, metrics, and logs under `out/`. The test checks native OTLP data points
and scrapes `/metrics` independently. [Test script](./scripts/score.sh) · [Rubric](./rubric.yaml)

Cited by [docs/design/observability.md](../../docs/design/observability.md) and the
store design §13.

## Quick start (CI / no Docker)

From the repository root:

```bash
cargo build --locked -p seikika-cli
experiments/otel-testbed/scripts/score.sh
OTEL_TESTBED_PROTOCOL=http/protobuf experiments/otel-testbed/scripts/score.sh
```

The script downloads `otelcol-contrib` **0.142.0** (SHA-256 in [`pin.env`](./pin.env))
into `.bin/`, starts it, runs `seikika serve examples/reference-portal --fixtures`
with the selected transport, exercises HTTP and `/metrics`, then asserts the rubric.
CI runs both commands. The HTTP run uses an authenticated receiver on port 4318,
configures a base URL so the SDK must append each signal path, and applies the
demo Collector's public filter. It checks that the private snapshot request and
extra attributes stay out of the exports. Each run replaces the previous files in
`out/`.

## Optional Docker Collector

```bash
mkdir -p experiments/otel-testbed/out
docker compose -f experiments/otel-testbed/docker-compose.yaml up -d
OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317 \
  target/debug/seikika serve examples/reference-portal --fixtures --listen 127.0.0.1:18180
# then curl healthz / metrics; inspect experiments/otel-testbed/out/
```

## Rubric (design §13.2)

| Check | What passes |
|---|---|
| `service.name` | All three OTLP signal checks use the `seikika` resource |
| HTTP span | `http.server.request` for `GET /healthz` with numeric status 200 |
| Catalog metrics (file) | Native OTLP points for the two ingest counters and a measured `seikika_snapshot_lag_seconds` before any Prometheus scrape |
| Log ↔ trace | Matching trace and span IDs on the OTLP request log and span, and the same trace ID in stderr |
| `/metrics` | Scrape still exposes catalog `# TYPE` lines |
| Testbed present | This directory + `score.sh` |

## Layout

```
experiments/otel-testbed/
  docker-compose.yaml      # optional otelcol container
  otelcol.yaml             # host score.sh template (placeholders)
  otelcol.public.yaml      # apply the demo filter to the HTTP file exporters
  otelcol.compose.yaml     # compose-mounted config
  pin.env                  # version + SHA-256
  rubric.yaml              # expectations
  scripts/score.sh         # exercise + assert (CI gate)
  scripts/check_exports.py # assert the exported OTLP records
  README.md
```

`.bin/` and `out/` are gitignored.
