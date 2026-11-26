#!/usr/bin/env bash
# Start one Seikika server per arm C run of a round, each on its own workspace and port
# (README §2), in the tmux session `gm-serve-<round>`, and wait until each serves every crate.
#
#   serve.sh <round>         start
#   serve.sh <round> stop    stop the session
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
RND=$1
WORK=${WORK:-/tmp/graphify-monorepo-ab}
SESSION="gm-serve-$RND"
TMUX_CMD=(tmux)
[[ -f /exec-daemon/tmux.portal.conf ]] && TMUX_CMD=(tmux -f /exec-daemon/tmux.portal.conf)
if [[ "${2:-}" == stop ]]; then
  "${TMUX_CMD[@]}" kill-session -t "$SESSION" 2>/dev/null || true
  exit 0
fi
"${TMUX_CMD[@]}" has-session -t "=$SESSION" 2>/dev/null || "${TMUX_CMD[@]}" new-session -d -s "$SESSION"
while IFS=$'\t' read -r run arm port; do
  [[ "$arm" == C ]] || continue
  dest="$WORK/r$RND/$run"
  "${TMUX_CMD[@]}" new-window -t "$SESSION" -n "$run" -c "$dest" \
    "seikika serve .seikika --listen 127.0.0.1:$port > $WORK/serve-r$RND-$run.log 2>&1"
done < "$HERE/../prompts/rendered/r$RND/arms.tsv"
while IFS=$'\t' read -r run arm port; do
  [[ "$arm" == C ]] || continue
  for _ in $(seq 1 120); do
    n=$(cd "$WORK/r$RND/$run" && SEIKIKA_URL="http://127.0.0.1:$port" SEIKIKA_AS=agent \
      seikika query --kind Crate --fields path -o json 2>/dev/null |
      python3 -c 'import json,sys; print(len(json.load(sys.stdin)["items"]))' 2>/dev/null || echo 0)
    [[ "$n" == 120 ]] && break
    sleep 2
  done
  echo "$run: port $port, $n crates"
done < "$HERE/../prompts/rendered/r$RND/arms.tsv"
