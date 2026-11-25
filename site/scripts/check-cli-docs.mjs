#!/usr/bin/env node
/**
 * Drift check: re-load CLI catalog and ensure build-time fingerprint is stable.
 * Pages CI runs it as `npm run check:cli-docs`.
 *
 * This script invokes toolbox describe directly (mirrors cli-describe.ts logic)
 * so it does not require a TypeScript build step.
 */
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = resolve(siteRoot, '..');
const snapshotPath = join(siteRoot, 'src/data/cli-catalog.fingerprint.json');

function resolveToolbox() {
  if (process.env.TOOLBOX_BIN && existsSync(process.env.TOOLBOX_BIN)) {
    return process.env.TOOLBOX_BIN;
  }
  for (const c of [
    join(repoRoot, 'target/debug/toolbox'),
    join(repoRoot, 'target/release/toolbox'),
  ]) {
    if (existsSync(c)) return c;
  }
  const build = spawnSync('cargo', ['build', '-p', 'toolbox-cli', '--locked'], {
    cwd: repoRoot,
    encoding: 'utf8',
  });
  if (build.status !== 0) {
    console.error(build.stderr);
    process.exit(1);
  }
  return join(repoRoot, 'target/debug/toolbox');
}

function describe(toolbox) {
  const r = spawnSync(toolbox, ['describe'], { cwd: repoRoot, encoding: 'utf8' });
  if (r.status !== 0) {
    console.error(r.stderr || r.stdout);
    process.exit(1);
  }
  const commands = [];
  const mcpTools = [];
  for (const line of r.stdout.split(/\r?\n/)) {
    if (!line.trim()) continue;
    const rec = JSON.parse(line);
    if (rec.kind === 'command') {
      const help = spawnSync(toolbox, [...String(rec.data.name).split(/\s+/), '--help'], {
        cwd: repoRoot,
        encoding: 'utf8',
      });
      const helpText = (help.stdout || help.stderr || '').trimEnd();
      const usage =
        helpText
          .split(/\r?\n/)
          .find((l) => l.startsWith('Usage:'))
          ?.replace(/^Usage:\s*/, '')
          .trim() ?? null;
      commands.push({
        name: rec.data.name,
        about: rec.data.about ?? null,
        mcpTool: rec.data.mcpTool ?? null,
        args: (rec.data.args || []).map((a) => ({
          name: a.name,
          long: a.long,
          positional: a.positional,
        })),
        usage,
      });
    } else if (rec.kind === 'mcp-tool') {
      mcpTools.push({
        name: rec.data.name,
        cliCommands: rec.data.cliCommands || [],
      });
    }
  }
  commands.sort((a, b) => a.name.localeCompare(b.name));
  mcpTools.sort((a, b) => a.name.localeCompare(b.name));
  return { commands, mcpTools };
}

const mode = process.argv.includes('--write') ? 'write' : 'check';
const toolbox = resolveToolbox();
const catalog = describe(toolbox);
const body = `${JSON.stringify(catalog, null, 2)}\n`;

if (mode === 'write') {
  mkdirSync(dirname(snapshotPath), { recursive: true });
  writeFileSync(snapshotPath, body);
  console.log(`wrote ${snapshotPath}`);
  process.exit(0);
}

if (!existsSync(snapshotPath)) {
  console.error(`missing ${snapshotPath}; run: npm run generate:cli-docs`);
  process.exit(1);
}
const current = readFileSync(snapshotPath, 'utf8');
if (current !== body) {
  console.error(`${snapshotPath} is stale. Run: npm run generate:cli-docs`);
  process.exit(1);
}
console.log(`${snapshotPath} is up to date`);
