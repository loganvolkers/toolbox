#!/usr/bin/env bash
# Regenerate npm/site/.generated the same way .github/workflows/pages.yml does.
#
# Pages CI installs actionlint 1.7.12, osv-scanner 2.7.1 and the pinned RustSec database
# (scripts/ci-tools.sh), then zizmor and ruff from the portal's
# pins (seikika tools install),
# runs seikika doctor, then:
#   seikika report .seikika --skip runner,github-actions --baseline .seikika/results.json -o web --out npm/site/.generated
# Without those pins, ci-lint rules become `unknown` and the preview diverges from CI
# (D01M7YAXH89ZJQ7Q4HP77NMHBX8). Use this script instead of a bare report command.
#
# Usage: scripts/regen-site-report.sh
set -euo pipefail
cd "$(dirname "$0")/.."

TOOLS="${SEIKIKA_CI_TOOLS:-${TMPDIR:-/tmp}/seikika-ci-tools}"
scripts/ci-tools.sh "$TOOLS" actionlint scorecard osv-scanner rustsec-osv shellcheck shfmt bats
export PATH="$TOOLS:$PATH" OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY="$TOOLS"

pnpm install --frozen-lockfile
pnpm build
cargo build --locked
# zizmor and ruff from the portal's own pins (tools:), with uv, pipx or pip: whichever is here.
export PATH="${HOME}/.local/bin:$PATH"
target/debug/seikika tools install .seikika

actionlint -version
zizmor --version
ruff --version
osv-scanner --version

target/debug/seikika doctor .seikika
target/debug/seikika report .seikika --skip runner,github-actions --baseline .seikika/results.json -o web --out npm/site/.generated
echo "wrote npm/site/.generated/report.json for the Astro report integration"
