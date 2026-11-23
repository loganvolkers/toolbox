# How to run (arm C — Backstage EntityProvider)

## Configure Forge base URL and token

Defaults match tenant A in `FORGE.md`:

| Variable | Default | Meaning |
|---|---|---|
| `FORGE_BASE_URL` | `http://127.0.0.1:9743` | Forge tenant A |
| `FORGE_TOKEN` | `tenant-a-token` | Bearer token |
| `OUT_PATH` | `OUT/entities.ndjson` (under this arm) | Catalog output |

Override for tenant B later with `FORGE_BASE_URL=http://127.0.0.1:9744` and `FORGE_TOKEN=tenant-b-token`.

## Resync → OUT/entities.ndjson

From this directory (`/tmp/connector-ab/arms/C`):

```bash
npm run resync
# equivalent: node resync.mjs
```

That runs `src/resync-runner.ts` via Node `--experimental-strip-types` (no `npm install` / `tsc` required). It constructs `ForgeEntityProvider`, calls `connect` + `run`, and writes mapped `project` and `issue` records (phase 1; no components).

## Where ForgeEntityProvider is registered

- Implementation: `src/ForgeEntityProvider.ts` (`getProviderName` → `"forge"`, `connect`, `run` with full `applyMutation`).
- Harness registration / drive: `src/resync-runner.ts` (imported by `resync.mjs`).
- Package export: `src/index.ts` re-exports the provider for a real Backstage catalog module.
