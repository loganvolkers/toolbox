#!/usr/bin/env bash
# Loads the same synthetic world as the Rust run into a throwaway local Postgres and runs sql/bench.sql.
# Usage: CSV_DIR=/scratch/csv PGDIR=/dir/writable/by/postgres ./run-postgres.sh [entities]
# Needs Postgres 16 binaries. As root, the server runs as the `postgres` user; psql runs as the caller.
set -euo pipefail
cd "$(dirname "$0")"
N=${1:-100000}; CSV=${CSV_DIR:-$PWD/target/csv}; PGDIR=${PGDIR:-/var/lib/postgresql/dss}
PGBIN=${PGBIN:-/usr/lib/postgresql/16/bin}; RUNAS=""
[ "$(id -u)" = 0 ] && RUNAS="runuser -u postgres --"
cargo run --release -q -- "$N" 4 --csv "$CSV"
$RUNAS rm -rf "$PGDIR"; $RUNAS mkdir -p "$PGDIR"; $RUNAS "$PGBIN/initdb" -A trust -U postgres -D "$PGDIR/data" >/dev/null
$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -o "-p 54329 -k $PGDIR -c listen_addresses='' -c shared_buffers=2GB -c work_mem=256MB -c maintenance_work_mem=1GB -c max_parallel_workers_per_gather=4" -l "$PGDIR/pg.log" -w start >/dev/null
trap '$RUNAS "$PGBIN/pg_ctl" -D "$PGDIR/data" -m fast stop >/dev/null' EXIT
P="psql -q -X -h $PGDIR -p 54329 -U postgres -d postgres -v ON_ERROR_STOP=1"
(cd "$CSV" && $P -f "$OLDPWD/sql/schema.sql")
$P -f sql/pipeline.sql
$P -At -c "SELECT version(); SHOW jit;"
$P -v id="${ID:-0}" -f sql/bench.sql
