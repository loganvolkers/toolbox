// Keep the docs catalog in step with the PNPM workspace's public packages.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const catalog = join(root, "site/src/data/npm-packages.json");
const workspaces = JSON.parse(execFileSync("pnpm", ["-r", "list", "--depth", "-1", "--json"], { cwd: root, encoding: "utf8" }));

const entries = workspaces.filter(workspace => !workspace.private).map(workspace => {
  const pkg = JSON.parse(readFileSync(join(workspace.path, "package.json"), "utf8"));
  if (pkg.publishConfig?.access !== "public" || !pkg.version || !pkg.description || !pkg.exports) {
    throw new Error("Declare access, version, description, and exports for " + pkg.name);
  }
  return {
    name: pkg.name,
    description: pkg.description,
    source: relative(root, workspace.path).replaceAll("\\", "/"),
    registry: "https://www.npmjs.com/package/" + pkg.name,
  };
}).sort((a, b) => a.name.localeCompare(b.name));
const expected = JSON.stringify(entries, null, 2) + "\n";

if (process.argv.includes("--write")) {
  writeFileSync(catalog, expected);
} else if (readFileSync(catalog, "utf8") !== expected) {
  throw new Error("Public package catalog is stale. Run pnpm check:packages --write.");
}
