# ClusterFuzzLite build image. OpenSSF Scorecard's Fuzzing check detects this path
# when the file contains build commands (not comments alone).
# Full OSS-Fuzz project registration is optional and owner-gated.
FROM gcr.io/oss-fuzz-base/base-builder-rust@sha256:3428d0a90b1e7efbe2ba947bd1d0c9a70594986b4b6553804547f1fb24cfc92f

RUN apt-get update && apt-get install -y --no-install-recommends \
    make pkg-config \
 && rm -rf /var/lib/apt/lists/*

COPY . $SRC/toolbox
WORKDIR $SRC/toolbox
COPY .clusterfuzzlite/build.sh $SRC/
