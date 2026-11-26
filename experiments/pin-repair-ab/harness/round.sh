#!/usr/bin/env bash
# Set up one round: $RUNS workspaces per arm (default 3) under /tmp/pin-repair-ab/r<round>/<arm>-<n>,
# and the rendered prompts under prompts/rendered/r<round>/ (P1: commit them before the first launch).
#
#   [RUNS=n] [PREV_TARGET=dir [PREV_CHECKOUT=dir]] round.sh <round> <openfga-src> <seikika-checkout> <seikika-target-dir>
#
# With PREV_TARGET, a third arm, `seikika-prev`, runs the Seikika binaries in that directory, and
# vendors packages/github-actions from PREV_CHECKOUT (default: <seikika-checkout>). Its
# workspaces are numbered on from arm A's (seikika-<RUNS+1> …), so a workspace's name doesn't say
# which Seikika it has; prompts/rendered/r<round>/arms.tsv maps each run to its arm.
set -euo pipefail
round=$1 src=$2 checkout=$3 target=$4
runs_per_arm=${RUNS:-3}
here=$(cd "$(dirname "$0")" && pwd)
runs=/tmp/pin-repair-ab/r$round
rendered="$here/../prompts/rendered/r$round"
mkdir -p "$rendered"
: > "$rendered/arms.tsv"
one() { # <arm> <kind> <run> <target> [checkout]
  local dest="$runs/$3"
  "$here/setup.sh" "$src" "$2" "$dest" "${5:-$checkout}" "$4"
  sed "s|{DEST}|$dest|g" "$here/../prompts/$2.md" > "$rendered/$3.md"
  printf '%s\t%s\n' "$3" "$1" >> "$rendered/arms.tsv"
}
for n in $(seq 1 "$runs_per_arm"); do
  one seikika seikika "seikika-$n" "$target"
  one reviewdog reviewdog "reviewdog-$n" "$target"
  if [ -n "${PREV_TARGET:-}" ]; then
    one seikika-prev seikika "seikika-$((runs_per_arm + n))" "$PREV_TARGET" "${PREV_CHECKOUT:-$checkout}"
  fi
done
ls "$runs"
