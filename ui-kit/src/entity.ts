/** A labeled web link shown on an entity page. */
export interface EntityLink {
  /** Text displayed to the reader. */
  label: string;
  /** Destination URL. {@link entityOverview} accepts absolute HTTP and HTTPS links. */
  href: string;
}

/** Data displayed by the `EntityPage` component. */
export interface EntityOverview {
  /** Catalog entity ID, such as `service:checkout`. */
  id: string;
  /** Display label for the kind of entity, such as `Service`. */
  kind: string;
  /** Heading displayed for the entity. */
  title: string;
  /** Optional text below the heading. Omit it when the catalog has no description. */
  description?: string;
  /** Owner's entity ID, such as `group:payments`. */
  owner?: string;
  /** Ownership status supplied by the catalog, if available. */
  ownershipStatus?: string;
  /** Links displayed in order. An empty array hides the link list. */
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

/**
 * Returns a browser-safe absolute HTTP or HTTPS URL without embedded credentials.
 *
 * @param value - A URL from catalog or connector data.
 */
export function safeWebHref(value: unknown): string | undefined {
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

/**
 * Builds the data displayed by an entity page from catalog fields.
 *
 * The title uses the first nonempty value from `title`, `displayName`, `name`, and `id`.
 * `ownership.owner` takes precedence over `owner`. Explicit `links` come before known
 * URL fields such as `repositoryUrl` and `runbookUrl`. Only absolute HTTP and HTTPS
 * URLs without embedded credentials are included. The first link for each URL wins.
 *
 * @param id - Catalog entity ID, such as `service:checkout`.
 * @param kind - Kind label displayed on the page, such as `Service`.
 * @param fields - Optional catalog fields. Omit them to use the ID as the title.
 */
export function entityOverview(id: string, kind: string, fields: Record<string, unknown> = {}): EntityOverview {
  const ownership = record(fields.ownership);
  const links = new Map<string, EntityLink>();
  const add = (label: unknown, url: unknown) => {
    const href = safeWebHref(url);
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

/**
 * Builds the hosted portal's root-relative entity URL.
 *
 * @param id - An unencoded entity ID. The helper encodes it as one path segment.
 * @returns A path such as `/e/service%3Acheckout`. Supply your own resolver on hosts with other routes.
 * @throws URIError if `id` contains an unpaired UTF-16 surrogate.
 */
export const entityHref = (id: string) => "/e/" + encodeURIComponent(id);
