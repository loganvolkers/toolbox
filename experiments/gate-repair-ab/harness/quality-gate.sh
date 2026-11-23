#!/usr/bin/env bash
# Quality gate: zizmor and actionlint over .github/workflows/, failing only on findings this branch adds
# (reviewdog -filter-mode=added against the base branch). Exit 0 means pass.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
base="${GATE_BASE:-main}"
status=0
zizmor --offline --no-exit-codes --format github .github/workflows/ 2>/dev/null \
  | reviewdog -efm='::%trror file=%f,line=%l,title=%.%#::%m' -efm='::%tarning file=%f,line=%l,title=%.%#::%m' \
      -efm='::%totice file=%f,line=%l,title=%.%#::%m' \
      -name=zizmor -reporter=local -filter-mode=added -fail-level=any -diff="git diff --no-ext-diff $base" || status=1
actionlint -shellcheck= -pyflakes= -oneline \
  | reviewdog -efm='%f:%l:%c: %m' -name=actionlint -reporter=local -filter-mode=added -fail-level=any \
      -diff="git diff --no-ext-diff $base" || status=1
exit $status
