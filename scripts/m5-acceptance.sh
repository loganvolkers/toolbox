#!/usr/bin/env bash
# M5 exit criteria (docs/design/federated-actions.md §7): self-service actions at the repo level
# and through a federating org portal, on a local org of git repositories. No network, no
# Postgres: the CLI path (`toolbox run`) and the server path (`toolbox serve`, REST and MCP).
# Usage: scripts/m5-acceptance.sh
set -euo pipefail
cd "$(dirname "$0")/.."
BIN=$(cd "${TOOLBOX_BIN_DIR:-target/debug}" && pwd); EX=$PWD/examples/federated-actions; WORK=$(mktemp -d)
API=127.0.0.1:${API_PORT:-18095}; URL=http://$API
export TOOLBOX_STATE_DIR=$WORK/state TOOLBOX_CONNECTORS_DIR=$BIN
G=(-c user.name=acceptance -c user.email=acceptance@example.com)
PIDS=(); cleanup() { kill "${PIDS[@]}" 2>/dev/null || true; [ -n "${KEEP:-}" ] && echo "kept $WORK" || rm -rf "$WORK"; }
trap cleanup EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }
field() { python3 -c "import json,sys; d=json.load(sys.stdin); print(eval('d'+sys.argv[1]))" "$1"; }
run() { (cd "$1" && shift && "$BIN/toolbox" run "$@" -o json); }
merge() { # merge a proposed branch into main, as a reviewer would
  git -C "$WORK/review/$1" fetch -q origin
  git -C "$WORK/review/$1" checkout -q main && git -C "$WORK/review/$1" pull -q origin main
  git -C "$WORK/review/$1" "${G[@]}" merge -q --no-edit "origin/$2"
  git -C "$WORK/review/$1" push -q origin main
}

echo "0. a local org: four repositories with repo portals, and an org portal"
mkdir -p "$WORK/org"
for r in alpha beta gamma delta; do
  git init -q --bare -b main "$WORK/origin-$r.git"
  git init -q -b main "$WORK/$r"; cp -r "$EX/repo/.toolbox" "$WORK/$r/"
  sed -i "s/example-repo/$r/g" "$WORK/$r/.toolbox/toolbox.yaml"
  echo "# $r" > "$WORK/$r/README.md"
  git -C "$WORK/$r" remote add origin "$WORK/origin-$r.git"
  git -C "$WORK/$r" add -A && git -C "$WORK/$r" "${G[@]}" commit -qm init && git -C "$WORK/$r" push -q -u origin main
  git clone -q "$WORK/origin-$r.git" "$WORK/review/$r"
done
# delta declares no runner: a served parent must refuse to run its actions.
python3 - "$WORK/delta/.toolbox/toolbox.yaml" <<'PY'
import re, sys
p = sys.argv[1]; s = open(p).read()
s, n = re.subn(r"(?m)^federation:.*\n(?:[ \t]+.*\n)*", "", s)
assert n == 1, "no federation block to strip"
open(p, "w").write(s)
PY
git -C "$WORK/delta" "${G[@]}" commit -qam "no runner" && git -C "$WORK/delta" push -q
cp "$EX/org/toolbox.yaml" "$WORK/org/"
"$BIN/toolbox" validate "$WORK/alpha/.toolbox" | tail -1
"$BIN/toolbox" validate "$WORK/org" | tail -1

echo "1. repo level: list, preview, confirm, observed"
run "$WORK/alpha" | field "['actions'].__len__()" | grep -qx 4 || fail "alpha should offer 4 actions"
TOKEN=$(run "$WORK/alpha" write-readme -i text='Alpha does one thing well.' | field "['confirm_token']")
OUT=$(run "$WORK/alpha" write-readme -i text='Alpha does one thing well.' --confirm "$TOKEN")
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "write-readme: $OUT"
echo "   write-readme succeeded, verified: $(echo "$OUT" | field "['verification']['postcondition']")"

echo "2. a stale preview and invalid inputs are refused"
TOKEN=$(run "$WORK/alpha" write-readme -i text='A second README for alpha.' | field "['confirm_token']")
echo "# changed meanwhile, by someone else" > "$WORK/alpha/README.md"
set +e; OUT=$(run "$WORK/alpha" write-readme -i text='A second README for alpha.' --confirm "$TOKEN" 2>&1); CODE=$?; set -e
[ $CODE = 1 ] && echo "$OUT" | grep -q preview_stale || fail "stale token accepted ($CODE): $OUT"
set +e; run "$WORK/alpha" write-readme -i text=short >/dev/null 2>&1; CODE=$?; set -e
[ $CODE = 2 ] || fail "invalid inputs should exit 2, got $CODE"
echo "   preview_stale (exit 1), invalid inputs (exit 2)"

echo "3. a repo portal holds no approvals"
set +e; OUT=$(run "$WORK/alpha" set-owners -i team=@acme/alpha --yes); CODE=$?; set -e
[ $CODE = 3 ] && [ "$(echo "$OUT" | field "['state']")" = awaiting_approval ] || fail "set-owners: $CODE $OUT"
echo "   set-owners is awaiting_approval (exit 3)"

echo "4. a change is a pull request; review is its approval; done when merged and observed"
OUT=$(run "$WORK/alpha" add-license -i spdx=MIT --yes)
[ "$(echo "$OUT" | field "['state']")" = proposed ] || fail "add-license: $OUT"
RID=$(echo "$OUT" | field "['id']"); BRANCH=$(echo "$OUT" | field "['outputs']['branch']")
git -C "$WORK/alpha" log -1 --format=%B "origin/$BRANCH" 2>/dev/null | grep -q "Toolbox-Run: $RID" \
  || git --git-dir "$WORK/origin-alpha.git" log -1 --format=%B "$BRANCH" | grep -q "Toolbox-Run: $RID" || fail "no Toolbox-Run trailer"
[ "$(run "$WORK/alpha" --verify "$RID" | field "['state']")" = proposed ] || fail "unmerged change should stay proposed"
merge alpha "$BRANCH"
[ "$(run "$WORK/alpha" --verify "$RID" | field "['state']")" = succeeded ] || fail "merged change not observed"
OUT=$(run "$WORK/beta" add-license -i spdx=MIT --yes); RID=$(echo "$OUT" | field "['id']")
echo "MIT License" > "$WORK/review/beta/LICENSE"
git -C "$WORK/review/beta" add LICENSE && git -C "$WORK/review/beta" "${G[@]}" commit -qm "a licence, by hand" && git -C "$WORK/review/beta" push -q origin main
[ "$(run "$WORK/beta" --verify "$RID" | field "['state']")" = proposed ] || fail "an unmerged change succeeded because the base already had a licence"
echo "   $BRANCH proposed, merged, succeeded; on beta, a licence pushed by hand does not complete the unmerged run"

echo "5. the org portal offers every child's actions, packages included"
N=$(run "$WORK/org" | field "['actions'].__len__()")
[ "$N" = 16 ] || fail "org should offer 16 actions (4 portals x 4), got $N"
OUT=$(run "$WORK/org" write-readme --entity portal:beta -i text='Beta, run from the org portal.' --yes)
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "federated write-readme: $OUT"
[ "$(echo "$OUT" | field "['verification']['by']")" = "child portal" ] || fail "not verified by the child: $OUT"
echo "   write-readme on portal:beta succeeded, verified by the child portal"

echo "6. a fleet change: one org action, a pull request per repo, each verified by its repo"
declare -A FLEET
for r in alpha beta gamma; do
  OUT=$(run "$WORK/org" add-contributing --entity "portal:$r" -i team="@acme/$r" --yes)
  [ "$(echo "$OUT" | field "['state']")" = proposed ] || fail "add-contributing on $r: $OUT"
  FLEET[$r]=$(echo "$OUT" | field "['id']")
  merge "$r" "$(echo "$OUT" | field "['child']['outputs']['branch']")"
done
for r in alpha beta gamma; do
  OUT=$(run "$WORK/org" --verify "${FLEET[$r]}") || true; [ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "fleet run on $r not observed: $OUT"
done
echo "   add-contributing: 3 pull requests merged, 3 runs succeeded"

echo "7. the org portal served: REST runs, signed approvals by the parent, MCP"
export TOOLBOX_APPROVAL_KEY=$(python3 -c 'import secrets; print(secrets.token_hex(32))')
(cd "$WORK/org" && exec "$BIN/toolbox" serve --listen "$API" . >"$WORK/server.log" 2>&1) & PIDS+=($!)
for _ in $(seq 100); do
  curl -fs -H 'x-toolbox-user: ann' "$URL/v1/actions" 2>/dev/null | grep -q add-contributing && break; sleep 0.2
done
kill -0 "${PIDS[-1]}" 2>/dev/null || fail "the server did not start: $(cat "$WORK/server.log")"
rest() { local who=$1 method=$2 path=$3; shift 3
  curl -s -X "$method" -H "x-toolbox-user: $who" -H 'content-type: application/json' "$URL$path" "$@"; }
OUT=$(rest ann POST /v1/actions/write-readme/previews -d '{"entity":"portal:delta","inputs":{"text":"Delta has no runner here."}}')
[ "$(echo "$OUT" | field "['code']")" = no-runner ] || fail "delta should be refused: $OUT"
BODY='{"entity":"portal:gamma","inputs":{"team":"@acme/gamma"}}'
OUT=$(rest ann POST /v1/actions/set-owners/previews -d "$BODY")
echo "$OUT" | grep -q child-refuses-approvals || fail "gamma pins another key, so the preview should say approvals here can't start it: $OUT"
KEY=$(rest ann GET /v1/actions | field "['approval_key']")
grep -q "$KEY" "$WORK/server.log" || fail "serve should log its approval key"
sed -i "s/ed25519:0\{64\}/$KEY/" "$WORK/gamma/.toolbox/toolbox.yaml"
git -C "$WORK/gamma" "${G[@]}" commit -qam "accept the org portal's approvals"
TOKEN=$(rest ann POST /v1/actions/set-owners/previews -d "$BODY" | field "['confirm_token']")
OUT=$(rest ann POST /v1/actions/set-owners/runs -d "{\"entity\":\"portal:gamma\",\"inputs\":{\"team\":\"@acme/gamma\"},\"confirm_token\":\"$TOKEN\"}")
[ "$(echo "$OUT" | field "['state']")" = awaiting_approval ] || fail "set-owners via the org: $OUT"
RID=$(echo "$OUT" | field "['id']")
OUT=$(rest ann POST /v1/actions/set-owners/runs -d "{\"entity\":\"portal:gamma\",\"inputs\":{\"team\":\"@acme/gamma\"},\"confirm_token\":\"$TOKEN\"}")
[ "$(echo "$OUT" | field "['code']")" = token-used ] || fail "a confirm token started a second run: $OUT"
[ "$(rest ann POST "/v1/runs/$RID/approvals" -d '{}' | field "['code']")" = not-an-approver ] || fail "self-approval accepted"
[ "$(rest carol POST "/v1/runs/$RID/approvals" -d '{}' | field "['code']")" = not-an-approver ] || fail "an approval by someone not listed was accepted"
set +e; OUT=$(cd "$WORK/gamma" && "$BIN/toolbox" run set-owners -i team=@acme/gamma --yes --parent-run org/run_forged --grant 00.00 -o json); CODE=$?; set -e
[ $CODE = 3 ] && echo "$OUT" | grep -q "approvals were refused" || fail "a forged grant was accepted ($CODE): $OUT"
rest bob POST "/v1/runs/$RID/approvals" -d '{"note":"gamma is ours"}' >/dev/null
for _ in $(seq 60); do
  STATE=$(rest ann GET "/v1/runs/$RID" | field "['state']"); case $STATE in succeeded|failed|unverified) break;; esac; sleep 0.5
done
[ "$STATE" = succeeded ] || fail "approved set-owners ended $STATE: $(rest ann GET "/v1/runs/$RID")"
echo "   set-owners on gamma: blocked until gamma pinned the org's key; a reused confirm token, ann (the requester), carol (not listed) and a forged grant refused; approved by bob in the org portal, signed, succeeded in gamma"
mcp() { curl -s -X POST -H 'x-toolbox-user: ann' -H 'content-type: application/json' "$URL/mcp" \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"$1\",\"arguments\":$2}}"; }
curl -s -X POST -H 'x-toolbox-user: bob' -H 'content-type: application/json' "$URL/mcp" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | grep -q approve_run && fail "MCP should offer no approval tool (D57)"
ARGS='{"action":"write-readme","id":"portal:gamma","inputs":{"text":"Gamma, written by an agent over MCP."}}'
TOKEN=$(mcp preview_action "$ARGS" | field "['result']['structuredContent']['confirm_token']")
RID=$(mcp run_action "${ARGS%\}},\"confirm_token\":\"$TOKEN\"}" | field "['result']['structuredContent']['id']")
for _ in $(seq 60); do
  TEXT=$(mcp get_run "{\"run\":\"$RID\"}" | field "['result']['content'][0]['text']"); echo "$TEXT" | grep -q '\*\*succeeded\*\*' && break; sleep 0.5
done
echo "$TEXT" | grep -q '\*\*succeeded\*\*' || fail "MCP run: $TEXT"
[ "$(rest ann GET "/v1/runs/$RID" | field "['via']")" = mcp ] || fail "the MCP run should record via mcp"
echo "   MCP: preview_action, run_action, get_run → $(echo "$TEXT" | head -1)"
python3 - "$URL" "$WORK/state/runs" <<'PY' || fail "approvals and cancels raced"
import glob, json, sys, threading, time, urllib.error, urllib.request
url, runs = sys.argv[1:]
def rest(who, method, path, body=None):
    r = urllib.request.Request(url + path, method=method, headers={"x-toolbox-user": who,
        "content-type": "application/json"}, data=None if body is None else json.dumps(body).encode())
    try:
        with urllib.request.urlopen(r) as f: return json.load(f)
    except urllib.error.HTTPError as e: return json.load(e)
body = {"entity": "portal:gamma", "inputs": {"team": "@acme/gamma"}}
bad = []
for mode in ["bob twice"] * 4 + ["bob and a cancel"] * 4:
    token = rest("ann", "POST", "/v1/actions/set-owners/previews", body)["confirm_token"]
    run = rest("ann", "POST", "/v1/actions/set-owners/runs", dict(body, confirm_token=token))
    if "id" not in run: sys.exit(f"set-owners: {run}")
    rid = run["id"]
    calls = [("bob", "approvals"), ("bob", "approvals")] if mode == "bob twice" else [("bob", "approvals"), ("ann", "cancel")]
    gate, ok = threading.Barrier(2), {}
    def call(i, who, what):
        gate.wait(); ok[i] = "code" not in rest(who, "POST", f"/v1/runs/{rid}/{what}", {})
    threads = [threading.Thread(target=call, args=(i, *c)) for i, c in enumerate(calls)]
    [t.start() for t in threads]; [t.join() for t in threads]
    for _ in range(120):
        state = rest("ann", "GET", f"/v1/runs/{rid}")["state"]
        if state not in ("awaiting_approval", "approved", "dispatched", "executor_succeeded", "verifying"): break
        time.sleep(0.25)
    child = sum(f"parent:org/{rid}" in open(f).read() for f in glob.glob(f"{runs}/*.json"))
    if child > 1 or (mode != "bob twice" and all(ok.values()) and child) or (mode == "bob twice" and all(ok.values())):
        bad.append(f"{mode}: {rid} accepted {ok}, ran {child} times, ended {state}")
print("\n".join(bad) or "   races: bob approving twice at once is accepted once and runs once; a cancel that is accepted stops the run (4 tries each)")
sys.exit(1 if bad else 0)
PY
echo "M5 acceptance: all checks passed"
