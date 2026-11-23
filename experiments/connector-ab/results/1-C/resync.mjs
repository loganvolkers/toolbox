#!/usr/bin/env node
/**
 * Drive ForgeEntityProvider.run() and write OUT/entities.ndjson.
 *
 * Config (env, with tenant-A defaults from FORGE.md):
 *   FORGE_BASE_URL  default http://127.0.0.1:9743
 *   FORGE_TOKEN     default tenant-a-token
 *   OUT_PATH        default <cwd>/OUT/entities.ndjson
 *
 * Uses Node's built-in TypeScript strip-types so no tsc / npm install is required.
 */
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const runner = path.join(here, 'src', 'resync-runner.ts');

const result = spawnSync(
  process.execPath,
  ['--experimental-strip-types', runner],
  {
    cwd: here,
    stdio: 'inherit',
    env: process.env,
  },
);

process.exit(result.status ?? 1);
