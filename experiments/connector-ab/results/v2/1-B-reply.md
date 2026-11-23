## What built
- Ocean-style Forge connector for tenant A: `@ocean.on_resync("project")` and `@ocean.on_resync("issue")` in `integration.py`, Forge client (`forge_client.py`, stdlib `urllib`), thin `ocean_stub.py` (full Ocean needs fastapi), `.port/spec.yaml` with `baseUrl`/`token` + kinds, `resync.py` → `OUT/entities.ndjson` (5 entities).
- Run: `cd /tmp/connector-ab/arms/B && python3 resync.py`

## Could not do
- Import real `port_ocean` (fastapi missing; no installs). Stub preserves decorator shape. No `component` (phase 3).

## Developer-experience feedback
1. Ocean pin unusable without its dependency tree; scaffold’s “vendor a thin copy” note was essential but easy to miss.
2. Real Jira integration uses `/project/search` and `/search/jql`; Forge documents older `/project` and `/search` — mismatch vs the pin.
3. `requirements.txt` lists `httpx` but setup did not install it; stdlib was the workable path.
4. Handlers live in `main.py` in Ocean examples, but the task requires them in `integration.py`.

**Tool calls (transcript):** 37 (agent reported ~18)
**Agent:** [Connector A/B v2 phase1 B](bc-0c51db1a-e909-51c7-9492-4a8c35124dd9)
