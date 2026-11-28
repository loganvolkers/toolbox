#!/usr/bin/env bash
# Exercise seikika serve against a pinned otelcol-contrib and score the rubric.
# Usage (from repo root): experiments/otel-testbed/scripts/score.sh
# Optional: SEIKIKA=target/debug/seikika  OTEL_TESTBED_PORT=18180
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
BED="$ROOT/experiments/otel-testbed"
# shellcheck disable=SC1091
source "$BED/pin.env"

SEIKIKA="${SEIKIKA:-$ROOT/target/debug/seikika}"
PORT="${OTEL_TESTBED_PORT:-18180}"
API="127.0.0.1:${PORT}"
URL="http://${API}"
PROTOCOL="${OTEL_TESTBED_PROTOCOL:-grpc}"
case "$PROTOCOL" in
  grpc) OTLP_PORT=4317 ;;
  http/protobuf) OTLP_PORT=4318 ;;
  *) echo "Use OTEL_TESTBED_PROTOCOL=grpc or http/protobuf" >&2; exit 2 ;;
esac
OUT="$BED/out"
WORK="$(mktemp -d)"
COL_PID=""
SRV_PID=""
PASS=0
FAIL=0

cleanup() {
  if [ -n "${SRV_PID}" ]; then
    kill -TERM "$SRV_PID" 2>/dev/null || true
    for _ in $(seq 40); do kill -0 "$SRV_PID" 2>/dev/null || break; sleep 0.1; done
    kill -KILL "$SRV_PID" 2>/dev/null || true
  fi
  if [ -n "${COL_PID}" ]; then
    kill -TERM "$COL_PID" 2>/dev/null || true
    for _ in $(seq 40); do kill -0 "$COL_PID" 2>/dev/null || break; sleep 0.1; done
    kill -KILL "$COL_PID" 2>/dev/null || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

ok() { echo "  PASS: $1"; PASS=$((PASS + 1)); }
bad() { echo "  FAIL: $1"; FAIL=$((FAIL + 1)); }

need_seikika() {
  if [ ! -x "$SEIKIKA" ]; then
    echo "building seikika-cli…"
    pnpm install --frozen-lockfile
    pnpm build
    cargo build --locked -p seikika-cli
    SEIKIKA="$ROOT/target/debug/seikika"
  fi
}

install_otelcol() {
  local bin="$BED/.bin/otelcol-contrib"
  if [ -x "$bin" ] && "$bin" --version 2>/dev/null | grep -q "$OTELCOL_VERSION"; then
    printf '%s\n' "$bin"
    return
  fi
  mkdir -p "$BED/.bin" "$WORK/dl"
  curl -fsSL --retry 3 -o "$WORK/dl/otelcol.tgz" "$OTELCOL_URL"
  # Verify on stderr so command substitution only captures the binary path.
  echo "${OTELCOL_SHA256}  $WORK/dl/otelcol.tgz" | sha256sum -c - >&2
  tar -xzf "$WORK/dl/otelcol.tgz" -C "$BED/.bin" otelcol-contrib
  chmod +x "$bin"
  printf '%s\n' "$bin"
}

need_seikika
COL="$(install_otelcol)"

rm -rf "$OUT"
mkdir -p "$OUT"

# Render collector config (placeholders; avoids confmap env-syntax drift).
sed -e "s|__OTEL_OUT__|${OUT}|g" \
    "$BED/otelcol.yaml" >"$WORK/otelcol.yaml"
collector_config=(--config "$WORK/otelcol.yaml")
export_check=("$OUT")
if [[ "$PROTOCOL" == http/protobuf ]]; then
  # The pinned Collector predates the otlp_http exporter spelling used by LGTM.
  sed 's/otlp_http/otlphttp/g' "$ROOT/examples/cloudflare-containers/observability/collector.yaml" \
    >"$WORK/collector-public.yaml"
  collector_config=(--config "$WORK/collector-public.yaml" "${collector_config[@]}" --config "$BED/otelcol.public.yaml")
  export_check+=(--public)
fi

echo "1. start otelcol-contrib ${OTELCOL_VERSION}"
"$COL" "${collector_config[@]}" >"$OUT/otelcol.log" 2>&1 & COL_PID=$!
for _ in $(seq 50); do
  if (echo >"/dev/tcp/127.0.0.1/$OTLP_PORT") >/dev/null 2>&1; then break; fi
  sleep 0.1
done

echo "2. serve reference portal with OTLP ($PROTOCOL) → collector"
unset OTEL_EXPORTER_OTLP_TRACES_ENDPOINT OTEL_EXPORTER_OTLP_LOGS_ENDPOINT OTEL_EXPORTER_OTLP_METRICS_ENDPOINT
unset OTEL_EXPORTER_OTLP_TRACES_HEADERS OTEL_EXPORTER_OTLP_LOGS_HEADERS OTEL_EXPORTER_OTLP_METRICS_HEADERS
export OTEL_EXPORTER_OTLP_ENDPOINT="http://127.0.0.1:$OTLP_PORT"
export OTEL_EXPORTER_OTLP_PROTOCOL="$PROTOCOL"
if [[ "$PROTOCOL" == http/protobuf ]]; then
  export OTEL_EXPORTER_OTLP_HEADERS=authorization=Bearer%20seikika-testbed
  export OTEL_SERVICE_NAME=seikika-demo
else
  unset OTEL_EXPORTER_OTLP_HEADERS
  export OTEL_SERVICE_NAME=seikika
fi
export OTEL_TRACES_SAMPLER=always_on
export OTEL_METRIC_EXPORT_INTERVAL=500
export SEIKIKA_TELEMETRY=ci
export SEIKIKA_LOG_FORMAT=json
export SEIKIKA_CONNECTORS=off
export RUST_LOG=info
"$SEIKIKA" serve examples/reference-portal --fixtures --listen "$API" \
  >"$OUT/server.stdout" 2>"$OUT/server.log" & SRV_PID=$!

for _ in $(seq 200); do
  curl -fs "$URL/readyz" >/dev/null 2>&1 && break
  if ! kill -0 "$SRV_PID" 2>/dev/null; then
    echo "serve exited early:" >&2
    cat "$OUT/server.log" >&2 || true
    exit 1
  fi
  sleep 0.1
done
curl -fs "$URL/readyz" >/dev/null

echo "3. exercise HTTP + metrics scrape surface"
curl -fs "$URL/healthz" >/dev/null
curl -fs "$URL/v1/snapshot" -H 'x-seikika-user: dave' -o "$OUT/snapshot.json"
# Native OTLP snapshot lag must arrive before anything scrapes /metrics.
for _ in $(seq 60); do
  if python3 "$BED/scripts/check_exports.py" "${export_check[@]}" 2>"$WORK/export-error"; then
    break
  fi
  sleep 0.2
done
if [ -s "$WORK/export-error" ]; then
  cat "$WORK/export-error" >&2
  bad "OTLP export or correlation contract before metrics scrape"
else
  ok "OTLP traces, metrics and logs before metrics scrape"
fi
curl -fs "$URL/metrics" -o "$OUT/metrics.prom"

echo "4. stop seikika (flush OTLP batches) then collector"
kill -TERM "$SRV_PID" 2>/dev/null || true
for _ in $(seq 50); do kill -0 "$SRV_PID" 2>/dev/null || break; sleep 0.1; done
SRV_PID=""
sleep 1
kill -TERM "$COL_PID" 2>/dev/null || true
for _ in $(seq 50); do kill -0 "$COL_PID" 2>/dev/null || break; sleep 0.1; done
COL_PID=""

echo "5. score rubric"
# --- /metrics scrape catalog ---
if grep -qs '# TYPE seikika_snapshot_lag_seconds' "$OUT/metrics.prom" \
  && grep -qs 'seikika_ingest_records_total' "$OUT/metrics.prom"; then
  ok "/metrics scrape returns catalog types"
else
  bad "/metrics scrape missing catalog types"
fi

# --- testbed paths ---
if [ -f "$BED/README.md" ] && [ -x "$BED/scripts/score.sh" ]; then
  ok "testbed README + score.sh present"
else
  bad "testbed paths missing"
fi

echo
echo "otel-testbed score: $PASS passed, $FAIL failed"
if [ "$FAIL" -ne 0 ]; then
  echo "--- diagnostics ---" >&2
  ls -la "$OUT" >&2 || true
  echo "--- otelcol.log (tail) ---" >&2
  tail -40 "$OUT/otelcol.log" >&2 || true
  echo "--- server.log (tail) ---" >&2
  tail -40 "$OUT/server.log" >&2 || true
  exit 1
fi
echo "otel-testbed: PASS"
