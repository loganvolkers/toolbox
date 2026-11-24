#!/usr/bin/env bash
# Regenerate report-ui/fixtures: the report data of the reference portals, rendered by the
# toolbox binary built from this checkout. Stories render these; nothing in them is hand-made.
#
#   t4t.json                 Toolbox for Toolbox (portal/), as pages CI renders it
#   openfga.json             examples/openfga-portal over OpenFGA v1.23.0
#   openfga-unmeasured.json  the same with the comparison records left out (--skip portal)
#   starter.json             examples/repo-portal-starter over this repository
#   starter-empty.json       the starter with no connector run: a portal before its first data
#
# Needs the tools `toolbox doctor` lists for portal/ and examples/openfga-portal (actionlint,
# zizmor, ruff, staticcheck); without them, rules read `unknown` and the fixtures drift from CI.
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build --locked -q
tb="$PWD/target/debug/toolbox"
out="$PWD/report-ui/fixtures"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
mkdir -p "$out"

"$tb" report portal --skip runner,github-actions -o web --out "$tmp/t4t" >/dev/null
examples/openfga-portal/fetch.sh >/dev/null
(
  cd examples/openfga-portal/checkout
  "$tb" report .. --skip github-actions -o web --out "$tmp/openfga" 2>/dev/null
  "$tb" report .. --skip github-actions,portal -o web --out "$tmp/openfga-unmeasured" 2>/dev/null
)
"$tb" report examples/repo-portal-starter -o web --out "$tmp/starter" >/dev/null
"$tb" report examples/repo-portal-starter --skip workspace -o web --out "$tmp/starter-empty" >/dev/null

for name in t4t openfga openfga-unmeasured starter starter-empty; do
  python3 -c 'import json,sys; json.dump(json.load(open(sys.argv[1])), open(sys.argv[2], "w"), sort_keys=True, separators=(",", ":"))' \
    "$tmp/$name/report.json" "$out/$name.json"
  echo "$out/$name.json"
done
