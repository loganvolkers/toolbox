#!/usr/bin/env bash
# Build the Backstage arm's app as setup.sh copies it ($BACKSTAGE_APP, default /tmp/bs/app):
# Node 24.22.3, then `@backstage/create-app@0.9.4` (Backstage 1.57.0, with the scaffolder backend
# and its GitHub module), which installs the dependencies itself.
#   build-app.sh [dir]   (default /tmp/bs; writes <dir>/node24 and <dir>/app)
set -euo pipefail
dir=${1:-/tmp/bs} node=v24.22.3
mkdir -p "$dir"
if [ ! -x "$dir/node24/bin/node" ]; then
  curl -fsSL "https://nodejs.org/dist/$node/node-$node-linux-x64.tar.xz" | tar xJ -C "$dir"
  ln -sfn "$dir/node-$node-linux-x64" "$dir/node24"
fi
export PATH=$dir/node24/bin:$PATH
corepack enable --install-directory "$dir/node24/bin"
[ -e "$dir/app" ] && { echo "refusing: $dir/app exists" >&2; exit 1; }
echo backstage | npx -y @backstage/create-app@0.9.4 --path "$dir/app"
