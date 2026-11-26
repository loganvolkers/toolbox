import snapshot from "../fixtures/s4s.json";
import { fixture, page, routes } from "./report";

const s4s = fixture(snapshot);

const r = routes(s4s, { track: "delivery", kind: "Crate", entity: "crate:seikika-cli", scorecard: "maintainer-skill", rule: "has-example", compare: "vs-backstage-capabilities" });

export default { title: "Pages/Seikika for Seikika" };

export const Overview = page(s4s, r.overview);
export const Tracks = page(s4s, r.tracks);
export const Track = page(s4s, r.track);
export const Catalog = page(s4s, r.catalog);
export const CatalogKind = page(s4s, r.catalogKind);
export const Entity = page(s4s, r.entity);
export const Campaigns = page(s4s, r.campaigns);
export const Campaign = page(s4s, r.campaign);
export const Scorecards = page(s4s, r.scorecards);
export const Scorecard = page(s4s, r.scorecard);
export const Rule = page(s4s, r.rule);
export const Compare = page(s4s, r.compare);
export const CompareFactor = page(s4s, r.compareFactor);
export const NotFound = page(s4s, r.notFound);
