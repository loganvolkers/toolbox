#!/usr/bin/env bash
# The Toolbox report action's main step (action.yml). Runs `toolbox report`, writes the job summary
# and the step outputs, and never fails itself: the action's last step fails on the exit code, so
# the report is uploaded first. Runnable locally with the same environment variables.
set -uo pipefail

bin=${TOOLBOX_BIN:?TOOLBOX_BIN: directory with the toolbox binaries}
bin=$(cd "$bin" && pwd)                       # absolute, before changing directory
cd "${INPUT_WORKING_DIRECTORY:-.}" || { echo "::error title=Toolbox report::no directory ${INPUT_WORKING_DIRECTORY}"; exit 0; }
portal=${INPUT_PORTAL:-.toolbox}
baseline=${INPUT_BASELINE:-}
mode=${INPUT_MODE:-ci}
out=${RUNNER_TEMP:-/tmp}/toolbox-report
summary=${GITHUB_STEP_SUMMARY:-/dev/null}
outputs=${GITHUB_OUTPUT:-/dev/null}
mkdir -p "$out"

[[ -z "$baseline" ]] && baseline="$portal/results.json"
verdict="$out/verdict.json"
rm -f "$verdict"
args=(report "$portal" -o md --out "$out/report.md" --verdict "$verdict")
[[ -n "${INPUT_SKIP:-}" ]] && args+=(--skip "$INPUT_SKIP")
[[ "${INPUT_HTML:-true}" == "true" ]] && args+=(--html "$out/report.html")
case "$mode" in
  ci) args+=(--baseline "$baseline" --ci) ;;
  check) args+=(--baseline "$baseline") ;;
  update) args+=(--baseline "$baseline" --update) ;;
  report) ;;
  *) echo "::error title=Toolbox report::unknown mode '$mode' (ci, check, update or report)"
     echo "result=error" >> "$outputs"; echo "exit-code=2" >> "$outputs"; exit 0 ;;
esac

if [[ ! -f "$portal/toolbox.yaml" ]]; then
  echo "::error title=Toolbox report::no $portal/toolbox.yaml; copy examples/repo-portal-starter/toolbox.yaml from the toolbox repository"
  echo "result=error" >> "$outputs"; echo "exit-code=2" >> "$outputs"; exit 0
fi

# Connectors are found next to the toolbox binary (TOOLBOX_CONNECTORS_DIR for good measure).
TOOLBOX_CONNECTORS_DIR="$bin" "$bin/toolbox" "${args[@]}" 2> "$out/ratchet.txt"
code=$?
cat "$out/ratchet.txt" >&2

# The verdict file is the contract (D96): result, exit code, and each change as data.
result=$(sed -n 's/^ *"result": "\([a-z-]*\)".*/\1/p' "$verdict" 2>/dev/null | head -1)
[[ -z "$result" ]] && result=error
# One annotation per regression, on the run and the pull request (needs jq, as GitHub's runners have).
if [[ "$result" == regressed ]] && command -v jq >/dev/null; then
  jq -r '.regressions[] | "::error title=Ratchet: \(.scorecard)::\(.entity): \(.message | gsub("%"; "%25") | gsub("\n"; "%0A"))"' "$verdict"
fi

{
  echo "result=$result"
  echo "exit-code=$code"
  [[ -f "$out/report.md" ]] && echo "report=$out/report.md"
  [[ -f "$out/report.html" ]] && echo "html=$out/report.html"
  [[ -f "$verdict" ]] && echo "verdict=$verdict"
  echo "baseline=$(realpath -m "$baseline")"
} >> "$outputs"

{
  echo "## Toolbox report: $result"
  echo
  case "$result" in
    pass) echo "Nothing got worse than \`$baseline\`." ;;
    regressed) echo "Something got worse than \`$baseline\`. Fix it, or (if the change is intended) update the baseline in this PR: \`toolbox report $portal --baseline $baseline --update\`." ;;
    unknown) echo "Some rules couldn't be evaluated (a tool or its data is missing), so the ratchet can't compare them; install the tools the portal declares (\`toolbox doctor $portal\`)." ;;
    stale) echo "Nothing got worse, but \`$baseline\` doesn't record the improvements yet: update it in this PR (\`toolbox report $portal --baseline $baseline --update\`, or run this action with \`mode: update\` and commit the uploaded file)." ;;
    missing-baseline) echo "There is no \`$baseline\` yet: run this action once with \`mode: update\` and commit the \`results.json\` it uploads, or run \`toolbox report $portal --baseline $baseline --update\` locally." ;;
    error) echo "\`toolbox report\` failed (exit $code); its output is below." ;;
  esac
  if [[ -s "$out/ratchet.txt" ]]; then
    echo; echo '```'; cat "$out/ratchet.txt"; echo '```'
  fi
  if [[ -f "$out/report.md" ]]; then
    echo; echo "<details><summary>Full report</summary>"; echo; cat "$out/report.md"; echo; echo "</details>"
  fi
} >> "$summary"
exit 0
