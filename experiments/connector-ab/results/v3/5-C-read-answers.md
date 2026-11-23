# Read answers (arm C, tenant B)

Source: `OUT/entities.ndjson` (already synced to tenant B) plus `RUN.md` / `resync.mjs` / `FORGE.md` for config.

## 1. How many projects does tenant B have?

**1** — project `GAMMA`.

## 2. List issue keys on tenant B

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1 (account id)?

`u-cara`

## 4. Name every component on GAMMA

- `core` (catalog id `GAMMA:core`)

## 5. Which config fields switch tenants?

Env (and matching `ForgeConfig` fields):

| Env | Config field | Tenant A (default) | Tenant B |
|---|---|---|---|
| `FORGE_BASE_URL` | `baseUrl` | `http://127.0.0.1:9743` | `http://127.0.0.1:9744` |
| `FORGE_TOKEN` | `token` | `tenant-a-token` | `tenant-b-token` |

Only those two; documented in `RUN.md` and wired in `resync.mjs`.

## DX feedback

- **Good:** Tenant switch is env-only (`FORGE_BASE_URL` + `FORGE_TOKEN`); no code edits. `RUN.md` table makes the A→B mapping obvious.
- **Good:** `entities.ndjson` is flat kind/id/data rows — grepping projects, issues, assignees, and components is one file, no schema hunt.
- **Friction:** Current `OUT/entities.ndjson` is already tenant B with no label in the file or filename; without `FORGE.md` / `RUN.md` you cannot tell which tenant the snapshot is. A tenant annotation or separate OUT path would help.
- **Friction:** Component “name” is under `data.name` while catalog `id` is `{project}:{name}` — clear once you read `RUN.md` C3 rematch note, less obvious from the NDJSON alone.
- **Friction:** Assignee is an opaque account id (`u-cara`) with no user entity in the catalog; fine for the question, but identity lookup would need another Forge call outside this arm’s OUT.
