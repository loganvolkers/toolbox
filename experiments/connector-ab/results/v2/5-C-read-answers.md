# Read answers (tenant B)

Source: `OUT/entities.ndjson` (already synced for tenant B) and `RUN.md` / `src/resync.ts` for tenant-switch config.

## 1. How many projects does tenant B have?

**1** — project key `GAMMA`.

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

`u-cara`

## 4. Name every component on GAMMA.

- `core`

## 5. Which config fields switch tenants?

`FORGE_BASE_URL` and `FORGE_TOKEN` (constructor options `baseUrl` + `token` on `ForgeEntityProvider`). Tenant B: `http://127.0.0.1:9744` + `tenant-b-token`.

