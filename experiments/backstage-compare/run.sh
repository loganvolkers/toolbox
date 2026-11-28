#!/usr/bin/env bash
# The scored run (README.md): performance for both arms at each size, REPS runs per arm with the
# arms alternating, then the security and agent-surface probes, the footprint and the upgrade
# facts. Raw results go to $OUT; snapshot.py turns them into .seikika/comparisons/*.yaml.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
OUT=${OUT:-/tmp/bc/r}
CATALOGS=${CATALOGS:-/tmp/bc}
REPS=${REPS:-5}
SIZES=${SIZES:-"1000 10000"}
export CATALOGS
mkdir -p "$OUT"
for n in $SIZES; do
  [ -f "$CATALOGS/n$n/catalog.ndjson" ] || python3 "$here/gen.py" "$n" "$CATALOGS/n$n"
  for r in $(seq "$REPS"); do
    for arm in seikika backstage; do
      python3 "$here/measure.py" "$arm" "$CATALOGS/n$n" "$OUT/perf-$arm-$n-$r.json"
    done
  done
done
for arm in seikika backstage; do
  python3 "$here/surface.py" "$arm" "$CATALOGS/n1000" "$OUT/surface-$arm.json"
done
python3 "$here/footprint.py" "$OUT/footprint.json"
python3 "$here/upgrade.py" "$OUT/upgrade.json"
