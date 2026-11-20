#!/usr/bin/env bash
# Apply schema.sql to a throwaway Postgres 16 and run test.sql. As root, the server runs as `postgres`.
set -euo pipefail
cd "$(dirname "$0")"
PGBIN=${PGBIN:-/usr/lib/postgresql/16/bin}; PGDIR=${PGDIR:-/var/lib/postgresql/durable}; RUNAS=""
[ "$(id -u)" = 0 ] && RUNAS="runuser -u postgres --"
$RUNAS rm -rf "$PGDIR"; $RUNAS mkdir -p "$PGDIR"; $RUNAS "$PGBIN/initdb" -A trust -U postgres -D "$PGDIR/data" >/dev/null
$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -o "-p 54331 -k $PGDIR -c listen_addresses=''" -l "$PGDIR/pg.log" -w start >/dev/null
trap '$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -m fast stop >/dev/null' EXIT
P="psql -q -X -h $PGDIR -p 54331 -U postgres -d postgres -v ON_ERROR_STOP=1"
$P -f schema.sql
$P -At -f test.sql
