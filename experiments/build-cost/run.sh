#!/usr/bin/env bash
# Measures clean and incremental build times for the "boring" and "full" dependency sets.
# Usage: TARGET_ROOT=/some/scratch ./run.sh > results.txt   (dependencies must already be fetched)
set -euo pipefail
cd "$(dirname "$0")"
ROOT=${TARGET_ROOT:-/tmp/build-cost-target}
t() { local s=$(date +%s.%N); "$@" >/dev/null 2>&1 || { echo "FAILED: $*"; "$@" 2>&1 | tail -30; exit 1; }; echo "$(echo "$(date +%s.%N) - $s" | bc)"; }
bump() { sed -i "s/^\/\/ EDIT-MARKER .*/\/\/ EDIT-MARKER $RANDOM/" crates/app/src/main.rs; }
bumpleaf() { echo "// $RANDOM" >> crates/leaf-cel/src/lib.rs; }
mkdir -p "$ROOT"; cp crates/app/src/main.rs $ROOT/.bc-main.rs; cp crates/leaf-cel/src/lib.rs $ROOT/.bc-cel.rs
trap 'cp $ROOT/.bc-main.rs crates/app/src/main.rs; cp $ROOT/.bc-cel.rs crates/leaf-cel/src/lib.rs' EXIT
echo "machine: $(nproc) vCPU, $(rustc --version)"
for set in boring full; do
  export CARGO_TARGET_DIR=$ROOT/$set; rm -rf "$CARGO_TARGET_DIR"
  echo "$set clean_dev_build_s $(t cargo build --offline -p app --features $set)"
  echo "$set crates_compiled $(ls $CARGO_TARGET_DIR/debug/deps/*.rlib | sed 's/-[0-9a-f]*\.rlib//' | sort -u | wc -l)"
  echo "$set dev_binary_bytes $(stat -c %s $CARGO_TARGET_DIR/debug/app)"
  bump;     echo "$set incr_edit_app_s $(t cargo build --offline -p app --features $set)"
  bump;     echo "$set incr_edit_app_s $(t cargo build --offline -p app --features $set)"
  bumpleaf; echo "$set incr_edit_leaf_cel_s $(t cargo build --offline -p app --features $set)"
  bumpleaf; echo "$set check_edit_leaf_cel_s $(t cargo check --offline -p app --features $set)"
  bumpleaf; echo "$set check_edit_leaf_cel_s $(t cargo check --offline -p app --features $set)"
done
for set in boring full; do
  export CARGO_TARGET_DIR=$ROOT/rel-$set; rm -rf "$CARGO_TARGET_DIR"
  echo "$set clean_release_build_s $(t cargo build --offline --release -p app --features $set)"
  echo "$set release_binary_bytes $(stat -c %s $CARGO_TARGET_DIR/release/app)"
done
