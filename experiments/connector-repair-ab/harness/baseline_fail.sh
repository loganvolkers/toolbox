#!/usr/bin/env bash
# Drive each broken arm once and confirm truth fails (pre-agent baseline).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
REPAIR_HOME="${REPAIR_HOME:-/tmp/connector-repair-ab}"
export PATH="$REPAIR_HOME/bin:/usr/local/cargo/bin:$PATH"

score() {
  python3 -I "$ROOT/experiments/connector-repair-ab/harness/score.py" "$1" a || true
}

echo "=== Arm A: build + drive ==="
(
  cd "$REPAIR_HOME/arms/A"
  cargo build -q
  toolbox connector drive --kinds project,issue,component --format catalog \
    --out OUT/entities.ndjson \
    --config '{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}' \
    -- target/debug/forge-connector
)
echo "A score (expect fail):"
score "$REPAIR_HOME/arms/A"

echo "=== Arm B: resync ==="
(
  cd "$REPAIR_HOME/arms/B"
  python3 -I resync.py
)
echo "B score (expect fail):"
score "$REPAIR_HOME/arms/B"

echo "=== Arm C: resync ==="
(
  cd "$REPAIR_HOME/arms/C"
  npm run resync
)
echo "C score (expect fail):"
score "$REPAIR_HOME/arms/C"
