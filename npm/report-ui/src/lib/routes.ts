import type { CatalogReport, MetaDetail, ReportData } from "../types";
import { el, enc, link } from "./dom";
import { known } from "./levels";
import { title } from "./rings";
import type { AssessmentFilters } from "./assessments";
import { campaignNamesSorted } from "./summary";

export interface Query extends AssessmentFilters {
  level?: string;
  outcome?: string;
  rule?: string;
  entity?: string;
  group?: string;
  source?: string;
  filter?: string;
}

export interface Route {
  mode: string;
  kind?: string;
  id?: string;
  name?: string;
  scorecard?: string;
  rule?: string;
  bare?: boolean;
  query?: Query;
  catalogId?: string;
}

export function parseHash(hash: string, extraViews: readonly string[] = []): Route {
  var h = hash.replace(/^#/, "");
  var query: Query = {};
  var qi = h.indexOf("?");
  if (qi >= 0) {
    var params = new URLSearchParams(h.slice(qi + 1));
    ["q", "level", "outcome", "rule", "entity", "group", "source", "rubric", "verdict", "filter"].forEach(function (k) {
      if (params.has(k)) query[k as keyof Query] = params.get(k) || "";
    });
    h = h.slice(0, qi);
  }
  try {
    return { ...(extraViews.includes(h) ? { mode: h } : parsePath(h)), query };
  } catch {
    return { mode: "missing" };
  }
}

export function parsePath(h: string): Route {
  if (!h || h === "overview") return { mode: "overview" };
  if (h === "actions") return { mode: "actions" };
  if (h.startsWith("action/")) return { mode: "action", id: decodeURIComponent(h.slice(7)) };
  if (h === "assessments" || h === "rubrics") return { mode: h };
  if (h.startsWith("assessment/")) return { mode: "assessment", id: decodeURIComponent(h.slice(11)) };
  if (h.startsWith("rubric/")) return { mode: "rubric", id: decodeURIComponent(h.slice(7)) };
  if (h === "catalog") return { mode: "catalog" };
  if (h.indexOf("catalog/") === 0) return { mode: "catalog", kind: decodeURIComponent(h.slice(8)) };
  if (h.indexOf("entity/") === 0) return { mode: "entity", id: decodeURIComponent(h.slice(7)) };
  if (h === "tracks") return { mode: "tracks" };
  if (h.indexOf("track/") === 0) return { mode: "tracks", name: decodeURIComponent(h.slice(6)) };
  if (h.indexOf("rule/") === 0) {
    var rest = h.slice(5);
    var cut = rest.indexOf("/");
    return { mode: "rule", scorecard: decodeURIComponent(rest.slice(0, cut)), rule: decodeURIComponent(rest.slice(cut + 1)) };
  }
  if (h === "campaigns") return { mode: "campaigns" };
  if (h.indexOf("campaign/") === 0) return { mode: "campaign", name: decodeURIComponent(h.slice(9)) };
  if (h === "compare") return { mode: "compare" };
  if (h.indexOf("compare/") === 0) return { mode: "compare", name: decodeURIComponent(h.slice(8)) };
  if (h.indexOf("scorecard/") === 0) return { mode: "scorecard", name: decodeURIComponent(h.slice(10)) };
  if (h === "scorecards") return { mode: "scorecards" };
  return { mode: "scorecard", name: decodeURIComponent(h), bare: true };
}

export function slugOf(text) {
  return String(text)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

// Track links read as their titles; the manifest name still resolves, so older links keep working.
export function trackPath(D: ReportData, name) {
  var def = (D.tracks || {})[name] || {};
  var slug = def.title ? slugOf(def.title) : "";
  var clash = Object.keys(D.tracks || {}).some(function (n) {
    return n !== name && (n === slug || slugOf(((D.tracks || {})[n] || {}).title || "") === slug);
  });
  return "track/" + enc(slug && !clash ? slug : name);
}

export function trackByPath(D: ReportData, key) {
  if ((D.tracks || {})[key]) return key;
  var hit = Object.keys(D.tracks || {}).filter(function (n) {
    return trackPath(D, n) === "track/" + enc(key);
  })[0];
  return hit || key;
}

type DetailRoute = Route & (
  | { mode: "scorecard" | "compare" | "campaign" | "tracks"; name: string }
  | { mode: "rule"; scorecard: string; rule: string }
  | { mode: "rubric" | "assessment" | "action"; id: string }
);

function metaRoute(data: ReportData, id: string, detail: MetaDetail): DetailRoute {
  switch (detail.kind) {
    case "Scorecard": return { mode: data.comparisons?.[detail.name] ? "compare" : "scorecard", name: detail.name };
    case "Rule": return { mode: "rule", scorecard: detail.name, rule: detail.rule };
    case "Track": return { mode: "tracks", name: detail.name };
    case "Campaign": return { mode: "campaign", name: detail.name };
    case "Rubric": return { mode: "rubric", id: detail.name };
    case "Assessment": return { mode: "assessment", id };
    case "Action": return { mode: "action", id };
  }
}

export function catalogEntityPath(data: CatalogReport, id: string): string {
  const detail = data.metaDetails[id];
  if (!detail) return "entity/" + enc(id);
  const route = metaRoute(data, id, detail);
  switch (route.mode) {
    case "rule": return "rule/" + enc(route.scorecard) + "/" + enc(route.rule);
    case "tracks": return trackPath(data, route.name);
    case "scorecard":
    case "compare":
    case "campaign": return route.mode + "/" + enc(route.name);
    default: return route.mode + "/" + enc(route.id);
  }
}

/** Give old catalog detail links the same address as links from the report sections. */
export function canonicalDetailHash(data: CatalogReport, hash: string): string {
  if (data.publicCatalog || !hash.startsWith("#entity/")) return hash;
  const { id } = parseHash(hash);
  if (!id || !data.metaDetails[id] && !data.entityRedirects?.[id]) return hash;
  const query = hash.indexOf("?");
  return "#" + catalogEntityPath(data, data.entityRedirects?.[id] ?? id) + (query >= 0 ? hash.slice(query) : "");
}

/** Resolve catalog and section links to the same detail and keep their existing URL formats. */
export function reportRoute(data: CatalogReport, route: Route): Route {
  if (route.mode === "entity" && route.id && data.entityRedirects?.[route.id]) {
    route = { ...route, id: data.entityRedirects[route.id] };
  }
  if (data.viewerMethod === "guest" && data.organization && route.mode === "entity" && route.id) {
    const legacy = /^repository:([^/]+)$/.exec(route.id);
    const id = legacy && `repository:${data.organization.toLowerCase()}/${legacy[1]}`;
    if (id && data.catalog.Repository?.includes(id)) route = { ...route, id };
  }
  if (route.mode === "campaigns") {
    const name = campaignNamesSorted(data)[0];
    if (name) route = { ...route, mode: "campaign", name };
  }
  if (route.mode === "tracks" && route.name) route = { ...route, name: trackByPath(data, route.name) };
  if (route.mode === "scorecard" && route.name && data.comparisons?.[route.name]) route = { ...route, mode: "compare" };
  if (route.mode === "entity" && route.id && data.metaDetails[route.id]) {
    return { ...metaRoute(data, route.id, data.metaDetails[route.id]), query: route.query, catalogId: route.id };
  }
  if (!route.id && !route.name && !route.rule) return route;
  for (const [id, detail] of Object.entries(data.metaDetails)) {
    const target = metaRoute(data, id, detail);
    if (target.mode === route.mode && target.id === route.id && target.name === route.name
        && target.scorecard === route.scorecard && target.rule === route.rule) return { ...route, catalogId: id };
  }
  return route;
}

export function missingTarget(D: ReportData, route: Route) {
  if (route.bare && !(D.scorecards || {})[route.name || ""]) return "The link “#" + route.name + "” doesn't match anything in snapshot " + D.snapshot + ".";
  if (route.mode === "scorecard" && route.name && !(D.scorecards || {})[route.name]) return "No scorecard is named “" + route.name + "”.";
  if (route.mode === "tracks" && route.name && !(D.tracks || {})[route.name]) return "No track is named “" + route.name + "”.";
  if (route.mode === "rule") {
    var card = route.scorecard || "";
    var rsc = (D.scorecards || {})[card];
    if (!rsc) return "No scorecard is named “" + card + "”.";
    if (!(rsc.rules || {})[route.rule || ""]) return title(D, card) + " has no rule named “" + route.rule + "”.";
  }
  if (route.mode === "campaign" && !(D.campaigns || {})[route.name || ""]) return "No campaign is named “" + route.name + "”.";
  if (route.mode === "compare" && route.name && !(D.comparisons || {})[route.name]) return "No comparison is named “" + route.name + "”.";
  if (route.mode === "catalog" && route.kind && !(D.catalog || {})[route.kind]) return "No kind is named “" + route.kind + "”.";
  if (route.mode === "entity") {
    var known = Object.values(D.catalog || {}).some(function (ids) {
      return ids.indexOf(route.id || "") >= 0;
    });
    if (!known) return "No entity has the id “" + route.id + "” in this snapshot.";
  }
  return null;
}

export function notFound(message) {
  return el(
    "section",
    {},
    el("h2", { textContent: "Not found" }),
    el("p", { className: "lede", textContent: message + " The link may be from an older report." }),
    el("p", {}, link("overview", "", "Go to the overview"), " · ", link("catalog", "", "Browse the catalog")),
  );
}
