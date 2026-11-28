// Keep the docs catalog in step with the PNPM workspace's public packages.
import { globSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const catalog = join(root, "npm/site/src/data/npm-packages.json");
const entries = globSync("npm/*/package.json", { cwd: root }).flatMap(manifest => {
  const pkg = JSON.parse(readFileSync(join(root, manifest), "utf8"));
  if (pkg.private) return [];
  if (pkg.publishConfig?.access !== "public" || !pkg.version || !pkg.description || !pkg.exports) {
    throw new Error("Declare access, version, description, and exports for " + pkg.name);
  }
  return [{
    name: pkg.name,
    description: pkg.description,
    source: dirname(manifest).replaceAll("\\", "/"),
    registry: "https://www.npmjs.com/package/" + pkg.name,
  }];
}).sort((a, b) => a.name.localeCompare(b.name));
const expected = JSON.stringify(entries, null, 2) + "\n";

if (process.argv.includes("--write")) {
  writeFileSync(catalog, expected);
} else if (readFileSync(catalog, "utf8") !== expected) {
  throw new Error("Public package catalog is stale. Run pnpm check:packages --write.");
}
