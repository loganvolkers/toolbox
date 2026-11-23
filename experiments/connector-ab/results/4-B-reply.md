From [Connector A/B phase4 Ocean](bc-7898db12-af3f-57b6-bf41-8e71c69ffbcc):

No sync-logic rewrite. `FORGE_BASE_URL` / `FORGE_TOKEN` switch tenants. Wrote tenant B entities (4); updated `RUN.md`.

**DX:** Env-based reuse worked. Overwriting a single `OUT/entities.ndjson` means you must re-export to flip tenants (no side-by-side OUT). Defaults still hardcode tenant A.

**Tool calls:** 15 (transcript).
