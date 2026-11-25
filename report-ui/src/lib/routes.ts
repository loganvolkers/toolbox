import type { ReportData } from "../types";
import { el, enc, link } from "./dom";
import { known } from "./levels";
import { title } from "./rings";

export interface Query {
  q?: string;
  level?: string;
  outcome?: string;
  rule?: string;
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
}

export function parseHash(hash: string): Route {
  var h = hash.replace(/^#/, "");
  var query: Query = {};
  var qi = h.indexOf("?");
  if (qi >= 0) {
    var params = new URLSearchParams(h.slice(qi + 1));
    ["q", "level", "outcome", "rule"].forEach(function (k) {
      if (params.has(k)) query[k as keyof Query] = params.get(k) || "";
    });
    h = h.slice(0, qi);
  }
  try {
    return { ...parsePath(h), query };
  } catch {
    return { mode: "missing" };
  }
}

export function parsePath(h: string): Route {
  if (!h || h === "overview") return { mode: "overview" };
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
