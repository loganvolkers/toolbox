#!/usr/bin/env bash
# Set up one round: $RUNS workspaces per arm (default 3) under /tmp/actions-gitlab/r<round>/<arm>-<n>,
# each with its own GitLab group, and the rendered prompts under prompts/rendered/r<round>/ (commit
# them before the first launch).
#
#   [RUNS=n] [ARMS="seikika gitlab"] [PORT_BASE=7300] round.sh <round> <src-dir>
#
# Run i of round r gets the group acme-w<10r+i>, its engineer ops-w<10r+i>, and port PORT_BASE+10i.
set -euo pipefail
round=$1 src=$2
runs_per_arm=${RUNS:-3} arms=${ARMS:-seikika gitlab}
here=$(cd "$(dirname "$0")" && pwd)
runs=/tmp/actions-gitlab/r$round
rendered="$here/../prompts/rendered/r$round"
mkdir -p "$rendered" "$runs"
: > "$rendered/arms.tsv"
i=0
for n in $(seq 1 "$runs_per_arm"); do
  for arm in $arms; do
    i=$((i + 1))
    dest="$runs/$arm-$n" port=$((${PORT_BASE:-7300} + 10 * i)) k=$((10 * round + i))
    "$here/setup.sh" "$arm" "$dest" "$src" "$k" >/dev/null
    python3 - "$here/../prompts" "$arm" "$dest" "$port" "r$round-$arm-$n" "acme-w$k" "ops-w$k" \
      > "$rendered/$arm-$n.md" <<'EOF'
import sys
d, arm, dest, port, run, group, ops = sys.argv[1:]
body = open(f"{d}/task.md").read().replace("{PLATFORM}", open(f"{d}/platform-{arm}.md").read().strip())
for k, v in {"{RUN}": run, "{DEST}": dest, "{PORT}": port, "{GROUP}": group, "{OPS}": ops}.items():
    body = body.replace(k, v)
print(body, end="")
EOF
    printf '%s\t%s\t%s\t%s\tacme-w%s\n' "$arm-$n" "$arm" "$dest" "$port" "$k" >> "$rendered/arms.tsv"
  done
done
cat "$rendered/arms.tsv"
