# Phase 1 diagnosis — Backstage arm C

Compared `npm run resync` → `OUT/entities.ndjson` to Forge tenant A (`FORGE.md`) and the target schema in `RUN.md` / prompt.

- **Issue `assignee` always null:** Forge search returns `fields.assignee.accountId` for assigned issues (`ALPHA-2` → `u-anne`, `BETA-1` → `u-bob`), but every issue row is written with `assignee: null`. Mapping must use `accountId` or `null` when unassigned (`ALPHA-1`).
- **Component catalog `id` is name-only:** Target id is `{project}:{name}` (e.g. `ALPHA:api`). Output uses bare names (`api`, `web`, `billing`), which can collide across projects and fails the catalog id rule.
- **Component `data` includes extra `description`:** Catalog component data must be exactly `{id,name,project}` with Forge’s numeric id only in `data.id`. Rows also emit `description` (e.g. `"HTTP API"`), which must be omitted.
