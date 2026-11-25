#!/usr/bin/env bash
# Regenerate site/public/report the same way .github/workflows/pages.yml does.
#
# Pages CI installs actionlint 1.7.12, osv-scanner 2.7.1 and the pinned RustSec database
# (scripts/ci-tools.sh), then zizmor and ruff from the portal's
# pins (toolbox tools install),
# runs toolbox doctor, then:
#   toolbox report portal --skip runner,github-actions --baseline portal/results.json -o web --out site/public/report
# Without those pins, ci-lint rules become `unknown` and the preview diverges from CI
# (D01M7YAXH89ZJQ7Q4HP77NMHBX8). Use this script instead of a bare report command.
#
# Usage: scripts/regen-site-report.sh
set -euo pipefail
cd "$(dirname "$0")/.."

TOOLS="${TOOLBOX_CI_TOOLS:-${TMPDIR:-/tmp}/toolbox-ci-tools}"
scripts/ci-tools.sh "$TOOLS" actionlint scorecard osv-scanner rustsec-osv
export PATH="$TOOLS:$PATH" OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY="$TOOLS"

cargo build --locked
# zizmor and ruff from the portal's own pins (tools:), with uv, pipx or pip: whichever is here.
export PATH="${HOME}/.local/bin:$PATH"
target/debug/toolbox tools install portal

actionlint -version
zizmor --version
ruff --version
osv-scanner --version

target/debug/toolbox doctor portal
target/debug/toolbox report portal --skip runner,github-actions --baseline portal/results.json -o web --out site/public/report
echo "wrote site/public/report (preview only; not committed — pages CI regenerates it)"
