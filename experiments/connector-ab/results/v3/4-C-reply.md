**What changed + resync:** No sync rewrite. `FORGE_BASE_URL` / `FORGE_TOKEN` already drove the provider. Ran `FORGE_BASE_URL=http://127.0.0.1:9744 FORGE_TOKEN=tenant-b-token npm run resync`. `OUT/entities.ndjson` is now tenant B: `GAMMA`, `GAMMA-1`, `GAMMA-2`, component `GAMMA:core` with data `{id,name,project}`. `RUN.md` documents the tenant-B env command.

**Blockers:** None.

**DX feedback:** Reuse was one env pair + the same `npm run resync`; component `{project}:{name}` mapping already matched FORGE.md, so tenant B was documentation + a command, not code.

**Tool-call count:** 16