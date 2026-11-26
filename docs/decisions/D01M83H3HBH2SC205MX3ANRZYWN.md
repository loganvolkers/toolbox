---
id: D01M83H3HBH2SC205MX3ANRZYWN
decided: 2026-11-23T06:25:00Z
---

# Composite catalog ids for `connector drive --format catalog`

**A package-author SDLC fix (not a benchmark-bar change).** Connector-ab v3
phase 3 failed truth on Seikika and Backstage when agents used Forge's numeric
component id as the catalog/`externalId` suffix (`1`) instead of
`{project}:{name}` (`ALPHA:api`). C3 requires fixing unclear docs and rematching
before scoring cost.

**Decision:**

1. Document in `author-connector` / `connector-drive` skills and the connector-ab
   scaffold/`FORGE.md` that composite kinds use `externalId`
   `component:{project}:{name}`, with the upstream id only in `data.id`.
2. When `--format catalog` sees `data.project` and `data.name` on a record, prefer
   `{project}:{name}` as the catalog `id` over a bare stripped `externalId` that
   looks like a lone numeric token — so a mistaken `component:1` still dumps a
   usable catalog line when the payload carries project/name.

**Consequences:** The C1–C3 bar is unchanged. Phase-3 arms that failed this id
rule are discarded and rematched after the fix.
