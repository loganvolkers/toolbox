import { h } from "vue";
import PageFrame from "@toolbox-dev/ui/components/PageFrame.vue";
import ReportApp from "../src/ReportApp.vue";
import ReportContent from "../src/ReportContent.vue";
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
    render: (args: { route: string | null }) => ({
      setup: () => () => h(ReportApp, { data, initialRoute: args.route ?? "overview" }),
    }),
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
    render: () => ({
      setup: () => () => h(PageFrame, { class: "toolbox-report" }, {
        default: () => h(ReportContent, { build }),
      }),
    }),
  };
}
