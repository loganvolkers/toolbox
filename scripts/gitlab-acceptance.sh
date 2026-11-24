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
cp -r "$EX/gitlab-org" "$WORK/org"
sed "s#https://gitlab.acme.dev#$GITLAB_URL#; s#acme/#$ORG/#g; s#group: acme\$#group: $ORG#" "$EX/gitlab-org/toolbox.yaml" > "$WORK/org/toolbox.yaml"
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

echo "4. inputs reach the child's job as data: quotes, \$VARS, \$(commands), newlines and GitLab's own \$[[ ]]"
cat > "$WORK/review/gamma/.toolbox/actions/fingerprint.yaml" <<'YAML'
actions:
  - name: fingerprint
    kind: Portal
    description: Print the SHA-256 of the text it was given.
    unverified: true
    inputs: {type: object, required: [text], properties: {text: {type: string}}}
    executor:
      type: command
      run: >-
        printf '{"sha256": "%s"}\n'
        "$(printf '%s' "$TOOLBOX_INPUT_TEXT" | sha256sum | cut -c1-64)"
YAML
git -C "$WORK/review/gamma" pull -q --rebase && git -C "$WORK/review/gamma" add -A
git -C "$WORK/review/gamma" commit -qm fingerprint && git -C "$WORK/review/gamma" push -q 2>/dev/null
TEXT=$(printf '%s\n%s' "it's \"quoted\" \$HOME \$(id) \`id\` \$[[ inputs.grant ]] %s {a: [b]}" 'second line: # not a comment')
set +e; OUT=$(run "$WORK/org" fingerprint --entity portal:gamma -i "text=$TEXT" --as ann --yes --wait); CODE=$?; set -e
[ $CODE = 4 ] && [ "$(echo "$OUT" | field "['state']")" = unverified ] || fail "fingerprint on gamma should end unverified ($CODE): $OUT"
WANT=$(printf '%s' "$TEXT" | sha256sum | cut -c1-64)
[ "$(echo "$OUT" | field "['child']['outputs']['sha256']")" = "$WANT" ] || fail "the job saw other text than was sent: $OUT"
PID=$(echo "$OUT" | field "['outputs']['pipeline_id']")
[ "$(gl GET "/projects/$GAMMA/pipelines/$PID" | field "['status']")" = success ] || fail "an unverified run should not fail its pipeline"
echo "   the job hashed exactly the text sent; unverified (exit 4: the job is a warning, the pipeline passed)"

echo "5. what a child can't choose, a project the forge entry doesn't admit, and a protected main"
sed -i 's/runner: {type: ci}/runner: {type: ci, project: someone\/else}/' "$WORK/review/gamma/.toolbox/toolbox.yaml"
git -C "$WORK/review/gamma" commit -qam "point the runner elsewhere" && git -C "$WORK/review/gamma" push -q 2>/dev/null
COC=(add-code-of-conduct --entity portal:gamma -i contact=other@acme.dev --as ann)
set +e; OUT=$(run "$WORK/org" "${COC[@]}" --yes 2>&1); CODE=$?; set -e
[ $CODE = 2 ] && echo "$OUT" | grep -q "portal:gamma offers no actions: .*project" || fail "a child naming its own project should be refused ($CODE): $OUT"
git -C "$WORK/review/gamma" revert --no-edit HEAD >/dev/null && git -C "$WORK/review/gamma" push -q 2>/dev/null
sed -i "s#projects: \[$ORG/\*, $ORG/\*/\*\]#projects: [$ORG/*]#" "$WORK/org/toolbox.yaml"
set +e; OUT=$(run "$WORK/org" "${COC[@]}" --yes 2>&1); CODE=$?; set -e
[ $CODE = 1 ] && echo "$OUT" | grep -q "does not list" || fail "a project outside forges[].projects should be refused ($CODE): $OUT"
sed -i "s#projects: \[$ORG/\*\]#projects: [$ORG/*, $ORG/*/*]#" "$WORK/org/toolbox.yaml"
gl DELETE "/projects/$GAMMA/protected_branches/main" >/dev/null
gl POST "/projects/$GAMMA/protected_branches" --data name=main --data push_access_level=0 --data merge_access_level=40 >/dev/null
set +e; OUT=$(run "$WORK/org" "${COC[@]}" --yes); CODE=$?; set -e
RID=$(echo "$OUT" | field "['id']")
set +e; OUT=$(run "$WORK/org" --approve "$RID" --as bob --wait 2>&1); CODE=$?; set -e
[ $CODE = 1 ] && echo "$OUT" | grep -qi "protected branch" || fail "a push GitLab refuses should fail the run with GitLab's message ($CODE): $OUT"
gl GET "/projects/$GAMMA/repository/files/CODE_OF_CONDUCT.md/raw?ref=main" | grep -q other@acme.dev && fail "the refused push landed"
echo "   refused: a child naming its project (its manifest is invalid, exit 2), a project outside projects: (exit 1),"
echo "   and a push to a protected main failed the run with GitLab's words: $(echo "$OUT" | grep -o "GitLab: [^\\]*" | head -1)"

echo "6. one org, two kinds of host: the plain git child runs in the org's clone"
git clone -q "$WORK/hosts/delta.git" "$WORK/review/delta"
sed -i "s/ed25519:0\{64\}/$KEY/" "$WORK/review/delta/.toolbox/actions/add-code-of-conduct.yaml"
git -C "$WORK/review/delta" commit -qam "accept the org portal's approvals" && git -C "$WORK/review/delta" push -q
set +e; OUT=$(run "$WORK/org" add-code-of-conduct --entity portal:delta -i contact=conduct@acme.dev --as ann --yes); set -e
RID=$(echo "$OUT" | field "['id']")
OUT=$(run "$WORK/org" --approve "$RID" --as bob --wait) || fail "approve on delta: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "add-code-of-conduct on delta: $OUT"
git --git-dir "$WORK/hosts/delta.git" show main:CODE_OF_CONDUCT.md | grep -q conduct@acme.dev || fail "CODE_OF_CONDUCT.md is not on delta's main"
echo "   delta (a bare repository, no forge): approved by bob, pushed and verified from the same org portal"

echo "7. the org portal's own action starts a pipeline: the gitlab-pipeline executor"
DEPLOY=$(project "$G" deploy); mkdir -p "$WORK/src/deploy"
cat > "$WORK/src/deploy/.gitlab-ci.yml" <<'YAML'
spec:
  inputs:
    environment: {options: [staging, production]}
    toolbox_run: {default: ""}
---
deploy:
  variables:
    ENVIRONMENT: $[[ inputs.environment ]]
    RUN: $[[ inputs.toolbox_run ]]
  script:
    - echo "deploying to $ENVIRONMENT for $RUN"
YAML
publish "$WORK/src/deploy" "$ORG/deploy"
cat >> "$WORK/org/toolbox.yaml" <<YAML
actions:
  - name: deploy
    kind: Portal
    description: Deploy a project, in the deploy project's pipeline.
    unverified: true
    inputs: {type: object, required: [environment], properties: {environment: {type: string}}}
    executor: {type: gitlab-pipeline, forge: "$GITLAB_URL", project: $ORG/deploy}
YAML
"$BIN/toolbox" validate "$WORK/org" | tail -1
set +e; OUT=$(run "$WORK/org" deploy --entity portal:gamma -i environment=staging --as ann --yes --wait); CODE=$?; set -e
[ $CODE = 4 ] && [ "$(echo "$OUT" | field "['state']")" = unverified ] || fail "deploy to staging ($CODE): $OUT"
JOB=$(gl GET "/projects/$DEPLOY/pipelines/$(echo "$OUT" | field "['outputs']['pipeline_id']")/jobs" | field "[0]['id']")
gl GET "/projects/$DEPLOY/jobs/$JOB/trace" | grep -q "deploying to staging for $(echo "$OUT" | field "['id']")" || fail "the job did not get the inputs"
set +e; OUT=$(run "$WORK/org" deploy --entity portal:gamma -i environment=moon --as ann --yes 2>&1); CODE=$?; set -e
[ $CODE = 1 ] && echo "$OUT" | grep -q "moon" || fail "an input the pipeline refuses should fail with GitLab's reason ($CODE): $OUT"
echo "   deploy: a pipeline with typed inputs, its job saw them; an option GitLab refuses failed the run with its reason"
echo "8. approvers are a GitLab group: the org portal reads it with its gitlab connector"
RELEASE=$(gl POST /groups --data name=release --data path=release --data "parent_id=$G" --data visibility=private | field "['id']")
for u in dave erin; do gl POST "/groups/$RELEASE/members" --data "user_id=$(user $u)" --data access_level=30 >/dev/null; done
gl POST "/groups/$RELEASE/members" --data "user_id=$(user carol)" --data access_level=10 >/dev/null
"$BIN/toolbox" connector test --name gitlab --portal "$WORK/org" -- "$BIN/toolbox-connector-gitlab" | tail -1
git -C "$WORK/review/delta" pull -q
sed -i "s#approval: .*#approval: \"['group:$ORG/release']\"#" "$WORK/review/delta/.toolbox/actions/add-code-of-conduct.yaml"
git -C "$WORK/review/delta" commit -qam "the release group approves" && git -C "$WORK/review/delta" push -q
set +e; OUT=$(run "$WORK/org" add-code-of-conduct --entity portal:delta -i contact=release@acme.dev --as dave --yes); CODE=$?; set -e
RID=$(echo "$OUT" | field "['id']")
[ $CODE = 3 ] || fail "a group-approved request should await approval ($CODE): $OUT"
for who in dave carol; do
  set +e; run "$WORK/org" --approve "$RID" --as $who >/dev/null 2>&1; CODE=$?; set -e
  [ $CODE = 2 ] || fail "$who should not approve for $ORG/release ($CODE)"
done
gl DELETE "/groups/$RELEASE/members/$(user erin)" >/dev/null
set +e; run "$WORK/org" --approve "$RID" --as erin >/dev/null 2>&1; CODE=$?; set -e
[ $CODE = 2 ] || fail "erin, removed from $ORG/release in GitLab, should not approve ($CODE)"
gl POST "/groups/$RELEASE/members" --data "user_id=$(user erin)" --data access_level=30 >/dev/null
OUT=$(run "$WORK/org" --approve "$RID" --as erin --wait) || fail "erin, back in $ORG/release: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved by a GitLab group member: $OUT"
git --git-dir "$WORK/hosts/delta.git" log -1 --format=%B main | grep -q "Approved-By: user:erin" || fail "no Approved-By trailer for erin"
echo "   group:$ORG/release from GitLab: dave (the requester) and carol (a Guest) refused; erin refused once removed in GitLab, then approved once re-added, and it landed"
echo "9. identity: true on the forge: who asks and who approves is whoever GitLab says owns their token"
sed -i '/^    type: gitlab$/a\    identity: true' "$WORK/org/toolbox.yaml"
"$BIN/toolbox" validate "$WORK/org" | tail -1
pat() { # pat NAME: a read_user token for NAME, in a file
  gl POST "/users/$(user "$1")/personal_access_tokens" --data name=identity --data "scopes[]=read_user" \
    --data "expires_at=$(date -d '+2 days' +%F)" | field "['token']" > "$WORK/$1.token"
}
pat dave; pat erin
NOTICE=(add-code-of-conduct --entity portal:delta -i contact=identity@acme.dev)
set +e; OUT=$(run "$WORK/org" "${NOTICE[@]}" --as dave --yes 2>&1); CODE=$?; set -e
[ $CODE = 2 ] && echo "$OUT" | grep -q "identity: true" || fail "--as alone should be refused ($CODE): $OUT"
set +e; OUT=$(run "$WORK/org" "${NOTICE[@]}" --as erin --identity-token-file "$WORK/dave.token" --yes 2>&1); CODE=$?; set -e
[ $CODE = 2 ] && echo "$OUT" | grep -q "user:dave" || fail "--as erin with dave's token should be refused ($CODE): $OUT"
set +e; OUT=$(run "$WORK/org" "${NOTICE[@]}" --identity-token-file "$WORK/dave.token" --yes); CODE=$?; set -e
[ $CODE = 3 ] && [ "$(echo "$OUT" | field "['requested_by']")" = user:dave ] || fail "dave's token should request as user:dave ($CODE): $OUT"
RID=$(echo "$OUT" | field "['id']")
set +e; run "$WORK/org" --approve "$RID" --as erin >/dev/null 2>&1; CODE=$?; set -e
[ $CODE = 2 ] || fail "an approval with --as erin alone should be refused ($CODE)"
set +e; TOOLBOX_IDENTITY_TOKEN=glpat-not-a-token run "$WORK/org" --approve "$RID" >/dev/null 2>&1; CODE=$?; set -e
[ $CODE = 2 ] || fail "a token GitLab refuses should be refused ($CODE)"
OUT=$(TOOLBOX_IDENTITY_TOKEN=$(cat "$WORK/erin.token") run "$WORK/org" --approve "$RID" --wait) || fail "erin's token: $OUT"
[ "$(echo "$OUT" | field "['state']")" = succeeded ] || fail "approved with erin's token: $OUT"
grep -rqs "$(cat "$WORK/erin.token")" "$TOOLBOX_STATE_DIR" && fail "a run record holds an identity token"
echo "   --as alone refused, --as against another's token refused, a bad token refused; dave's token requested, erin's approved, and no record holds a token"
echo "GitLab acceptance: all checks passed"
