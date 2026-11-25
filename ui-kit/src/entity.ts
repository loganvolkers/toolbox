export interface EntityLink {
  label: string;
  href: string;
}

export interface EntityOverview {
  id: string;
  kind: string;
  title: string;
  description?: string;
  owner?: string;
  ownershipStatus?: string;
  links: EntityLink[];
}

const linkFields = {
  repositoryUrl: "Source",
  sourceUrl: "Source",
  docsUrl: "Documentation",
  runbookUrl: "Runbook",
  grafanaUrl: "Grafana",
  dashboardUrl: "Dashboard",
  onCallUrl: "On-call",
  registryUrl: "npm registry",
} as const;

function text(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function record(value: unknown): Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function safeLink(value: unknown): string | undefined {
  const url = text(value);
  if (!url) return;
  try {
    const parsed = new URL(url);
    if ((parsed.protocol === "https:" || parsed.protocol === "http:") && !parsed.username && !parsed.password) {
      return parsed.href;
    }
  } catch {
    // The catalog contains external links; the host supplies internal entity URLs.
  }
}

export function entityOverview(id: string, kind: string, fields: Record<string, unknown> = {}): EntityOverview {
  const ownership = record(fields.ownership);
  const links = new Map<string, EntityLink>();
  const add = (label: unknown, url: unknown) => {
    const href = safeLink(url);
    if (href && !links.has(href)) links.set(href, { href, label: text(label) ?? new URL(href).hostname });
  };
  if (Array.isArray(fields.links)) {
    for (const value of fields.links) {
      const link = record(value);
      add(link.title ?? link.label, link.url ?? link.href);
    }
  }
  for (const [field, label] of Object.entries(linkFields)) add(label, fields[field]);
  return {
    id,
    kind,
    title: text(fields.title) ?? text(fields.displayName) ?? text(fields.name) ?? id,
    description: text(fields.description),
    owner: text(ownership.owner) ?? text(fields.owner),
    ownershipStatus: text(ownership.status),
    links: [...links.values()],
  };
}

export const entityHref = (id: string) => "/e/" + encodeURIComponent(id);
