#!/usr/bin/env node
/**
 * Drive ForgeEntityProvider.run() and write OUT/entities.ndjson.
 *
 * Config (env, with tenant-A defaults from FORGE.md):
 *   FORGE_BASE_URL  default http://127.0.0.1:9743
 *   FORGE_TOKEN     default tenant-a-token
 */
import { pathToFileURL } from 'node:url';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const providerUrl = pathToFileURL(
  path.join(root, 'src', 'ForgeEntityProvider.ts'),
).href;

const { ForgeEntityProvider, createNdjsonConnection } = await import(
  providerUrl
);

const baseUrl = process.env.FORGE_BASE_URL || 'http://127.0.0.1:9743';
const token = process.env.FORGE_TOKEN || 'tenant-a-token';

const provider = new ForgeEntityProvider({ baseUrl, token, rootDir: root });
await provider.connect(createNdjsonConnection(provider));

const outPath = path.join(root, 'OUT', 'entities.ndjson');
console.log(`wrote ${outPath} via ${provider.getProviderName()}`);
