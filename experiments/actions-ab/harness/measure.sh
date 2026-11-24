#!/usr/bin/env bash
# One run's measures, written to results/r<round>/<run>.*.json, with a one-line summary.
#
#   harness/measure.sh <round> <run> <transcript-dir>
#
# <transcript-dir> is the directory `batch-fetch-details` (with `includeTranscripts`) wrote: it
# holds <bcId>/transcript.json for the run's agent, whose bcId is in results/r<round>/runs.tsv.
set -euo pipefail
cd "$(dirname "$0")/.."
r=$1 run=$2 tdir=$3
agent=$(awk -v k="$run" '$1==k{print $2}' "results/r$r/runs.tsv")
t=$tdir/$agent/transcript.json
d=/tmp/actions-ab/r$r/$run o=results/r$r
python3 harness/score.py check "$d" > "$o/$run.check.json"
python3 harness/score.py lines "$d" > "$o/$run.lines.json"
python3 harness/count.py "$t" > "$o/$run.count.json"
python3 harness/isolation.py "$t" "$r" "$run" > "$o/$run.isolation.json"
python3 harness/prompt_check.py "$t" "prompts/rendered/r$r/$run.md" > "$o/$run.prompt.json"
cp "$d/org/HANDOFF.md" "$o/$run.HANDOFF.md"
python3 - "$o" "$run" <<'EOF'
import json, sys
o, run = sys.argv[1:]
j = lambda k: json.load(open(f"{o}/{run}.{k}.json"))
c, l, n, i, p = j("check"), j("lines"), j("count"), j("isolation"), j("prompt")
print(run, "prompt", p["identical"], "isolation", len(i["crossWorkspace"]), len(i["roundListing"]),
      "check", c["pass"], "lines", l["linesToMaintain"], "calls", n["toolCalls"], n["byKind"],
      "wall", n["wallSeconds"])
EOF
