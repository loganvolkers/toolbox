# Phase 4 — Read answers (Toolbox arm A, tenant B)

Answers from `OUT/entities.ndjson`, `RUN.md`, `FORGE.md`, and portal/connector config only. No live Forge.

## 1. How many projects does tenant B have?

**1** — one project entity: `GAMMA` (`kind: project`, name Gamma).

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1? (accountId)

`u-cara`

## 4. Name every component on GAMMA.

- `core` (catalog id `GAMMA:core`)

## 5. What config field switches tenants?

`token` in the connector JSON config (`TOOLBOX_CONNECTOR_CONFIG` / `connector drive --config`). Tenant B uses `tenant-b-token` (tenant A used `tenant-a-token`). `baseUrl` also changes with the tenant (`http://127.0.0.1:9744` vs `…9743` per `FORGE.md` / last dump in `RUN.md`).

## DX feedback

Reuse of the phase-3 dump was enough for Q1–Q4; `RUN.md` last-dump block and `FORGE.md` made the tenant switch fields obvious without resync. Catalog NDJSON is easy to grep by `kind`. Minor friction: Q5 is singular “field” while both `token` and `baseUrl` change between tenants in the documented config.
