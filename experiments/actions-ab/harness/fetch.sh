#!/usr/bin/env bash
# Download the pinned forks (repos.tsv) as tarballs into <dir>, and Backstage's documentation at
# the arm's release into <dir>/backstage-docs: the whole docs/ tree of backstage/backstage at
# v1.57.0, and the scaffolder and catalog backends' OpenAPI specs, uncurated. The gha arm's docs go
# into <dir>/gha-docs, whole subtrees at pinned commits (triage G1): from github/docs, the Actions
# pages, branch rules, code owners, teams, the Actions reusables and the REST reference for actions,
# deployments, repos and teams (fit audit r8, correction 4); act's README at the binary's tag;
# and the act user guide's src/.
#   fetch.sh <dir>
set -euo pipefail
dir=$1 here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$dir"
while read -r _ fork sha; do
  [ -s "$dir/$fork.tgz" ] || curl -fsSL -o "$dir/$fork.tgz" "https://codeload.github.com/loganvolkers/$fork/tar.gz/$sha"
done < "$here/repos.tsv"
bs=6b4aecbbe1159f48a3937dfd839e3018b85b608d # backstage/backstage v1.57.0
if [ ! -d "$dir/backstage-docs" ]; then
  mkdir -p "$dir/backstage-docs.tmp"
  curl -fsSL "https://codeload.github.com/backstage/backstage/tar.gz/$bs" |
    tar xz -C "$dir/backstage-docs.tmp" --strip-components=1 "backstage-$bs/docs" \
      "backstage-$bs/plugins/scaffolder-backend/src/schema/openapi.yaml" \
      "backstage-$bs/plugins/catalog-backend/src/schema/openapi.yaml"
  mv "$dir/backstage-docs.tmp" "$dir/backstage-docs"
fi
gh=44f009da482ef1470b37ffea456821014d892a1f    # github/docs main, 2026-11-24
actdocs=35cab6107d2afb3269a61dc5de5c15c151a4574f # nektos/act-docs, last changed 2025-05-01
if [ ! -d "$dir/gha-docs" ]; then
  t="$dir/gha-docs.tmp"
  rm -rf "$t" && mkdir -p "$t/github-docs" "$t/act" "$t/act-docs"
  git -C "$t/github-docs" init -q
  git -C "$t/github-docs" remote add origin https://github.com/github/docs
  git -C "$t/github-docs" sparse-checkout set --no-cone /content/actions/ \
    /content/repositories/configuring-branches-and-merges-in-your-repository/ \
    /content/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners.md \
    /content/organizations/organizing-members-into-teams/ \
    /data/reusables/actions/ /src/rest/data/fpt-2022-11-28/actions.json \
    /src/rest/data/fpt-2022-11-28/deployments.json /src/rest/data/fpt-2022-11-28/repos.json \
    /src/rest/data/fpt-2022-11-28/teams.json
  git -C "$t/github-docs" fetch -q --depth 1 --filter=blob:none origin "$gh"
  git -C "$t/github-docs" checkout -q FETCH_HEAD
  rm -rf "$t/github-docs/.git"
  curl -fsSL -o "$t/act/README.md" https://raw.githubusercontent.com/nektos/act/v0.2.98/README.md
  curl -fsSL "https://codeload.github.com/nektos/act-docs/tar.gz/$actdocs" |
    tar xz -C "$t/act-docs" --strip-components=1 "act-docs-$actdocs/src"
  mv "$t" "$dir/gha-docs"
fi
ls -la "$dir"
