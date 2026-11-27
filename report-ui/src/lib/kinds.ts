import type { CatalogReport, ReportData } from "../types";
import { title } from "./rings";
import { catalogKindsSorted } from "./summary";
import { fmt, plural } from "./words";
import { kindTitle } from "./kind-title";
export { kindTitle } from "./kind-title";

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
export function scorecardSplit(D: ReportData & Partial<Pick<CatalogReport, "metaDetails">>, k: string) {
  if (k !== "Scorecard" || !D.metaDetails) return "";
  const ids = D.catalog[k] ?? [];
  let standards = 0;
  let comparisons = 0;
  for (const id of ids) {
    const detail = D.metaDetails[id];
    if (detail?.kind !== "Scorecard") continue;
    if (D.comparisons?.[detail.name]) comparisons++;
    else standards++;
  }
  const examples = ids.length - standards - comparisons;
  const parts = [
    standards ? plural(standards, "report standard") : null,
    comparisons ? plural(comparisons, "comparison factor") : null,
    examples ? fmt(examples) + " from described portals" : null,
  ].filter(Boolean);
  return parts.length > 1 ? " (" + parts.join(", ") + ")" : "";
}
