#!/usr/bin/env bash
# M2 exit criteria (docs/v1-spec.md §3) against a throwaway Postgres 16:
#  1. fixtures sent through POST /v1/ingest give the same outputs as M1 (byte-identical files);
#  2. min_snapshot gives read-your-writes;
#  3. a restart loses nothing (same snapshot id, same outputs);
#  4. the publisher wrote the derived tuples to Postgres.
# Usage: scripts/m2-acceptance.sh   (needs Postgres 16 binaries and a built target/debug/seikika)
set -euo pipefail
cd "$(dirname "$0")/.."
PGBIN=${PGBIN:-/usr/lib/postgresql/16/bin}; PGDIR=${PGDIR:-/var/lib/postgresql/seikika-m2}; PORT=${PGPORT:-54340}
[ -x "$PGBIN/initdb" ] || { echo "no initdb in $PGBIN: install PostgreSQL 16 (e.g. the PGDG apt repository) or set PGBIN" >&2; exit 2; }
API=127.0.0.1:${API_PORT:-18080}; URL=http://$API; RUNAS=""
[ "$(id -u)" = 0 ] && RUNAS="runuser -u postgres --"
SEIKIKA=${SEIKIKA:-target/debug/seikika}; PORTAL=examples/reference-portal
WORK=$(mktemp -d); SERVER_PID=""
cleanup() { [ -n "$SERVER_PID" ] && kill "$SERVER_PID" 2>/dev/null || true; $RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -m fast stop >/dev/null 2>&1 || true; rm -rf "$WORK"; }
trap cleanup EXIT
$RUNAS rm -rf "$PGDIR"; $RUNAS mkdir -p "$PGDIR"
$RUNAS "$PGBIN/initdb" -A trust -U postgres -D "$PGDIR/data" >/dev/null
$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -o "-p $PORT -k $PGDIR -c listen_addresses=127.0.0.1" -l "$PGDIR/pg.log" -w start >/dev/null
export DATABASE_URL="postgres://postgres@127.0.0.1:$PORT/postgres"
export SEIKIKA_CONNECTORS=off  # M2 feeds inputs through the ingest API only
start() { "$SEIKIKA" serve "$PORTAL" --listen "$API" >>"$WORK/server.log" 2>&1 & SERVER_PID=$!
  for _ in $(seq 100); do curl -fs "$URL/readyz" >/dev/null 2>&1 && return; sleep 0.1; done; cat "$WORK/server.log"; exit 1; }
compare() { # compare the server's reference-format export with the committed files
  curl -fs -H x-seikika-user:dave "$URL/v1/ops/derived?min_snapshot=$1" > "$WORK/derived.json"
  python3 - "$WORK/derived.json" "$PORTAL" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); bad = 0
for rel, text in sorted(d["files"].items()):
    same = open(f"{sys.argv[2]}/{rel}").read() == text
    bad += not same
    print(f"  {'identical' if same else 'DIFFERENT'} {rel}")
print(f"  snapshot {d['snapshot']}")
sys.exit(1 if bad else 0)
PY
}
echo "1. fixtures through the ingest API"
start
"$SEIKIKA" push "$PORTAL" --url "$URL" | tail -1
V=$(f=$(mktemp); curl -fs -H x-seikika-user:dave "$URL/v1/snapshot" -o "$f"; python3 -c 'import json,sys; print(json.load(sys.stdin)["input_version"])' < "$f"; rm -f "$f")
compare "$V"
SNAP1=$(f=$(mktemp); curl -fs -H x-seikika-user:dave "$URL/v1/snapshot" -o "$f"; python3 -c 'import json,sys; print(json.load(sys.stdin)["snapshot"])' < "$f"; rm -f "$f")

echo "2. read-your-writes"
# user:erin has a single source (the IdP), so deleting its record removes the entity.
REC=$(grep '"login":"erin"' "$PORTAL/fixtures/idp.ndjson")
DEL=$(python3 -c 'import json,sys; r=json.loads(sys.argv[1]); print(json.dumps({"type":"delete","kind":r["kind"],"externalId":r["externalId"]}))' "$REC")
N=$(f=$(mktemp); curl -fs -H x-seikika-user:dave -X POST "$URL/v1/ingest" -H 'x-seikika-connector: idp' --data-binary "$DEL" -o "$f"; python3 -c 'import json,sys; print(json.load(sys.stdin)["input_version"])' < "$f"; rm -f "$f")
CODE=$(curl -s -H x-seikika-user:dave -o /dev/null -w '%{http_code}' "$URL/v1/entities/user:erin?min_snapshot=$N")
[ "$CODE" = 404 ] && echo "  deleted user:erin is gone at min_snapshot=$N" || { echo "  expected 404, got $CODE"; exit 1; }
N=$(f=$(mktemp); curl -fs -H x-seikika-user:dave -X POST "$URL/v1/ingest" -H 'x-seikika-connector: idp' --data-binary "$REC" -o "$f"; python3 -c 'import json,sys; print(json.load(sys.stdin)["input_version"])' < "$f"; rm -f "$f")
f=$(mktemp); curl -fs -H x-seikika-user:dave "$URL/v1/entities/user:erin?min_snapshot=$N" -o "$f"
python3 -c 'import json,sys; e=json.load(sys.stdin); assert e["fields"]["displayName"]=="Erin Park"; print("  restored user:erin visible at min_snapshot='"$N"'")' < "$f"
rm -f "$f"
compare "$N" >/dev/null && echo "  outputs identical again after delete + restore"
SNAP2=$(f=$(mktemp); curl -fs -H x-seikika-user:dave "$URL/v1/snapshot" -o "$f"; python3 -c 'import json,sys; print(json.load(sys.stdin)["snapshot"])' < "$f"; rm -f "$f")

echo "3. restart"
kill "$SERVER_PID"; wait "$SERVER_PID" 2>/dev/null || true; SERVER_PID=""
start
SNAP3=$(f=$(mktemp); curl -fs -H x-seikika-user:dave "$URL/v1/snapshot" -o "$f"; python3 -c 'import json,sys; print(json.load(sys.stdin)["snapshot"])' < "$f"; rm -f "$f")
[ "$SNAP3" = "$SNAP2" ] && echo "  same snapshot after restart: $SNAP3" || { echo "  snapshot changed: $SNAP2 → $SNAP3"; exit 1; }
compare "${SNAP3%%.*}"

echo "4. derived tuples in Postgres"
COUNT=$(psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c "SELECT count(*) FROM seikika.derived_tuples")
WANT=$(python3 -c 'import json; print(len(json.load(open("'"$PORTAL"'/authz/tuples.expected.json"))))')
[ "$COUNT" = "$WANT" ] && echo "  $COUNT tuples published (expected $WANT)" || { echo "  $COUNT tuples, expected $WANT"; exit 1; }
SNAPS=$(psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c "SELECT count(*) FROM seikika.snapshots")
echo "  $SNAPS snapshots recorded (first: $SNAP1)"
echo "M2 acceptance: PASS"
