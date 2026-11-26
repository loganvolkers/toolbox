# Phase 1 diagnosis — Seikika arm A (Forge connector)

Compared `seikika connector drive --format catalog` (tenant A) to FORGE.md + target schema and live Forge JSON on `127.0.0.1:9743`.

## Bugs

- **Component catalog `id` is bare name, not `{project}:{name}`.** Dump emits `"id":"api"`, `"web"`, `"billing"`; schema/FORGE require `ALPHA:api`, `ALPHA:web`, `BETA:billing` (Forge numeric id stays in `data.id` only).
- **Component `data` includes forbidden `description`.** Each component carries `"description":"…"`; allowed fields are exactly `{id,name,project}`.
- **Issue `assignee` is always `null`.** Forge `/search` returns `fields.assignee.accountId` `u-anne` (ALPHA-2) and `u-bob` (BETA-1); catalog still has `"assignee":null` for every issue (ALPHA-1 correctly null).

## Looks correct

- Projects: `ALPHA` / `BETA` with `key`, `name`, `type` (`software` / `business`).
- Issues: `key`, `summary`, `status`, `project` match Forge; all three issues present.
- Components: all three present (`api`, `web`, `billing`) with correct `data.id` / `name` / `project` values aside from the bugs above.
