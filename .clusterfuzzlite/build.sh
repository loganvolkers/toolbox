#!/usr/bin/env bash
# ClusterFuzzLite build entrypoint (invoked by CFLite / OSS-Fuzz runners).
# Produces a no-op libFuzzer binary so local CFLite dry-runs have an $OUT artifact.
set -euo pipefail

cat > "$SRC/fuzz_noop.cc" <<'EOF'
#include <stddef.h>
#include <stdint.h>
extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
  (void)data;
  (void)size;
  return 0;
}
EOF

$CXX $CXXFLAGS -std=c++17 "$SRC/fuzz_noop.cc" \
  -o "$OUT/fuzz_noop" $LIB_FUZZING_ENGINE
