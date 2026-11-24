#!/usr/bin/env bash
# Once per round (README §4 step 1): the frozen Toolbox binaries, the GitLab harness with its
# runner mounting them, the runner's job tools, glab, GitLab's own docs, the pinned forks, and the
# five people as GitLab users.
#
#   prepare.sh <src-dir>
#
# Writes into <src-dir>: bin/ (the frozen toolbox binaries), glab, gitlab-docs/, the forks'
# tarballs (actions-ab's repos.tsv), and gitlab.env. TOOLBOX_BIN_DIR (default target/debug) is
# copied, so a rebuild during the round changes neither the arms' bin/ nor the runner's.
set -euo pipefail
src=$1 here=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$here/../../.." && pwd)
mkdir -p "$src/bin"
while read -r _ fork sha; do
  [ -s "$src/$fork.tgz" ] || curl -fsSL -o "$src/$fork.tgz" "https://codeload.github.com/loganvolkers/$fork/tar.gz/$sha"
done < "$repo_root/experiments/actions-ab/harness/repos.tsv"
for b in "${TOOLBOX_BIN_DIR:-$repo_root/target/debug}"/toolbox "${TOOLBOX_BIN_DIR:-$repo_root/target/debug}"/toolbox-connector-*; do
  [ -f "$b" ] && [ -x "$b" ] && cp "$b" "$src/bin/"
done
glab=1.132.0 glab_sha=24d654844cae9f8f24193219ad481a4ebf01c1dc7340ea4107ee2bedd93f9e53
if [ ! -x "$src/glab" ]; then
  curl -fsSL -o "$src/glab.tgz" "https://gitlab.com/gitlab-org/cli/-/releases/v$glab/downloads/glab_${glab}_linux_amd64.tar.gz"
  echo "$glab_sha  $src/glab.tgz" | sha256sum -c --quiet
  tar xzf "$src/glab.tgz" -C "$src" bin/glab && mv "$src/bin/glab" "$src/glab" && rm "$src/glab.tgz"
fi
DOCKER=docker; $DOCKER info >/dev/null 2>&1 || DOCKER="sudo docker"
# The runner mounts the frozen binaries: recreate it if it mounts anything else.
mount=$($DOCKER inspect -f '{{range .Mounts}}{{if eq .Destination "/toolbox/bin"}}{{.Source}}{{end}}{{end}}' toolbox-gitlab-runner 2>/dev/null || true)
[ -n "$mount" ] && [ "$mount" != "$(cd "$src/bin" && pwd)" ] && $DOCKER rm -f toolbox-gitlab-runner >/dev/null
TOOLBOX_BIN_DIR="$src/bin" "$repo_root/scripts/gitlab/harness.sh" up > "$src/gitlab.env"
# Job tools beyond the runner image's bash, git and curl: what an operator's job image would have.
$DOCKER exec toolbox-gitlab-runner sh -c 'command -v python3 >/dev/null && command -v jq >/dev/null ||
  (apt-get update -qq && apt-get install -y -qq python3 jq >/dev/null)'
# GitLab's own documentation at the running version, Markdown only, uncurated except for the
# docs' contributor instructions (AGENTS.md, CLAUDE.md) and the guide to developing GitLab itself.
if [ ! -d "$src/gitlab-docs" ]; then
  rm -rf "$src/gitlab-docs.tmp" && mkdir -p "$src/gitlab-docs.tmp"
  $DOCKER exec toolbox-gitlab sh -c 'cd /opt/gitlab/embedded/service/gitlab-rails/doc &&
    find . -name "*.md" ! -path "./development/*" ! -name AGENTS.md ! -name CLAUDE.md | tar -c -T -' |
    tar -x -C "$src/gitlab-docs.tmp"
  mv "$src/gitlab-docs.tmp" "$src/gitlab-docs"
fi
. "$src/gitlab.env"
gl() { local m=$1 p=$2; shift 2; curl -sf -X "$m" -H "PRIVATE-TOKEN: $GITLAB_ROOT_TOKEN" "$GITLAB_URL/api/v4$p" "$@"; }
# Auto DevOps would start a pipeline on every push to a project with no .gitlab-ci.yml, which
# fails on this runner and holds it: off, so the only pipelines are the ones a run starts.
gl PUT /application/settings --data auto_devops_enabled=false >/dev/null
for u in ann bob carol dave erin; do
  gl GET "/users?username=$u" | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin) else 1)' ||
    gl POST /users --data "username=$u" --data "name=$u" --data "email=$u@acme.test" \
      --data "password=Pw-$(openssl rand -hex 12)" --data skip_confirmation=true >/dev/null
done
echo "prepared $src (GitLab $(gl GET /version | python3 -c 'import json,sys; print(json.load(sys.stdin)["version"])'))"
