#!/usr/bin/env bash
# One workspace for one run of one arm (README §4): an org of three open-source repositories, each
# a bare origin plus a checkout, an empty org directory, and the arm's platform.
#
#   setup.sh <arm: toolbox|gha|backstage|vanilla> <dest> <src-dir> [port]
#
# <src-dir> holds the pinned tarballs (fetch.sh). The toolbox arm takes TOOLBOX_BIN_DIR (default:
# target/debug of this checkout) and copies the operator skills and two shipped examples; the gha
# arm copies act ($ACT, default /tmp/tools/act); the backstage arm copies the create-app backend
# ($BACKSTAGE_APP, default /tmp/bs/app) and serves it on <port> (default 7007).
set -euo pipefail
arm=$1 dest=$2 src=$3 port=${4:-7007}
here=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$here/../../.." && pwd)
[ -e "$dest" ] && { echo "refusing: $dest exists" >&2; exit 1; }
mkdir -p "$dest/origins" "$dest/bin"

import() { # <name> <tarball> <upstream sha>
  local tmp; tmp=$(mktemp -d)
  tar xzf "$2" -C "$tmp" --strip-components=1
  git -C "$tmp" init -q -b main
  git -C "$tmp" add -A
  GIT_AUTHOR_DATE=2026-11-24T00:00:00Z GIT_COMMITTER_DATE=2026-11-24T00:00:00Z \
    git -C "$tmp" -c user.name=Upstream -c user.email=upstream@example.com \
    commit -qm "import loganvolkers/${4}@${3}"
  git clone -q --bare "$tmp" "$dest/origins/$1.git"
  rm -rf "$tmp"
  git clone -q "$dest/origins/$1.git" "$dest/$1"
  git -C "$dest/$1" config user.name ann
  git -C "$dest/$1" config user.email ann@acme.example
}
while read -r name fork sha; do
  import "$name" "$src/$fork.tgz" "$sha" "$fork"
done < "$here/repos.tsv"

git init -q --bare -b main "$dest/origins/org.git"
git clone -q "$dest/origins/org.git" "$dest/org" 2>/dev/null
git -C "$dest/org" config user.name ann
git -C "$dest/org" config user.email ann@acme.example

case $arm in
  toolbox)
    bin=$(cd "${TOOLBOX_BIN_DIR:-$repo_root/target/debug}" && pwd)
    for b in "$bin"/toolbox "$bin"/toolbox-connector-*; do
      [ -f "$b" ] && [ -x "$b" ] && cp "$b" "$dest/bin/"
    done
    mkdir -p "$dest/toolbox-docs/examples"
    cp -r "$repo_root/plugins/toolbox-operator/skills" "$dest/toolbox-docs/skills"
    cp -r "$repo_root/examples/federated-actions" "$repo_root/examples/repo-portal-starter" \
      "$dest/toolbox-docs/examples/"
    cp "$repo_root/schema/manifest.schema.json" "$dest/toolbox-docs/"
    ;;
  gha)
    cp "${ACT:-/tmp/tools/act}" "$dest/bin/act"
    ;;
  backstage)
    app=${BACKSTAGE_APP:-/tmp/bs/app}
    cp -a "$app" "$dest/backstage"
    sed -e "s|{PORT}|$port|g" "$here/backstage/app-config.local.yaml" > "$dest/backstage/app-config.local.yaml"
    ;;
  vanilla) ;;
  *) echo "unknown arm $arm" >&2; exit 2 ;;
esac

# The baseline the authored-lines measure compares against (score.py).
python3 "$here/score.py" snapshot "$dest" > "$dest/.baseline.json"
echo "$dest ($arm)"
