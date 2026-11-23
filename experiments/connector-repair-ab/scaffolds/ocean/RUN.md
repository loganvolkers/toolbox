# How to run (inherited connector — currently failing truth)

This tree fails the harness truth check. Diagnose in phase 1; fix in phase 2.

# How to run (arm B — Port Ocean / Forge)

## Configure

Defaults target **tenant A** (`FORGE.md`). Reuse the same connector for **tenant B**
with env overrides only — do not rewrite sync logic.

| Variable | Tenant A (default) | Tenant B |
|---|---|---|
| `FORGE_BASE_URL` | `http://127.0.0.1:9743` | `http://127.0.0.1:9744` |
| `FORGE_TOKEN` | `tenant-a-token` | `tenant-b-token` |

Ocean-style aliases also work: `OCEAN__INTEGRATION__CONFIG__BASEURL`,
`OCEAN__INTEGRATION__CONFIG__TOKEN`.

## Resync

From this directory:

```bash
# tenant A (defaults)
python3 resync.py

# tenant B (env overrides — same code path)
FORGE_BASE_URL=http://127.0.0.1:9744 FORGE_TOKEN=tenant-b-token python3 resync.py
```

Writes mapped catalog entities to `OUT/entities.ndjson` (sorted by `kind`, then `id`).
Uses stdlib only (no Port SaaS, no Ocean runtime install). Current `OUT/` is from
the last resync (tenant B when run with the overrides above).

## `@ocean.on_resync` kinds

Implemented in `integration.py` (via local `ocean_shim` — full `port_ocean` needs fastapi):

- `project` — `GET /rest/api/3/project` (paged `values`)
- `issue` — `GET /rest/api/3/search` (paged `issues`)
- `component` — per project `GET /rest/api/3/project/{key}/components` (bare JSON array; not paged)

Components are enriched with the parent project key before mapping (same idea as
Ocean jira's `_enrich_with_project`). Catalog id is `{project}:{name}` (e.g.
`ALPHA:api`); Forge's numeric component id goes in `data.id` only. Component
`data` is exactly `{id,name,project}` — no `description` or other extras.
