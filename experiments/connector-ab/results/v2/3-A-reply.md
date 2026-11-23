## What changed
- `src/main.rs`: sync `component` via bare-array components endpoint; `portal/toolbox.yaml` records + kind map; `RUN.md` drive kinds include component.
- `OUT/entities.ndjson`: 8 entities. `toolbox connector test` 12 passed.

## Could not do
- Nothing blocking.

## DX feedback
1. Catalog mapping from `connector drive` records is a second job — wants a sample mapper / `drive --map`.
2. Phase-3 API shape (bare array) differs from paged endpoints.
3. `connector drive` + `connector test` are excellent once built.
4. externalId vs catalog id still easy to confuse for components.

**Tool calls (transcript):** 23
**Agent:** [Connector A/B v2 phase3 A](bc-a2dd38c6-ec52-59ec-bd76-7b3bd57611a8)
