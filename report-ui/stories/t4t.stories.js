import t4t from "../fixtures/t4t.json";
import { page, routes } from "./report.js";

const r = routes(t4t, { track: "delivery", kind: "Crate", entity: "crate:toolbox-cli", scorecard: "maintainer-skill", rule: "has-example", compare: "vs-backstage-capabilities" });

export default { title: "Pages/Toolbox for Toolbox" };

export const Overview = page(t4t, r.overview);
export const Tracks = page(t4t, r.tracks);
export const Track = page(t4t, r.track);
export const Catalog = page(t4t, r.catalog);
export const CatalogKind = page(t4t, r.catalogKind);
export const Entity = page(t4t, r.entity);
export const Campaigns = page(t4t, r.campaigns);
export const Campaign = page(t4t, r.campaign);
export const Scorecards = page(t4t, r.scorecards);
export const Scorecard = page(t4t, r.scorecard);
export const Rule = page(t4t, r.rule);
export const Compare = page(t4t, r.compare);
export const CompareFactor = page(t4t, r.compareFactor);
export const NotFound = page(t4t, r.notFound);
