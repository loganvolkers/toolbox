# Phase 4 READ — arm B (Ocean / Forge)

Answers from arm artifacts only (`OUT/entities.ndjson`, `RUN.md`, `.port/spec.yaml`, `integration.py`). No live Forge.

## 1. How many projects does tenant B have?

**1** — project `GAMMA` (`Gamma`, type `software`).

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1? (accountId)

`u-cara`

## 4. Name every component on GAMMA.

- `core` (catalog id `GAMMA:core`)

## 5. What config field switches tenants?

Ocean config fields **`baseUrl`** and **`token`** (`.port/spec.yaml`). Env overrides: `FORGE_BASE_URL` / `FORGE_TOKEN`, or Ocean aliases `OCEAN__INTEGRATION__CONFIG__BASEURL` / `OCEAN__INTEGRATION__CONFIG__TOKEN`. Tenant B: `http://127.0.0.1:9744` + `tenant-b-token` (`RUN.md`).

## DX feedback

none
