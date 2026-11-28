import type { CatalogAlias, CatalogReport, ReportAction, ReportChild, ReportData, ReportEntity, Score } from "../types";
import { withMetaCatalog } from "./meta-catalog";

const map = <T>(): Record<string, T> => Object.create(null);
const entries = <T>(value?: Record<string, T>) => Object.entries(value ?? {});
const kindName = /^[A-Z][A-Za-z0-9]*$/;
const forbidden = new Set(["__proto__", "prototype", "constructor"]);
const record = (value: unknown): value is Record<string, unknown> =>
  Boolean(value && typeof value === "object" && !Array.isArray(value));
const strings = (value: unknown): value is string[] =>
  Array.isArray(value) && value.every(item => typeof item === "string");
const validMap = (value: unknown, check: (value: unknown, key: string) => boolean) =>
  record(value) && Object.entries(value).every(([key, value]) => !forbidden.has(key) && check(value, key));
const optionalMap = (value: unknown, check: (value: unknown, key: string) => boolean) =>
  value == null || validMap(value, check);
const textFields = (value: Record<string, unknown>, fields: string[]) =>
  fields.every(field => value[field] == null || typeof value[field] === "string");
const links = (value: unknown) =>
  value == null || Array.isArray(value) && value.every(link => record(link)
    && typeof link.url === "string" && textFields(link, ["title"]));
const number = (value: unknown) => value == null || typeof value === "number" && Number.isFinite(value);
const level = (value: unknown) => typeof value === "string" && /^[a-z][a-z0-9-]*$/.test(value) && !forbidden.has(value);
const score = (value: unknown) => record(value) && level(value.level)
  && optionalMap(value.rules, item => typeof item === "string")
  && optionalMap(value.evidence, () => true)
  && optionalMap(value.measures, number) && optionalMap(value.values, () => true);
const input = (value: unknown) => record(value) && textFields(value, ["type", "title", "description"])
  && (value.enum == null || Array.isArray(value.enum) && value.enum.every(item => ["string", "number", "boolean"].includes(typeof item)));
const action = (value: unknown) => record(value) && textFields(value, ["description", "scope", "approval", "permission"])
  && typeof value.name === "string" && typeof value.kind === "string" && typeof value.executor === "string"
  && optionalMap(value.choices, targets => validMap(targets, strings))
  && strings(value.entities) && (value.inputs == null || record(value.inputs)
    && optionalMap(value.inputs.properties, input) && (value.inputs.required == null || strings(value.inputs.required)));
const edge = (value: unknown, endpoint: string) => record(value)
  && typeof value.relation === "string" && typeof value[endpoint] === "string";

function valid(data: unknown): data is ReportData {
  return record(data) && typeof data.portal === "string" && textFields(data, ["portalName"])
    && validMap(data.catalog, (value, kind) => kindName.test(kind) && strings(value))
    && validMap(data.scorecards, value => record(value)
      && textFields(value, ["title", "description", "rationale", "owner", "track", "appliesTo"])
      && typeof value.kind === "string" && kindName.test(value.kind) && strings(value.levels) && value.levels.every(level)
      && links(value.links) && validMap(value.rules, rule => record(rule)
        && textFields(rule, ["level", "fix", "title", "rationale", "check", "basis", "unit"]) && links(rule.links)))
    && validMap(data.scores, rows => validMap(rows, score))
    && (data.kindGroups == null || Array.isArray(data.kindGroups) && data.kindGroups.every(group => record(group)
      && typeof group.name === "string" && textFields(group, ["title", "description"]) && strings(group.kinds)))
    && optionalMap(data.tracks, value => record(value) && textFields(value, ["title", "owner", "description"]) && links(value.links))
    && optionalMap(data.campaigns, value => record(value)
      && textFields(value, ["title", "track", "owner", "description", "start", "end", "status"])
      && ["targetPercent", "inScope", "meeting", "percent", "measure"].every(key => number(value[key]))
      && Array.isArray(value.includes) && value.includes.every(item => record(item) && typeof item.scorecard === "string"
        && textFields(item, ["atLeast"]) && (item.rules == null || strings(item.rules)))
      && (value.milestones == null || Array.isArray(value.milestones)
        && value.milestones.every(item => record(item) && typeof item.name === "string" && number(item.targetPercent)))
      && optionalMap(value.entities, item => record(item) && typeof item.meeting === "boolean" && (item.gaps == null || strings(item.gaps))))
    && optionalMap(data.kinds, (value, kind) => record(value) && kindName.test(kind) && textFields(value, ["title", "description"]))
    && optionalMap(data.entityOverview, value => record(value)
      && textFields(value, ["title", "displayName", "name", "description", "owner", "repositoryUrl", "sourceUrl"])
      // A catalog kind can use `links` as a metric, such as the Rule kind's link count.
      && (links(value.links) || number(value.links))
      && (value.ownership == null || record(value.ownership) && textFields(value.ownership, ["owner", "status"])))
    && optionalMap(data.entities, value => record(value) && record(value.fields) && optionalMap(value.provenance, item => typeof item === "string")
      && (value.relations == null || record(value.relations) && Array.isArray(value.relations.in) && Array.isArray(value.relations.out)
        && value.relations.in.every(item => edge(item, "from")) && value.relations.out.every(item => edge(item, "to"))))
    && (data.assessmentKinds == null || strings(data.assessmentKinds))
    && (data.repoRollup == null || strings(data.repoRollup))
    && optionalMap(data.rubrics, value => record(value) && typeof value.name === "string"
      && typeof value.kind === "string" && textFields(value, ["description", "subject"]) && links(value.links)
      && Array.isArray(value.criteria) && value.criteria.every(criterion => record(criterion)
        && typeof criterion.name === "string" && typeof criterion.question === "string"
        && textFields(criterion, ["guidance"])))
    && optionalMap(data.unassessed, strings)
    && optionalMap(data.comparisons, value => record(value) && typeof value.subject === "string" && strings(value.peers)
      && validMap(value.verdict, item => typeof item === "string")
      && validMap(value.rules, item => validMap(item, standing => typeof standing === "string"))
      && optionalMap(value.scores, score) && optionalMap(value.titles, item => item == null || typeof item === "string")
      && optionalMap(value.reads, strings) && optionalMap(value.facts, record)
      && optionalMap(value.fields, item => item == null || typeof item === "string"))
    && (data.actions == null || Array.isArray(data.actions) && data.actions.every(action))
    && (data.children == null || Array.isArray(data.children));
}

function aliasesFor(rules: CatalogAlias[], parent: CatalogReport, child: CatalogReport) {
  const aliases = new Map<string, string>();
  const fromCounts = new Map<string, number>();
  const toCounts = new Map<string, number>();
  for (const { from, to } of rules) {
    fromCounts.set(from, (fromCounts.get(from) ?? 0) + 1);
    toCounts.set(to, (toCounts.get(to) ?? 0) + 1);
  }
  for (const { from, to, kind } of rules) {
    if (fromCounts.get(from) === 1 && toCounts.get(to) === 1
        && child.catalog[kind]?.includes(from) && parent.catalog[kind]?.includes(to)
        && !child.metaDetails[from] && !parent.metaDetails[to]) aliases.set(from, to);
  }
  return aliases;
}

function rewriteRelations(detail: ReportEntity, id: (original: string) => string): ReportEntity {
  return {
    ...detail,
    relations: {
      in: (detail.relations?.in ?? []).map(row => ({ ...row, from: id(row.from) })),
      out: (detail.relations?.out ?? []).map(row => ({ ...row, to: id(row.to) })),
    },
  };
}

function withTitle(value: { title?: string | null }, fallback: string, source: string) {
  return (value.title || fallback) + " · " + source;
}

function reportScore(row: Score): Score {
  if (!row.evidence) return row;
  const text = (value: unknown) => typeof value === "string" ? value : JSON.stringify(value);
  return { ...row, evidence: Object.fromEntries(entries(row.evidence).map(([rule, evidence]) => [
    rule, Array.isArray(evidence) ? evidence.map(text) : text(evidence),
  ])) };
}

/** Flatten snapshots while keeping the parent policy and every child's data independently keyed. */
export function aggregateReport(snapshot: ReportData): CatalogReport {
  const parent = withMetaCatalog(snapshot);
  if (!parent.children?.length && !parent.organization) return parent;
  const out: CatalogReport = {
    ...parent, catalog: map(), scorecards: map(), scores: map(), campaigns: map(), tracks: map(),
    entityOverview: map(), entities: map(), kinds: map(), unassessed: map(), comparisons: map(), rubrics: map(),
    sources: map(), entitySources: map(), entityObservations: map(), entityRedirects: map(), comparisonSources: map(),
    repoRollupsByCard: map(),
    actions: [], kindGroups: [], assessmentIds: [], metaDetails: map(),
  };
  const sources = out.sources!;
  const entitySources = out.entitySources!;
  const observations = out.entityObservations!;
  const actionRows = out.actions!;
  const parentKey = "local";
  sources[parentKey] = { title: parent.portal, local: true };
  const label = (key: string) => sourceLabel(out, key);
  const names = new Set([
    ...Object.values(parent.catalog).flat(), ...Object.keys(parent.scorecards), ...Object.keys(parent.tracks ?? {}),
    ...Object.keys(parent.campaigns ?? {}), ...Object.keys(parent.rubrics ?? {}),
  ]);
  const forwarded = new Map<string, ReportAction>();
  for (const action of parent.actions ?? []) {
    if (action.portal) forwarded.set(JSON.stringify([action.portal.key, action.name]), action);
  }
  const matched = new Set<ReportAction>();
  const grouped = new Set<string>();
  const aliasRules = new Map<string, CatalogAlias[]>();
  for (const rule of parent.catalogAliases ?? []) {
    const key = JSON.stringify(rule.source);
    const rules = aliasRules.get(key) ?? [];
    rules.push(rule);
    aliasRules.set(key, rules);
  }
  function add(doc: CatalogReport, key: string, path: string[], depth: number) {
    const mapped = new Map<string, string>();
    function id(original: string) {
      if (!path.length) return original;
      let result = mapped.get(original);
      if (result) return result;
      result = "@" + path.map(encodeURIComponent).join("/") + "/" + original;
      while (names.has(result)) result = "@" + result;
      names.add(result);
      mapped.set(original, result);
      return result;
    }
    const aliases = aliasesFor(aliasRules.get(JSON.stringify(path)) ?? [], parent, doc);
    const entityId = (original: string) => aliases.get(original) ?? id(original);
    const owner = <T>(value: T) => typeof value === "string" && doc.entityOverview?.[value] ? entityId(value) : value;
    for (const [kind, metadata] of entries(doc.kinds)) out.kinds![kind] ??= metadata;
    for (const [kind, ids] of entries(doc.catalog)) {
      const values = out.catalog[kind] ??= [];
      for (const original of [...new Set(ids)]) {
        const canonical = aliases.get(original);
        const detail = doc.entities?.[original];
        if (canonical) {
          out.entityRedirects![id(original)] = canonical;
          (observations[canonical] ??= []).push({ source: key, original, detail: detail && rewriteRelations(detail, entityId) });
          continue;
        }
        const entity = id(original);
        const meta = doc.metaDetails[original];
        if (meta) out.metaDetails[entity] = "name" in meta ? { ...meta, name: id(meta.name) } : meta;
        values.push(entity);
        if (meta?.kind === "Assessment") out.assessmentIds!.push(entity);
        const overview = doc.entityOverview?.[original];
        const repository = !path.length && parent.organization
          ? overview?.repository ?? (kind === "Repository" ? overview?.name : undefined)
          : undefined;
        const source = typeof repository === "string" ? "repository:" + repository : key;
        if (source !== key) sources[source] ??= { title: String(repository), repository: true };
        entitySources[entity] = { source, original };
        if (overview) {
          const ownership = record(overview.ownership) ? overview.ownership : undefined;
          out.entityOverview![entity] = {
            ...overview, owner: owner(overview.owner as string | undefined),
            ...(ownership ? { ownership: { ...ownership, owner: owner(ownership.owner as string | undefined) } } : {}),
          };
        }
        if (detail) out.entities![entity] = rewriteRelations(detail, entityId);
      }
    }
    for (const group of doc.kindGroups ?? []) {
      const kinds = group.kinds.filter(kind => !grouped.has(kind) && out.catalog[kind]?.length);
      if (!kinds.length) continue;
      kinds.forEach(kind => grouped.add(kind));
      out.kindGroups!.push({ ...group, name: id(group.name), kinds });
    }
    for (const [name, card] of entries(doc.scorecards)) out.scorecards[id(name)] = {
      ...card, title: withTitle(card, name, label(key)), owner: owner(card.owner),
      track: card.track ? id(card.track) : card.track,
    };
    const rollup = doc.repoRollup?.map(id) ?? [];
    for (const name of rollup) out.repoRollupsByCard![name] = rollup;
    for (const [name, rubric] of entries(doc.rubrics)) {
      const entity = id(name);
      out.rubrics![entity] = rubric;
      entitySources[entity] ??= { source: key, original: name };
    }
    for (const [name, rows] of entries(doc.scores)) out.scores[id(name)] =
      Object.fromEntries(entries(rows).map(([target, row]) => [entityId(target), path.length ? reportScore(row) : row]));
    for (const [name, track] of entries(doc.tracks)) out.tracks![id(name)] = {
      ...track, title: withTitle(track, name, label(key)), owner: owner(track.owner),
    };
    for (const [name, campaign] of entries(doc.campaigns)) out.campaigns![id(name)] = {
      ...campaign, title: withTitle(campaign, name, label(key)), owner: owner(campaign.owner),
      track: campaign.track ? id(campaign.track) : campaign.track,
      includes: campaign.includes.map(include => ({ ...include, scorecard: id(include.scorecard) })),
      entities: campaign.entities && Object.fromEntries(entries(campaign.entities).map(([target, row]) => [entityId(target), {
        ...row, gaps: row.gaps?.map(gap => gap.replace(/^([^:]+)(:.*)$/, (_, card, rest) => id(card) + rest)),
      }])),
    };
    for (const [name, values] of entries(doc.unassessed)) out.unassessed![id(name)] = values.map(entityId);
    for (const [name, comparison] of entries(doc.comparisons)) {
      out.comparisonSources![id(name)] = key;
      out.comparisons![id(name)] = {
        ...comparison, subject: entityId(comparison.subject), peers: comparison.peers.map(entityId),
        verdict: Object.fromEntries(entries(comparison.verdict).map(([target, verdict]) => [entityId(target), verdict])),
        rules: Object.fromEntries(entries(comparison.rules).map(([rule, scores]) =>
          [rule, Object.fromEntries(entries(scores).map(([target, value]) => [entityId(target), value]))])),
        titles: Object.fromEntries(entries(comparison.titles).map(([target, value]) => [entityId(target), value])),
        facts: Object.fromEntries(entries(comparison.facts).map(([target, value]) => [entityId(target), value])),
        scores: Object.fromEntries(entries(comparison.scores).map(([target, value]) => [entityId(target), path.length ? reportScore(value) : value])),
      };
    }
    for (const item of doc.actions ?? []) {
      if (!path.length && item.portal) continue;
      const link = path.length === 1 && !item.portal ? forwarded.get(JSON.stringify([path[0], item.name])) : undefined;
      if (link) matched.add(link);
      actionRows.push({
        ...item, ...(link ? { inputs: link.inputs, approval: link.approval, permission: link.permission } : {}),
        catalogId: item.catalogId && id(item.catalogId),
        source: key, entities: item.entities.map(entityId), target: link?.entities[0],
        choices: item.choices && Object.fromEntries(entries(item.choices).map(([input, targets]) =>
          [input, Object.fromEntries(entries(targets).map(([target, values]) => [entityId(target), values]))])),
        previewOnly: path.length > 0 && (!link || item.entities.length > 1),
      });
    }
    if (depth === 8) return;
    for (const item of doc.children ?? []) {
      if (!record(item) || typeof item.key !== "string" || !item.key || item.key.length > 200) continue;
      const child = item as unknown as ReportChild;
      const next = [...path, child.key];
      const sourceKey = "child:" + JSON.stringify(next);
      if (sources[sourceKey]) continue;
      const title = typeof child.title === "string" ? child.title : child.key;
      sources[sourceKey] = {
        title: path.length ? sources[key].title + " / " + title : title,
        commit: typeof child.commit === "string" ? child.commit : undefined,
        digest: typeof child.digest === "string" ? child.digest : undefined,
        error: typeof child.error === "string" ? child.error : undefined,
      };
      if (valid(child.report)) add(withMetaCatalog(child.report), sourceKey, next, depth + 1);
      else sources[sourceKey].error ??= "The published report could not be loaded.";
    }
  }
  add(parent, parentKey, [], 0);
  for (const action of parent.actions ?? []) {
    if (!action.portal) continue;
    if (matched.has(action)) {
      const id = action.catalogId!;
      delete out.metaDetails[id];
      if (!snapshot.catalog.Action?.includes(id)) {
        out.catalog.Action = out.catalog.Action.filter(item => item !== id);
        delete out.entityOverview![id];
        delete entitySources[id];
      }
      continue;
    }
    const key = "child:" + JSON.stringify([action.portal.key]);
    actionRows.push({ ...action, source: sources[key] ? key : undefined });
  }
  return out;
}

export function sourceLabel(data: ReportData, key: string | undefined): string {
  if (!key) return "";
  const source = data.sources?.[key];
  if (!source) return "";
  const label = source.local ? (data.organization ? "Federation standard" : "Local")
    : source.repository ? "Source repository" : "Child-defined";
  return label + " · " + source.title;
}

export function entitySource(data: ReportData, id: string): string {
  return sourceLabel(data, data.entitySources?.[id]?.source);
}

export function entitySourceKeys(data: ReportData, id: string): string[] {
  const primary = data.entitySources?.[id]?.source;
  return [...new Set([...(primary ? [primary] : []), ...(data.entityObservations?.[id] ?? []).map(item => item.source)])];
}
