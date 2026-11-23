#!/usr/bin/env bash
# Unpack the repo-portal binaries (toolbox, workspace, runner) into one directory
# after checking each release archive's SHA-256. Same layout as actions/report/install.sh.
#
# The consumer install is the installer on the GitHub release.
# See site/src/content/docs/start/install.md. For v0.1.1:
#   curl --proto '=https' --tlsv1.2 -LsSf \
#     https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.sh | sh
#
# This script is for a checkout that already exists:
#   TOOLBOX_VERSION=0.1.1 ./scripts/install-release.sh
#   TOOLBOX_VERSION=0.1.1 TOOLBOX_BIN=~/.local/bin ./scripts/install-release.sh
# Private repos: set GH_TOKEN or GITHUB_TOKEN (contents:read on the Toolbox repo).
set -euo pipefail
TOOLBOX_REPO="${TOOLBOX_REPO:-loganvolkers/toolbox}"
TOOLBOX_VERSION="${TOOLBOX_VERSION:?set TOOLBOX_VERSION, e.g. 0.1.0}"
TOOLBOX_VERSION="${TOOLBOX_VERSION#v}"
if [ -n "${TOOLBOX_BIN:-}" ]; then
  dest="$TOOLBOX_BIN"
elif [ -n "${CARGO_HOME:-}" ]; then
  dest="${CARGO_HOME}/bin"
else
  dest="${HOME}/.cargo/bin"
fi
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)   target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64)  target=aarch64-unknown-linux-gnu ;;
  Darwin-arm64)   target=aarch64-apple-darwin ;;
  Darwin-x86_64)  target=x86_64-apple-darwin ;;
  MINGW*|MSYS*|CYGWIN*) target=x86_64-pc-windows-msvc ;;
  *) echo "toolbox: no release build for $(uname -s)-$(uname -m)" >&2; exit 2 ;;
esac
ext=tar.xz; [ "$target" = x86_64-pc-windows-msvc ] && ext=zip
mkdir -p "$dest"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
tag="v${TOOLBOX_VERSION}"
download() {
  local file="$1"
  if command -v gh >/dev/null 2>&1; then
    gh release download "$tag" --repo "$TOOLBOX_REPO" --dir "$work" --pattern "$file" --pattern "$file.sha256" --clobber
  else
    local base="https://github.com/${TOOLBOX_REPO}/releases/download/${tag}"
    local auth=()
    local tok="${GH_TOKEN:-${GITHUB_TOKEN:-}}"
    if [ -n "$tok" ]; then
      auth=(-H "Authorization: Bearer ${tok}" -H "Accept: application/octet-stream")
    fi
    curl -fsSL --retry 3 "${auth[@]}" -o "$work/$file" "$base/$file"
    curl -fsSL --retry 3 "${auth[@]}" -o "$work/$file.sha256" "$base/$file.sha256"
  fi
}
for app in toolbox-cli toolbox-connector-workspace toolbox-connector-runner; do
  file="$app-$target.$ext"
  download "$file"
  want=$(cut -d' ' -f1 "$work/$file.sha256")
  if command -v sha256sum >/dev/null 2>&1; then
    got=$(sha256sum "$work/$file" | cut -c1-64)
  else
    got=$(shasum -a 256 "$work/$file" | cut -c1-64)
  fi
  [ "$want" = "$got" ] || { echo "toolbox: checksum mismatch for $file" >&2; exit 2; }
  if [ "$ext" = zip ]; then unzip -q -o "$work/$file" -d "$work"; else tar -xJf "$work/$file" -C "$work"; fi
  find "$work/$app-$target" -maxdepth 1 -type f \( -name 'toolbox*' \) -exec cp {} "$dest/" \;
done
chmod +x "$dest"/toolbox* 2>/dev/null || true
echo "toolbox ${TOOLBOX_VERSION} ($target) -> $dest"
echo "Add $dest to PATH if needed, then: toolbox --version && toolbox doctor <portal>"
