#!/usr/bin/env bash
# Regenerate site/public/report the same way .github/workflows/pages.yml does.
#
# Pages CI installs actionlint 1.7.12 (scripts/ci-tools.sh), zizmor==1.32.0 and ruff==0.14.0,
# runs toolbox doctor, then:
#   toolbox report portal --skip runner,github-actions -o web --out site/public/report
# Without those pins, ci-lint rules become `unknown` and the preview diverges from CI
# (D01M7YAXH89ZJQ7Q4HP77NMHBX8). Use this script instead of a bare report command.
#
# Usage: scripts/regen-site-report.sh
set -euo pipefail
cd "$(dirname "$0")/.."

TOOLS="${TOOLBOX_CI_TOOLS:-${TMPDIR:-/tmp}/toolbox-ci-tools}"
scripts/ci-tools.sh "$TOOLS" actionlint
export PATH="$TOOLS:$PATH"

need_zizmor=1
if command -v zizmor >/dev/null 2>&1; then
  ver=$(zizmor --version 2>/dev/null || true)
  case "$ver" in
    *1.32.0*) need_zizmor=0 ;;
  esac
fi
if [[ "$need_zizmor" -eq 1 ]]; then
  if command -v pipx >/dev/null 2>&1; then
    pipx install --force "zizmor==1.32.0"
  else
    python3 -m pip install --user "zizmor==1.32.0"
    export PATH="${HOME}/.local/bin:$PATH"
  fi
fi

if ! ruff --version 2>/dev/null | grep -q '0.14.0'; then
  if command -v pipx >/dev/null 2>&1; then
    pipx install --force "ruff==0.14.0"
  else
    python3 -m pip install --user "ruff==0.14.0"
    export PATH="${HOME}/.local/bin:$PATH"
  fi
fi

actionlint -version
zizmor --version
ruff --version

cargo build --locked
target/debug/toolbox doctor portal
target/debug/toolbox report portal --skip runner,github-actions -o web --out site/public/report
echo "wrote site/public/report (preview only; not committed — pages CI regenerates it)"
