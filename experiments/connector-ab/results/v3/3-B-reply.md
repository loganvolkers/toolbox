**What changed + resync**
- `integration.py` `map_component`: catalog `id` → `{project}:{name}`; `data` → `{id,name,project}` with Forge numeric in `data.id`; dropped `description`.
- `RUN.md`: documented that catalog id vs Forge id split.
- `python3 resync.py` → 8 entities; components are `ALPHA:api` / `ALPHA:web` / `BETA:billing` with `data.id` `1`/`2`/`3`. Projects/issues unchanged.

**Blockers**
- None.

**DX feedback**
- FORGE tip made the rematch clear (numeric → `data.id` only; omit `description`). Prior failure was easy once the tip named both mistakes.
- Ocean jira’s enrich-with-project pattern still fit; only the catalog id/data mapping needed the rematch fix.

**Tool-call count:** 14