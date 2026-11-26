#!/bin/sh
# One container: the portal, the SCIM connector, then the directory this image ships.
# SIGTERM reaches the server, which exits on its own (the Cloud Run window).
set -eu
port="${PORT:-8080}"
seikika serve /portal &
pid=$!
trap 'kill -TERM "$pid"; wait "$pid"' TERM INT
(
  i=0
  while [ "$i" -lt 90 ]; do
    if curl -sf "http://127.0.0.1:${port}/readyz" >/dev/null; then
      seikika-connector-scim --seikika "http://127.0.0.1:${port}" --listen 0.0.0.0:9100 &
      scim=$!
      j=0
      while [ "$j" -lt 30 ]; do
        code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 2 \
          -H "Authorization: Bearer ${SEIKIKA_SCIM_TOKEN:-}" \
          "http://127.0.0.1:9100/scim/v2/Users/u-alice" || true)
        if [ -n "$code" ] && [ "$code" != "000" ]; then
          break
        fi
        j=$((j + 1))
        sleep 1
      done
      if [ -x /usr/local/bin/scim_seed.sh ]; then
        /usr/local/bin/scim_seed.sh "http://127.0.0.1:9100/scim/v2" || echo "scim: directory push failed" >&2
      fi
      wait "$scim"
      exit
    fi
    i=$((i + 1))
    sleep 1
  done
  echo "scim: portal did not become ready" >&2
) &
wait "$pid"
