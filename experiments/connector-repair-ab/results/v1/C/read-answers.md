# Phase 4 READ — Backstage arm C

Answers from arm artifacts only (`OUT/entities.ndjson`, `RUN.md`, `FORGE.md`, `resync.mjs`). No live Forge.

## 1. How many projects does tenant B have?

**1** — only `GAMMA` (`OUT/entities.ndjson`).

## 2. List issue keys on tenant B.

- `GAMMA-1`
- `GAMMA-2`

## 3. Who is assigned to GAMMA-1? (accountId)

`u-cara`

## 4. Name every component on GAMMA.

- `core` (catalog id `GAMMA:core`)

## 5. What config field switches tenants?

Env overrides (do not change sync code), per `RUN.md` / `resync.mjs`:

- `FORGE_BASE_URL` (ForgeConfig `baseUrl`) — tenant B: `http://127.0.0.1:9744`
- `FORGE_TOKEN` (ForgeConfig `token`) — tenant B: `tenant-b-token`

Both must change together to point at another tenant.

## DX feedback

Readable path for this READ: `OUT/entities.ndjson` already held tenant B after the prior resync, so Q1–Q4 were one-file lookups (kind/id/data). Tenant switch for Q5 is documented in `RUN.md`’s env table and mirrored in `resync.mjs` defaults — no code dive required. Mild friction: `OUT/diagnosis.md` still describes tenant-A bug symptoms, so an agent skimming OUT/ can briefly think the catalog is still broken/A-side; the live NDJSON is the source of truth for READ. Otherwise none.
