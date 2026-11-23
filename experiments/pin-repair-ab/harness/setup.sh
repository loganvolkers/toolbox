#!/usr/bin/env bash
# Build one run's workspace: <dest>/openfga on branch `pr` (the planted PR) over `main` (the fork at
# the pinned commit plus the arm's gate), and <dest>/bin with the tools on PATH.
#
#   setup.sh <openfga-src> <arm: toolbox|reviewdog> <dest> <toolbox-checkout> <toolbox-target-dir>
#
# <openfga-src> is an export of loganvolkers/openfga@7d89a566 (README §3). <toolbox-checkout> is the
# frozen Toolbox commit the round runs on; its packages/github-actions is vendored into arm A, and
# <toolbox-target-dir> holds its `toolbox` and `toolbox-connector-*` binaries.
set -euo pipefail
src=$1 arm=$2 dest=$3 checkout=$4 target=$5
here=$(cd "$(dirname "$0")" && pwd)
dest=$(realpath -m "$dest")
[ ! -e "$dest" ] || { echo "setup: $dest exists" >&2; exit 1; }
mkdir -p "$dest/bin"
for tool in zizmor actionlint reviewdog; do ln -s "$(command -v "$tool")" "$dest/bin/$tool"; done
cp "$target/toolbox" "$target"/toolbox-connector-* "$dest/bin/"
rm -f "$dest"/bin/*.d
export PATH="$dest/bin:$PATH"

repo="$dest/openfga"
cp -a "$src" "$repo"
cd "$repo"
git init -q -b main
git config user.name "openfga maintainer"
git config user.email "maintainer@example.invalid"
cp "$checkout/packages/github-actions/actionlint.yaml" .github/actionlint.yaml
case "$arm" in
  toolbox)
    mkdir -p .toolbox
    cp "$here/toolbox.yaml" .toolbox/toolbox.yaml
    toolbox vendor "$checkout/packages/github-actions" --portal .toolbox >/dev/null
    toolbox validate .toolbox >/dev/null
    toolbox report .toolbox --skip github-actions --baseline .toolbox/results.json --update >/dev/null
    ;;
  reviewdog)
    cp "$here/quality-gate.sh" scripts/quality-gate.sh
    ;;
  *) echo "setup: unknown arm $arm" >&2; exit 1 ;;
esac
git add -A
git commit -qm "openfga at 7d89a566 with the quality gate"
git checkout -qb pr
python3 "$here/plant.py" .
git add -A
git commit -qm "Add a weekly checks workflow"
