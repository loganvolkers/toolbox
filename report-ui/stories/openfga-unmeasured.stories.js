import openfgaUnmeasured from "../fixtures/openfga-unmeasured.json";
import { page, routes } from "./report.js";

const r = routes(openfgaUnmeasured, { track: "vs-spicedb", compare: "vs-spicedb-performance" });

export default { title: "States/Comparison not measured yet" };

export const Overview = page(openfgaUnmeasured, r.overview);
export const Tracks = page(openfgaUnmeasured, r.tracks);
export const Track = page(openfgaUnmeasured, r.track);
export const Compare = page(openfgaUnmeasured, r.compare);
export const CompareFactor = page(openfgaUnmeasured, r.compareFactor);
