#!/usr/bin/env bash
# Regenerate report-ui/fixtures: the report data of the reference portals, rendered by the
# seikika binary built from this checkout. Stories render these; nothing in them is hand-made.
#
#   s4s.json                 Seikika for Seikika (portal/), as pages CI renders it
#   s4s-assessments.json     a subset of that report for the assessment reader tasks
#   openfga.json             examples/openfga-portal over OpenFGA v1.23.0
#   openfga-unmeasured.json  the same with the comparison records left out (--skip portal)
#   starter.json             examples/repo-portal-starter over this repository
#   starter-empty.json       the starter with no connector run: a portal before its first data
#
# Installs the tools `seikika doctor` lists for portal/ and examples/openfga-portal at their pins:
# actionlint, scorecard and staticcheck as SHA-256-pinned release binaries (scripts/ci-tools.sh),
# zizmor and ruff from portal/'s `tools:` (seikika tools install). It stops if one is still
# missing, since rules would read `unknown` and drift from CI. Linux x86_64, like CI.
set -euo pipefail
cd "$(dirname "$0")/../.."
pnpm build
cargo build --locked -q
seikika_bin="$PWD/target/debug/seikika"
out="$PWD/report-ui/fixtures"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"

TOOLS="${SEIKIKA_CI_TOOLS:-${TMPDIR:-/tmp}/seikika-ci-tools}"
scripts/ci-tools.sh "$TOOLS" actionlint scorecard staticcheck >/dev/null
export PATH="$TOOLS:${HOME}/.local/bin:$PATH"
"$seikika_bin" tools install portal >/dev/null

examples/openfga-portal/fetch.sh >/dev/null
"$seikika_bin" doctor portal >/dev/null
(cd examples/openfga-portal/checkout && "$seikika_bin" doctor .. >/dev/null)

# With each portal's baseline, as pages CI renders it, so campaigns show progress since their start.
"$seikika_bin" report portal --skip runner,github-actions --baseline portal/results.json -o web --out "$tmp/s4s" >/dev/null
(
  cd examples/openfga-portal/checkout
  "$seikika_bin" report .. --skip github-actions --baseline ../results.json -o web --out "$tmp/openfga" 2>/dev/null
  "$seikika_bin" report .. --skip github-actions,portal -o web --out "$tmp/openfga-unmeasured" 2>/dev/null
)
"$seikika_bin" report examples/repo-portal-starter -o web --out "$tmp/starter" >/dev/null
"$seikika_bin" report examples/repo-portal-starter --skip workspace -o web --out "$tmp/starter-empty" >/dev/null

for name in s4s openfga openfga-unmeasured starter starter-empty; do
  python3 -c 'import json,sys; json.dump(json.load(open(sys.argv[1])), open(sys.argv[2], "w"), sort_keys=True, separators=(",", ":"))' \
    "$tmp/$name/report.json" "$out/$name.json"
  echo "$out/$name.json"
done
node report-ui/scripts/assessment-fixture.mjs "$tmp/s4s/report.json" > "$out/s4s-assessments.json"
