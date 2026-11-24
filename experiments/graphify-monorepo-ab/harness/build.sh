#!/usr/bin/env bash
# Build the frozen export and each arm's additions (README §3).
#
#   build.sh            export the fork at the pinned commit to $WORK/src/codex, build Graphify's
#                       graph in $WORK/build/graphify/codex, and validate arm C's portal on a copy
#
# Env: CODEX_CHECKOUT (a clone of loganvolkers/codex with 859dbe27), WORK (default
# /tmp/graphify-monorepo-ab). Needs graphify 0.9.118 and toolbox on PATH, and cargo for the
# Rust toolchain codex-rs pins (1.95.0), which the portal's `cargo metadata` runs.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
WORK=${WORK:-/tmp/graphify-monorepo-ab}
COMMIT=859dbe27616c593238bad63be63e13a2d80579d9

rm -rf "$WORK/src" "$WORK/build"
mkdir -p "$WORK/src/codex" "$WORK/build/graphify"
(cd "${CODEX_CHECKOUT:?}" && git archive "$COMMIT") | tar -x -C "$WORK/src/codex"
echo "export: $(find "$WORK/src/codex" -type f | wc -l) files"

# Arm B: Graphify's headless code-only build, which needs no LLM, plus its Cargo pass. Communities
# get the LLM-free hub labels (GRAPHIFY_BACKEND=none), as in graphify-ab.
cp -a "$WORK/src/codex" "$WORK/build/graphify/codex"
cd "$WORK/build/graphify/codex"
export GRAPHIFY_BACKEND=none GRAPHIFY_QUERY_LOG_DISABLE=1
start=$(date +%s)
graphify extract . --code-only --cargo
graphify cluster-only . --no-viz
echo "graphify build: $(($(date +%s) - start)) s"

# Arm C: the repo portal; validate it on a copy of the export.
mkdir -p "$WORK/build/toolbox"
cp -a "$WORK/src/codex" "$WORK/build/toolbox/codex"
mkdir -p "$WORK/build/toolbox/codex/.toolbox"
cp "$HERE/portal/toolbox.yaml" "$WORK/build/toolbox/codex/.toolbox/toolbox.yaml"
(cd "$WORK/build/toolbox/codex" && toolbox validate .toolbox)
