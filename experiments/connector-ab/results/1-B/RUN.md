# How to run (tenant A, phase 1)

## Configure

Defaults target tenant A. Override with env:

```bash
export FORGE_BASE_URL=http://127.0.0.1:9743
export FORGE_TOKEN=tenant-a-token
```

`.port/spec.yaml` declares the same `baseUrl` / `token` configuration fields.

## Resync → OUT/entities.ndjson

From this directory (stdlib only; no extra installs):

```bash
cd /tmp/connector-ab/arms/B
python3 resync.py
```

Writes mapped `project` and `issue` entities to `OUT/entities.ndjson`, sorted by kind then id.

## `@ocean.on_resync` kinds

Implemented in `integration.py` (via local `ocean_shim` — full Port Ocean needs fastapi):

- `project` — `GET /rest/api/3/project`
- `issue` — `GET /rest/api/3/search`

`component` is deferred to phase 3.
