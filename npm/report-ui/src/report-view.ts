import type { CatalogReport } from "./types";
import type { Summary } from "./lib/summary";
import { missingTarget, notFound, type Route } from "./lib/routes";
import { overviewViews } from "./views/overview";
import { scorecardsViews } from "./views/scorecards";
import { tracksViews } from "./views/tracks";
import { campaignsViews } from "./views/campaigns";
import { catalogViews } from "./views/catalog";
import { entityViews } from "./views/entity";
import { ruleViews } from "./views/rule";
import { compareViews } from "./views/compare";

const views = {
  ...overviewViews, ...scorecardsViews, ...tracksViews, ...campaignsViews,
  ...catalogViews, ...entityViews, ...ruleViews, ...compareViews,
};
type Methods = typeof overviewViews & typeof scorecardsViews & typeof tracksViews &
  typeof campaignsViews & typeof catalogViews & typeof entityViews & typeof ruleViews & typeof compareViews;
export interface ReportViews extends Methods {}

export function buildView(data: CatalogReport, summary: Summary, route: Route, catalogIds?: ReadonlySet<string>): HTMLElement {
  const missing = missingTarget(data, route);
  if (missing) return notFound(missing);
  switch (route.mode) {
    case "overview": return views.overview(data, summary);
    case "catalog": return views.catalog(data, summary, route.kind, route.query ?? {}, catalogIds);
    case "entity": return views.entity(data, summary, route.id);
    case "campaigns":
    case "campaign": return views.campaigns(data, summary, route.name);
    case "compare": return views.compare(data, route.name);
    case "tracks": return views.tracks(data, summary, route.name);
    case "rule": return views.rule(data, summary, route.scorecard, route.rule);
    case "scorecards": return views.scorecardIndex(data, summary, route.query ?? {});
    case "scorecard": return views.scorecard(data, summary, route.name, route.query ?? {});
    default: return notFound("This link could not be read.");
  }
}
