#!/usr/bin/env bash
# Install released seikika binaries (D90) instead of building from source: download the archives of
# seikika and the connectors this action runs for the runner's platform from a GitHub release,
# check each against its published SHA-256, and unpack them into $SEIKIKA_BIN.
# Usage: SEIKIKA_REPO=owner/repo SEIKIKA_VERSION=0.2.1 SEIKIKA_BIN=dir install.sh
# Private repos: prefer `gh` (GITHUB_TOKEN on Actions) or set GH_TOKEN for curl.
set -euo pipefail
: "${SEIKIKA_REPO:?}" "${SEIKIKA_VERSION:?}" "${SEIKIKA_BIN:?}"
SEIKIKA_VERSION="${SEIKIKA_VERSION#v}"
tag="v${SEIKIKA_VERSION}"
case "${RUNNER_OS:-$(uname -s)}-${RUNNER_ARCH:-$(uname -m)}" in
  Linux-X64 | Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-ARM64 | Linux-aarch64) target=aarch64-unknown-linux-gnu ;;
  macOS-ARM64 | Darwin-arm64) target=aarch64-apple-darwin ;;
  macOS-X64 | Darwin-x86_64) target=x86_64-apple-darwin ;;
  Windows-X64) target=x86_64-pc-windows-msvc ;;
  *)
    echo "seikika: no release build for ${RUNNER_OS:-?}-${RUNNER_ARCH:-?}" >&2
    exit 2
    ;;
esac
ext=tar.xz
[ "$target" = x86_64-pc-windows-msvc ] && ext=zip
base="https://github.com/${SEIKIKA_REPO}/releases/download/${tag}"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$SEIKIKA_BIN"
download() {
  local file="$1"
  if command -v gh >/dev/null 2>&1; then
    gh release download "$tag" --repo "$SEIKIKA_REPO" --dir "$work" --pattern "$file" --pattern "$file.sha256" --clobber
  else
    local auth=()
    local tok="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
    if [ -n "$tok" ]; then
      auth=(-H "Authorization: Bearer ${tok}" -H "Accept: application/octet-stream")
    fi
    curl -fsSL --retry 3 "${auth[@]}" -o "$work/$file" "$base/$file"
    curl -fsSL --retry 3 "${auth[@]}" -o "$work/$file.sha256" "$base/$file.sha256"
  fi
}
for app in seikika-cli seikika-connector-workspace seikika-connector-runner; do
  file="$app-$target.$ext"
  download "$file"
  want=$(cut -d' ' -f1 "$work/$file.sha256")
  got=$(sha256sum "$work/$file" | cut -d' ' -f1)
  [ "$want" = "$got" ] || {
    echo "seikika: checksum mismatch for $file" >&2
    exit 2
  }
  if [ "$ext" = zip ]; then unzip -q -o "$work/$file" -d "$work"; else tar -xJf "$work/$file" -C "$work"; fi
  find "$work/$app-$target" -maxdepth 1 -type f \( -name 'seikika*' \) -exec cp {} "$SEIKIKA_BIN/" \;
done
chmod +x "$SEIKIKA_BIN"/seikika* 2>/dev/null || true
echo "seikika ${SEIKIKA_VERSION} ($target) from $base"
