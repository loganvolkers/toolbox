#!/usr/bin/env bash
# Quality gate: zizmor and actionlint over .github/workflows/, failing only on findings this branch adds
# (reviewdog -filter-mode=added against the base branch). Exit 0 means pass. Each tool's default
# format quotes the source lines of a finding, and reviewdog's local reporter prints them as is.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
base="${GATE_BASE:-main}"
status=0
zizmor --offline --no-exit-codes --format plain .github/workflows/ 2>/dev/null \
  | reviewdog -efm='%E%trror[%.%#]: %m' -efm='%W%tarning[%.%#]: %m' -efm='%I%tnfo[%.%#]: %m' \
      -efm='%Whelp[%.%#]: %m' -efm='%C%\s%#--> %f:%l:%c' -efm='%-G%\d%\+ finding%.%#' -efm='%C%.%#' \
      -name=zizmor -reporter=local -filter-mode=added -fail-level=any -diff="git diff --no-ext-diff $base" || status=1
actionlint -shellcheck= -pyflakes= \
  | reviewdog -efm='%A%f:%l:%c: %m' -efm='%C%.%#' -name=actionlint -reporter=local -filter-mode=added -fail-level=any \
      -diff="git diff --no-ext-diff $base" || status=1
[ "$status" -eq 0 ] && echo "quality gate: passed"
exit $status
