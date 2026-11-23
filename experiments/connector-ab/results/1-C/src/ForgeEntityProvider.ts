/**
 * Backstage-style EntityProvider for the Forge (mock Jira Cloud) API.
 *
 * Pattern mirrors GithubEntityProvider: getProviderName + connect, then a
 * scheduled/manual refresh that applyMutation({ type: 'full', entities }).
 *
 * For this benchmark harness, run() also writes the experiment catalog schema
 * to OUT/entities.ndjson (sorted by kind then id).
 */

import * as fs from 'node:fs';
import * as path from 'node:path';
import * as https from 'node:https';
import * as http from 'node:http';
import { URL } from 'node:url';

/** Minimal stand-in for @backstage/plugin-catalog-node EntityProviderConnection. */
export interface EntityProviderConnection {
  applyMutation(mutation: EntityProviderMutation): Promise<void>;
  refresh(options: { keys: string[] }): Promise<void>;
}

export type EntityProviderMutation =
  | { type: 'full'; entities: DeferredEntity[] }
  | {
      type: 'delta';
      added: DeferredEntity[];
      removed: Array<DeferredEntity | { entityRef: string; locationKey?: string }>;
    };

export type DeferredEntity = {
  entity: Record<string, unknown>;
  locationKey?: string;
};

/** Experiment catalog line (not a Backstage entity). */
export type CatalogRecord =
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
      data: { id: string; name: string; project: string };
    };

export type ForgeEntityProviderOptions = {
  baseUrl: string;
  token: string;
  /** Absolute or cwd-relative path for entities.ndjson */
  outPath?: string;
};

type ForgeProject = {
  id: string;
  key: string;
  name: string;
  projectTypeKey: string;
};

type ForgeIssue = {
  id: string;
  key: string;
  fields: {
    summary: string;
    status: { name: string };
    project: { key: string };
    assignee: { accountId: string; displayName?: string } | null;
  };
};

async function httpGetJson<T>(
  url: string,
  token: string,
): Promise<T> {
  const u = new URL(url);
  const lib = u.protocol === 'https:' ? https : http;
  return new Promise((resolve, reject) => {
    const req = lib.request(
      {
        protocol: u.protocol,
        hostname: u.hostname,
        port: u.port,
        path: `${u.pathname}${u.search}`,
        method: 'GET',
        headers: {
          Authorization: `Bearer ${token}`,
          Accept: 'application/json',
        },
      },
      res => {
        const chunks: Buffer[] = [];
        res.on('data', (c: Buffer) => chunks.push(c));
        res.on('end', () => {
          const body = Buffer.concat(chunks).toString('utf8');
          if (
            res.statusCode &&
            (res.statusCode < 200 || res.statusCode >= 300)
          ) {
            reject(
              new Error(
                `Forge GET ${url} → ${res.statusCode}: ${body.slice(0, 200)}`,
              ),
            );
            return;
          }
          try {
            resolve(JSON.parse(body) as T);
          } catch (e) {
            reject(e);
          }
        });
      },
    );
    req.on('error', reject);
    req.end();
  });
}

async function paginateProjects(
  baseUrl: string,
  token: string,
): Promise<ForgeProject[]> {
  const out: ForgeProject[] = [];
  let startAt = 0;
  const maxResults = 50;
  for (;;) {
    const page = await httpGetJson<{
      values: ForgeProject[];
      isLast?: boolean;
      total?: number;
      maxResults?: number;
    }>(
      `${baseUrl}/rest/api/3/project?startAt=${startAt}&maxResults=${maxResults}`,
      token,
    );
    const values = page.values ?? [];
    out.push(...values);
    if (page.isLast === true || values.length === 0) break;
    startAt += page.maxResults ?? values.length;
    if (page.total !== undefined && startAt >= page.total) break;
  }
  return out;
}

async function paginateIssues(
  baseUrl: string,
  token: string,
): Promise<ForgeIssue[]> {
  const out: ForgeIssue[] = [];
  let startAt = 0;
  const maxResults = 50;
  for (;;) {
    const page = await httpGetJson<{
      issues: ForgeIssue[];
      total: number;
      maxResults: number;
      startAt: number;
    }>(
      `${baseUrl}/rest/api/3/search?startAt=${startAt}&maxResults=${maxResults}`,
      token,
    );
    const issues = page.issues ?? [];
    out.push(...issues);
    startAt += page.maxResults ?? issues.length;
    if (issues.length === 0 || startAt >= (page.total ?? 0)) break;
  }
  return out;
}

function sortRecords(records: CatalogRecord[]): CatalogRecord[] {
  const kindOrder: Record<string, number> = {
    component: 0,
    issue: 1,
    project: 2,
  };
  // Prompt: sorted by kind then id. Lexicographic kind: component < issue < project.
  return [...records].sort((a, b) => {
    if (a.kind !== b.kind) {
      return (kindOrder[a.kind] ?? 99) - (kindOrder[b.kind] ?? 99) ||
        a.kind.localeCompare(b.kind);
    }
    return a.id.localeCompare(b.id);
  });
}

function toBackstageEntities(records: CatalogRecord[]): DeferredEntity[] {
  return records.map(r => ({
    locationKey: `forge:${r.kind}:${r.id}`,
    entity: {
      apiVersion: 'backstage.io/v1alpha1',
      kind: r.kind === 'project' ? 'System' : r.kind === 'issue' ? 'Component' : 'Resource',
      metadata: {
        name: r.id.replace(/[^a-zA-Z0-9\-_.]/g, '-').toLowerCase(),
        title: 'name' in r.data ? (r.data as { name: string }).name : r.id,
        annotations: {
          'forge/kind': r.kind,
          'forge/id': r.id,
        },
      },
      spec: { ...r.data, type: r.kind },
    },
  }));
}

/**
 * Discovers Forge projects and issues and emits them into the catalog.
 */
export class ForgeEntityProvider {
  private connection?: EntityProviderConnection;
  private readonly baseUrl: string;
  private readonly token: string;
  private readonly outPath: string;

  constructor(options: ForgeEntityProviderOptions) {
    this.baseUrl = options.baseUrl.replace(/\/$/, '');
    this.token = options.token;
    this.outPath =
      options.outPath ??
      path.join(process.cwd(), 'OUT', 'entities.ndjson');
  }

  /** {@inheritdoc EntityProvider.getProviderName} */
  getProviderName(): string {
    return 'forge';
  }

  /** {@inheritdoc EntityProvider.connect} */
  async connect(connection: EntityProviderConnection): Promise<void> {
    this.connection = connection;
  }

  /**
   * Full resync: pull Forge project + issue, apply a full mutation when
   * connected, and always rewrite OUT/entities.ndjson.
   */
  async run(): Promise<CatalogRecord[]> {
    const projects = await paginateProjects(this.baseUrl, this.token);
    const issues = await paginateIssues(this.baseUrl, this.token);

    const records: CatalogRecord[] = [];

    for (const p of projects) {
      records.push({
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
      records.push({
        kind: 'issue',
        id: issue.key,
        data: {
          key: issue.key,
          summary: issue.fields.summary,
          status: issue.fields.status.name,
          project: issue.fields.project.key,
          assignee: issue.fields.assignee?.accountId ?? null,
        },
      });
    }

    const sorted = sortRecords(records);

    if (this.connection) {
      await this.connection.applyMutation({
        type: 'full',
        entities: toBackstageEntities(sorted),
      });
    }

    await this.writeOut(sorted);
    return sorted;
  }

  private async writeOut(records: CatalogRecord[]): Promise<void> {
    const dir = path.dirname(this.outPath);
    fs.mkdirSync(dir, { recursive: true });
    const body = records.map(r => JSON.stringify(r)).join('\n') + (records.length ? '\n' : '');
    fs.writeFileSync(this.outPath, body, 'utf8');
  }
}
