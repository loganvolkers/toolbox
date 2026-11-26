#!/usr/bin/env bash
# Replay a repository's recent history through the ratchet (docs/design/repo-value.md R4): check out
# each of the last N first-parent commits, oldest first, and run `seikika report --baseline` against
# the previous commit's results, as CI would have on that change. Upstream merged every commit, so
# after recording the verdict the baseline is rebuilt from that commit and the replay moves on.
#
# Usage: experiments/out-of-box/replay.sh <checkout> <portal dir> <commits> <out.tsv>
# Needs: a checkout with that much history (git fetch --depth=<commits>), seikika built
# (target/debug), and zizmor and actionlint on the PATH. Prints one TSV row per commit:
# commit, date, result, regressions, improvements, subject, and the first regression.
set -euo pipefail
repo=$(cd "$1" && pwd); portal=$(cd "$2" && pwd); n=$3; out=$4
here=$(cd "$(dirname "$0")/../.." && pwd)
seikika=${SEIKIKA:-$here/target/debug/seikika}
work=$(mktemp -d); tree="$work/tree"; base="$work/results.json"; verdict="$work/verdict.json"
trap 'git -C "$repo" worktree remove --force "$tree" >/dev/null 2>&1 || true; rm -rf "$work"' EXIT

mapfile -t commits < <(git -C "$repo" rev-list --first-parent --max-count="$n" HEAD | tac)
git -C "$repo" worktree add -q --detach "$tree" "${commits[0]}"
report() { (cd "$tree" && "$seikika" report "$portal" --skip github-actions -o json --out /dev/null "$@" 2>/dev/null) || true; }

printf 'commit\tdate\tresult\tregressions\timprovements\tsubject\tfirst regression\n' > "$out"
report --baseline "$base" --update
for c in "${commits[@]:1}"; do
  git -C "$tree" checkout -q --detach "$c"
  rm -f "$verdict"
  report --baseline "$base" --verdict "$verdict"
  python3 - "$verdict" "$c" "$tree" >> "$out" <<'PY'
import json, subprocess, sys
v = json.load(open(sys.argv[1]))
meta = subprocess.run(["git", "-C", sys.argv[3], "log", "-1", "--format=%ad\t%s", "--date=short", sys.argv[2]],
                      capture_output=True, text=True).stdout.strip()
date, subject = meta.split("\t", 1)
first = v["regressions"][0] if v.get("regressions") else None
first = f"{first['scorecard']} · {first['entity']}: {first['message']}" if first else ""
print("\t".join([sys.argv[2][:9], date, v["result"], str(len(v.get("regressions", []))),
                 str(len(v.get("improvements", []))), subject[:80].replace("\t", " "), first[:160]]))
PY
  rm -f "$base"
  report --baseline "$base" --update
done
echo "replayed $(( ${#commits[@]} - 1 )) commits of $repo into $out" >&2
