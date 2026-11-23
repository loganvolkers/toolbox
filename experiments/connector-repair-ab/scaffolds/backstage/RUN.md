# How to run (inherited connector — currently failing truth)

This tree fails the harness truth check. Diagnose in phase 1; fix in phase 2.

# How to run

## Configure Forge

Defaults match tenant A in `FORGE.md`. Point at another tenant with env overrides only — do not change sync code.

| Env | Default (tenant A) | Tenant B reuse |
|---|---|---|
| `FORGE_BASE_URL` | `http://127.0.0.1:9743` | `http://127.0.0.1:9744` |
| `FORGE_TOKEN` | `tenant-a-token` | `tenant-b-token` |

## Resync

From this arm directory:

```bash
# Tenant A (defaults)
npm run resync

# Tenant B (same provider; env overrides only)
FORGE_BASE_URL=http://127.0.0.1:9744 FORGE_TOKEN=tenant-b-token npm run resync
```

That constructs `ForgeEntityProvider`, calls `connect` (which runs a full resync), and writes mapped `project` + `issue` + `component` rows to `OUT/entities.ndjson` (sorted by kind, then id).

Components come from `GET /rest/api/3/project/{key}/components` (bare JSON array, one request per project key).

**C3 rematch:** component catalog `id` is `{project}:{name}` (e.g. `ALPHA:api`). Forge's numeric component id is only in `data.id`. Component `data` is exactly `{id,name,project}` — no description.

## Where the provider is registered

- Implementation: `src/ForgeEntityProvider.ts` (`getProviderName`, `connect`, `run`).
- Harness entry: `resync.mjs` imports the provider, wires `createNdjsonConnection`, and calls `connect`.
- Re-export: `src/index.ts`.
