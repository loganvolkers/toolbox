/**
 * Build-time CLI catalog loader for the Starlight CLI reference.
 * Copy to site/src/lib/cli-describe.ts
 *
 * Invokes `toolbox describe` (+ optional --help enrichment) from the repo root.
 */
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
/** site/src/lib → repo root is ../../.. */
export const REPO_ROOT = resolve(here, '../../..');

export type CliArg = {
  name: string;
  long: string | null;
  positional: boolean;
  described: boolean;
  help?: string | null;
};

export type CliCommand = {
  name: string;
  slug: string;
  about: string | null;
  args: CliArg[];
  mcpTool: string | null;
  structuredOutput: boolean;
  usage?: string | null;
  helpText?: string | null;
};

export type McpTool = {
  name: string;
  description: string | null;
  cliCommands: string[];
};

export type CliCatalog = {
  commands: CliCommand[];
  mcpTools: McpTool[];
};

function resolveToolbox(): string {
  const env = process.env.TOOLBOX_BIN;
  if (env && existsSync(env)) return env;
  for (const c of [
    join(REPO_ROOT, 'target/debug/toolbox'),
    join(REPO_ROOT, 'target/release/toolbox'),
  ]) {
    if (existsSync(c)) return c;
  }
  const which = spawnSync('which', ['toolbox'], { encoding: 'utf8' });
  if (which.status === 0 && which.stdout.trim()) return which.stdout.trim();
  const build = spawnSync('cargo', ['build', '-p', 'toolbox-cli', '--locked'], {
    cwd: REPO_ROOT,
    encoding: 'utf8',
  });
  if (build.status !== 0) {
    throw new Error(`cargo build failed:\n${build.stderr}`);
  }
  const bin = join(REPO_ROOT, 'target/debug/toolbox');
  if (!existsSync(bin)) throw new Error('toolbox binary missing after build');
  return bin;
}

function runToolbox(toolbox: string, args: string[]): string {
  const r = spawnSync(toolbox, args, { cwd: REPO_ROOT, encoding: 'utf8' });
  if (r.status !== 0) {
    throw new Error(`toolbox ${args.join(' ')} failed:\n${r.stderr || r.stdout}`);
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
  const toolbox = resolveToolbox();
  const raw = runToolbox(toolbox, ['describe']);
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
      mcpTools.push({
        name: String(d.name),
        description: (d.description as string | null) ?? null,
        cliCommands: (d.cliCommands as string[]) ?? [],
      });
    }
  }

  commands.sort((a, b) => a.name.localeCompare(b.name));
  mcpTools.sort((a, b) => a.name.localeCompare(b.name));

  if (enrichHelp) {
    for (const cmd of commands) {
      const r = spawnSync(toolbox, [...cmd.name.split(/\s+/), '--help'], {
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

/** Stable JSON for drift checks (sorted, no helpText churn from whitespace). */
export function catalogFingerprint(catalog: CliCatalog): string {
  const slim = {
    commands: catalog.commands.map((c) => ({
      name: c.name,
      about: c.about,
      mcpTool: c.mcpTool,
      args: c.args.map((a) => ({
        name: a.name,
        long: a.long,
        positional: a.positional,
      })),
      usage: c.usage ?? null,
    })),
    mcpTools: catalog.mcpTools.map((t) => ({
      name: t.name,
      cliCommands: t.cliCommands,
    })),
  };
  return `${JSON.stringify(slim, null, 2)}\n`;
}
