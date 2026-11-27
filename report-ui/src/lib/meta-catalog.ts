import type { CatalogReport, MetaDetail, ReportData } from "../types";
import { rubricTitle } from "./assessments";

// New snapshots include the manifest name. Rubric keys identify it in older snapshots with a
// display title. For older reports without rubrics, use the catalog's scorecard namespaces.
function portalName(data: ReportData) {
  if (data.portalName) return data.portalName;
  const rubricPortals = new Set(Object.keys(data.rubrics ?? {}).flatMap(id => {
    const match = /^rubric:([^/]+)\//.exec(id);
    return match ? [match[1]] : [];
  }));
  if (rubricPortals.size === 1) return [...rubricPortals][0];
  const slug = data.portal.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "portal";
  const matches = new Map<string, number>();
  for (const id of data.catalog.Scorecard ?? []) {
    const match = /^scorecard:([^/]+)\/(.+)$/.exec(id);
    if (match && Object.hasOwn(data.scorecards, match[2])) {
      matches.set(match[1], (matches.get(match[1]) ?? 0) + 1);
    }
  }
  if (matches.has(slug)) return slug;
  const [first, second] = [...matches].sort((a, b) => b[1] - a[1]);
  return first && first[1] > (second?.[1] ?? 0) ? first[0] : slug;
}

export function withMetaCatalog(data: ReportData): CatalogReport {
  const metaDetails: CatalogReport["metaDetails"] = Object.create(null);
  if (data.publicCatalog) return { ...data, metaDetails };
  const catalog = Object.fromEntries(Object.entries(data.catalog).map(([kind, ids]) => [kind, [...ids]]));
  const entityOverview = { ...data.entityOverview };
  const kindsById = new Map(Object.entries(catalog).flatMap(([kind, ids]) => ids.map(id => [id, kind] as const)));
  const portal = portalName(data);

  function add(detail: MetaDetail, id: string, title?: string) {
    while ((kindsById.has(id) && kindsById.get(id) !== detail.kind) || metaDetails[id]) id = "@" + id;
    if (!kindsById.has(id)) {
      (catalog[detail.kind] ??= []).push(id);
      kindsById.set(id, detail.kind);
    }
    metaDetails[id] = detail;
    if (title) entityOverview[id] = { ...entityOverview[id], title: entityOverview[id]?.title ?? title };
    return id;
  }

  for (const [name, card] of Object.entries(data.scorecards)) {
    add({ kind: "Scorecard", name }, `scorecard:${portal}/${name}`, card.title || name);
    for (const [rule, definition] of Object.entries(card.rules)) {
      add({ kind: "Rule", name, rule }, `rule:${portal}/${name}/${rule}`, definition.title || rule);
    }
  }
  for (const [name, campaign] of Object.entries(data.campaigns ?? {})) {
    add({ kind: "Campaign", name }, `campaign:${portal}/${name}`, campaign.title || name);
  }
  for (const [name, track] of Object.entries(data.tracks ?? {})) {
    add({ kind: "Track", name }, `track:${portal}/${name}`, track.title || name);
  }
  for (const [id, rubric] of Object.entries(data.rubrics ?? {})) {
    add({ kind: "Rubric", name: id }, id, rubricTitle(rubric.name));
  }
  const assessmentKinds = data.assessmentKinds ?? (catalog.Assessment ? ["Assessment"] : []);
  for (const id of data.assessmentIds ?? assessmentKinds.flatMap(kind => catalog[kind] ?? [])) {
    if (kindsById.has(id) && !metaDetails[id]) metaDetails[id] = { kind: "Assessment" };
  }
  const actions = data.actions?.map(action => ({
    ...action,
    catalogId: add({ kind: "Action" }, `action:${portal}/${action.name}`, action.name),
  }));
  return { ...data, catalog, entityOverview, metaDetails, actions };
}
