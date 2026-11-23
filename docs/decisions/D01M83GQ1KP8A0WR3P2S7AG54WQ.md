---
id: D01M83GQ1KP8A0WR3P2S7AG54WQ
decided: 2026-11-23T06:20:00Z
---

# Catalog-shaped `connector drive` output

**A package-author SDLC fix (not a benchmark-bar change).** After the learning-tax
rerun ([RESULTS-v2](../../experiments/connector-ab/RESULTS-v2.md),
D01M83FFM2K6XS4EAVHVP92TBDZ) Toolbox held C1–C3, but agents still spent tool calls
hand-mapping drive NDJSON (`kind`/`externalId`/`data`) into catalog lines
(`kind`/`id`/`data`). That map script was the top remaining friction.

**Decision:** Extend `toolbox connector drive` with:

1. `--format catalog` — emit sorted `{kind,id,data}` NDJSON where `id` is
   `externalId` with a leading `{kind}:` stripped (so `project:ALPHA` → `ALPHA`,
   `component:ALPHA:api` → `ALPHA:api`).
2. `--out PATH` — write that NDJSON to a file (no shell redirect + edit loop).

Default `--format records` keeps the existing dump for protocol debugging.

**Consequences:** Package authors can dump a harness-shaped catalog in one command.
The connector-ab bar (C1–C3) is unchanged; a rematch measures whether call counts
drop further.
