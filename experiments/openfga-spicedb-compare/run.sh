#!/usr/bin/env bash
# OpenFGA v1.23.0 against SpiceDB v1.57.1 (README.md): download and verify both releases, build the
# load generator, generate the corpus, run 5 rounds with the arms alternating (OpenFGA first in odd
# rounds, SpiceDB first in even ones), then decide the counted and probed facts (facts.py).
# Writes raw/; python3 snapshot.py turns raw/ into the product records.
#
# Needs: curl, sha256sum, python3 with PyYAML, Go (GOTOOLCHAIN, default go1.27.4), and gh
# authenticated for facts.py. Ports used: 18051, 18080, 18081, 18090, 18112, 18317, and for the
# probes each server's default ports.
set -euo pipefail
cd "$(dirname "$0")"
HERE=$(pwd)
export GOTOOLCHAIN=${GOTOOLCHAIN:-go1.27.4}
export PATH="$(go env GOPATH)/bin:$HOME/.local/bin:$PATH"
ROUNDS=${ROUNDS:-5}
LOAD_MAX=${LOAD_MAX:-1.0}       # wait for the 1-minute load average to drop below this
LOAD_WAIT_S=${LOAD_WAIT_S:-600} # then give up waiting and proceed
T0=$(date +%s)
STARTED=$(date -u +%Y-%m-%dT%H:%M:%SZ)
cd ../..
HARNESS=$(git rev-parse --short HEAD)
if [ -n "$(git status --porcelain -- experiments/openfga-spicedb-compare)" ]; then HARNESS="$HARNESS+uncommitted"; fi
cd "$HERE"

say() { echo "run.sh: $*" >&2; }

fetch() { # repo tag asset dir [saved-as]
  local url="https://github.com/$1/releases/download/$2/$3" to="$4/${5:-$3}"
  [ -s "$to" ] || { say "download $url"; curl -fsSL --retry 4 -o "$to" "$url"; }
}

verify() { # dir checksums-file asset
  (cd "$1" && grep -E "  $3\$" "$2" > ".$3.sum" && sha256sum -c ".$3.sum" && rm ".$3.sum") >&2
}

mkdir -p downloads/openfga downloads/spicedb downloads/cli bin raw logs

say "tools"
command -v govulncheck >/dev/null || go install golang.org/x/vuln/cmd/govulncheck@v1.10.0
command -v slsa-verifier >/dev/null || go install github.com/slsa-framework/slsa-verifier/v2/cli/slsa-verifier@v2.7.1
command -v cosign >/dev/null || go install github.com/sigstore/cosign/v2/cmd/cosign@v2.6.1

say "releases"
OF=openfga_1.23.0_linux_amd64.tar.gz
for a in checksums.txt checksums.txt.sig checksums.txt.pem openfga.intoto.jsonl "$OF" "$OF.sbom.json"; do
  fetch openfga/openfga v1.23.0 "$a" downloads/openfga
done
verify downloads/openfga checksums.txt "$OF"
verify downloads/openfga checksums.txt "$OF.sbom.json"
SD=spicedb_1.57.1_linux_amd64.tar.gz
for a in checksums.txt "$SD"; do fetch authzed/spicedb v1.57.1 "$a" downloads/spicedb; done
verify downloads/spicedb checksums.txt "$SD"
# The projects' CLIs, for the modelTests probe.
fetch openfga/cli v0.9.0 checksums.txt downloads/cli fga-checksums.txt
fetch openfga/cli v0.9.0 fga_0.9.0_linux_amd64.tar.gz downloads/cli
verify downloads/cli fga-checksums.txt fga_0.9.0_linux_amd64.tar.gz
fetch authzed/zed v1.2.1 checksums.txt downloads/cli zed-checksums.txt
fetch authzed/zed v1.2.1 zed_1.2.1_linux_amd64_gnu.tar.gz downloads/cli
verify downloads/cli zed-checksums.txt zed_1.2.1_linux_amd64_gnu.tar.gz
tar xzf "downloads/openfga/$OF" -C bin openfga
tar xzf "downloads/spicedb/$SD" -C bin spicedb
tar xzf downloads/cli/fga_0.9.0_linux_amd64.tar.gz -C downloads/cli fga
tar xzf downloads/cli/zed_1.2.1_linux_amd64_gnu.tar.gz -C downloads/cli zed

say "load generator and corpus"
(cd load && go build -o loadgen .)
python3 gen.py >&2

KEY=$(python3 -c 'import secrets; print(secrets.token_hex(16))')
OPENFGA=(bin/openfga run --datastore-engine memory
  --grpc-addr 127.0.0.1:18081 --http-addr 127.0.0.1:18080 --metrics-addr 127.0.0.1:18112)
SPICEDB=(bin/spicedb serve --datastore-engine memory --grpc-preshared-key "$KEY"
  --grpc-addr 127.0.0.1:18051 --metrics-addr 127.0.0.1:18090)

wait_load() { # sets WAITED and GAVE_UP
  WAITED=0 GAVE_UP=false
  while :; do
    read -r l1 _ < /proc/loadavg
    if python3 -c "import sys; sys.exit(0 if $l1 < $LOAD_MAX else 1)"; then return; fi
    if [ "$WAITED" -ge "$LOAD_WAIT_S" ]; then
      say "load average $l1 still >= $LOAD_MAX after ${WAITED}s; proceeding"
      GAVE_UP=true
      return
    fi
    say "load average $l1 >= $LOAD_MAX; waiting"
    sleep 10
    WAITED=$((WAITED + 10))
  done
}

round() { # n order arm
  local n=$1 order=$2 arm=$3 out="raw/round-$1-$3.json"
  wait_load
  say "round $n: $arm (load waited ${WAITED}s, gave up: $GAVE_UP)"
  if [ "$arm" = openfga ]; then
    load/loadgen round -dir . -arm openfga -addr 127.0.0.1:18081 -round "$n" -order "$order" \
      -log "logs/round-$n-openfga.log" -out "$out" -- "${OPENFGA[@]}" > /dev/null
  else
    load/loadgen round -dir . -arm spicedb -addr 127.0.0.1:18051 -key "$KEY" -round "$n" -order "$order" \
      -log "logs/round-$n-spicedb.log" -out "$out" -- "${SPICEDB[@]}" > /dev/null
  fi
  python3 - "$out" "$WAITED" "$GAVE_UP" "$KEY" <<'EOF'
import glob, json, sys
path, waited, gave_up, key = sys.argv[1], int(sys.argv[2]), sys.argv[3] == "true", sys.argv[4]
r = json.load(open(path))
r["loadWaitS"], r["loadWaitGaveUp"] = waited, gave_up
r["server"] = ["<key>" if a == key else a for a in r["server"]]
json.dump(r, open(path, "w"), indent=2)
open(path, "a").write("\n")
e, c = r["equivalence"], r["check"]
print(f"run.sh:   ready {r['readyS']} s, rss {r['rssIdleMb']}/{r['rssLoadedMb']} MB, write {r['write']['perS']}/s, "
      f"check p95 {c['p95Ms']} ms at {c['rps']}/s, list p95 {r['list']['p95Ms']} ms, "
      f"oracle {'match' if e['oracleMatch'] else 'MISMATCH'}", file=sys.stderr)
# The run stops unless every round, of either arm, gave the same answers (README.md).
for other in sorted(glob.glob("raw/round-*.json")):
    o = json.load(open(other))["equivalence"]
    if (o["decisionsDigest"], o["listsDigest"]) != (e["decisionsDigest"], e["listsDigest"]):
        sys.exit(f"run.sh: {path} and {other} answered differently; stopping")
EOF
}

rm -f raw/round-*.json
for n in $(seq 1 "$ROUNDS"); do
  if [ $((n % 2)) -eq 1 ]; then first=openfga second=spicedb; else first=spicedb second=openfga; fi
  round "$n" 1 "$first"
  round "$n" 2 "$second"
done

say "facts"
python3 facts.py

python3 - "$STARTED" "$HARNESS" "$(( $(date +%s) - T0 ))" <<'EOF'
import json, os, platform, subprocess, sys
cpu = [l.split(":", 1)[1].strip() for l in open("/proc/cpuinfo") if l.startswith("model name")]
mem = next(l.split()[1] for l in open("/proc/meminfo") if l.startswith("MemTotal"))
env = {"startedAt": sys.argv[1], "harness": sys.argv[2], "wallS": int(sys.argv[3]),
       "rounds": int(os.environ.get("ROUNDS", "5")),
       "cpu": {"model": cpu[0] if cpu else None, "count": len(cpu)}, "memTotalKb": int(mem),
       "kernel": platform.release(),
       "go": subprocess.run(["go", "version"], capture_output=True, text=True).stdout.strip(),
       "python": platform.python_version()}
json.dump(env, open("raw/env.json", "w"), indent=2)
open("raw/env.json", "a").write("\n")
print(f"run.sh: done in {env['wallS']} s; harness {env['harness']}", file=sys.stderr)
EOF
