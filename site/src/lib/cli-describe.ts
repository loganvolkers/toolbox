/**
 * Build-time catalog loader for the Starlight CLI and MCP references.
 *
 * Invokes `seikika describe` (+ optional --help enrichment) from the repo root.
 */
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CliArg, CliCatalog, CliCommand, McpTool } from './cli-catalog';

// Astro also bundles this loader under dist/.prerender during a docs build.
function repoRoot(start: string): string {
  let directory = start;
  while (directory !== dirname(directory)) {
    if (existsSync(join(directory, "Cargo.toml")) && existsSync(join(directory, "site/package.json"))) return directory;
    directory = dirname(directory);
  }
  throw new Error("Could not locate the Seikika repository for the CLI reference");
}
export const REPO_ROOT = repoRoot(dirname(fileURLToPath(import.meta.url)));

function resolveSeikika(): string {
  const env = process.env.SEIKIKA_BIN;
  if (env && existsSync(env)) return env;
  for (const c of [
    join(REPO_ROOT, 'target/debug/seikika'),
    join(REPO_ROOT, 'target/release/seikika'),
  ]) {
    if (existsSync(c)) return c;
  }
  const which = spawnSync('which', ['seikika'], { encoding: 'utf8' });
  if (which.status === 0 && which.stdout.trim()) return which.stdout.trim();
  const build = spawnSync('cargo', ['build', '-p', 'seikika-cli', '--locked'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
  });
  if (build.status !== 0) {
    throw new Error(`cargo build failed:\n${build.error?.message ?? build.stderr}`);
  }
  const bin = join(REPO_ROOT, 'target/debug/seikika');
  if (!existsSync(bin)) throw new Error('seikika binary missing after build');
  return bin;
}

function runSeikika(seikika: string, args: string[]): string {
  const r = spawnSync(seikika, args, { cwd: REPO_ROOT, encoding: 'utf8' });
  if (r.status !== 0) {
    throw new Error(`seikika ${args.join(' ')} failed:\n${r.stderr || r.stdout}`);
  }
  return r.stdout;
}

function parseUsage(help: string): string | null {
  const line = help.split(/\r?\n/).find((l) => l.startsWith('Usage:'));
  return line ? line.replace(/^Usage:\s*/, '').trim() : null;
}

/** Load catalog from live CLI. Call only at build / check time (Node). */
export function loadCliCatalog(opts: { enrichHelp?: boolean } = {}): CliCatalog {
  const enrichHelp = opts.enrichHelp !== false;
  const seikika = resolveSeikika();
  const raw = runSeikika(seikika, ['describe']);
  const commands: CliCommand[] = [];
  const mcpTools: McpTool[] = [];

  for (const line of raw.split(/\r?\n/)) {
    if (!line.trim()) continue;
    const rec = JSON.parse(line) as {
      kind: string;
      data: Record<string, unknown>;
    };
    if (rec.kind === 'command') {
      const d = rec.data;
      commands.push({
        name: String(d.name),
        slug: String(d.slug ?? String(d.name).replace(/ /g, '-')),
        about: (d.about as string | null) ?? null,
        args: (d.args as CliArg[]) ?? [],
        mcpTool: (d.mcpTool as string | null) ?? null,
        structuredOutput: Boolean(d.structuredOutput),
      });
    } else if (rec.kind === 'mcp-tool') {
      const d = rec.data;
      if (!d.inputSchema) {
        throw new Error('The reference needs the tool input schemas from seikika describe. Rebuild seikika from this checkout.');
      }
      mcpTools.push({
        name: String(d.name),
        description: (d.description as string | null) ?? null,
        cliCommands: (d.cliCommands as string[]) ?? [],
        inputSchema: d.inputSchema as McpTool['inputSchema'],
      });
    }
  }

  commands.sort((a, b) => a.name.localeCompare(b.name));
  mcpTools.sort((a, b) => a.name.localeCompare(b.name));

  if (enrichHelp) {
    for (const cmd of commands) {
      const r = spawnSync(seikika, [...cmd.name.split(/\s+/), '--help'], {
        cwd: REPO_ROOT,
        encoding: 'utf8',
      });
      const help = (r.stdout || r.stderr || '').trimEnd();
      cmd.helpText = help || null;
      cmd.usage = help ? parseUsage(help) : null;
    }
  }

  return { commands, mcpTools };
}
