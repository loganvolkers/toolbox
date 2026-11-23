## What changed
Added `component` via iter_components, `@ocean.on_resync("component")`, map_component, spec + RUN.md. 8 entities.

## Could not do
Real port_ocean still unavailable; kept ocean_shim.

## DX feedback
1. Ocean pin heavy for local resync; shim required.
2. API shape drift vs Ocean Jira (`/component` paginated vs Forge bare `/components`).
3. Components need a project loop (no global list).
4. Mapping lives outside Ocean handlers in resync.py.
5. Phase 1 scaffold made add-kind easy.

**Tool calls:** ~22.
