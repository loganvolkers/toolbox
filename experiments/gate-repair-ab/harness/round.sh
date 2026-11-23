#!/usr/bin/env bash
# Set up one round: $RUNS workspaces per arm (default 3) under /tmp/gate-repair-ab/r<round>/<arm>-<n>,
# and the rendered prompts under prompts/rendered/r<round>/ (P1: commit them before the first launch).
#
#   [RUNS=n] [PREV_TARGET=dir] round.sh <round> <openfga-src> <toolbox-checkout> <toolbox-target-dir>
#
# With PREV_TARGET, a third arm, `toolbox-prev`, runs the Toolbox binaries in that directory. Its
# workspaces are numbered on from arm A's (toolbox-<RUNS+1> …), so a workspace's name doesn't say
# which Toolbox it has; prompts/rendered/r<round>/arms.tsv maps each run to its arm.
set -euo pipefail
round=$1 src=$2 checkout=$3 target=$4
runs_per_arm=${RUNS:-3}
here=$(cd "$(dirname "$0")" && pwd)
runs=/tmp/gate-repair-ab/r$round
rendered="$here/../prompts/rendered/r$round"
mkdir -p "$rendered"
: > "$rendered/arms.tsv"
one() { # <arm> <kind> <run> <target>
  local dest="$runs/$3"
  "$here/setup.sh" "$src" "$2" "$dest" "$checkout" "$4" 2>/dev/null
  sed "s|{DEST}|$dest|g" "$here/../prompts/$2.md" > "$rendered/$3.md"
  printf '%s\t%s\n' "$3" "$1" >> "$rendered/arms.tsv"
}
for n in $(seq 1 "$runs_per_arm"); do
  one toolbox toolbox "toolbox-$n" "$target"
  one reviewdog reviewdog "reviewdog-$n" "$target"
  if [ -n "${PREV_TARGET:-}" ]; then
    one toolbox-prev toolbox "toolbox-$((runs_per_arm + n))" "$PREV_TARGET"
  fi
done
ls "$runs"
