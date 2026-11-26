#!/usr/bin/env bash
# OpenFGA as a sidecar reading derived tuples through a Postgres view (no tuple writes, no FFI).
# Run after ../run-postgres.sh (reuses its data directory). Needs the `openfga` (v1.22.1) and `fga` binaries:
#   GOTOOLCHAIN=go1.26.11 go install github.com/openfga/openfga/cmd/openfga@v1.22.1
#   go install github.com/openfga/cli/cmd/fga@v0.9.0
set -euo pipefail
cd "$(dirname "$0")"
PGDIR=${PGDIR:-/var/lib/postgresql/dss}; PGBIN=${PGBIN:-/usr/lib/postgresql/16/bin}; OPENFGA=${OPENFGA:-openfga}; FGA=${FGA:-fga}
RUNAS=""; [ "$(id -u)" = 0 ] && RUNAS="runuser -u postgres --"
$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -o "-p 54329 -k $PGDIR -c listen_addresses='' -c shared_buffers=2GB -c work_mem=256MB" -l "$PGDIR/pg.log" -w start >/dev/null
URI="postgres://postgres@localhost:54329/postgres?host=$PGDIR&sslmode=disable"
"$OPENFGA" migrate --datastore-engine postgres --datastore-uri "$URI" >/dev/null 2>&1
"$OPENFGA" run --datastore-engine postgres --datastore-uri "$URI" --playground-enabled=false --metrics-enabled=false \
  --grpc-addr 127.0.0.1:8081 --http-addr 127.0.0.1:8080 --log-level warn >/dev/null 2>&1 &
FPID=$!
trap 'kill $FPID; $RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -m fast stop >/dev/null' EXIT
until curl -s 127.0.0.1:8080/healthz | grep -q SERVING; do sleep 0.5; done
export FGA_API_URL=http://127.0.0.1:8080
STORE=$("$FGA" store create --name seikika | python3 -c "import json,sys; print(json.load(sys.stdin)['store']['id'])")
"$FGA" model write --store-id "$STORE" --file model.fga >/dev/null
P="psql -q -X -h $PGDIR -p 54329 -U postgres -v ON_ERROR_STOP=1"
$P -v store="$STORE" -f tuple-view.sql
python3 -I differential.py "$STORE" "$PGDIR" 2000
echo "== propagation: change service 5's owner to a team of user 42, refresh the derived views, re-check"
"$FGA" query check --store-id "$STORE" user:42 can_view_details service:5
$P -c '\timing on' -c "UPDATE git_frag SET owner_team = 556 WHERE id = 5" -c "REFRESH MATERIALIZED VIEW mv_scores" \
   -c "REFRESH MATERIALIZED VIEW mv_tuples" -c "REFRESH MATERIALIZED VIEW fga_tuple"
"$FGA" query check --store-id "$STORE" user:42 can_view_details service:5
