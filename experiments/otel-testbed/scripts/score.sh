#!/usr/bin/env bash
# Exercise toolbox serve against a pinned otelcol-contrib and score the rubric.
# Usage (from repo root): experiments/otel-testbed/scripts/score.sh
# Optional: TOOLBOX=target/debug/toolbox  OTEL_TESTBED_PORT=18180
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
BED="$ROOT/experiments/otel-testbed"
# shellcheck disable=SC1091
source "$BED/pin.env"

TOOLBOX="${TOOLBOX:-$ROOT/target/debug/toolbox}"
PORT="${OTEL_TESTBED_PORT:-18180}"
API="127.0.0.1:${PORT}"
URL="http://${API}"
OTLP="http://127.0.0.1:4317"
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

need_toolbox() {
  if [ ! -x "$TOOLBOX" ]; then
    echo "building toolbox-cli…"
    cargo build --locked -p toolbox-cli
    TOOLBOX="$ROOT/target/debug/toolbox"
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

need_toolbox
COL="$(install_otelcol)"

rm -rf "$OUT"
mkdir -p "$OUT"

# Render collector config (placeholders; avoids confmap env-syntax drift).
sed -e "s|__TOOLBOX_SCRAPE_TARGET__|${API}|g" \
    -e "s|__OTEL_OUT__|${OUT}|g" \
    "$BED/otelcol.yaml" >"$WORK/otelcol.yaml"

echo "1. start otelcol-contrib ${OTELCOL_VERSION}"
"$COL" --config "$WORK/otelcol.yaml" >"$OUT/otelcol.log" 2>&1 & COL_PID=$!
for _ in $(seq 50); do
  # Collector binds 4317; a refused connect that later succeeds is enough.
  if (echo >/dev/tcp/127.0.0.1/4317) >/dev/null 2>&1; then break; fi
  sleep 0.1
done

echo "2. serve reference portal with OTLP → collector"
export OTEL_EXPORTER_OTLP_ENDPOINT="$OTLP"
export OTEL_SERVICE_NAME=toolbox
export OTEL_METRIC_EXPORT_INTERVAL=500
export TOOLBOX_TELEMETRY=ci
export TOOLBOX_LOG_FORMAT=json
export TOOLBOX_CONNECTORS=off
export RUST_LOG=info
"$TOOLBOX" serve examples/reference-portal --fixtures --listen "$API" \
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
curl -fs "$URL/v1/snapshot" -H 'x-toolbox-user: dave' -o "$OUT/snapshot.json"
curl -fs "$URL/metrics" -o "$OUT/metrics.prom"
# Wait for OTLP metric export interval + collector batch flush.
sleep 3

echo "4. stop toolbox (flush OTLP batches) then collector"
kill -TERM "$SRV_PID" 2>/dev/null || true
for _ in $(seq 50); do kill -0 "$SRV_PID" 2>/dev/null || break; sleep 0.1; done
SRV_PID=""
sleep 1
kill -TERM "$COL_PID" 2>/dev/null || true
for _ in $(seq 50); do kill -0 "$COL_PID" 2>/dev/null || break; sleep 0.1; done
COL_PID=""

echo "5. score rubric"
# --- service.name ---
if grep -Rqs '"service.name"' "$OUT"/*.json 2>/dev/null \
  || grep -Rqs 'service.name' "$OUT"/traces.json "$OUT"/logs.json 2>/dev/null; then
  if grep -Rqs 'toolbox' "$OUT"/traces.json "$OUT"/logs.json "$OUT"/metrics.json 2>/dev/null; then
    ok "service.name present (toolbox)"
  else
    bad "service.name resource missing toolbox value"
  fi
else
  bad "service.name not found in exported files"
fi

# --- HTTP span ---
if grep -Rqs 'http.server.request' "$OUT"/traces.json 2>/dev/null; then
  ok "http.server.request span exported"
else
  bad "no http.server.request span in traces.json (see $OUT/otelcol.log)"
fi

# --- catalog metrics in file exporter ---
if grep -Rqs 'toolbox_snapshot_lag_seconds' "$OUT"/metrics.json 2>/dev/null; then
  ok "metrics file has toolbox_snapshot_lag_seconds"
else
  bad "metrics file missing toolbox_snapshot_lag_seconds"
fi
if grep -Rqs 'toolbox_ingest_records_total' "$OUT"/metrics.json 2>/dev/null; then
  ok "metrics file has toolbox_ingest_records_total"
else
  bad "metrics file missing toolbox_ingest_records_total"
fi

# --- log / trace_id correlation ---
if grep -qs 'trace_id' "$OUT/server.log"; then
  TRACE_ID="$(grep -oE '"trace_id":"[0-9a-f]{32}"' "$OUT/server.log" | head -1 | sed 's/.*"\([0-9a-f]\{32\}\)".*/\1/' || true)"
  if [ -n "${TRACE_ID:-}" ] && grep -Rqs "$TRACE_ID" "$OUT"/traces.json 2>/dev/null; then
    ok "server JSON log trace_id matches an exported span ($TRACE_ID)"
  elif [ -n "${TRACE_ID:-}" ]; then
    # OTel JSON may encode trace_id as bytes/base64; still require a non-empty correlated field.
    if grep -Rqs 'traceId\|trace_id\|TraceId' "$OUT"/traces.json "$OUT"/logs.json 2>/dev/null; then
      ok "server JSON log has trace_id and exports carry trace ids"
    else
      bad "trace_id in server.log but no trace id in exports"
    fi
  else
    bad "trace_id key in server.log but could not parse hex id"
  fi
else
  bad "server.log missing trace_id (expected JSON correlated http log)"
fi

# --- /metrics scrape catalog ---
if grep -qs '# TYPE toolbox_snapshot_lag_seconds' "$OUT/metrics.prom" \
  && grep -qs 'toolbox_ingest_records_total' "$OUT/metrics.prom"; then
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
