#!/usr/bin/env bash
# GitLab acceptance (docs/design/forges.md §6): federated actions on a real GitLab started on this
# machine (scripts/gitlab/harness.sh: GitLab CE and a GitLab Runner in Docker). An org portal
# lists private GitLab projects and a plain git repository; changes become merge requests; a
# child that runs in its own CI gets a pipeline, whose job runs `toolbox run` with the parent's
# signed grant and keeps the run record as an artifact. No Postgres.
# Usage: scripts/gitlab-acceptance.sh   (starts the harness if it isn't running; KEEP=1 keeps $WORK)
set -euo pipefail
cd "$(dirname "$0")/.."
BIN=$(cd "${TOOLBOX_BIN_DIR:-target/debug}" && pwd); EX=$PWD/examples/federated-actions; WORK=$(mktemp -d)
export TOOLBOX_BIN_DIR=$BIN
ENVFILE=$(scripts/gitlab/harness.sh env 2>/dev/null || scripts/gitlab/harness.sh up)
eval "$(echo "$ENVFILE" | grep -E '^GITLAB_(URL|ROOT_TOKEN)=')"
HOST=${GITLAB_URL#http://}
export TOOLBOX_STATE_DIR=$WORK/state TOOLBOX_CONNECTORS_DIR=$BIN HOME=$WORK/home
mkdir -p "$HOME"
git config --global user.name acceptance && git config --global user.email acceptance@example.com
git config --global credential.helper store && git config --global init.defaultBranch main
cleanup() { [ -n "${KEEP:-}" ] && echo "kept $WORK" || rm -rf "$WORK"; }
trap cleanup EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }
field() { python3 -c "import json,sys; d=json.load(sys.stdin); print(eval('d'+sys.argv[1]))" "$1"; }
run() { (cd "$1" && shift && "$BIN/toolbox" run "$@" -o json); }
# gl METHOD PATH [curl args]: the GitLab API as root; prints the JSON body, fails on an HTTP error.
gl() { local m=$1 p=$2; shift 2; curl -sf -X "$m" -H "PRIVATE-TOKEN: $GITLAB_ROOT_TOKEN" "$GITLAB_URL/api/v4$p" "$@"; }
enc() { python3 -c "import sys, urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=''))" "$1"; }
user() { # user NAME: create (or find) a GitLab user and print its id
  gl GET "/users?username=$1" | field "[0]['id']" 2>/dev/null ||
    gl POST /users --data "username=$1" --data "name=$1" --data "email=$1@acme.test" \
      --data "password=Pw-$(openssl rand -hex 12)" --data skip_confirmation=true | field "['id']"
}
merge() { # merge PROJECT_ID IID [curl args]: once GitLab has worked out that it can
  for _ in $(seq 1 30); do
    [ "$(gl GET "/projects/$1/merge_requests/$2" | field "['detailed_merge_status']")" = mergeable ] && break
    sleep 1
  done
  gl PUT "/projects/$1/merge_requests/$2/merge" "${@:3}" >/dev/null
}
publish() { # publish DIR PROJECT: push DIR's files as PROJECT's main
  git -C "$1" init -q && git -C "$1" add -A && git -C "$1" commit -qm init
  git -C "$1" push -q "$GITLAB_URL/$2.git" HEAD:main 2>&1 | grep -v "^remote:" || true
}

ORG=acme-$(date +%s)
echo "0. a GitLab org ($ORG): users, a group and a subgroup, private projects with repo portals; a plain git repository"
for u in ann bob carol dave erin; do user $u >/dev/null; done
BOT=$(user toolbox)
BOT_TOKEN=$(gl POST "/users/$BOT/personal_access_tokens" --data name=acceptance --data "scopes[]=api" \
  --data "scopes[]=write_repository" --data "expires_at=$(date -d '+2 days' +%F)" | field "['token']")
printf 'http://toolbox:%s@%s\n' "$BOT_TOKEN" "$HOST" > "$HOME/.git-credentials"
G=$(gl POST /groups --data "name=$ORG" --data "path=$ORG" --data visibility=private | field "['id']")
PLATFORM=$(gl POST /groups --data name=platform --data path=platform --data "parent_id=$G" --data visibility=private | field "['id']")
gl POST "/groups/$G/members" --data "user_id=$BOT" --data access_level=40 >/dev/null
project() { # project NAMESPACE_ID NAME: a private project the bot can push to with its job token
  local id
  id=$(gl POST /projects --data "name=$2" --data "namespace_id=$1" --data visibility=private | field "['id']")
  gl PUT "/projects/$id" --data ci_push_repository_for_job_token_allowed=true >/dev/null
  echo "$id"
}
ALPHA=$(project "$G" alpha); GAMMA=$(project "$PLATFORM" gamma)
mkdir -p "$WORK/src"
cp -r "$EX/gitlab-repo" "$WORK/src/alpha" && cp -r "$EX/gitlab-repo" "$WORK/src/gamma"
rm "$WORK/src/alpha/.gitlab-ci.yml"
sed -i 's/name: gitlab-repo/name: alpha/; s/runner: {type: ci}/runner: {type: local}/' "$WORK/src/alpha/.toolbox/toolbox.yaml"
sed -i 's/name: gitlab-repo/name: gamma/' "$WORK/src/gamma/.toolbox/toolbox.yaml"
for r in alpha gamma; do echo "# $r" > "$WORK/src/$r/README.md"; done
publish "$WORK/src/alpha" "$ORG/alpha"; publish "$WORK/src/gamma" "$ORG/platform/gamma"
git init -q --bare -b main "$WORK/hosts/delta.git" && cp -r "$EX/minimal-repo" "$WORK/src/delta"
sed -i 's/name: minimal-repo/name: delta/' "$WORK/src/delta/.toolbox/toolbox.yaml"
git -C "$WORK/src/delta" init -q && git -C "$WORK/src/delta" add -A && git -C "$WORK/src/delta" commit -qm init
git -C "$WORK/src/delta" push -q "$WORK/hosts/delta.git" HEAD:main
mkdir -p "$WORK/org"
sed "s#https://gitlab.acme.dev#$GITLAB_URL#; s#acme/#$ORG/#g" "$EX/gitlab-org/toolbox.yaml" > "$WORK/org/toolbox.yaml"
echo "    - git: ../hosts/delta.git" >> "$WORK/org/toolbox.yaml"
export GITLAB_TOKEN=$BOT_TOKEN
"$BIN/toolbox" validate "$WORK/org" | tail -1

echo "1. the org portal lists every child's actions: two private GitLab projects and a plain git repository"
OUT=$(run "$WORK/org")
N=$(echo "$OUT" | field "['actions'].__len__()")
[ "$N" = 5 ] || fail "the org should offer 5 actions (alpha 2, gamma 2, delta 1), got $N: $OUT"
KEY=$(echo "$OUT" | field "['approval_key']")
echo "   5 actions from 3 children; the org signs with $KEY"

echo "2. a change on a GitLab child is a merge request; merged, the run succeeds"
OUT=$(run "$WORK/org" add-license --entity portal:alpha -i spdx=MIT --as ann --yes)
[ "$(echo "$OUT" | field "['state']")" = proposed ] || fail "add-license on alpha: $OUT"
RID=$(echo "$OUT" | field "['id']")
MR=$(echo "$OUT" | field "['outputs']['review']['url']") || fail "no merge request in the record: $OUT"
IID=${MR##*/}
[ "$(gl GET "/projects/$ALPHA/merge_requests/$IID" | field "['state']")" = opened ] || fail "the merge request is not open: $MR"
[ "$(run "$WORK/org" --verify "$RID" --as ann | field "['state']")" = proposed ] || fail "an open merge request should stay proposed"
merge "$ALPHA" "$IID"
[ "$(run "$WORK/org" --verify "$RID" --as ann | field "['state']")" = succeeded ] || fail "the merged merge request was not observed"
echo "   $MR: opened by the org portal, proposed while open, succeeded once merged"
OUT=$(run "$WORK/org" add-license --entity portal:alpha -i spdx=Apache-2.0 --as ann --yes)
RID=$(echo "$OUT" | field "['id']"); MR=$(echo "$OUT" | field "['outputs']['review']['url']"); IID=${MR##*/}
merge "$ALPHA" "$IID" --data squash=true
OUT=$(run "$WORK/org" --verify "$RID" --as ann)
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "a squash-merged merge request was not observed: $OUT"
echo "   $MR, squash-merged: succeeded"

echo "3. a child with runner {type: ci} runs in its own pipeline, with the org's signed approval"
git clone -q "$GITLAB_URL/$ORG/platform/gamma.git" "$WORK/review/gamma"
sed -i "s/ed25519:0\{64\}/$KEY/" "$WORK/review/gamma/.toolbox/toolbox.yaml"
git -C "$WORK/review/gamma" commit -qam "accept the org portal's approvals" && git -C "$WORK/review/gamma" push -q 2>/dev/null
set +e; OUT=$(run "$WORK/org" add-code-of-conduct --entity portal:gamma -i contact=conduct@acme.dev --as ann --yes); CODE=$?; set -e
[ $CODE = 3 ] || fail "add-code-of-conduct on gamma should await approval ($CODE): $OUT"
RID=$(echo "$OUT" | field "['id']")
set +e; OUT=$(run "$WORK/org" --approve "$RID" --as bob --wait); CODE=$?; set -e
[ $CODE = 0 ] && [ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved add-code-of-conduct on gamma ($CODE): $OUT"
PIPE=$(echo "$OUT" | field "['outputs']['pipeline']")
gl GET "/projects/$GAMMA/repository/files/CODE_OF_CONDUCT.md/raw?ref=main" | grep -q conduct@acme.dev || fail "CODE_OF_CONDUCT.md is not on gamma's main"
echo "$OUT" | field "['verification']['by']" | grep -qx "child pipeline" || fail "the verification should be the child pipeline's: $OUT"
echo "   $PIPE: ann requested, bob approved, gamma's own job ran it and pushed to main; the org read the record from the job's artifact"
echo "GitLab acceptance: all checks passed"
