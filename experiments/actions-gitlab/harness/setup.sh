#!/usr/bin/env bash
# One workspace for one run of one arm (README §4): a private GitLab group with the three forks
# as projects and an empty `org` project, the release subgroup, the people's tokens, checkouts,
# and the arm's platform.
#
#   setup.sh <arm: seikika|gitlab> <dest> <src-dir> <k>
#
# <src-dir> is what prepare.sh wrote. <k> numbers the run's GitLab group, acme-w<k>, and its
# platform engineer, ops-w<k>. Beside the workspace it writes <dest>.baseline.json (score.py),
# <dest>.settings.json (settings.py) and <dest>.group.
set -euo pipefail
arm=$1 dest=$2 src=$3 k=$4
here=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$here/../../.." && pwd)
[ -e "$dest" ] && { echo "refusing: $dest exists" >&2; exit 1; }
. "$src/gitlab.env"
group=acme-w$k ops=ops-w$k host=${GITLAB_URL#http://}
gl() { local m=$1 p=$2; shift 2; curl -sf -X "$m" -H "PRIVATE-TOKEN: $GITLAB_ROOT_TOKEN" "$GITLAB_URL/api/v4$p" "$@"; }
field() { python3 -c "import json,sys; d=json.load(sys.stdin); print(eval('d'+sys.argv[1]))" "$1"; }
uid() { gl GET "/users?username=$1" | field "[0]['id']"; }
token() { # token USER_ID NAME: an api token for a person, valid for the round
  gl POST "/users/$1/personal_access_tokens" --data "name=$2" --data "scopes[]=api" \
    --data "expires_at=$(date -d '+7 days' +%F)" | field "['token']"
}
gl GET "/groups/$group" >/dev/null 2>&1 && { echo "refusing: group $group exists" >&2; exit 1; }
mkdir -p "$dest/people" "$dest/home" "$dest/bin"

OPS=$(gl GET "/users?username=$ops" | field "[0]['id']" 2>/dev/null ||
  gl POST /users --data "username=$ops" --data "name=$ops" --data "email=$ops@acme.test" \
    --data "password=Pw-$(openssl rand -hex 12)" --data skip_confirmation=true | field "['id']")
G=$(gl POST /groups --data "name=$group" --data "path=$group" --data visibility=private | field "['id']")
R=$(gl POST /groups --data name=release --data path=release --data "parent_id=$G" --data visibility=private | field "['id']")
gl POST "/groups/$G/members" --data "user_id=$OPS" --data access_level=50 >/dev/null
for p in ann:40 bob:40 carol:30 dave:30 erin:30; do
  gl POST "/groups/$G/members" --data "user_id=$(uid "${p%%:*}")" --data "access_level=${p#*:}" >/dev/null
done
for u in dave erin; do gl POST "/groups/$R/members" --data "user_id=$(uid $u)" --data access_level=30 >/dev/null; done
# root created both groups, which made it a direct Owner of each: the release group is dave and erin.
ROOT=$(uid root)
gl DELETE "/groups/$R/members/$ROOT" >/dev/null && gl DELETE "/groups/$G/members/$ROOT" >/dev/null
token "$OPS" "$group" > "$dest/people/ops.token"
for u in ann bob carol dave erin; do token "$(uid $u)" "$group" > "$dest/people/$u.token"; done
chmod 600 "$dest"/people/*.token

# git as ops, for every command run with HOME=$dest/home.
cat > "$dest/home/.gitconfig" <<EOF
[user]
	name = $ops
	email = $ops@acme.test
[credential]
	helper = store --file=$dest/home/.git-credentials
[init]
	defaultBranch = main
[commit]
	gpgsign = false
[core]
	fsmonitor = false
EOF
printf 'http://%s:%s@%s\n' "$ops" "$(cat "$dest/people/ops.token")" "$host" > "$dest/home/.git-credentials"
chmod 600 "$dest/home/.git-credentials"
export HOME=$dest/home

import() { # <name> <tarball> <upstream sha> <fork>: a project in the group, its main the fork
  local tmp; tmp=$(mktemp -d)
  gl POST /projects --data "name=$1" --data "path=$1" --data "namespace_id=$G" --data visibility=private >/dev/null
  tar xzf "$2" -C "$tmp" --strip-components=1
  git -C "$tmp" init -q -b main && git -C "$tmp" add -A
  GIT_AUTHOR_DATE=2026-11-24T00:00:00Z GIT_COMMITTER_DATE=2026-11-24T00:00:00Z \
    git -C "$tmp" -c user.name=Upstream -c user.email=upstream@example.com \
    commit -qm "import loganvolkers/${4}@${3}"
  git -C "$tmp" push -q "$GITLAB_URL/$group/$1.git" main 2>&1 | grep -v '^remote:' || true
  rm -rf "$tmp"
  git clone -q "$GITLAB_URL/$group/$1.git" "$dest/$1"
}
while read -r name fork sha; do
  import "$name" "$src/$fork.tgz" "$sha" "$fork"
done < "$repo_root/experiments/actions-ab/harness/repos.tsv"
gl POST /projects --data name=org --data path=org --data "namespace_id=$G" --data visibility=private >/dev/null
git clone -q "$GITLAB_URL/$group/org.git" "$dest/org" 2>/dev/null

cp "$src/glab" "$dest/bin/glab"
cp -r "$src/gitlab-docs" "$dest/gitlab-docs"
case $arm in
  seikika)
    cp "$src"/bin/* "$dest/bin/"
    mkdir -p "$dest/seikika-docs/examples"
    cp -r "$repo_root/plugins/seikika-operator/skills" "$dest/seikika-docs/skills"
    cp -r "$repo_root/examples/federated-actions" "$repo_root/examples/repo-portal-starter" \
      "$dest/seikika-docs/examples/"
    cp "$repo_root/crates/seikika-model/schema/manifest.schema.json" "$dest/seikika-docs/"
    ;;
  gitlab) ;;
  *) echo "unknown arm $arm" >&2; exit 2 ;;
esac

echo "$group" > "$dest.group"
python3 "$here/score.py" snapshot "$dest" > "$dest.baseline.json"
python3 "$here/settings.py" "$dest" > "$dest.settings.json"
echo "$dest ($arm, $group)"
