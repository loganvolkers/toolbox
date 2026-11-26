## What built
- `forge-connector` (`src/main.rs`) as a stdio `Source` via `run_stdio`: syncs Forge tenant A `project` + `issue`.
- `OUT/entities.ndjson`: 2 projects + 3 issues (sorted by kind, then id).
- `seikika connector test`: 11/11 pass.
- Resync via `RUN.md` (`cargo build` → `seikika connector drive` → map into `OUT/entities.ndjson`).

## Could not do
- Nothing required for this phase. Components intentionally omitted (phase 3).

## Developer-experience feedback
1. **`externalId` pattern `^[a-z0-9-]+:.+$`** is easy to miss: bare keys like `ALPHA` pass resync functionally but fail “every … message is valid”; scaffold/`RUN.md` don’t show a SaaS-key example.
2. **Two id spaces**: connector `externalId` (`issue:ALPHA-1`) vs catalog `id` (`ALPHA-1`) — mapping step isn’t called out next to the entities schema.
3. First `cargo build` pulled a large dep graph (~36s); fine once warm.
4. `connector drive` + `connector test` + scaffold/skills were clear and usable.

**Tool calls (transcript):** 36
**Agent:** [Connector A/B v2 phase1 A](bc-5133c227-6bc1-5efa-b07f-9c22a1bb6e30)
