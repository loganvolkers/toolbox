#!/usr/bin/env bash
# Install CI's third-party tools from their releases, SHA-256-pinned, instead of compiling them
# with `go install` on every run (D90: reuse released artifacts). Linux x86_64 only, like CI.
# Usage: scripts/ci-tools.sh <bin-dir> <tool>...
#   (tools: openfga actionlint fga scorecard osv-scanner rustsec-osv)
# For the site report preview (pages.yml pins), prefer scripts/regen-site-report.sh, which
# installs actionlint from here plus zizmor==1.32.0, then runs doctor + report.
set -euo pipefail
dir=$1; shift; mkdir -p "$dir"; tmp=$(mktemp -d)
fetch() { # name url sha256 member
  curl -fsSL --retry 3 -o "$tmp/$1.tgz" "$2"
  echo "$3  $tmp/$1.tgz" | sha256sum -c - >/dev/null
  tar -xzf "$tmp/$1.tgz" -C "$dir" "$4"
  echo "installed $1 from $2"
}
for tool in "$@"; do
  case "$tool" in
    openfga) fetch openfga https://github.com/openfga/openfga/releases/download/v1.22.1/openfga_1.22.1_linux_amd64.tar.gz \
      80d2d4b582762c02dc86b1246f7f6f54d6854259727b36e5b5c43ba90792814a openfga ;;
    actionlint) fetch actionlint https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_linux_amd64.tar.gz \
      8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8 actionlint ;;
    fga) fetch fga https://github.com/openfga/cli/releases/download/v0.8.2/fga_0.8.2_linux_amd64.tar.gz \
      e48c4a73cf68031fe9fc0827f1462a9b2d6981e3d2e9b82f1e1c2e120c293443 fga ;;
    scorecard) fetch scorecard https://github.com/ossf/scorecard/releases/download/v5.7.0/scorecard_5.7.0_linux_amd64.tar.gz \
      3b08ec0a8681806c3e744ec9210742a502c7d66f06f25523b70f7128738235d3 scorecard ;;
    osv-scanner)
      curl -fsSL --retry 3 -o "$tmp/osv-scanner" https://github.com/google/osv-scanner/releases/download/v2.7.1/osv-scanner_linux_amd64
      echo "fb7dab19a8bd9bbe5c3189f09d4b7ee1820c9553a23bac202648a56efd2f6ee8  $tmp/osv-scanner" | sha256sum -c - >/dev/null
      install -m 755 "$tmp/osv-scanner" "$dir/osv-scanner"
      echo "installed osv-scanner 2.7.1" ;;
    rustsec-osv)
      # RustSec's OSV export at one commit, as osv-scanner's offline crates.io database: point
      # OSV_SCANNER_LOCAL_DB_CACHE_DIRECTORY at <bin-dir>. The commit is the pin; findings change
      # only when it moves (D01M885GT32STM506V32XKPP37G).
      rev=944cab374a9b34d9607c8b64b5185a1e481125cb
      git init -q "$tmp/advisory-db"
      git -C "$tmp/advisory-db" fetch -q --depth 1 https://github.com/rustsec/advisory-db "$rev"
      git -C "$tmp/advisory-db" checkout -q FETCH_HEAD
      mkdir -p "$dir/osv-scalibr/crates.io"
      python3 - "$tmp/advisory-db/crates" "$dir/osv-scalibr/crates.io/all.zip" <<'EOF'
import pathlib, sys, zipfile
src, out = pathlib.Path(sys.argv[1]), sys.argv[2]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for f in sorted(src.glob("*.json")):
        z.write(f, f.name)
EOF
      echo "installed the RustSec OSV database at $rev" ;;
    *) echo "unknown tool $tool" >&2; exit 2 ;;
  esac
done
