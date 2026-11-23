# Read answers (tenant B)

Source: `OUT/entities.ndjson`, `FORGE.md`, `RUN.md`, `.port/spec.yaml`.

1. **Projects on tenant B:** 1 (`GAMMA`)
2. **Issue keys on tenant B:** `GAMMA-1`, `GAMMA-2`
3. **Assignee of GAMMA-1 (account id):** `u-cara`
4. **Components on GAMMA:** `core`
5. **Config fields that switch tenants:**
   - Env (local resync): `FORGE_BASE_URL`, `FORGE_TOKEN`
   - Ocean config (`.port/spec.yaml` / `ocean.integration_config`): `baseUrl`, `token`
