/**
 * Backstage-style EntityProvider for Forge (mock Jira Cloud).
 *
 * Pattern follows GithubEntityProvider / ConfigLocationEntityProvider:
 * getProviderName + connect(store connection) + run/refresh that applyMutation(full).
 *
 * For this harness, applyMutation also writes OUT/entities.ndjson in the
 * experiment schema (kind/id/data), sorted by kind then id.
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

export type ForgeConfig = {
  baseUrl: string;
  token: string;
  /** Directory that contains OUT/ (defaults to arm root). */
  rootDir?: string;
};

/** Minimal EntityProviderConnection (catalog-node shape). */
export interface EntityProviderConnection {
  applyMutation(mutation: {
    type: 'full' | 'delta';
    entities: DeferredEntity[];
  }): Promise<void>;
  refresh?(options: { keys: string[] }): Promise<void>;
}

export type DeferredEntity = {
  entity: {
    apiVersion: string;
    kind: string;
    metadata: { name: string; annotations?: Record<string, string> };
    spec?: Record<string, unknown>;
  };
  locationKey?: string;
};

/** Experiment catalog row written to OUT/entities.ndjson */
export type CatalogRow =
  | {
      kind: 'project';
      id: string;
      data: { key: string; name: string; type: string };
    }
  | {
      kind: 'issue';
      id: string;
      data: {
        key: string;
        summary: string;
        status: string;
        project: string;
        assignee: string | null;
      };
    }
  | {
      kind: 'component';
      id: string;
      data: { id: string; name: string; project: string; description?: string };
    };

function armRoot(): string {
  const here = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(here, '..');
}

async function forgeGet<T>(
  baseUrl: string,
  token: string,
  pathname: string,
  query: Record<string, string | number> = {},
): Promise<T> {
  const url = new URL(pathname, baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`);
  for (const [k, v] of Object.entries(query)) {
    url.searchParams.set(k, String(v));
  }
  const res = await fetch(url, {
    headers: { Authorization: `Bearer ${token}`, Accept: 'application/json' },
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Forge ${url} → ${res.status}: ${body}`);
  }
  return (await res.json()) as T;
}

type ProjectPage = {
  values: Array<{
    id: string;
    key: string;
    name: string;
    projectTypeKey: string;
  }>;
  startAt: number;
  isLast: boolean;
  maxResults: number;
};

type SearchPage = {
  issues: Array<{
    id: string;
    key: string;
    fields: {
      summary: string;
      status: { name: string };
      project: { key: string };
      assignee: { accountId: string; displayName?: string } | null;
    };
  }>;
  startAt: number;
  maxResults: number;
  total: number;
};

/** Response element from GET /rest/api/3/project/{key}/components (bare array). */
type ForgeComponent = {
  id: string | number;
  name: string;
  description?: string;
};

export class ForgeEntityProvider {
  private readonly baseUrl: string;
  private readonly token: string;
  private readonly rootDir: string;
  private connection?: EntityProviderConnection;
  private lastRows: CatalogRow[] = [];

  constructor(config: ForgeConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, '');
    this.token = config.token;
    this.rootDir = config.rootDir ?? armRoot();
  }

  getProviderName(): string {
    return 'forge';
  }

  async connect(connection: EntityProviderConnection): Promise<void> {
    this.connection = connection;
    await this.run();
  }

  /** Full resync: fetch Forge → map → applyMutation(full) → write NDJSON. */
  async run(): Promise<CatalogRow[]> {
    const rows = await this.fetchMapped();
    this.lastRows = rows;

    const entities: DeferredEntity[] = rows.map(row => ({
      entity: {
        apiVersion: 'forge.local/v1alpha1',
        kind: row.kind === 'project' ? 'ForgeProject' : row.kind === 'issue' ? 'ForgeIssue' : 'ForgeComponent',
        metadata: {
          name: row.id,
          annotations: {
            'forge.local/kind': row.kind,
            'forge.local/id': row.id,
          },
        },
        spec: row.data as unknown as Record<string, unknown>,
      },
      locationKey: `forge:${row.kind}:${row.id}`,
    }));

    if (this.connection) {
      await this.connection.applyMutation({ type: 'full', entities });
    } else {
      await this.writeNdjson(rows);
    }

    return rows;
  }

  /** Fetch and map projects, issues, and per-project components. */
  async fetchMapped(): Promise<CatalogRow[]> {
    const projects = await this.fetchAllProjects();
    const issues = await this.fetchAllIssues();

    const rows: CatalogRow[] = [];

    for (const p of projects) {
      rows.push({
        kind: 'project',
        id: p.key,
        data: {
          key: p.key,
          name: p.name,
          type: p.projectTypeKey,
        },
      });
    }

    for (const issue of issues) {
      rows.push({
        kind: 'issue',
        id: issue.key,
        data: {
          key: issue.key,
          summary: issue.fields.summary,
          status: issue.fields.status.name,
          project: issue.fields.project.key,
          assignee: null,
        },
      });
    }

    for (const p of projects) {
      const components = await this.fetchProjectComponents(p.key);
      for (const c of components) {
        // Catalog id is {project}:{name} (FORGE.md); Forge numeric stays in data.id only.
        const catalogId = c.name;
        rows.push({
          kind: 'component',
          id: catalogId,
          data: {
            id: String(c.id),
            name: c.name,
            project: p.key,
            description: (c as { description?: string }).description ?? '',
          },
        });
      }
    }

    rows.sort((a, b) => {
      if (a.kind !== b.kind) return a.kind.localeCompare(b.kind);
      return a.id.localeCompare(b.id);
    });

    return rows;
  }

  private async fetchAllProjects(): Promise<ProjectPage['values']> {
    const all: ProjectPage['values'] = [];
    let startAt = 0;
    const maxResults = 50;
    for (;;) {
      const page = await forgeGet<ProjectPage>(
        this.baseUrl,
        this.token,
        'rest/api/3/project',
        { startAt, maxResults },
      );
      all.push(...page.values);
      if (page.isLast || page.values.length === 0) break;
      startAt += page.maxResults || maxResults;
    }
    return all;
  }

  private async fetchAllIssues(): Promise<SearchPage['issues']> {
    const all: SearchPage['issues'] = [];
    let startAt = 0;
    const maxResults = 50;
    for (;;) {
      const page = await forgeGet<SearchPage>(
        this.baseUrl,
        this.token,
        'rest/api/3/search',
        { startAt, maxResults },
      );
      all.push(...page.issues);
      startAt += page.issues.length;
      if (startAt >= page.total || page.issues.length === 0) break;
    }
    return all;
  }

  /** Bare JSON array (not paged): GET /rest/api/3/project/{key}/components */
  private async fetchProjectComponents(
    projectKey: string,
  ): Promise<ForgeComponent[]> {
    return forgeGet<ForgeComponent[]>(
      this.baseUrl,
      this.token,
      `rest/api/3/project/${encodeURIComponent(projectKey)}/components`,
    );
  }

  async writeNdjson(rows: CatalogRow[] = this.lastRows): Promise<string> {
    const outDir = path.join(this.rootDir, 'OUT');
    fs.mkdirSync(outDir, { recursive: true });
    const outPath = path.join(outDir, 'entities.ndjson');
    const body = rows.map(r => JSON.stringify(r)).join('\n') + (rows.length ? '\n' : '');
    fs.writeFileSync(outPath, body, 'utf8');
    return outPath;
  }
}

/** Connection that materializes the experiment NDJSON on full mutation. */
export function createNdjsonConnection(
  provider: ForgeEntityProvider,
): EntityProviderConnection {
  return {
    async applyMutation(mutation) {
      if (mutation.type !== 'full') {
        throw new Error('only full mutations supported in harness');
      }
      // Re-read mapped rows from provider via writeNdjson after run sets lastRows;
      // connection is invoked from run() after lastRows is set.
      await provider.writeNdjson();
    },
    async refresh() {
      /* no-op for harness */
    },
  };
}
