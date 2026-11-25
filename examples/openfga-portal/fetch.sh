#!/usr/bin/env bash
# Clone OpenFGA at the commit this portal scores (v1.23.0) into checkout/, which git ignores.
# Run reports from inside the checkout: connector roots resolve against the working directory.
#
#   examples/openfga-portal/fetch.sh
#   cd examples/openfga-portal/checkout && toolbox report .. --skip runner,github-actions
set -euo pipefail
cd "$(dirname "$0")"
COMMIT=d67ba7b1d2eaf945cffab3a9129ca1a4c7ad5251
if [ ! -d checkout/.git ]; then
  git init -q checkout
  git -C checkout remote add origin https://github.com/openfga/openfga
fi
if [ "$(git -C checkout rev-parse -q --verify HEAD || true)" != "$COMMIT" ]; then
  git -C checkout fetch -q --depth 1 origin "$COMMIT"
  git -C checkout checkout -q --detach FETCH_HEAD
fi
git -C checkout rev-parse HEAD
