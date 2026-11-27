#!/usr/bin/env bats

setup() {
  adapter="$BATS_TEST_DIRNAME/../../packages/shell/check-shell.py"
  cd "$BATS_TEST_TMPDIR" || return
  git init -q
}

@test "the shell adapter reports each tool and preserves paths with spaces" {
  printf '#!/bin/sh\nprintf "%%s\\n" "ok"\n' >"clean script.sh"
  printf '#!/bin/bash\nf() {\n    local value=$(false)\n    printf "%%s\\n" "$value"\n}\nf\n' >"with findings.sh"

  python3 "$adapter" '*.sh' >records.ndjson
  python3 - <<'PY'
import json
from pathlib import Path

records = [json.loads(line) for line in Path("records.ndjson").read_text().splitlines()]
clean, changed = records
assert clean["externalId"] == "file:clean script.sh"
assert clean["data"]["findings"] == []
assert clean["data"]["formatted"] is True
assert changed["externalId"] == "file:with findings.sh"
assert any("SC2155" in finding for finding in changed["data"]["findings"])
assert changed["data"]["formatted"] is False
PY
  grep -Fx '    local value=$(false)' "with findings.sh"
}

@test "the shell adapter fails when no scripts match" {
  run python3 "$adapter" '*.sh'
  [ "$status" -eq 1 ]
  [[ "$output" == *"no shell sources match"* ]]
}
