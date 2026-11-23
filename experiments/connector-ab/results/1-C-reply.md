## 1. What I built / how to resync

**Built** a Backstage-style `ForgeEntityProvider` under `/tmp/connector-ab/arms/C`:

- `src/ForgeEntityProvider.ts` — `getProviderName()` (`"forge"`), `connect()`, `run()`; pulls Forge projects + issues (paginated), applies a full `applyMutation`, writes experiment-schema lines to `OUT/entities.ndjson`
- `src/resync-runner.ts` + `resync.mjs` — harness entry that connects a stub catalog connection and runs the provider
- `RUN.md` — config + run instructions
- `package.json` — `"type": "module"` so Node strip-types runs cleanly

**Resync (tenant A defaults):**
```bash
cd /tmp/connector-ab/arms/C
npm run resync
```

**OUT/entities.ndjson** exists (5 lines).

## 2. Could not do

- No real Backstage catalog process — only the EntityProvider pattern + harness write path (scaffold has no Backstage runtime deps; TypeScript was not installed, so resync uses Node `--experimental-strip-types`).
- No components (phase 1 only).

## 3. Developer-experience feedback

1. **Scaffold ships TypeScript but no `node_modules` / no install step** — `package.json` lists `typescript` yet setup didn’t install it; had to use Node strip-types and avoid `tsc`.
2. **Dual output schemas** — Backstage `DeferredEntity` vs experiment `{kind,id,data}` NDJSON; the prompt requires both the EntityProvider pattern and a non-Backstage file format, which is easy to misread.
3. **“Sorted by kind then id”** vs example order (project → issue → component) — used lexicographic kind (`issue` before `project`); the example listing may imply a fixed kind order instead.
4. **Allowed Backstage pin paths are large** — `GithubEntityProvider.ts` is a heavy reference for a minimal connect/run provider; a short excerpt or stub interface in the scaffold would be faster.
5. **Network-only-to-loopback** is fine; Forge responses matched the sketched schema.

**Tool calls used:** ~28 (under the 50 budget).
