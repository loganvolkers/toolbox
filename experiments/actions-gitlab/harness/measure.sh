#!/usr/bin/env bash
# One run's measures, written to results/r<round>/<run>.*.json, with a one-line summary. Run it as
# soon as the run ends: the probes change GitLab.
#
#   harness/measure.sh <round> <run> <transcript-dir>
#
# <transcript-dir> is the directory `batch-fetch-details` (with `includeTranscripts`) wrote: it
# holds <bcId>/transcript.json for the run's agent, whose bcId is in results/r<round>/runs.tsv.
set -euo pipefail
cd "$(dirname "$0")/.."
ab=../actions-ab/harness
r=$1 run=$2 tdir=$3
agent=$(awk -v k="$run" '$1==k{print $2}' "results/r$r/runs.tsv")
t=$tdir/$agent/transcript.json
d=/tmp/actions-gitlab/r$r/$run o=results/r$r
python3 harness/score.py check "$d" > "$o/$run.check.json"
python3 harness/score.py lines "$d" > "$o/$run.lines.json"
python3 harness/settings.py diff "$d" > "$o/$run.settings.json"
python3 $ab/count.py "$t" > "$o/$run.count.json"
python3 harness/isolation.py "$t" "$r" "$run" "$(cat "$d.group")" > "$o/$run.isolation.json"
python3 $ab/prompt_check.py "$t" "prompts/rendered/r$r/$run.md" > "$o/$run.prompt.json"
cp "$d.remote/org/HANDOFF.md" "$o/$run.HANDOFF.md" 2>/dev/null || echo "no HANDOFF.md on org's main" >&2
python3 - "$o" "$run" <<'EOF'
import json, sys
o, run = sys.argv[1:]
j = lambda k: json.load(open(f"{o}/{run}.{k}.json"))
c, l, n, i, p, s = j("check"), j("lines"), j("count"), j("isolation"), j("prompt"), j("settings")
print(run, "prompt", p["identical"], "isolation", {k: len(v) for k, v in i.items()},
      "check", c["pass"], "lines", l["linesToMaintain"], "settings", s["changed"],
      "calls", n["toolCalls"], n["byKind"], "wall", n["wallSeconds"])
EOF
