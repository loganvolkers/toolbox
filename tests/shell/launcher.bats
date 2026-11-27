#!/usr/bin/env bats

setup() {
  launcher="$BATS_TEST_DIRNAME/../../plugins/seikika-operator/bin/seikika-mcp"
  unset SEIKIKA_BIN
}

@test "the MCP launcher forwards a data directory containing spaces" {
  export SEIKIKA_BIN="$BATS_TEST_TMPDIR/seikika mock"
  cat >"$SEIKIKA_BIN" <<'MOCK'
#!/bin/sh
printf '<%s>\n' "$@"
exit 7
MOCK
  chmod +x "$SEIKIKA_BIN"

  run "$launcher" --ignored --data "$BATS_TEST_TMPDIR/plugin data"
  [ "$status" -eq 7 ]
  [ "${lines[0]}" = '<mcp>' ]
  [ "${lines[1]}" = '<--dev>' ]
  [ "${lines[2]}" = '<--data>' ]
  [ "${lines[3]}" = "<$BATS_TEST_TMPDIR/plugin data>" ]
  [ "${#lines[@]}" -eq 4 ]
}

@test "the MCP launcher diagnoses a missing binary" {
  bats_require_minimum_version 1.5.0
  run --separate-stderr "$launcher"
  [ "$status" -eq 1 ]
  [ -z "$output" ]
  [[ "$stderr" == *"the Seikika binary is not available"* ]]
}
