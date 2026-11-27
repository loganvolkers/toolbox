#!/usr/bin/env bash
# End-to-end with real connectors and no fixture push: github and git-files run in-image (against a
# mock GitHub API and a local catalog checkout), the IdP arrives through the SCIM connector, the
# native tier through the API. The derived outputs must be byte-identical to the reference model's.
set -euo pipefail
cd "$(dirname "$0")/.."
T=${T:-target/debug}
PORTAL=examples/reference-portal
WORK=$(mktemp -d)
API=127.0.0.1:${API_PORT:-18082}
URL=http://$API
GH=${GH_PORT:-18997}
SCIM=127.0.0.1:${SCIM_PORT:-19100}
PIDS=()
cleanup() {
  kill "${PIDS[@]}" 2>/dev/null || true
  rm -rf "$WORK"
}
trap cleanup EXIT
python3 "$PORTAL/tools/mock_github.py" "$GH" &
PIDS+=($!)
python3 - "$PORTAL" "$WORK/catalog" <<'PY'
import json, os, sys
for l in open(f"{sys.argv[1]}/fixtures/git-files.ndjson"):
    if l.strip():
        d = json.loads(l)["data"]; os.makedirs(f"{sys.argv[2]}/{d['repo']}", exist_ok=True)
        json.dump(d["content"], open(f"{sys.argv[2]}/{d['repo']}/{d['path']}", "w"))
PY
export SEIKIKA_CONNECTORS_DIR="$PWD/$T"
export SEIKIKA_CONNECTOR_CONFIG_OVERRIDES="{\"github\":{\"apiUrl\":\"http://127.0.0.1:$GH\"},\"git-files\":{\"root\":\"$WORK/catalog\"}}"
"$T/seikika" serve "$PORTAL" --listen "$API" >"$WORK/server.log" 2>&1 &
PIDS+=($!)
for _ in $(seq 100); do
  curl -fs "$URL/readyz" >/dev/null 2>&1 && break
  sleep 0.1
done
"$T/seikika-connector-scim" --seikika "$URL" --listen "$SCIM" --connector idp >"$WORK/scim.log" 2>&1 &
PIDS+=($!)
for _ in $(seq 50); do
  curl -s "http://$SCIM/scim/v2/Users/x" >/dev/null 2>&1 && break
  sleep 0.1
done
python3 "$PORTAL/tools/scim_feed.py" "http://$SCIM/scim/v2"
# Wait for the in-image connectors' initial resyncs (both connectors read every declared kind).
for _ in $(seq 200); do
  N=$(
    f=$(mktemp)
    curl -fs -H x-seikika-user:dave "$URL/v1/connectors" -o "$f"
    python3 -c 'import json,sys; d=json.load(sys.stdin); print(sum(1 for c in d["connectors"] for v in c["last_read"].values() if v))' <"$f"
    rm -f "$f"
  )
  [ "$N" -ge 5 ] && break
  sleep 0.1
done
curl -fs -X PUT -H x-seikika-user:dave -H content-type:application/json \
  --data '{"value":"1","reason":"Runs shared CI and DNS; tier agreed in the platform review"}' \
  "$URL/v1/entities/repository:platform-infra/native/tier" >/dev/null
V=$(
  f=$(mktemp)
  curl -fs -H x-seikika-user:dave "$URL/v1/snapshot" -o "$f"
  python3 -c 'import json,sys; print(json.load(sys.stdin)["input_version"])' <"$f"
  rm -f "$f"
)
sleep 1
curl -fs -H x-seikika-user:dave "$URL/v1/ops/derived?min_snapshot=$((V + 1))" >"$WORK/derived.json" || curl -fs -H x-seikika-user:dave "$URL/v1/ops/derived" >"$WORK/derived.json"
python3 - "$WORK/derived.json" "$PORTAL" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); bad = 0
for rel, text in sorted(d["files"].items()):
    same = open(f"{sys.argv[2]}/{rel}").read() == text; bad += not same
    print(f"  {'identical' if same else 'DIFFERENT'} {rel}")
print(f"  snapshot {d['snapshot']}")
sys.exit(1 if bad else 0)
PY
echo "connectors smoke: PASS"
