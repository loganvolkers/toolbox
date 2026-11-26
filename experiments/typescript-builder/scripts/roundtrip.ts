// Compare the builder's output with the hand-written reference manifest.
// CEL and prose strings are compared with whitespace runs collapsed (YAML folding vs TS templates);
// everything else must be equal.
import { readFileSync } from "node:fs";
import { parse } from "yaml";
import { canonical } from "../src/index.js";
import { manifest } from "../portal/acme.js";

const norm = (v: unknown): unknown =>
  typeof v === "string" ? v.replace(/\s+/g, " ").replace(/([{[(]) /g, "$1").replace(/ ([}\])])/g, "$1").trim()
  : Array.isArray(v) ? v.map(norm)
  : v && typeof v === "object" ? Object.fromEntries(Object.entries(v).map(([k, x]) => [k, norm(x)])) : v;

const yaml = parse(readFileSync(new URL("../../../examples/reference-portal/seikika.yaml", import.meta.url), "utf8"));
const a = canonical(norm(manifest)).split("\n");
const b = canonical(norm(yaml)).split("\n");
const diffs = a.map((l, i) => [i, l, b[i]] as const).filter(([, l, r]) => l !== r);
if (diffs.length || a.length !== b.length) {
  for (const [i, l, r] of diffs.slice(0, 20)) console.log(`line ${i + 1}\n  builder: ${l}\n  yaml:    ${r}`);
  console.log(`DIFFERENT (${diffs.length} lines, ${a.length} vs ${b.length})`);
  process.exit(1);
}
console.log(`identical: builder output equals examples/reference-portal/seikika.yaml (${a.length} canonical lines)`);
