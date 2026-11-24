#!/usr/bin/env bash
# Set up one round: $RUNS workspaces per arm (default 3) under /tmp/actions-ab/r<round>/<arm>-<n>,
# and the rendered prompts under prompts/rendered/r<round>/ (P1: commit them before the first
# launch). Each run gets its own ports, so a batch can run every arm at once.
#
#   [RUNS=n] [ARMS="toolbox gha backstage vanilla"] [PORT_BASE=7100] round.sh <round> <src-dir>
set -euo pipefail
round=$1 src=$2
runs_per_arm=${RUNS:-3}
arms=${ARMS:-toolbox gha backstage vanilla}
here=$(cd "$(dirname "$0")" && pwd)
runs=${RUNS_DIR:-/tmp/actions-ab}/r$round
rendered="$here/../prompts/rendered/r$round"
mkdir -p "$rendered" "$runs"
: > "$rendered/arms.tsv"
i=0
for n in $(seq 1 "$runs_per_arm"); do
  for arm in $arms; do
    i=$((i + 1))
    dest="$runs/$arm-$n" port=$((${PORT_BASE:-7100} + 10 * i))
    "$here/setup.sh" "$arm" "$dest" "$src" "$port" >/dev/null
    python3 - "$here/../prompts" "$arm" "$dest" "$port" "r$round-$arm-$n" > "$rendered/$arm-$n.md" <<'EOF'
import sys
d, arm, dest, port, run = sys.argv[1:]
body = open(f"{d}/task.md").read().replace("{PLATFORM}", open(f"{d}/platform-{arm}.md").read().strip())
print(body.replace("{RUN}", run).replace("{DEST}", dest).replace("{PORT2}", str(int(port) + 1)).replace("{PORT}", port), end="")
EOF
    printf '%s\t%s\t%s\t%s\n' "$arm-$n" "$arm" "$dest" "$port" >> "$rendered/arms.tsv"
  done
done
cat "$rendered/arms.tsv"
