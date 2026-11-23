# Arm C (Backstage) — Phase 5 Read answers

Source: `OUT/entities.ndjson` (tenant B catalog already present), `RUN.md`, `FORGE.md`, `src/resync-runner.ts`. No resync; connector code unchanged.

## 1. How many projects does tenant B have?

**1** — project key `GAMMA` (`{"kind":"project","id":"GAMMA",...}`).

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

**`u-cara`** (`data.assignee` on the `GAMMA-1` issue record).

## 4. Name every component on GAMMA.

- **`core`** (id `10`, catalog id `GAMMA:core`)

## 5. Which config fields switch tenants?

**`FORGE_BASE_URL`** and **`FORGE_TOKEN`** (env → `ForgeEntityProvider` constructor `baseUrl` + `token`).

| Tenant | `FORGE_BASE_URL` | `FORGE_TOKEN` |
|---|---|---|
| A (default) | `http://127.0.0.1:9743` | `tenant-a-token` |
| B | `http://127.0.0.1:9744` | `tenant-b-token` |

`OUT_PATH` only changes where the NDJSON is written; it does not select a tenant.

## Could not do

Nothing required for these five answers. Did not call Forge or regenerate entities (per instructions). Catalog assumed to be tenant B from prior arm setup (GAMMA only).

## DX feedback (arm C Read)

- **Answerable from files alone:** Once `entities.ndjson` exists, project/issue/component/assignee questions are one-pass NDJSON scans; no Backstage catalog API needed in this harness.
- **Tenant identity is implicit:** The file does not label which tenant it came from. You infer B from keys (`GAMMA`) or from knowing which env was used for the last resync. A `tenant` field or filename would make Read safer.
- **Switching tenants is clear:** `RUN.md` documents the two env vars and the exact B command; constructor options match. No sync-logic edits required.
- **Friction:** Read depends on someone having already run resync for B into `OUT/`. Without that file (or with a stale A catalog), Read fails or silently answers the wrong tenant.
