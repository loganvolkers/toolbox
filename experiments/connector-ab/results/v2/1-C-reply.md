## What built
- Backstage-style `ForgeEntityProvider` (`getProviderName`, `connect`, `run`) mapping Forge tenant A projects + issues to the experiment schema → `OUT/entities.ndjson` (5 entities).
- Resync: `npm run resync` / `node resync.mjs` from `/tmp/connector-ab/arms/C`.

## Could not do
- Nothing blocking. Components left off (phase 3). No real Backstage catalog process—stub `EntityProviderConnection` only.

## Developer-experience feedback
1. No install path / no README while `package.json` lists TypeScript and `build` uses `tsc || true`—had to use Node `--experimental-strip-types` and add `"type":"module"`.
2. Scaffold `resync.mjs` is a hard fail stub with little guidance on wiring `run()` vs writing NDJSON.
3. Sort order (“by kind then id”) isn’t shown in the example block.
4. Backstage pin is large; useful surface is small vs reading the whole provider.

**Tool calls (transcript):** 38 (agent reported ~37)
**Agent:** [Connector A/B v2 phase1 C](bc-22217758-3292-54f0-b08c-b3f6e652cce4)
