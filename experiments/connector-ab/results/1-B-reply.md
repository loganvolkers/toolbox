## What built
- `integration.py`, `ocean_shim.py`, `forge_client.py`, `resync.py`, `.port/spec.yaml`, `RUN.md`
- Shim used because full `port_ocean` needs fastapi
- `OUT/entities.ndjson`: 5 lines

## Could not do
- Could not import real `port_ocean` (fastapi/deps missing; no installs) → local shim

## Developer-experience feedback
1. **Ocean pin unusable as-is** — `from port_ocean.context.ocean import ocean` fails without fastapi/loguru/etc.; scaffold hints at vendoring but that undercuts “use Ocean”.
2. **requirements.txt lists `httpx` but it isn’t installed** and installs are discouraged — stdlib `urllib` was the only reliable path.
3. **Jira Ocean client ≠ Forge endpoints** — pin uses `/project/search` and `/search/jql`; Forge uses `/project` and `/search`. Following the pin literally would break.
4. **Handlers live in `main.py` in real Ocean integrations**, scaffold puts them in `integration.py` — small mismatch when copying patterns.
5. **No worked offline-resync example** in the readable pin paths for “write NDJSON without Port SaaS”.

**Tool calls:** 41.
