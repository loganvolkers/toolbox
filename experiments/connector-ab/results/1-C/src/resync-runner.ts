/**
 * Standalone resync entry: construct ForgeEntityProvider, connect a no-op
 * catalog connection (harness only needs OUT/entities.ndjson), then run().
 */
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  ForgeEntityProvider,
  type EntityProviderConnection,
} from './ForgeEntityProvider.ts';

const here = path.dirname(fileURLToPath(import.meta.url));
const armRoot = path.resolve(here, '..');

const baseUrl = process.env.FORGE_BASE_URL ?? 'http://127.0.0.1:9743';
const token = process.env.FORGE_TOKEN ?? 'tenant-a-token';
const outPath =
  process.env.OUT_PATH ?? path.join(armRoot, 'OUT', 'entities.ndjson');

const connection: EntityProviderConnection = {
  async applyMutation() {
    // Catalog engine not running in this harness; OUT write is authoritative.
  },
  async refresh() {},
};

const provider = new ForgeEntityProvider({ baseUrl, token, outPath });
await provider.connect(connection);
const records = await provider.run();
console.log(
  `forge provider=${provider.getProviderName()} wrote ${records.length} entities → ${outPath}`,
);
