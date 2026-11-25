import starterJson from "../fixtures/starter.json";
import { fixture, page, routes } from "./report";

const starter = fixture(starterJson);

const r = routes(starter, { kind: "Doc" });

export default { title: "Pages/Repo portal starter" };

export const Overview = page(starter, r.overview);
export const Tracks = page(starter, r.tracks);
export const Catalog = page(starter, r.catalog);
export const CatalogKind = page(starter, r.catalogKind);
export const Entity = page(starter, r.entity);
export const Campaigns = page(starter, r.campaigns);
export const Scorecards = page(starter, r.scorecards);
export const Scorecard = page(starter, r.scorecard);
export const Rule = page(starter, r.rule);
export const Compare = page(starter, r.compare);
