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
PIDS=(); cleanup() { kill "${PIDS[@]}" 2>/dev/null || true; [ -n "${DETACHED:-}" ] && "$BIN/toolbox" serve "$DETACHED" --stop >/dev/null 2>&1; [ -n "${KEEP:-}" ] && echo "kept $WORK" || rm -rf "$WORK"; }
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
run "$WORK/alpha" | field "['actions'].__len__()" | grep -qx 5 || fail "alpha should offer 5 actions"
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
set +e; OUT=$(run "$WORK/alpha" add-security-policy -i contact=security@acme.dev --yes); CODE=$?; set -e
[ $CODE = 3 ] || fail "a change that lands without review needs an approval a checkout can't hold ($CODE): $OUT"
git --git-dir "$WORK/origin-alpha.git" cat-file -e main:SECURITY.md 2>/dev/null && fail "an unapproved push landed"
echo "   set-owners and add-security-policy (land: push) are awaiting_approval (exit 3); nothing was pushed"

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
[ "$N" = 20 ] || fail "org should offer 20 actions (4 portals x 5), got $N"
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
echo "8. the served client: toolbox run --url; a person approves; the push lands on main; runs outlive a restart"
TB=("$BIN/toolbox" run --url "$URL" -o json)
set +e; OUT=$("${TB[@]}" add-security-policy --entity portal:gamma -i contact=security@acme.dev --as ann --yes --wait); CODE=$?; set -e
[ $CODE = 3 ] && [ "$(echo "$OUT" | field "['state']")" = awaiting_approval ] || fail "add-security-policy via --url ($CODE): $OUT"
RID=$(echo "$OUT" | field "['id']")
git --git-dir "$WORK/origin-gamma.git" cat-file -e main:SECURITY.md 2>/dev/null && fail "pushed before approval"
OUT=$("${TB[@]}" --approve "$RID" --as bob --note "ship it" --wait) || fail "approve: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved add-security-policy: $OUT"
[ "$(echo "$OUT" | field "['verification']['child']['by']")" = base ] || fail "the child should observe the push on main: $OUT"
git --git-dir "$WORK/origin-gamma.git" show main:SECURITY.md | grep -q security@acme.dev || fail "SECURITY.md is not on gamma's main"
kill "${PIDS[-1]}"; wait "${PIDS[-1]}" 2>/dev/null || true
(cd "$WORK/org" && exec "$BIN/toolbox" serve --listen "$API" . >"$WORK/server2.log" 2>&1) & PIDS+=($!)
for _ in $(seq 100); do curl -fs "$URL/healthz" >/dev/null 2>&1 && break; sleep 0.2; done
OUT=$("${TB[@]}" --runs --as ann)
echo "$OUT" | python3 -c "import json,sys; sys.exit(not any(r['id'] == sys.argv[1] and r['state'] == 'succeeded' for r in json.load(sys.stdin)['runs']))" "$RID" \
  || fail "the run did not outlive the restart: $OUT"
echo "   add-security-policy on gamma: awaiting bob (exit 3), approved with --approve, pushed to main, observed there; listed again after a restart"

echo "9. an org portal that lists its children by git URL: Toolbox keeps the clones and the key"
export TOOLBOX_STATE_DIR=$WORK/state-git; unset TOOLBOX_APPROVAL_KEY
git init -q --bare -b main "$WORK/hosts/mini.git" && git clone -q "$WORK/hosts/mini.git" "$WORK/review/mini" 2>/dev/null
cp -r "$EX/minimal-repo/.toolbox" "$WORK/review/mini/"
git -C "$WORK/review/mini" add -A && git -C "$WORK/review/mini" "${G[@]}" commit -qm init && git -C "$WORK/review/mini" push -q -u origin main
BEFORE=$(git --git-dir "$WORK/hosts/mini.git" rev-parse main)
OUT=$(cd "$WORK/review/mini" && "$BIN/toolbox" test .toolbox) || fail "the minimal repo's action cases: $OUT"
echo "$OUT" | grep -q "5 passed, 0 failed" || fail "the minimal repo should run 5 action cases: $OUT"
[ "$(git --git-dir "$WORK/hosts/mini.git" rev-parse main)" = "$BEFORE" ] || fail "toolbox test changed the real origin"
mkdir -p "$WORK/gitorg"
sed 's#- git: https://github.com/acme/\([a-z]*\)\.git#- {git: ../origin-\1.git, key: \1}#' "$EX/git-org/toolbox.yaml" >"$WORK/gitorg/toolbox.yaml"
echo "    - git: ../hosts/mini.git" >>"$WORK/gitorg/toolbox.yaml"
grep -q "origin-gamma.git, key: gamma" "$WORK/gitorg/toolbox.yaml" || fail "git-org: the children were not rewritten"
"$BIN/toolbox" validate "$WORK/gitorg" | tail -1
N=$(run "$WORK/gitorg" | field "['actions'].__len__()")
[ "$N" = 16 ] || fail "the git org should offer 16 actions (3 children x 5, and mini's one), got $N"
OUT=$(run "$WORK/gitorg" add-license --entity portal:beta -i spdx=Apache-2.0 --yes)
[ "$(echo "$OUT" | field "['state']")" = proposed ] || fail "add-license on a cloned beta: $OUT"
git --git-dir "$WORK/origin-beta.git" rev-parse -q --verify "$(echo "$OUT" | field "['child']['outputs']['branch']")" >/dev/null \
  || fail "the cloned child's branch is not on beta's origin: $OUT"
API2=127.0.0.1:$((${API_PORT:-18095} + 1)); TB2=("$BIN/toolbox" run --url "http://$API2" -o json)
OUT=$("$BIN/toolbox" serve "$WORK/gitorg" --listen "$API2" --detach) || fail "serve --detach: $OUT"; DETACHED=$WORK/gitorg
KEY=$(echo "$OUT" | sed -n 's/^approval key: \(ed25519:[0-9a-f]*\).*/\1/p')
N=$("${TB2[@]}" --as ann | field "['actions'].__len__()")
[ "$N" -gt 0 ] || fail "once --detach returns, the children's actions should be listed without waiting ($N)"
[ -n "$KEY" ] && [ "$KEY" = "$("${TB2[@]}" --as ann | field "['approval_key']")" ] || fail "--detach should print the key the server signs with: $OUT"
[ "$(stat -c %a "$TOOLBOX_STATE_DIR/served/org/signing-key")" = 600 ] || fail "the kept signing key should be private"
git -C "$WORK/review/alpha" pull -q origin main
sed -i "s/ed25519:0\{64\}/$KEY/" "$WORK/review/alpha/.toolbox/toolbox.yaml"
git -C "$WORK/review/alpha" "${G[@]}" commit -qam "accept the org portal's approvals" && git -C "$WORK/review/alpha" push -q origin main
set +e; OUT=$("${TB2[@]}" add-security-policy --entity portal:alpha -i contact=security@acme.dev --as ann --yes --wait); CODE=$?; set -e
[ $CODE = 3 ] || fail "add-security-policy on a cloned alpha should await approval ($CODE): $OUT"
RID=$(echo "$OUT" | field "['id']")
OUT=$("${TB2[@]}" --approve "$RID" --as bob --note "alpha is ours" --wait) || fail "approve: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved add-security-policy on alpha: $OUT"
git --git-dir "$WORK/origin-alpha.git" show main:SECURITY.md | grep -q security@acme.dev || fail "SECURITY.md is not on alpha's main"
echo "$OUT" | grep -q "alpha is ours" || fail "the approval note was not kept: $OUT"
COC=(add-code-of-conduct --entity portal:mini -i contact=conduct@acme.dev)
set +e; OUT=$("${TB2[@]}" "${COC[@]}" --as ann --yes 2>&1); CODE=$?; set -e
[ $CODE = 2 ] && echo "$OUT" | grep -q "acceptApprovalsFrom" && echo "$OUT" | grep -q "$KEY" \
  || fail "before the pin, a request should be refused with the line to pin ($CODE): $OUT"
git -C "$WORK/review/mini" pull -q origin main
sed -i "s/ed25519:0\{64\}/$KEY/" "$WORK/review/mini/.toolbox/actions/add-code-of-conduct.yaml"
git -C "$WORK/review/mini" "${G[@]}" commit -qam "accept the org portal's approvals" && git -C "$WORK/review/mini" push -q origin main
"$BIN/toolbox" run --url "http://$API2" "${COC[@]}" --as ann | grep -q "CODE_OF_CONDUCT.md" || fail "the preview should show the file"
set +e; OUT=$("${TB2[@]}" "${COC[@]}" --as ann --yes); CODE=$?; set -e
[ $CODE = 3 ] || fail "add-code-of-conduct should await approval ($CODE): $OUT"
RID=$(echo "$OUT" | field "['id']")
for who in ann carol; do
  set +e; "${TB2[@]}" --approve "$RID" --as $who >/dev/null 2>&1; CODE=$?; set -e
  [ $CODE = 2 ] || fail "$who should not be able to approve ($CODE)"
done
OUT=$("${TB2[@]}" --approve "$RID" --as bob --wait) || fail "approve: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved add-code-of-conduct: $OUT"
git --git-dir "$WORK/hosts/mini.git" show main:CODE_OF_CONDUCT.md | grep -q conduct@acme.dev || fail "CODE_OF_CONDUCT.md is not on mini's main"
git --git-dir "$WORK/hosts/mini.git" log -1 --format=%B main | grep -q "Requested-By: user:ann" || fail "no Requested-By trailer"
git --git-dir "$WORK/hosts/mini.git" log -1 --format=%B main | grep -q "Approved-By: user:bob" || fail "no Approved-By trailer"
[ "$(git --git-dir "$WORK/hosts/mini.git" log -1 --format=%an main)" = Toolbox ] || fail "the org portal's clone should commit as Toolbox"
echo "$OUT" | field "['verification']['child']['files'][0]['matches']" | grep -q True || fail "verification should compare the landed file: $OUT"
OUT=$("${TB2[@]}" --verify "$RID" --as ann) || fail "verify: $OUT"
echo "$OUT" | grep -q "user:bob" || fail "the record should name the approver: $OUT"
"${TB2[@]}" --runs --as ann | grep -q "$RID" || fail "the run is not listed"
printf '#!/bin/sh\necho "main is protected"\nexit 1\n' >"$WORK/hosts/mini.git/hooks/pre-receive" && chmod +x "$WORK/hosts/mini.git/hooks/pre-receive"
set +e; OUT=$("${TB2[@]}" add-code-of-conduct --entity portal:mini -i contact=other@acme.dev --as ann --yes); set -e
RID=$(echo "$OUT" | field "['id']")
set +e; OUT=$("${TB2[@]}" --approve "$RID" --as bob --wait 2>&1); CODE=$?; set -e
[ $CODE = 1 ] && echo "$OUT" | grep -q "main is protected" || fail "a rejected push should fail the run with the origin's message ($CODE): $OUT"
echo "   minimal-repo: toolbox test ran its 5 action cases in a scratch copy, leaving its origin as it was"
echo "   minimal-repo: previewed, awaited bob (ann and carol refused), pushed CODE_OF_CONDUCT.md with trailers, recorded; a rejected push failed with the origin's message"
echo "   no connector, kind or key declared: 16 actions from 4 git children; a pull request branch on beta's origin; alpha pinned the generated key on its origin, the clone followed, and bob's approval pushed SECURITY.md to alpha's main"

echo "10. no server: the org portal holds approvals in its checkout, signed with the same key"
"$BIN/toolbox" serve "$WORK/gitorg" --stop >/dev/null; DETACHED=
rm "$WORK/hosts/mini.git/hooks/pre-receive"
[ "$(run "$WORK/gitorg" --as ann | field "['approval_key']")" = "$KEY" ] || fail "the checkout should sign with the key the server kept"
COC=(add-code-of-conduct --entity portal:mini -i contact=local@acme.dev)
set +e; OUT=$(run "$WORK/gitorg" "${COC[@]}" --as ann --yes); CODE=$?; set -e
RID=$(echo "$OUT" | field "['id']")
[ $CODE = 3 ] && echo "$OUT" | grep -q -- "--approve $RID" || fail "a request should await approval and say how to give it ($CODE): $OUT"
for who in ann carol; do
  set +e; run "$WORK/gitorg" --approve "$RID" --as $who >/dev/null 2>&1; CODE=$?; set -e
  [ $CODE = 2 ] || fail "$who should not be able to approve in the checkout ($CODE)"
done
git --git-dir "$WORK/hosts/mini.git" show main:CODE_OF_CONDUCT.md | grep -q local@acme.dev && fail "pushed before approval"
OUT=$(run "$WORK/gitorg" --approve "$RID" --as bob --note "fine offline" --wait) || fail "approve in the checkout: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved in the checkout: $OUT"
REFUSED=$(echo "$OUT" | python3 -c "import json,sys; print(' '.join(e['by'] for e in json.load(sys.stdin)['events'] if e['state'] == 'approval_refused'))")
[ "$REFUSED" = "user:ann user:carol" ] || fail "the refused approvals are not in the record: $REFUSED"
git --git-dir "$WORK/hosts/mini.git" show main:CODE_OF_CONDUCT.md | grep -q local@acme.dev || fail "CODE_OF_CONDUCT.md is not on mini's main"
git --git-dir "$WORK/hosts/mini.git" log -1 --format=%B main | grep -q "Approved-By: user:bob" || fail "no Approved-By trailer"
set +e; run "$WORK/gitorg" --approve "$RID" --as bob >/dev/null 2>&1; CODE=$?; set -e
[ $CODE = 2 ] || fail "a finished run should not be approved again ($CODE)"
set +e; OUT=$(run "$WORK/gitorg" add-code-of-conduct --entity portal:mini -i contact=never@acme.dev --as ann --yes); set -e
RID=$(echo "$OUT" | field "['id']")
set +e; OUT=$(run "$WORK/gitorg" --reject "$RID" --as bob); CODE=$?; set -e
[ $CODE = 1 ] && [ "$(echo "$OUT" | field "['state']")" = rejected ] || fail "reject in the checkout ($CODE): $OUT"
git --git-dir "$WORK/hosts/mini.git" show main:CODE_OF_CONDUCT.md | grep -q never@acme.dev && fail "a rejected run landed"
echo "   add-code-of-conduct on mini with no server: awaited bob (ann and carol refused, both recorded), signed with the kept key, pushed and verified; not approved twice; a rejected run did not land"

echo "11. no server: a group the org portal declares approves a child's action in its checkout"
cat >"$WORK/gitorg/groups.fga" <<'FGA'
model
  schema 1.1
type user
type group
  relations
    define member: [user, group#member]
FGA
cat >>"$WORK/gitorg/toolbox.yaml" <<'YAML'
authz:
  model: groups.fga
  tuples:
    - static:
        - {user: user:dave, relation: member, object: group:security}
        - {user: group:leads#member, relation: member, object: group:security}
        - {user: user:erin, relation: member, object: group:leads}
YAML
"$BIN/toolbox" validate "$WORK/gitorg" | tail -1
git -C "$WORK/review/mini" pull -q origin main
sed -i "s/approval: .*/approval: \"['group:security']\"/" "$WORK/review/mini/.toolbox/actions/add-code-of-conduct.yaml"
git -C "$WORK/review/mini" "${G[@]}" commit -qam "group:security approves" && git -C "$WORK/review/mini" push -q origin main
COC=(add-code-of-conduct --entity portal:mini -i contact=security@acme.dev)
set +e; OUT=$(run "$WORK/gitorg" "${COC[@]}" --as dave --yes); CODE=$?; set -e
RID=$(echo "$OUT" | field "['id']")
[ $CODE = 3 ] || fail "a group-approved request should await approval ($CODE): $OUT"
for who in dave carol; do
  set +e; run "$WORK/gitorg" --approve "$RID" --as $who >/dev/null 2>&1; CODE=$?; set -e
  [ $CODE = 2 ] || fail "$who should not be able to approve for group:security ($CODE)"
done
OUT=$(run "$WORK/gitorg" --approve "$RID" --as erin --wait) || fail "erin, in group:security through group:leads: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved by a nested member: $OUT"
git --git-dir "$WORK/hosts/mini.git" log -1 --format=%B main | grep -q "Approved-By: user:erin" || fail "no Approved-By trailer for erin"
echo "   add-code-of-conduct needing group:security: dave (the requester) and carol (not a member) refused; erin, a member through group:leads, approved and it landed"
echo "M5 acceptance: all checks passed"
