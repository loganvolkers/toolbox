#!/usr/bin/env bash
# One workspace for one run of one arm (README §4): an org of three open-source repositories, each
# a bare origin plus a checkout, an empty org directory, and the arm's platform.
#
#   setup.sh <arm: seikika|gha|backstage|vanilla> <dest> <src-dir> [port]
#
# <src-dir> holds the pinned tarballs (fetch.sh). The seikika arm takes SEIKIKA_BIN_DIR (default:
# target/debug of this checkout) and copies the operator skills and two shipped examples; the gha
# arm copies act ($ACT, default /tmp/tools/act) and the GitHub and act docs (fetch.sh); the backstage arm copies the create-app app
# ($BACKSTAGE_APP, default /tmp/bs/app, built by backstage/build-app.sh), commits its install as it
# arrived, serves it on <port> (default 7007), and copies Backstage's docs (fetch.sh).
# The authored-lines baseline goes beside the workspace, in <dest>.baseline.json.
set -euo pipefail
arm=$1 dest=$2 src=$3 port=${4:-7007}
here=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$here/../../.." && pwd)
[ -e "$dest" ] && { echo "refusing: $dest exists" >&2; exit 1; }
mkdir -p "$dest/origins"

local_git() { # the host's signing and fsmonitor settings are not Acme's
  git -C "$1" config commit.gpgsign false
  git -C "$1" config core.fsmonitor false
}

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
  local_git "$dest/$1"
}
while read -r name fork sha; do
  import "$name" "$src/$fork.tgz" "$sha" "$fork"
done < "$here/repos.tsv"

git init -q --bare -b main "$dest/origins/org.git"
git clone -q "$dest/origins/org.git" "$dest/org" 2>/dev/null
git -C "$dest/org" config user.name ann
git -C "$dest/org" config user.email ann@acme.example
local_git "$dest/org"

case $arm in
  seikika)
    bin=$(cd "${SEIKIKA_BIN_DIR:-$repo_root/target/debug}" && pwd)
    mkdir -p "$dest/bin"
    for b in "$bin"/seikika "$bin"/seikika-connector-*; do
      [ -f "$b" ] && [ -x "$b" ] && cp "$b" "$dest/bin/"
    done
    mkdir -p "$dest/seikika-docs/examples"
    cp -r "$repo_root/plugins/seikika-operator/skills" "$dest/seikika-docs/skills"
    cp -r "$repo_root/examples/federated-actions" "$repo_root/examples/repo-portal-starter" \
      "$dest/seikika-docs/examples/"
    cp "$repo_root/crates/seikika-model/schema/manifest.schema.json" "$dest/seikika-docs/"
    ;;
  gha)
    mkdir -p "$dest/bin" && cp "${ACT:-/tmp/tools/act}" "$dest/bin/act"
    cp -r "$src/gha-docs" "$dest/gha-docs"
    ;;
  backstage)
    app=${BACKSTAGE_APP:-/tmp/bs/app}
    cp -a "$app" "$dest/backstage"
    local_git "$dest/backstage"
    git -C "$dest/backstage" add -A
    GIT_AUTHOR_DATE=2026-11-24T00:00:00Z GIT_COMMITTER_DATE=2026-11-24T00:00:00Z \
      git -C "$dest/backstage" -c user.name=create-app -c user.email=create-app@example.com \
      commit -qm "yarn install" || true
    git -C "$dest/backstage" config user.name ann
    git -C "$dest/backstage" config user.email ann@acme.example
    sed -e "s|{PORT}|$port|g" "$here/backstage/app-config.local.yaml" > "$dest/backstage/app-config.local.yaml"
    cp -r "$src/backstage-docs" "$dest/backstage-docs"
    ;;
  vanilla) ;;
  *) echo "unknown arm $arm" >&2; exit 2 ;;
esac

# The baseline the authored-lines measure compares against (score.py).
python3 "$here/score.py" snapshot "$dest" > "$dest.baseline.json"
echo "$dest ($arm)"
