## What changed
- Component sync: `forge_client.py` helpers, `@ocean.on_resync("component")`, `.port/spec.yaml`, `resync.py` KINDS, RUN.md.
- `OUT/entities.ndjson`: 8 entities.

## Could not do
- Full `port_ocean` (fastapi); local stub used.

## DX feedback
1. Ocean’s real Jira client is heavy to learn from for a bare-array Forge endpoint.
2. Catalog id `{project}:{name}` must be invented from the task schema.
3. Local stub vs real Ocean; httpx unused.
4. Pagination inconsistency across endpoints.

**Tool calls (transcript):** 26
**Agent:** [Connector A/B v2 phase3 B](bc-4b572909-11d8-5971-8b55-c918a5b57b0d)
