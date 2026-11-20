#!/usr/bin/env bash
# Toolbox for Toolbox (docs/design/toolbox-for-toolbox.md): run the portal in portal/ on this repo.
#
#   scripts/self-portal.sh serve [--listen ADDR]  live portal: API, MCP (/mcp), UI; connectors run
#                                                 in-process and re-read on refresh (default :8090)
#   scripts/self-portal.sh report [--static]      one-shot: run the connectors once, recompute,
#                                                 print every scorecard (--static skips the runner)
#   scripts/self-portal.sh snapshot DIR [--static] write the connectors' records to DIR/*.ndjson
#
# Agents: start `serve`, then use `toolbox mcp --url http://127.0.0.1:8090 --as maintainer`
# (stdio MCP) or the CLI reads (`toolbox score crate:toolbox-derive --url …`).
set -euo pipefail
cd "$(dirname "$0")/.."
T=target/debug
PORTAL=portal

build() { cargo build -q -p toolbox-cli -p toolbox-connector-workspace -p toolbox-connector-runner; }

# The connector's config from the manifest, as JSON.
config() {
  python3 -c 'import json,sys,yaml; m=yaml.safe_load(open(sys.argv[1])); print(json.dumps(next(c for c in m["connectors"] if c["name"]==sys.argv[2]).get("config",{})))' \
    "$PORTAL/toolbox.yaml" "$1"
}

# Run one connector's resync and keep only its record messages (fixture format).
resync() {
  local name=$1 out=$2
  echo '{"type":"command","command":"resync","command_id":"snapshot"}' \
    | TOOLBOX_CONNECTOR_CONFIG="$(config "$name")" "$T/toolbox-connector-$name" \
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
  if [[ "$static" == "--static" ]]; then
    "$T/toolbox" describe > "$dir/runner.ndjson"     # surfaces only; no test or clippy runs
  else
    resync runner "$dir/runner.ndjson"
  fi
}

case "${1:-report}" in
  serve)
    shift; build
    LISTEN=127.0.0.1:8090; [[ "${1:-}" == "--listen" ]] && LISTEN=$2
    exec env TOOLBOX_CONNECTORS_DIR="$PWD/$T" "$T/toolbox" serve "$PORTAL" --listen "$LISTEN" ;;
  snapshot)
    build; snapshot "${2:?usage: snapshot DIR [--static]}" "${3:-}"
    wc -l "$2"/*.ndjson ;;
  report)
    build
    WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT
    cp "$PORTAL/toolbox.yaml" "$WORK/"; mkdir -p "$WORK/tests"
    printf 'name: none\nassert: []\n' > "$WORK/tests/standards.test.yaml"
    snapshot "$WORK/fixtures" "${2:-}"
    "$T/toolbox" derive "$WORK" --out "$WORK/out" >/dev/null
    [[ "${2:-}" == "--static" ]] && echo "(--static: no test or clippy runs, so maintainer-crate's tests-pass and clippy-clean fail by design)"
    python3 - "$WORK/out/expected" <<'PY'
import json, sys
d = sys.argv[1]
scores = json.load(open(f"{d}/scorecards.json"))
order = ["maintainer-crate", "maintainer-docs", "operator-cli", "user-cli", "user-mcp"]
for card in order + sorted(set(scores) - set(order)):
    rows = scores.get(card, {})
    tally = {}
    for r in rows.values():
        tally[r["level"]] = tally.get(r["level"], 0) + 1
    print(f"\n## {card}  " + ", ".join(f"{k}: {v}" for k, v in sorted(tally.items())))
    for eid, r in sorted(rows.items()):
        if r["level"] == "skipped":
            continue
        failing = [k for k, v in r.get("rules", {}).items() if v != "pass"]
        print(f"  {r['level']:<7} {eid:<45} {', '.join(failing)}")
PY
    ;;
  *) sed -n '2,12p' "$0"; exit 2 ;;
esac
