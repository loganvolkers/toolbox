#!/usr/bin/env bash
# Prepare /tmp/connector-repair-ab from broken scaffolds. Reuses connector-ab Forge + pins.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
REPAIR_HOME="${REPAIR_HOME:-/tmp/connector-repair-ab}"
AB_HOME="${AB_HOME:-/tmp/connector-ab}"

mkdir -p "$REPAIR_HOME"/{arms,logs,bin,snapshots}
ln -sfn "$ROOT/experiments/connector-ab/fixtures" "$REPAIR_HOME/fixtures"
ln -sfn "$AB_HOME/pins" "$REPAIR_HOME/pins"
if [[ -x "$ROOT/target/debug/toolbox" ]]; then
  ln -sfn "$ROOT/target/debug/toolbox" "$REPAIR_HOME/bin/toolbox"
elif [[ -x "$AB_HOME/bin/toolbox" ]]; then
  ln -sfn "$AB_HOME/bin/toolbox" "$REPAIR_HOME/bin/toolbox"
fi

for arm in A B C; do
  dest="$REPAIR_HOME/arms/$arm"
  rm -rf "$dest"
  mkdir -p "$dest/OUT"
  case "$arm" in
    A)
      cp -a "$ROOT/experiments/connector-repair-ab/scaffolds/toolbox/." "$dest/"
      # Cargo.toml already points at /agent/repos/toolbox; keep pin.
      echo "export PATH=$REPAIR_HOME/bin:/usr/local/cargo/bin:\$PATH" > "$dest/env.sh"
      ;;
    B)
      cp -a "$ROOT/experiments/connector-repair-ab/scaffolds/ocean/." "$dest/"
      echo "$AB_HOME/pins/ocean" > "$dest/OCEAN_ROOT.txt"
      ;;
    C)
      cp -a "$ROOT/experiments/connector-repair-ab/scaffolds/backstage/." "$dest/"
      echo "$AB_HOME/pins/backstage" > "$dest/BACKSTAGE_ROOT.txt"
      ;;
  esac
done

# Snapshot sources so phase 1 can assert no edits.
python3 -I "$ROOT/experiments/connector-repair-ab/harness/snapshot.py" write "$REPAIR_HOME"

echo "REPAIR_HOME=$REPAIR_HOME ready"
cat "$AB_HOME/pins/PINS.txt" 2>/dev/null || true
