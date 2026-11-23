# Phase 5 (Read) — tenant B answers

Source: `OUT/entities.ndjson`, `RUN.md`, `.port/spec.yaml`, `FORGE.md`. No Forge calls; no entity regeneration.

## 1. How many projects does tenant B have?

**1**

Entity: `{"kind":"project","id":"GAMMA",...}`

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

`u-cara`

## 4. Name every component on GAMMA.

- `core`

(entity id `GAMMA:core`)

## 5. Which config fields switch tenants?

- **Ocean / `.port/spec.yaml`:** `baseUrl` and `token`
- **Env (same meaning, per `RUN.md`):** `FORGE_BASE_URL` and `FORGE_TOKEN`

Tenant B values: `http://127.0.0.1:9744` + `tenant-b-token`.
