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

if [[ "$portal_dir" -ef "$example_dir/seikika-forks" ]]; then
  organization="$destination/portal/examples/github-org-federation"
  source="$repo_dir/examples/github-org-federation"
  mkdir -p "$organization" "$destination/portal/packages"
  cp -- "$source"/{public,public-connector,repositories,standards,workflows,meta,standards.test}.yaml "$organization/"
  cp -R -- "$source"/{assessments,fixtures} "$organization/"
  cp -R -- "$repo_dir/packages/meta-scoring" "$destination/portal/packages/"
  bundle=$(cd -- "$destination/portal" && pwd)
  # The assessment connector verifies citations against these original files.
  (
    cd -- "$repo_dir"
    cp --parents -- .github/CODEOWNERS \
      crates/seikika-connector-workspace/src/{github_org,org_workflows,repo}.rs \
      docs/decisions/D01M7Y57FVX0XYNGMX0A0XJ4GK9.md "$bundle/"
  )
  sed -i 's|../../github-org-federation/|examples/github-org-federation/|g; s|../../../packages/|packages/|g' \
    "$destination/portal/seikika.yaml"
  bash "$repo_dir/scripts/ci-tools.sh" "$destination/bin" zizmor
fi
