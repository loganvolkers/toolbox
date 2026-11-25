import openfgaJson from "../fixtures/openfga.json";
import { fixture, page, routes } from "./report";

const openfga = fixture(openfgaJson);

const r = routes(openfga, { track: "vs-spicedb", kind: "GoFile", scorecard: "go-file-size", compare: "vs-spicedb-performance" });

export default { title: "Pages/OpenFGA" };

export const Overview = page(openfga, r.overview);
export const Tracks = page(openfga, r.tracks);
export const Track = page(openfga, r.track);
export const Catalog = page(openfga, r.catalog);
export const CatalogKind = page(openfga, r.catalogKind);
export const Entity = page(openfga, r.entity);
export const Campaigns = page(openfga, r.campaigns);
export const Campaign = page(openfga, r.campaign);
export const Scorecards = page(openfga, r.scorecards);
export const Scorecard = page(openfga, r.scorecard);
export const Rule = page(openfga, r.rule);
export const Compare = page(openfga, r.compare);
export const CompareFactor = page(openfga, r.compareFactor);
export const NotFound = page(openfga, r.notFound);
