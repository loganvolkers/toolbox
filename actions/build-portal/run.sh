#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../lib/portal.sh"

format=${INPUT_FORMAT:-html}
case "$format" in
  html|web) ;;
  *) echo '::error title=Build Seikika portal::format must be html or web.'; exit 2 ;;
esac
if ! executable=$(type -P seikika); then
  echo '::error title=Build Seikika portal::Run actions/setup or put seikika on PATH first.'
  exit 2
fi
executable=$(realpath "$executable")
portal=${INPUT_PORTAL:-.seikika}
prepare_portal 'Build Seikika portal' "$executable" "${INPUT_WORKING_DIRECTORY:-.}" "$portal"

out=$(mktemp -d "${RUNNER_TEMP:?}/seikika-portal.XXXXXX")
destination="$out/index.html"
if [[ "$format" == web ]]; then destination="$out"; fi
args=(report "$portal" -o "$format" --out "$destination")
if [[ -n "${INPUT_SKIP:-}" ]]; then args+=(--skip "$INPUT_SKIP"); fi
"$executable" "${args[@]}"

{
  printf 'directory=%s\nindex=%s/index.html\n' "$out" "$out"
  if [[ "$format" == web ]]; then printf 'report-json=%s/report.json\n' "$out"; fi
} >> "${GITHUB_OUTPUT:?}"
