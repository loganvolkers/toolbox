## What changed
Added `component` sync: GET /rest/api/3/project/{key}/components, catalog id `{project}:{name}`. Updated portal + RUN.md. OUT/entities.ndjson: 8 lines. `seikika connector test` 12/12.

## Could not do
Nothing required.

## DX feedback
1. Components endpoint is a bare array (not `{values,isLast}`) — easy to misuse the project paginator.
2. Two outputs (stdio records vs FORGE_WRITE_ENTITIES NDJSON) split mapping.
3. Toolchain pin still easy to miss.
4. Catalog id ≠ external id (`ALPHA:api` vs `forge:ALPHA:api`).

**Tool calls:** 12 (v2 agent bc-3da53731; truncated launch bc-1f45cf68 discarded per protocol).
