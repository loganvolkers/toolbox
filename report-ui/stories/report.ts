// Story helpers: a page is the whole <toolbox-report> over a fixture at one route; a component
// is a library function's output inside a shadow root that carries the report's stylesheet.
import { CSS } from "../src/lib/styles";
import type { ReportData } from "../src/types";

type Pins = Partial<Record<"scorecard" | "kind" | "entity" | "track" | "campaign" | "rule" | "compare", string>>;

// Fixtures are written by `toolbox report`, whose tests hold the shape; a JSON import types its
// strings too loosely to check against ReportData.
export function fixture(json: unknown): ReportData {
  return json as ReportData;
}

export function page(data: ReportData, route: string | null) {
  return {
    args: { route },
    render: (args) => {
      history.replaceState(null, "", "#" + args.route);
      const el = document.createElement("toolbox-report");
      const json = document.createElement("script");
      json.type = "application/json";
      json.textContent = JSON.stringify(data);
      el.append(json);
      return el;
    },
  };
}

// The pages a portal has, chosen from its data so the same list works for any fixture.
export function routes(D: ReportData, pins: Pins = {}) {
  const first = (o) => Object.keys(o || {}).sort()[0];
  const biggest = (o) => Object.keys(o || {}).sort((a, b) => (o[b] || []).length - (o[a] || []).length)[0];
  const scorecard = pins.scorecard || first(Object.fromEntries(Object.entries(D.scorecards || {}).filter(([n]) => !(D.comparisons || {})[n])));
  const rule = scorecard && first((D.scorecards[scorecard] || {}).rules);
  const kind = pins.kind || biggest(D.catalog);
  const entity = pins.entity || (kind && (D.catalog[kind] || [])[0]);
  const r = {
    overview: "overview",
    tracks: "tracks",
    track: Object.keys(D.tracks || {}).length ? "track/" + (pins.track || first(D.tracks)) : null,
    catalog: "catalog",
    catalogKind: kind ? "catalog/" + kind : null,
    entity: entity ? "entity/" + encodeURIComponent(entity) : null,
    campaigns: "campaigns",
    campaign: first(D.campaigns) ? "campaign/" + (pins.campaign || first(D.campaigns)) : null,
    scorecards: "scorecards",
    scorecard: scorecard ? "scorecard/" + scorecard : null,
    rule: rule ? "rule/" + scorecard + "/" + (pins.rule || rule) : null,
    compare: "compare",
    compareFactor: first(D.comparisons) ? "compare/" + (pins.compare || first(D.comparisons)) : null,
    notFound: "scorecard/no-such-scorecard",
  };
  return r;
}

export function component(build: () => Node | Node[]) {
  return {
    render: () => {
      const host = document.createElement("div");
      const root = host.attachShadow({ mode: "open" });
      const style = document.createElement("style");
      style.textContent = CSS;
      const main = document.createElement("main");
      main.style.padding = "24px";
      main.append(...([] as Node[]).concat(build()));
      root.append(style, main);
      return host;
    },
  };
}
