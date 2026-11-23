# Phase 1 diagnosis (arm B — Ocean / Forge)

Evidence: `python3 -I resync.py` → 8 entities (tenant A defaults); compared to `FORGE.md` / RUN.md schema and live Forge JSON on `127.0.0.1:9743`.

- **Component catalog `id` is name-only, not `{project}:{name}`.** Output has `"id":"api"|"web"|"billing"`; FORGE.md / RUN.md require e.g. `ALPHA:api`, `ALPHA:web`, `BETA:billing`. Root cause: `map_component` in `integration.py` sets `"id": name` instead of `f"{project}:{name}"`.
- **Component `data` includes a forbidden `description` field.** Schema requires exactly `{id,name,project}`; every component row also has `"description": ...`. Root cause: `map_component` copies `raw.get("description")` into `data`.
- **Issue `assignee` is always `null`.** Forge search returns `assignee.accountId` for ALPHA-2 (`u-anne`) and BETA-1 (`u-bob`); mapped entities force `"assignee": null` for all issues. Root cause: `map_issue` computes `assignee` from `accountId` then hardcodes `"assignee": None` in the returned `data` (ignores the computed value).

Projects map correctly (`key`/`name`/`type` from `projectTypeKey`). Fetch/pagination and component project enrichment in `forge_client.py` look fine for tenant A; the three bugs above are all in the mappers in `integration.py`.
