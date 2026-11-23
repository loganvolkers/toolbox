#!/usr/bin/env bash
# Bring up the local Git server, the GitHub façade, Postgres and (with --portal) the portal image.
# The benchmark harness stops before --portal: the agent starts the portal.
set -euo pipefail
cd "$(dirname "$0")"
root="$(cd ../../.. && pwd)"
portal=0
mirrors=0
for arg in "$@"; do
  case "$arg" in
    --portal) portal=1 ;;
    --mirrors) mirrors=1 ;;
    *) echo "usage: up.sh [--portal] [--mirrors]" >&2; exit 2 ;;
  esac
done

if ! docker info >/dev/null 2>&1; then
  echo "docker is not running" >&2
  exit 1
fi

pass_file=".secrets.env"
if [[ ! -f "$pass_file" ]]; then
  pass="$(openssl rand -hex 16)"
  umask 077
  printf 'FORGEJO_PASSWORD=%s\n' "$pass" > "$pass_file"
fi
# shellcheck disable=SC1090
source "$pass_file"
FORGEJO_PASSWORD="${FORGEJO_PASSWORD:?}"

docker compose up -d postgres forgejo registry
# Some Docker hosts keep an iptables-legacy FORWARD policy of DROP and never add the
# compose bridge to DOCKER-BRIDGE, so containers cannot reach each other by service name
# while published host ports still work. Accept same-bridge forwarding when that happens.
bridge="br-$(docker network inspect acme_default --format '{{.Id}}' | cut -c1-12)"
if [[ -d "/sys/class/net/${bridge}" ]] && command -v iptables-legacy >/dev/null; then
  sudo iptables-legacy -C FORWARD -i "$bridge" -o "$bridge" -j ACCEPT 2>/dev/null \
    || sudo iptables-legacy -I FORWARD 1 -i "$bridge" -o "$bridge" -j ACCEPT
fi
echo "waiting for forgejo"
for _ in $(seq 1 90); do
  if curl -sf "http://127.0.0.1:3030/api/healthz" >/dev/null; then
    break
  fi
  sleep 2
done
curl -sf "http://127.0.0.1:3030/api/healthz" >/dev/null

if ! docker compose exec -T -u 1000 forgejo forgejo admin user list 2>/dev/null | grep -q harness; then
  docker compose exec -T -u 1000 forgejo forgejo admin user create \
    --admin --username harness --password "$FORGEJO_PASSWORD" \
    --email harness@acme.test --must-change-password=false
fi

if [[ -z "${FORGEJO_TOKEN:-}" ]]; then
  token="$(docker compose exec -T -u 1000 forgejo forgejo admin user generate-access-token \
    --username harness --token-name facade --raw --scopes all)"
  token="$(printf '%s' "$token" | tr -d '\r\n')"
  printf 'FORGEJO_PASSWORD=%s\nFORGEJO_TOKEN=%s\n' "$FORGEJO_PASSWORD" "$token" > "$pass_file"
  # shellcheck disable=SC1090
  source "$pass_file"
fi

docker compose --env-file "$pass_file" up -d facade
echo "waiting for the façade"
for _ in $(seq 1 30); do
  if curl -sf "http://127.0.0.1:8091/healthz" >/dev/null; then
    break
  fi
  sleep 1
done
curl -sf "http://127.0.0.1:8091/healthz" >/dev/null

api() {
  curl -sf -H "Authorization: token ${FORGEJO_TOKEN}" -H "content-type: application/json" "$@"
}
if ! api "http://127.0.0.1:3030/api/v1/orgs/acme" >/dev/null; then
  api -X POST "http://127.0.0.1:3030/api/v1/orgs" -d '{"username":"acme"}' >/dev/null
fi

push_tree() {
  local name="$1" src="$2"
  local work
  work="$(mktemp -d)"
  cp -a "$src/." "$work/"
  git -C "$work" init -q -b main
  git -C "$work" config user.email harness@acme.test
  git -C "$work" config user.name Harness
  git -C "$work" add .
  git -C "$work" commit -q -m "seed ${name}"
  api -X POST "http://127.0.0.1:3030/api/v1/orgs/acme/repos" \
    -d "{\"name\":\"${name}\",\"auto_init\":false,\"default_branch\":\"main\"}" >/dev/null || true
  git -C "$work" remote add origin "http://harness:${FORGEJO_PASSWORD}@127.0.0.1:3030/acme/${name}.git"
  git -C "$work" push -q --force -u origin main
  rm -rf "$work"
  echo "pushed ${name}"
}

for repo in systems payments-api checkout-web ledger; do
  push_tree "$repo" "../seed/${repo}"
done

if [[ "$mirrors" -eq 1 ]]; then
  # The pins are fetched with --depth 1. Forgejo's git refuses that push unless the
  # receiving repo allows a shallow update.
  docker compose exec -T -u 1000 forgejo git config --global receive.shallowUpdate true
  mirror() {
    local name="$1" url="$2" pin="$3" extra="$4"
    local work
    work="$(mktemp -d)"
    git -C "$work" init -q
    git -C "$work" remote add origin "$url"
    git -C "$work" fetch -q --depth 1 origin "$pin"
    git -C "$work" checkout -q FETCH_HEAD
    if [[ -n "$extra" ]]; then
      mkdir -p "$work"
      printf '%s\n' "$extra" > "$work/catalog-info.yaml"
      git -C "$work" config user.email harness@acme.test
      git -C "$work" config user.name Harness
      git -C "$work" add catalog-info.yaml
      git -C "$work" commit -q -m "catalog-info for the deploy benchmark"
      echo "${name} catalog commit $(git -C "$work" rev-parse HEAD)"
    fi
    api -X POST "http://127.0.0.1:3030/api/v1/orgs/acme/repos" \
      -d "{\"name\":\"${name}\",\"auto_init\":false,\"default_branch\":\"main\"}" >/dev/null || true
    docker compose exec -T -u 1000 forgejo \
      git --git-dir="/data/git/repositories/acme/${name}.git" config receive.shallowUpdate true
    git -C "$work" remote add forge "http://harness:${FORGEJO_PASSWORD}@127.0.0.1:3030/acme/${name}.git"
    git -C "$work" push -q --force forge HEAD:refs/heads/main
    rm -rf "$work"
    echo "mirrored ${name}"
  }
  mirror backstage https://github.com/backstage/backstage.git feeadd4d4efcbda9f1573b95e356e0f3fc0b5d17 ""
  mirror openfga https://github.com/openfga/openfga.git d67ba7b1d2eaf945cffab3a9129ca1a4c7ad5251 \
    "$(cat <<'YAML'
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: openfga
spec:
  type: service
  lifecycle: production
  owner: group:platform
  system: checkout
YAML
)"
fi

if [[ "$portal" -eq 1 ]]; then
  stage="$(mktemp -d)"
  mkdir -p "$stage/bin" "$stage/portal/authz"
  cp "$root/target/debug/toolbox" \
    "$root/target/debug/toolbox-connector-github" \
    "$root/target/debug/toolbox-connector-git-files" \
    "$root/target/debug/toolbox-connector-scim" \
    "$stage/bin/"
  cp ../toolbox.yaml "$stage/portal/toolbox.yaml"
  cp -a ../authz/. "$stage/portal/authz/"
  cp Dockerfile entrypoint.sh scim_seed.sh "$stage/"
  docker build -t toolbox-acme:local "$stage"
  rm -rf "$stage"
  docker compose --env-file "$pass_file" --profile portal up -d portal
  echo "waiting for the portal"
  for _ in $(seq 1 90); do
    if curl -sf "http://127.0.0.1:8080/readyz" >/dev/null; then
      break
    fi
    sleep 2
  done
  curl -sf "http://127.0.0.1:8080/readyz" >/dev/null
  echo "waiting for the directory the image pushes"
  seeded=0
  for _ in $(seq 1 40); do
    code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 2 \
      -H "Authorization: Bearer scim-token" \
      "http://127.0.0.1:9100/scim/v2/Groups/g-platform" || true)"
    if [[ "$code" == "200" ]]; then
      seeded=1
      break
    fi
    sleep 1
  done
  if [[ "$seeded" -ne 1 ]]; then
    TOOLBOX_SCIM_TOKEN=scim-token ./scim_seed.sh "http://127.0.0.1:9100/scim/v2"
  fi
fi

install -d -m 700 /tmp/deploy-ab
umask 077
cat > /tmp/deploy-ab/env <<EOF
PORTAL_URL=http://127.0.0.1:8080
FORGEJO_URL=http://127.0.0.1:3030
FORGEJO_TOKEN=${FORGEJO_TOKEN}
FORGEJO_ORG=acme
FACADE_URL=http://127.0.0.1:8091
FACADE_TOKEN=acme-reader
GIT_USER=harness
GIT_PASSWORD=${FORGEJO_PASSWORD}
ADMIN_USER=bob
READER_USER=alice
OUTSIDER_USER=carol
EOF
echo "stack is up"
