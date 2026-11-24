#!/usr/bin/env bash
# Download the pinned forks (repos.tsv) as tarballs into <dir>.
#   fetch.sh <dir>
set -euo pipefail
dir=$1 here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$dir"
while read -r _ fork sha; do
  [ -s "$dir/$fork.tgz" ] || curl -fsSL -o "$dir/$fork.tgz" "https://codeload.github.com/loganvolkers/$fork/tar.gz/$sha"
done < "$here/repos.tsv"
ls -la "$dir"
