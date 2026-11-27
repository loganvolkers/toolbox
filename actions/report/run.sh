#!/usr/bin/env bash
# The Seikika report action's main step (action.yml). Runs `seikika report`, writes the job summary
# and the step outputs, and never fails itself: the action's last step fails on the exit code, so
# the report is uploaded first. Runnable locally with the same environment variables.
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../lib/portal.sh"

portal=${INPUT_PORTAL:-.seikika}
baseline=${INPUT_BASELINE:-}
mode=${INPUT_MODE:-ci}
summary=${GITHUB_STEP_SUMMARY:-/dev/null}
outputs=${GITHUB_OUTPUT:-/dev/null}

report_error() {
  printf 'result=error\nexit-code=2\n' >>"$outputs"
}

if [[ -n "${SEIKIKA_BIN:-}" ]]; then
  if ! bin=$(cd "$SEIKIKA_BIN" && pwd -P) || [[ ! -x "$bin/seikika" ]]; then
    echo '::error title=Seikika report::The binary directory must contain an executable named seikika.'
    report_error
    exit 0
  fi
  executable="$bin/seikika"
  export SEIKIKA_CONNECTORS_DIR="$bin"
elif executable=$(type -P seikika); then
  executable=$(realpath "$executable")
else
  echo '::error title=Seikika report::Run actions/setup or put seikika on PATH first.'
  report_error
  exit 0
fi
out=$(mktemp -d "${RUNNER_TEMP:-/tmp}/seikika-report.XXXXXX") || {
  report_error
  exit 0
}

[[ -z "$baseline" ]] && baseline="$portal/results.json"
verdict="$out/verdict.json"
args=(report "$portal" -o md --out "$out/report.md" --verdict "$verdict")
[[ -n "${INPUT_SKIP:-}" ]] && args+=(--skip "$INPUT_SKIP")
[[ "${INPUT_HTML:-true}" == "true" ]] && args+=(--html "$out/report.html")
case "$mode" in
  ci) args+=(--baseline "$baseline" --ci) ;;
  check) args+=(--baseline "$baseline") ;;
  update) args+=(--baseline "$baseline" --update) ;;
  report) ;;
  *)
    echo "::error title=Seikika report::unknown mode '$mode' (ci, check, update or report)"
    report_error
    exit 0
    ;;
esac

if ! prepare_portal 'Seikika report' "$executable" "${INPUT_WORKING_DIRECTORY:-.}" "$portal"; then
  report_error
  exit 0
fi

"$executable" "${args[@]}" 2>"$out/ratchet.txt"
code=$?
cat "$out/ratchet.txt" >&2

# The verdict file is the contract (D96): result, exit code, and each change as data.
result=$(sed -n 's/^ *"result": "\([a-z-]*\)".*/\1/p' "$verdict" 2>/dev/null | head -1)
[[ -z "$result" ]] && result=error
# One annotation per regression, on the run and the pull request (needs jq, as GitHub's runners have),
# with its root cause: the evidence, how to reproduce it, and for a repository level the scorecard
# that fell and the change below it (docs/design/root-causes.md).
if [[ "$result" == regressed ]] && command -v jq >/dev/null; then
  jq -r 'def lines: "\(.scorecard) · \(.entity): \(.message)", ((.detail // [])[] | "  " + .), ((.causes // [])[] | lines | "  " + .);
    .regressions[] | "::error title=Ratchet: \(.scorecard)::" + ([lines] | join("\n") | gsub("%"; "%25") | gsub("\n"; "%0A"))' "$verdict"
fi

{
  echo "result=$result"
  echo "exit-code=$code"
  [[ -f "$out/report.md" ]] && echo "report=$out/report.md"
  [[ -f "$out/report.html" ]] && echo "html=$out/report.html"
  [[ -f "$verdict" ]] && echo "verdict=$verdict"
  echo "baseline=$(realpath -m "$baseline")"
} >>"$outputs"

{
  echo "## Seikika report: $result"
  echo
  case "$result" in
    pass)
      if [[ "$mode" == report ]]; then
        echo "Scored the portal without a baseline."
      else
        echo "Nothing got worse than \`$baseline\`."
      fi
      ;;
    regressed) echo "Something got worse than \`$baseline\`. Fix it, or (if the change is intended) update the baseline in this PR: \`seikika report $portal --baseline $baseline --update\`." ;;
    unknown) echo "Some rules couldn't be evaluated (a tool or its data is missing), so the ratchet can't compare them; install the tools the portal declares (\`seikika doctor $portal\`)." ;;
    stale) echo "Nothing got worse, but \`$baseline\` doesn't record the improvements yet: update it in this PR (\`seikika report $portal --baseline $baseline --update\`, or run this action with \`mode: update\` and commit the uploaded file)." ;;
    missing-baseline) echo "There is no \`$baseline\` yet: run this action once with \`mode: update\` and commit the \`results.json\` it uploads, or run \`seikika report $portal --baseline $baseline --update\` locally." ;;
    error) echo "\`seikika report\` failed (exit $code); its output is below." ;;
  esac
  if [[ -s "$out/ratchet.txt" ]]; then
    echo
    echo '```'
    cat "$out/ratchet.txt"
    echo '```'
  fi
  if [[ -f "$out/report.md" ]]; then
    echo
    echo "<details><summary>Full report</summary>"
    echo
    cat "$out/report.md"
    echo
    echo "</details>"
  fi
} >>"$summary"
exit 0
