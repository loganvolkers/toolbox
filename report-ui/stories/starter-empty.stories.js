import starterEmpty from "../fixtures/starter-empty.json";
import { page, routes } from "./report.js";

const r = routes(starterEmpty, {});

export default { title: "States/No data yet" };

export const Overview = page(starterEmpty, r.overview);
export const Catalog = page(starterEmpty, r.catalog);
export const Scorecards = page(starterEmpty, r.scorecards);
export const Scorecard = page(starterEmpty, r.scorecard);
