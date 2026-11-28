import starterEmptyJson from "../fixtures/starter-empty.json";
import { fixture, page, routes } from "./report";

const starterEmpty = fixture(starterEmptyJson);

const r = routes(starterEmpty, {});

export default { title: "States/No data yet" };

export const Overview = page(starterEmpty, r.overview);
export const Catalog = page(starterEmpty, r.catalog);
export const Scorecards = page(starterEmpty, r.scorecards);
export const Scorecard = page(starterEmpty, r.scorecard);
