#!/usr/bin/env bash
# Assemble a Wrangler build context from a portal and the compiled Linux release binaries.
set -euo pipefail

portal_dir=${1:?usage: stage.sh PORTAL_DIR DESTINATION [BINARIES_DIR]}
destination=${2:?usage: stage.sh PORTAL_DIR DESTINATION [BINARIES_DIR]}
example_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_dir=$(cd -- "$example_dir/../.." && pwd)
binaries_dir=${3:-${CARGO_TARGET_DIR:-"$repo_dir/target"}/release}

test -f "$portal_dir/seikika.yaml"
mkdir -p "$destination"
cp -- "$example_dir"/{Dockerfile,worker.mjs,wrangler.toml,.dockerignore,package.json,package-lock.json} "$destination/"
cp -R -T -- "$portal_dir" "$destination/portal"
for binary in seikika seikika-connector-workspace seikika-connector-github \
    seikika-connector-git-files seikika-connector-gitlab seikika-connector-runner \
    seikika-connector-scim; do
  install -D -m 0755 "$binaries_dir/$binary" "$destination/bin/$binary"
done
bash "$repo_dir/scripts/ci-tools.sh" "$destination/bin" openfga
