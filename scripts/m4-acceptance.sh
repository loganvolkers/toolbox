#!/usr/bin/env bash
# M4 exit criteria (docs/v1-spec.md §3): A1–A5 and A11 on the production path (Postgres + OpenFGA
# over the tuple view), fed only by real connectors (github and git-files in-image against a mock
# GitHub API and a local checkout; the IdP through the SCIM connector), and the connector
# conformance kit for all three connectors.
# Usage: TOOLBOX_OPENFGA_BIN=/path/to/openfga scripts/m4-acceptance.sh
set -euo pipefail
cd "$(dirname "$0")/.."
PGBIN=${PGBIN:-/usr/lib/postgresql/16/bin}; PGDIR=${PGDIR:-/var/lib/postgresql/toolbox-m4}; PORT=${PGPORT:-54342}
T=${T:-target/debug}; PORTAL=examples/reference-portal; WORK=$(mktemp -d); RUNAS=""
[ "$(id -u)" = 0 ] && RUNAS="runuser -u postgres --"
API=127.0.0.1:${API_PORT:-18085}; URL=http://$API; GH=${GH_PORT:-18996}; SCIM=127.0.0.1:${SCIM_PORT:-19101}
: "${TOOLBOX_OPENFGA_BIN:?set TOOLBOX_OPENFGA_BIN}"; export TOOLBOX_OPENFGA_BIN
PIDS=(); cleanup() { kill "${PIDS[@]}" 2>/dev/null || true; sleep 0.5; $RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -m fast stop >/dev/null 2>&1 || true; rm -rf "$WORK"; }
trap cleanup EXIT
$RUNAS rm -rf "$PGDIR"; $RUNAS mkdir -p "$PGDIR"
$RUNAS "$PGBIN/initdb" -A trust -U postgres -D "$PGDIR/data" >/dev/null
$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -o "-p $PORT -k $PGDIR -c listen_addresses=127.0.0.1" -l "$PGDIR/pg.log" -w start >/dev/null
export DATABASE_URL="postgres://postgres@127.0.0.1:$PORT/postgres"
python3 "$PORTAL/tools/mock_github.py" "$GH" & PIDS+=($!)
python3 - "$PORTAL" "$WORK/catalog" <<'PY'
import json, os, sys
for l in open(f"{sys.argv[1]}/fixtures/git-files.ndjson"):
    if l.strip():
        d = json.loads(l)["data"]; os.makedirs(f"{sys.argv[2]}/{d['repo']}", exist_ok=True)
        json.dump(d["content"], open(f"{sys.argv[2]}/{d['repo']}/{d['path']}", "w"))
PY
ask() { "$T/toolbox" "$@" --url "$URL"; }

echo "1. connector conformance kit"
"$T/toolbox" connector test --name github --portal "$PORTAL" --config "{\"apiUrl\":\"http://127.0.0.1:$GH\"}" -- "$T/toolbox-connector-github" | tail -1
"$T/toolbox" connector test --name git-files --portal "$PORTAL" --config "{\"root\":\"$WORK/catalog\"}" -- "$T/toolbox-connector-git-files" | tail -1

echo "2. serve with real connectors only"
export TOOLBOX_CONNECTORS_DIR="$PWD/$T"
export TOOLBOX_CONNECTOR_CONFIG_OVERRIDES="{\"github\":{\"apiUrl\":\"http://127.0.0.1:$GH\"},\"git-files\":{\"root\":\"$WORK/catalog\"}}"
"$T/toolbox" serve "$PORTAL" --listen "$API" >"$WORK/server.log" 2>&1 & PIDS+=($!)
for _ in $(seq 200); do curl -fs "$URL/readyz" >/dev/null 2>&1 && break; sleep 0.1; done
"$T/toolbox-connector-scim" --toolbox "$URL" --listen "$SCIM" --connector idp >"$WORK/scim.log" 2>&1 & PIDS+=($!)
for _ in $(seq 50); do curl -s "http://$SCIM/scim/v2/Users/x" >/dev/null 2>&1 && break; sleep 0.1; done
echo "  SCIM (push-only connector): $(python3 "$PORTAL/tools/scim_feed.py" "http://$SCIM/scim/v2")"
for _ in $(seq 300); do
  N=$(curl -fs -H x-toolbox-user:dave "$URL/v1/views/ops" | python3 -c 'import json,sys; d=json.load(sys.stdin); print(sum(1 for c in d["connectors"] for v in c["last_read"].values() if v))' 2>/dev/null || echo 0)
  [ "$N" -ge 5 ] && break; sleep 0.1
done
curl -fs -H x-toolbox-user:dave "$URL/v1/views/ops" | python3 -c 'import json,sys; d=json.load(sys.stdin); print("  registered:", ", ".join(c["name"] for c in d["connectors"] if c["registered"]))'
curl -fs -X PUT -H x-toolbox-user:dave -H content-type:application/json --data '{"value":"1","reason":"Runs shared CI and DNS; tier agreed in the platform review"}' "$URL/v1/entities/repository:platform-infra/native/tier" >/dev/null
sleep 1.5
curl -fs -H x-toolbox-user:dave "$URL/v1/ops/derived" > "$WORK/derived.json"
python3 - "$WORK/derived.json" "$PORTAL" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); bad = 0
for rel, text in sorted(d["files"].items()):
    same = open(f"{sys.argv[2]}/{rel}").read() == text; bad += not same
    print(f"  {'identical' if same else 'DIFFERENT'} {rel}")
sys.exit(1 if bad else 0)
PY

echo "3. A1 (CLI → MCP owner_of as alice)"; ask owner repository:payments-api --as alice | head -1
echo "4. A2 (explain ledger)"; ask owner repository:ledger --as alice | sed -n '1p;3p'
echo "5. A3 (erin: levels only)"; ask score repository:checkout-web --as erin | grep -E 'ownership|rule details' | head -2
echo "6. A4 (bob: details)"; ask score repository:checkout-web --as bob | grep -E 'branch-protected'
echo "7. A5 (API filter)"
curl -fs -G -H x-toolbox-user:erin "$URL/v1/entities" --data-urlencode "kind=Repository" --data-urlencode "filter=entity.ownership.status != 'verified'" \
  | python3 -c 'import json,sys; ids=[e["id"] for e in json.load(sys.stdin)["items"]]; print(" ", ids); assert ids==["repository:ledger","repository:legacy-tool","repository:platform-infra"]'
echo "8. A11 (UI refresh in a browser)"
SCREENSHOT="${SCREENSHOT:-$WORK/a11.png}" node scripts/ui-a11.cjs "$URL" "http://127.0.0.1:$GH"
echo "9. SCIM revocation through the connector"
curl -fs -X PUT -H content-type:application/scim+json --data '{"displayName":"Payments","members":[],"urn:ietf:params:scim:schemas:extension:toolbox:2.0:Group":{"slug":"payments","parent":"engineering","githubTeam":"acme/payments"}}' "http://$SCIM/scim/v2/Groups/g-payments" >/dev/null
START=$(date +%s.%N)
until ask score repository:payments-api --as alice | grep -q 'rule details: 🔒'; do
  python3 -c "import sys,time; sys.exit(time.time()-$START > 60)" || { echo "  not revoked within 60 s"; exit 1; }; sleep 0.1
done
python3 -c "import time; print(f'  alice lost payments details {time.time()-$START:.2f} s after the SCIM update')"
echo "M4 acceptance: PASS"
