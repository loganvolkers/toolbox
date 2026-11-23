# Read answers (arm B, tenant B)

Source: `OUT/entities.ndjson` (last resync with tenant B env overrides) plus `RUN.md`, `FORGE.md`, `.port/spec.yaml`, `integration.py`. No Forge calls; no regeneration.

## 1. How many projects does tenant B have?

**1** — project `GAMMA` (`Gamma`, type `software`).

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

`u-cara`

## 4. Name every component on GAMMA.

- `core` (catalog id `GAMMA:core`)

## 5. Which config fields switch tenants?

`baseUrl` and `token` (spec names). Env equivalents:

| Spec field | Env | Ocean alias |
|---|---|---|
| `baseUrl` | `FORGE_BASE_URL` | `OCEAN__INTEGRATION__CONFIG__BASEURL` |
| `token` | `FORGE_TOKEN` | `OCEAN__INTEGRATION__CONFIG__TOKEN` |

Tenant B: `http://127.0.0.1:9744` + `tenant-b-token`. Defaults (without overrides) are tenant A.

## DX feedback

- **Catalog was enough:** All entity questions (1–4) answered from a single sorted `OUT/entities.ndjson` — kinds, ids, and assignee were self-explanatory; no need to re-hit Forge or open the client.
- **Tenant switch is documented twice:** `RUN.md` table and `.port/spec.yaml` both name `baseUrl`/`token`; `integration.py` confirms the four env names. Low ambiguity for question 5.
- **Friction:** `OUT/` does not stamp which tenant produced it — had to trust `RUN.md` (“Current OUT/ is from the last resync (tenant B…)”). A `tenant` or config fingerprint in the output would remove that guess.
- **Friction:** Component names live under `data.name` while catalog `id` is `{project}:{name}`; clear once you read `RUN.md`, but the ndjson alone needs that convention.
- **Budget-friendly:** Read-only path needed only docs + one entity dump; no Ocean runtime, no network.
