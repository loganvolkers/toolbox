**What changed + resync:** No sync-logic rewrite. `integration.get_client()` already reads `FORGE_BASE_URL` / `FORGE_TOKEN`. Ran `FORGE_BASE_URL=http://127.0.0.1:9744 FORGE_TOKEN=tenant-b-token python3 resync.py` → `OUT/entities.ndjson` (4 entities: GAMMA project, GAMMA-1/2 issues, `GAMMA:core` component with `data={id,name,project}`). Updated `RUN.md` with tenant B override table/command.

**Blockers:** None. Tenant B on `:9744` responded; components map correctly.

**DX feedback:** Env overrides for multi-tenant reuse work well once documented. Local `ocean_shim` (pinned Ocean needs fastapi) is fine for this harness but is a gap vs real Ocean. Component id rule (`{project}:{name}` vs Forge numeric `data.id`) is easy to get wrong without `FORGE.md`.

**Tool-call count:** 12