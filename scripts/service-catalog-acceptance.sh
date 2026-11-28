#!/usr/bin/env bash
# Grafana in a real local container plus Cloudflare and GitHub API emulation.
set -euo pipefail

repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
kit=$repo/examples/service-catalog
dashboards=$repo/examples/cloudflare-containers/observability
bin=${SEIKIKA_BIN_DIR:-"$repo/target/debug"}
seikika=$bin/seikika
work=$(mktemp -d)
container=
emulator=
server=
cleanup() {
  local result=$?
  if ((result != 0)); then
    cat "$work/"*.log >&2
    if [[ -s $work/report.json ]]; then
      jq '{catalog, entities, federationStatus}' "$work/report.json" >&2 || true
    fi
    if [[ -n $container ]]; then docker logs --tail 40 "$container" >&2; fi
  fi
  if [[ -n $server ]]; then
    kill "$server" 2>/dev/null || true
    wait "$server" 2>/dev/null || true
  fi
  if [[ -n $emulator ]]; then
    kill "$emulator" 2>/dev/null || true
    wait "$emulator" 2>/dev/null || true
  fi
  if [[ -n $container ]]; then docker rm -f "$container" >/dev/null; fi
  rm -rf -- "$work"
}
trap cleanup EXIT
export PATH="$bin:$PATH" SEIKIKA_STATE_DIR="$work/state"
cd "$kit"

python3 emulator.py >"$work/emulator.port" 2>"$work/emulator.log" &
emulator=$!
for _ in {1..40}; do
  [[ -s $work/emulator.port ]] && break
  sleep .25
done
emulator_url="http://127.0.0.1:$(cat "$work/emulator.port")"
image='grafana/otel-lgtm:0.41.2@sha256:79dbbe04d810bfe93fd0310610a1643ba4ed420e7432143a810e71e9cba2541e'
container=$(docker run -d -p 127.0.0.1::3000 -p 127.0.0.1::9090 \
  -e GF_AUTH_ANONYMOUS_ENABLED=true -e GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer \
  -e GF_ANALYTICS_REPORTING_ENABLED=false -e GF_PLUGINS_PREINSTALL_DISABLED=true \
  --mount "type=bind,src=$dashboards/dashboard.yml,dst=/otel-lgtm/grafana/conf/provisioning/dashboards/grafana-dashboards.yaml,readonly" \
  --mount "type=bind,src=$dashboards/dashboard.json,dst=/otel-lgtm/seikika-runtime/dashboard.json,readonly" "$image")
grafana="http://$(docker port "$container" 3000/tcp)"
prometheus="http://$(docker port "$container" 9090/tcp)"
for _ in {1..60}; do
  if curl -fsS "$grafana/api/dashboards/uid/seikika-runtime" >"$work/dashboard.json" 2>/dev/null &&
    curl -fsS "$prometheus/-/ready" >/dev/null 2>&1; then break; fi
  sleep 1
done
jq -e '.dashboard.uid == "seikika-runtime"' "$work/dashboard.json" >/dev/null
python3 seed_metrics.py "$prometheus"

cloudflare_config=$(jq -nc --arg api "$emulator_url/client/v4" '{
  apiUrl:$api, accountId:("a"*32), token:"local-cloudflare", targets:{"seikika-demo@production":"seikika"}
}')
grafana_config=$(jq -nc --arg url "$grafana" '{
  url:$url, datasourceUid:"prometheus",
  targets:{"seikika-demo@production":{
    serviceName:"seikika-demo",dashboardUid:"seikika-runtime",
    metrics:[
      {id:"snapshot-lag",label:"Snapshot lag",unit:"s",expr:"max(seikika_snapshot_lag_seconds{service_name={{service}}})"},
      {id:"unreported",label:"Missing metric",unit:"s",expr:"test_unreported{service_name={{service}}}"},
      {id:"zero",label:"Zero metric",unit:"records",expr:"0 * max(seikika_snapshot_lag_seconds{service_name={{service}}})"}
    ]
  }}
}')
"$seikika" validate .
"$seikika" test "$repo/packages/service-catalog/tests"
"$seikika" connector test --name cloudflare --portal . --config "$cloudflare_config" -- "$bin/seikika-connector-cloudflare"
"$seikika" connector test --name grafana --portal . --config "$grafana_config" -- "$bin/seikika-connector-grafana"
failed_config=$(jq -c '.targets["seikika-demo@production"] = "unavailable"' <<<"$cloudflare_config")
if "$seikika" connector drive --kinds worker-deployments --config "$failed_config" -- "$bin/seikika-connector-cloudflare" >"$work/unavailable.ndjson" 2>"$work/unavailable.log"; then
  echo "Cloudflare 503 must fail the resync" >&2
  exit 1
fi
test ! -s "$work/unavailable.ndjson"

export SEIKIKA_CONNECTOR_CONFIG_OVERRIDES
SEIKIKA_CONNECTOR_CONFIG_OVERRIDES=$(jq -nc \
  --argjson cf "$cloudflare_config" --argjson gf "$grafana_config" --arg gh "$emulator_url" \
  '{cloudflare:$cf,grafana:$gf,github:{apiUrl:$gh}}')
export SEIKIKA_AUTH_MODE=cloudflare-access SEIKIKA_CF_TEAM_DOMAIN=https://test.invalid SEIKIKA_AUDIENCE=service-catalog-test
export SEIKIKA_PUBLIC_REPOSITORIES=seikika/seikika
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
portal="http://127.0.0.1:$port"
"$seikika" serve . --listen "127.0.0.1:$port" >"$work/server.log" 2>&1 &
server=$!
for _ in {1..80}; do
  if curl -fsS "$portal/v1/views/home?public_catalog=true" >"$work/report.json" 2>/dev/null &&
    jq -e '.federationStatus == null' "$work/report.json" >/dev/null; then break; fi
  sleep .25
done
jq -e --arg dashboard "$grafana/d/seikika-runtime" '
  .viewerMethod == "guest" and (.catalog.Service == ["service:seikika-demo"]) and
  (.entities["service:seikika-demo"] as $service |
    $service.fields.code == "seikika/seikika" and $service.fields.repository == "repository:seikika"
    and $service.relations.in == [{from:"service-environment:seikika-demo@production",relation:"environmentOf"}]) and
  .entities["repository:seikika"].fields.githubId == 42 and
  (.entities["service-environment:seikika-demo@production"].fields as $env |
    $env.worker == "seikika" and $env.serviceName == "seikika-demo" and
    ($env.deployments.recent[0] | .versions[1].percentage == 20 and
      .id == "22222222-2222-4222-8222-222222222222") and
    $env.metrics.dashboardUrl == $dashboard and
    ([$env.metrics.metrics[].status] == ["ok","no-data","ok"]) and
    ([$env.metrics.metrics[].value] == [12.5,null,0]) and
    ($env | has("cloudflareObservation") or has("grafanaObservation") | not))
' "$work/report.json" >/dev/null
curl -fsS "$portal/v1/entities?kind=Service&q=Seikika%20demo" |
  jq -e '[.items[].id] == ["service:seikika-demo"]' >/dev/null
curl -fsS "$portal/v1/views/entity/service-environment%3Aseikika-demo%40production" |
  jq -e '.entity.fields.metrics != null and (.entity.fields | has("grafanaObservation") | not)' >/dev/null
[[ $(curl -sS -o /dev/null -w '%{http_code}' "$portal/v1/views/ops") == 401 ]]
node browser.mjs "$portal" "$grafana"
echo "Service catalog accepted through Grafana, Cloudflare, GitHub, API, and browser."
