#!/usr/bin/env bash
# Seikika for Seikika (docs/design/seikika-for-seikika.md): run the portal in portal/ on this repo.
#
#   scripts/self-portal.sh serve [--listen ADDR]  live portal: API, MCP (/mcp), UI; connectors run
#                                                 in-process and re-read on refresh (default :8090)
#   scripts/self-portal.sh report [ARGS]          = seikika report portal ARGS: run the connectors
#                                                 once, recompute, print every scorecard with evidence
#                                                 (--skip runner: no cargo test/clippy, about 1 s)
#   scripts/self-portal.sh snapshot DIR [--static] write the connectors' records to DIR/*.ndjson
#
# Agents: start `serve`, then use `seikika mcp --url http://127.0.0.1:8090 --as maintainer`
# (stdio MCP) or the CLI reads (`seikika score crate:seikika-derive --url …`).
set -euo pipefail
cd "$(dirname "$0")/.."
T=target/debug
PORTAL=portal

build() { cargo build -q -p seikika-cli -p seikika-connector-workspace -p seikika-connector-runner; }

# The connector's config from the manifest, as JSON.
config() {
  python3 -c 'import json,sys,yaml; m=yaml.safe_load(open(sys.argv[1])); print(json.dumps(next(c for c in m["connectors"] if c["name"]==sys.argv[2]).get("config",{})))' \
    "$PORTAL/seikika.yaml" "$1"
}

# Run one connector's resync and keep only its record messages (fixture format).
resync() {
  local name=$1 out=$2
  echo '{"type":"command","command":"resync","command_id":"snapshot"}' \
    | SEIKIKA_CONNECTOR_CONFIG="$(config "$name")" "$T/seikika-connector-$name" \
    | python3 -c 'import json,sys
for l in sys.stdin:
    m=json.loads(l)
    if m["type"]=="record":
        m.pop("command_id",None); print(json.dumps(m,sort_keys=True))
    elif m["type"]=="complete" and m["status"]!="succeeded":
        sys.exit("connector failed: "+json.dumps(m.get("errors")))' > "$out"
}

snapshot() {
  local dir=$1 static=${2:-}
  mkdir -p "$dir"
  resync workspace "$dir/workspace.ndjson"
  resync surfaces "$dir/surfaces.ndjson"
  [[ "$static" == "--static" ]] || resync runner "$dir/runner.ndjson"
}

case "${1:-report}" in
  serve)
    shift; build
    LISTEN=127.0.0.1:8090; [[ "${1:-}" == "--listen" ]] && LISTEN=$2
    exec env SEIKIKA_CONNECTORS_DIR="$PWD/$T" "$T/seikika" serve "$PORTAL" --listen "$LISTEN" ;;
  snapshot)
    build; snapshot "${2:?usage: snapshot DIR [--static]}" "${3:-}"
    wc -l "$2"/*.ndjson ;;
  report)
    # `seikika report` does this natively now (F1); extra args pass through, e.g.
    # `scripts/self-portal.sh report --skip runner --baseline portal/results.json`.
    shift; build
    exec env SEIKIKA_CONNECTORS_DIR="$PWD/$T" "$T/seikika" report "$PORTAL" "$@" ;;
  *) sed -n '2,12p' "$0"; exit 2 ;;
esac
