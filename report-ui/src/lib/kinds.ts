import type { ReportData } from "../types";
import { title } from "./rings";
import { catalogKindsSorted } from "./summary";
import { fmt, plural } from "./words";

// Kinds in their declared groups, then any kind no group names.
export function kindGroupsOf(D: ReportData) {
  var catalog = D.catalog || {};
  var seen: Record<string, boolean> = {};
  var groups: { name: string; title: string; description?: string | null; kinds: string[]; count?: number }[] = (D.kindGroups || [])
    .map(function (g) {
      var kinds = (g.kinds || []).filter(function (k) {
        seen[k] = true;
        return catalog[k];
      });
      return { name: g.name, title: g.title || g.name, description: g.description, kinds: kinds };
    })
    .filter(function (g) {
      return g.kinds.length;
    });
  var rest = catalogKindsSorted(D).filter(function (k) {
    return !seen[k];
  });
  if (rest.length) groups.push({ name: "", title: groups.length ? "Other kinds" : "Kinds", kinds: rest });
  groups.forEach(function (g) {
    g.count = g.kinds.reduce(function (n, k) {
      return n + catalog[k].length;
    }, 0);
  });
  return groups;
}

export function kindDescription(D: ReportData, k) {
  return (((D.kinds || {})[k] || {}).description || "").trim();
}

export var ACRONYMS = { Api: "API", Cli: "CLI", Json: "JSON", Mcp: "MCP", Sdlc: "SDLC", Ui: "UI", Url: "URL" };
// A kind's id (SdlcLayer) in the reader's words (SDLC layer), unless the kind names a title.
export function kindTitle(D: ReportData, k) {
  var t = ((D.kinds || {})[k] || {}).title;
  if (t) return t;
  var words = String(k || "").match(/OpenSSF|[A-Z]{2,}(?![a-z])|[A-Z][a-z0-9]*|[a-z0-9]+/g) || [k];
  return words
    .map(function (w, i) {
      if (ACRONYMS[w]) return ACRONYMS[w];
      if (w === "OpenSSF" || /^[A-Z]{2,}$/.test(w)) return w;
      return i ? w.toLowerCase() : w;
    })
    .join(" ");
}

// "15 action dependencies", "26 SDLC stages": a count of a kind, in running text.
export function kindCount(D: ReportData, k, n) {
  var t = kindTitle(D, k);
  if (!/^(?:[A-Z]{2,}|OpenSSF|GitHub|(?:Go|Python|Rust|Cargo|TypeScript|JavaScript)\b)/.test(t)) t = t.charAt(0).toLowerCase() + t.slice(1);
  var tail = (t.match(/ \([^)]*\)$/) || [""])[0];
  t = t.slice(0, t.length - tail.length);
  if (n !== 1) t = /[^aeiou]y$/.test(t) ? t.slice(0, -1) + "ies" : /(s|x|ch|sh)$/.test(t) ? t + "es" : t + "s";
  return fmt(n) + " " + t + tail;
}

// The Scorecard kind counts more than the Scorecards tab: say what the surplus is wherever its total appears.
export function scorecardSplit(D: ReportData, k) {
  if (k !== "Scorecard") return "";
  var ids = (D.catalog || {})[k] || [];
  var hits = {};
  var split = ids.map(function (id) {
    var m = /^[^:]+:([^/]+)\/(.+)$/.exec(id);
    if (m && (D.scorecards || {})[m[2]]) hits[m[1]] = (hits[m[1]] || 0) + 1;
    return m;
  });
  var mine = Object.keys(hits).sort(function (a, b) {
    return hits[b] - hits[a];
  })[0];
  var own = 0;
  var cmp = 0;
  split.forEach(function (m) {
    if (!m || m[1] !== mine || !(D.scorecards || {})[m[2]]) return;
    if ((D.comparisons || {})[m[2]]) cmp++;
    else own++;
  });
  var parts = [fmt(own) + " of this portal's own", cmp ? plural(cmp, "comparison factor") : null, ids.length - own - cmp ? fmt(ids.length - own - cmp) + " in example portals" : null].filter(Boolean);
  return parts.length > 1 ? " (" + parts.join(", ") + ")" : "";
}
