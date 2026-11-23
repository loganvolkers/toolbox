#!/bin/sh
# One container: the portal, then the SCIM connector once it can register.
# SIGTERM reaches the server, which exits on its own (the Cloud Run window).
set -eu
port="${PORT:-8080}"
toolbox serve /portal &
pid=$!
trap 'kill -TERM "$pid"; wait "$pid"' TERM INT
(
  i=0
  while [ "$i" -lt 90 ]; do
    if curl -sf "http://127.0.0.1:${port}/readyz" >/dev/null; then
      exec toolbox-connector-scim --toolbox "http://127.0.0.1:${port}" --listen 0.0.0.0:9100
    fi
    i=$((i + 1))
    sleep 1
  done
  echo "scim: portal did not become ready" >&2
) &
wait "$pid"
