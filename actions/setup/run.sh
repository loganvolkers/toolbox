#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${SEIKIKA_BIN:-}" && -n "${SEIKIKA_VERSION:-}" ]]; then
  echo '::error title=Set up Seikika::Choose either seikika-version or seikika-dir.'
  exit 2
fi
if [[ -n "${SEIKIKA_VERSION:-}" ]]; then
  SEIKIKA_BIN=$(mktemp -d "${RUNNER_TEMP:?}/seikika-bin.XXXXXX")
  export SEIKIKA_BIN
  bash "$(dirname "${BASH_SOURCE[0]}")/../report/install.sh"
elif [[ -z "${SEIKIKA_BIN:-}" ]]; then
  echo '::error title=Set up Seikika::Set seikika-version or seikika-dir.'
  exit 2
fi

if ! bin=$(cd "$SEIKIKA_BIN" && pwd -P) || [[ ! -x "$bin/seikika" ]]; then
  echo '::error title=Set up Seikika::The binary directory must contain an executable named seikika.'
  exit 2
fi
"$bin/seikika" --version
printf '%s\n' "$bin" >> "${GITHUB_PATH:?}"
printf 'bin-dir=%s\n' "$bin" >> "${GITHUB_OUTPUT:?}"
