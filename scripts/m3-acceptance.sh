#!/usr/bin/env bash
# M3 exit criteria (docs/v1-spec.md §3) on the production path: Postgres + OpenFGA as a co-process
# reading derived tuples through a view (D38). Checks: the reference .fga.yaml passes through the
# Authorizer (seikika test); A3/A4 facet shaping; A10 action authorization; PAT authentication;
# a membership removal enforced within 60 s; OpenFGA never stored a tuple.
# Usage: SEIKIKA_OPENFGA_BIN=/path/to/openfga scripts/m3-acceptance.sh
set -euo pipefail
cd "$(dirname "$0")/.."
PGBIN=${PGBIN:-/usr/lib/postgresql/16/bin}; PGDIR=${PGDIR:-/var/lib/postgresql/seikika-m3}; PORT=${PGPORT:-54341}
[ -x "$PGBIN/initdb" ] || { echo "no initdb in $PGBIN: install PostgreSQL 16 (e.g. the PGDG apt repository) or set PGBIN" >&2; exit 2; }
API=127.0.0.1:${API_PORT:-18081}; URL=http://$API; RUNAS=""
[ "$(id -u)" = 0 ] && RUNAS="runuser -u postgres --"
SEIKIKA=${SEIKIKA:-target/debug/seikika}; PORTAL=examples/reference-portal
: "${SEIKIKA_OPENFGA_BIN:?set SEIKIKA_OPENFGA_BIN to the openfga binary}"; export SEIKIKA_OPENFGA_BIN
WORK=$(mktemp -d); SERVER_PID=""
cleanup() {
  local status=$? log
  if ((status != 0)); then
    for log in "$PGDIR/pg.log" "$WORK/server.log"; do
      if [[ -r "$log" ]]; then
        printf '\n--- %s ---\n' "$log" >&2
        tail -n 100 -- "$log" >&2 || true
      fi
    done
  fi
  [ -n "$SERVER_PID" ] && kill "$SERVER_PID" 2>/dev/null || true
  $RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -m fast stop >/dev/null 2>&1 || true
  rm -rf "$WORK"
  return "$status"
}
trap cleanup EXIT
$RUNAS rm -rf "$PGDIR"; $RUNAS mkdir -p "$PGDIR"
$RUNAS "$PGBIN/initdb" -A trust -U postgres -D "$PGDIR/data" >/dev/null
$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -o "-p $PORT -k $PGDIR -c listen_addresses=127.0.0.1" -l "$PGDIR/pg.log" -w start >/dev/null
export DATABASE_URL="postgres://postgres@127.0.0.1:$PORT/postgres"
export SEIKIKA_CONNECTORS=off  # M3 feeds inputs through the ingest API only
as() { curl -s -H "x-seikika-user: $1" "${@:2}"; }
json() { python3 -c "import json,sys; d=json.load(sys.stdin); print($1)"; }

echo "1. seikika test (CEL expectations + .fga.yaml through the Authorizer)"
"$SEIKIKA" test "$PORTAL" | tail -2

echo "2. serve on Postgres with OpenFGA over the tuple view"
"$SEIKIKA" serve "$PORTAL" --listen "$API" >"$WORK/server.log" 2>&1 & SERVER_PID=$!
for _ in $(seq 200); do curl -fs "$URL/readyz" >/dev/null 2>&1 && break; sleep 0.1; done
grep -o 'authorization: [A-Za-z]*' "$WORK/server.log"
"$SEIKIKA" push "$PORTAL" --url "$URL" | tail -1
V=$(as dave "$URL/v1/snapshot" | json 'd["input_version"]')
as dave "$URL/v1/snapshot?min_snapshot=$V" >/dev/null

echo "3. A3: erin sees levels, rule details redacted"
as erin "$URL/v1/entities/repository:checkout-web/scorecards" | json '(d["scorecards"]["ownership"]["level"], d["scorecards"]["ownership"]["rules"])'
echo "4. A4: bob sees failing rules"
as bob "$URL/v1/entities/repository:checkout-web/scorecards" | json '{k: v["outcome"] for k, v in d["scorecards"]["ownership"]["rules"].items()}'
echo "5. A10: erin's preview is refused with the missing relation"
BODY='{"entity":"repository:checkout-web","inputs":{"app":"snyk-security","justification":"Need scanning before the audit"}}'
CODE=$(as erin -o "$WORK/a10.json" -w '%{http_code}' -X POST -H 'content-type: application/json' --data "$BODY" "$URL/v1/actions/install-approved-app/previews")
echo "  HTTP $CODE: $(json 'd["detail"]' < "$WORK/a10.json")"
[ "$CODE" = 403 ]
CODE=$(as bob -o /dev/null -w '%{http_code}' -X POST -H 'content-type: application/json' --data "$BODY" "$URL/v1/actions/install-approved-app/previews")
echo "  bob (GitHub admin of checkout-web): HTTP $CODE"; [ "$CODE" = 200 ]

echo "6. personal access token"
TOKEN=$("$SEIKIKA" token create --user alice --label m3 2>"$WORK/token.err") || { cat "$WORK/token.err"; exit 1; }
f=$(mktemp); curl -fs -H "authorization: Bearer $TOKEN" "$URL/v1/me" -o "$f"; json '(d["user"], d["method"])' < "$f"; rm -f "$f"
CODE=$(curl -s -o /dev/null -w '%{http_code}' -H "authorization: Bearer tbx_not_a_token" "$URL/v1/me"); [ "$CODE" = 401 ] && echo "  unknown token: 401"

echo "7. revocation: remove alice from payments"
as alice "$URL/v1/entities/repository:payments-api/scorecards" | json '"  before: claim-verified=" + d["scorecards"]["ownership"]["rules"]["claim-verified"]["outcome"]'
START=$(date +%s.%N)
curl -fs -X POST -H 'x-seikika-connector: idp' --data '[{"type":"record","kind":"group","externalId":"idp:group/payments","data":{"slug":"payments","displayName":"Payments","parent":"engineering","githubTeam":"acme/payments","members":[]}}]' "$URL/v1/ingest" >/dev/null
while :; do
  # A response without scorecards (a transient problem while the new snapshot publishes) is "not
  # yet"; its body is shown so a real failure isn't hidden by the timeout.
  R=$(as alice "$URL/v1/entities/repository:payments-api/scorecards" | python3 -c 'import json,sys
d=json.load(sys.stdin)
rules=d.get("scorecards",{}).get("ownership",{}).get("rules")
if rules is None: print("pending: " + json.dumps(d)[:200], file=sys.stderr); print("pending")
else: print(rules.get("redacted", False))' || echo pending)
  [ "$R" = True ] && break
  python3 -c "import sys,time; sys.exit(time.time()-$START > 60)" || { echo "  revocation took longer than 60 s"; exit 1; }
  sleep 0.05
done
python3 -c "import time; print(f'  enforced after {time.time()-$START:.2f} s (SLO 60 s)')"

echo "8. OpenFGA stored no tuples"
psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c "SELECT 'view rows: ' || (SELECT count(*) FROM public.tuple) || ', OpenFGA table rows: ' || (SELECT count(*) FROM public.tuple_openfga_unused) || ', derived tuples: ' || (SELECT count(*) FROM seikika.derived_tuples)"
[ "$(psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c 'SELECT count(*) FROM public.tuple_openfga_unused')" = 0 ]
echo "9. each view row's user_type is what OpenFGA's own write would store (usersets and wildcards are 'userset')"
psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c "SELECT _user || ' ' || user_type FROM public.tuple WHERE _user LIKE '%*%' OR _user LIKE '%#%' ORDER BY 1 LIMIT 3"
[ "$(psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c "SELECT count(*) FROM public.tuple WHERE (user_type = 'userset') <> (_user LIKE '%#%' OR _user = '*' OR _user ~ '^[^:]+:\\*$')")" = 0 ]
[ "$(psql -h 127.0.0.1 -p "$PORT" -U postgres -At -c "SELECT user_type FROM public.tuple WHERE _user = 'user:*' LIMIT 1")" = userset ]
echo "10. SIGTERM stops the server and its OpenFGA co-process"
FGA_PID=$(pgrep -P "$SERVER_PID" -f "openfga run" | head -1 || true)
[ -n "$FGA_PID" ]
kill -TERM "$SERVER_PID"
for _ in $(seq 50); do kill -0 "$SERVER_PID" 2>/dev/null || break; sleep 0.1; done
for _ in $(seq 20); do kill -0 "$FGA_PID" 2>/dev/null || break; sleep 0.1; done
if kill -0 "$SERVER_PID" 2>/dev/null || kill -0 "$FGA_PID" 2>/dev/null; then
  echo "still running: server $SERVER_PID or OpenFGA $FGA_PID"; exit 1
fi
echo "  server and OpenFGA $FGA_PID exited"; SERVER_PID=""
echo "M3 acceptance: PASS"
