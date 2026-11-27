#!/usr/bin/env bats

setup() {
  report_action="$BATS_TEST_DIRNAME/../../actions/report/run.sh"
  mkdir -p "$BATS_TEST_TMPDIR/bin" "$BATS_TEST_TMPDIR/checkout/.seikika"
  touch "$BATS_TEST_TMPDIR/checkout/.seikika/seikika.yaml"
  export SEIKIKA_BIN="$BATS_TEST_TMPDIR/bin"
  export INPUT_WORKING_DIRECTORY="$BATS_TEST_TMPDIR/checkout"
  export INPUT_MODE=ci INPUT_HTML=false INPUT_SKIP=runner
  export INPUT_PORTAL=.seikika INPUT_BASELINE=
  export RUNNER_TEMP="$BATS_TEST_TMPDIR"
  export GITHUB_OUTPUT="$BATS_TEST_TMPDIR/outputs"
  export GITHUB_STEP_SUMMARY="$BATS_TEST_TMPDIR/summary"
  export REPORT_ARGS="$BATS_TEST_TMPDIR/args"

  cat >"$SEIKIKA_BIN/seikika" <<'MOCK'
#!/bin/sh
printf '%s\n' "$@" >"$REPORT_ARGS"
while [ "$#" -gt 0 ]; do
  if [ "$1" = --verdict ]; then
    printf '{\n  "result": "stale"\n}\n' >"$2"
    exit 1
  fi
  shift
done
exit 2
MOCK
  chmod +x "$SEIKIKA_BIN/seikika"
}

@test "the report action passes CI options and preserves a failing verdict for the final step" {
  run "$report_action"
  [ "$status" -eq 0 ]
  grep -Fx -- --ci "$REPORT_ARGS"
  grep -Fx -- runner "$REPORT_ARGS"
  grep -Fx -- .seikika/results.json "$REPORT_ARGS"
  grep -Fx 'result=stale' "$GITHUB_OUTPUT"
  grep -Fx 'exit-code=1' "$GITHUB_OUTPUT"
  grep -Fx '## Seikika report: stale' "$GITHUB_STEP_SUMMARY"
}
