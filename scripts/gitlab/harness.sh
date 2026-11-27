#!/usr/bin/env bash
# A real GitLab on this machine: GitLab CE and a GitLab Runner in Docker, for the GitLab
# acceptance script and the actions-gitlab benchmark (docs/design/forges.md §6).
#
#   scripts/gitlab/harness.sh start   start GitLab without waiting for it to be ready
#   scripts/gitlab/harness.sh up      start (or reuse) GitLab and its runner; print the env file
#   scripts/gitlab/harness.sh env     print the env file of a running harness
#   scripts/gitlab/harness.sh down    remove both containers and their volumes
#
# The env file holds GITLAB_URL and GITLAB_ROOT_TOKEN (an admin token with `api` and `sudo`).
# The runner uses the shell executor on the host network, with SEIKIKA_BIN_DIR mounted at
# /seikika/bin, so a job runs `seikika` the way a self-hosted runner with Seikika installed would.
# Needs Docker and about 4 GB of free memory. GitLab's first boot takes several minutes.
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION=${GITLAB_VERSION:-19.6.1}
PORT=${GITLAB_PORT:-8929}
NAME=${GITLAB_NAME:-seikika-gitlab}
URL=http://127.0.0.1:$PORT
STATE=${GITLAB_HARNESS_DIR:-/tmp/seikika-gitlab}
ENVFILE=$STATE/env
# A fixed root token keeps the harness scriptable; it only ever exists on this machine.
ROOT_TOKEN=${GITLAB_ROOT_TOKEN:-glpat-seikika-harness-root-0001}
DOCKER=${DOCKER:-docker}
$DOCKER info >/dev/null 2>&1 || DOCKER="sudo docker"

ready() { [ "$(curl -s -o /dev/null -w '%{http_code}' -H "PRIVATE-TOKEN: $ROOT_TOKEN" "$URL/api/v4/version")" = 200 ]; }

start() {
  if ! $DOCKER inspect "$NAME" >/dev/null 2>&1; then
    $DOCKER run -d --name "$NAME" --shm-size 256m -p "127.0.0.1:$PORT:$PORT" \
      -e GITLAB_ROOT_PASSWORD="seikika-harness-$(date +%s)" \
      -e GITLAB_OMNIBUS_CONFIG="external_url '$URL'; nginx['listen_port'] = $PORT;
        puma['worker_processes'] = 0; sidekiq['concurrency'] = 8;
        prometheus_monitoring['enable'] = false; gitlab_kas['enable'] = false;
        registry['enable'] = false; gitlab_rails['usage_ping_enabled'] = false;
        gitlab_rails['gitlab_shell_ssh_port'] = 2222;
        gitlab_rails['initial_shared_runners_registration_token'] = nil" \
      "gitlab/gitlab-ce:$VERSION-ce.0" >/dev/null
  fi
}

up() {
  local bin
  bin=$(cd "${SEIKIKA_BIN_DIR:-target/debug}" && pwd)
  mkdir -p "$STATE"
  start
  echo "waiting for GitLab at $URL" >&2
  for _ in $(seq 1 120); do
    code=$(curl -s -o /dev/null -w '%{http_code}' "$URL/users/sign_in" || true)
    [ "$code" = 200 ] && break
    sleep 5
  done
  [ "$code" = 200 ] || {
    echo "GitLab did not answer at $URL" >&2
    exit 1
  }
  if ! ready; then
    $DOCKER exec "$NAME" gitlab-rails runner "
      u = User.find_by_username('root')
      t = u.personal_access_tokens.find_by(name: 'seikika-harness') ||
          u.personal_access_tokens.new(name: 'seikika-harness', scopes: %w[api sudo admin_mode],
                                       expires_at: 360.days.from_now)
      t.set_token('$ROOT_TOKEN'); t.save!
      ApplicationSetting.current.update!(signup_enabled: false, allow_local_requests_from_web_hooks_and_services: true)
    " >&2
  fi
  ready || {
    echo "the root token does not work" >&2
    exit 1
  }
  if ! $DOCKER inspect "$NAME-runner" >/dev/null 2>&1; then
    created=$(curl -sf -X POST -H "PRIVATE-TOKEN: $ROOT_TOKEN" "$URL/api/v4/user/runners" \
      --data runner_type=instance_type --data run_untagged=true --data description=seikika-harness)
    RUNNER_TOKEN=$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["token"])' "$created")
    $DOCKER run -d --name "$NAME-runner" --network host -v "$bin:/seikika/bin:ro" \
      "gitlab/gitlab-runner:ubuntu-v$VERSION" >/dev/null
    $DOCKER exec "$NAME-runner" gitlab-runner register --non-interactive --url "$URL" \
      --token "$RUNNER_TOKEN" --executor shell --name seikika-harness \
      --env PATH=/seikika/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin >&2
    $DOCKER exec "$NAME-runner" sed -i 's/^concurrent = .*/concurrent = 4/' /etc/gitlab-runner/config.toml
  fi
  printf 'GITLAB_URL=%s\nGITLAB_ROOT_TOKEN=%s\nGITLAB_CONTAINER=%s\n' "$URL" "$ROOT_TOKEN" "$NAME" >"$ENVFILE"
  cat "$ENVFILE"
}

case "${1:-}" in
  start) start ;;
  up) up ;;
  env) ready && cat "$ENVFILE" ;;
  down)
    $DOCKER rm -f -v "$NAME-runner" "$NAME" >/dev/null 2>&1 || true
    rm -rf "$STATE"
    ;;
  *)
    echo "usage: $0 start|up|env|down" >&2
    exit 2
    ;;
esac
