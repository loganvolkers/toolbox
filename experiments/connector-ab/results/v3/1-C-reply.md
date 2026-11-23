1. **Built / how to resync**
   - Implemented `ForgeEntityProvider` (`getProviderName`, `connect`, `run`) in `src/ForgeEntityProvider.ts`, following Backstage’s EntityProvider pattern (full mutation).
   - Syncs tenant A projects + issues from `http://127.0.0.1:9743` with `tenant-a-token`; maps to the experiment schema; writes `OUT/entities.ndjson` (sorted by kind, then id). No components.
   - `resync.mjs` constructs the provider, connects an NDJSON harness connection, and runs the resync.
   - Resync: `npm run resync` (or `node --experimental-strip-types resync.mjs`). Env overrides: `FORGE_BASE_URL`, `FORGE_TOKEN`. Details in `RUN.md`.

2. **Could not do:** nothing blocking for this phase.

3. **Developer-experience feedback**
   - Scaffold ships TypeScript but no `node_modules` / install step; had to use `node --experimental-strip-types` to run the provider without installing packages.
   - `RUN.md` was a stub; `resync.mjs` exited 2 until implemented — expected, but the strip-types requirement wasn’t documented in the scaffold.
   - Backstage pin path via `BACKSTAGE_ROOT.txt` was clear; `GithubEntityProvider` is a large reference for a minimal `connect`/`run` harness.

4. **Tool-call count:** 29