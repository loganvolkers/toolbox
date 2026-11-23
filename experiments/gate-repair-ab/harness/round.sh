#!/usr/bin/env bash
# Set up one round: three workspaces per arm under /tmp/gate-repair-ab/r<round>/<arm>-<n>, and the
# rendered prompts under prompts/rendered/r<round>/ (P1: commit them before the first launch).
#
#   round.sh <round> <openfga-src> <toolbox-checkout> <toolbox-target-dir>
set -euo pipefail
round=$1 src=$2 checkout=$3 target=$4
here=$(cd "$(dirname "$0")" && pwd)
runs=/tmp/gate-repair-ab/r$round
rendered="$here/../prompts/rendered/r$round"
mkdir -p "$rendered"
for arm in toolbox reviewdog; do
  for n in 1 2 3; do
    dest="$runs/$arm-$n"
    "$here/setup.sh" "$src" "$arm" "$dest" "$checkout" "$target" 2>/dev/null
    sed "s|{DEST}|$dest|g" "$here/../prompts/$arm.md" > "$rendered/$arm-$n.md"
  done
done
ls "$runs"
