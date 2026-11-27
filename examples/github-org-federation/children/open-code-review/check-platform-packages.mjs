import { existsSync, readdirSync, readFileSync } from "node:fs";

function manifest(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

const root = manifest("package.json");
const platformPackages = new Map(Object.entries(root.optionalDependencies ?? {})
  .filter(([name]) => name.startsWith("@alibaba-group/ocr-")));
const directories = readdirSync("npm", { withFileTypes: true }).filter(entry => entry.isDirectory());
const findings = [];
const workflowPath = ".github/workflows/release.yml";
const platformLists = [...readFileSync(workflowPath, "utf8").matchAll(
  /^\s*PLATFORMS="([^"]+)"/gm
)];
if (platformLists.length !== 1) {
  throw new Error(`${workflowPath} must define one release PLATFORMS list`);
}
const releasePlatforms = platformLists[0][1].trim().split(/\s+/).map((entry) => entry.split(":")[0]);
const installedPlatforms = new Set(directories.map(({ name }) => name));
if (new Set(releasePlatforms).size !== releasePlatforms.length) {
  findings.push(`${workflowPath} repeats a release platform`);
}
for (const platform of releasePlatforms) {
  if (!installedPlatforms.has(platform) || !existsSync(`npm/${platform}/package.json`)) {
    findings.push(`${workflowPath} requires npm/${platform}/package.json`);
  }
}
if (directories.length === 0) findings.push("npm contains no platform package manifests");
for (const { name: platform } of directories) {
  const path = `npm/${platform}/package.json`;
  if (!releasePlatforms.includes(platform)) findings.push(`${workflowPath} does not release ${path}`);
  if (!existsSync(path)) continue;
  const pkg = manifest(path);
  const [os, cpu] = platform.split("-");
  const expected = `@alibaba-group/ocr-${platform}`;
  if (pkg.name !== expected) findings.push(`${path} must declare name ${expected}`);
  if (pkg.version !== root.version) findings.push(`${path} has version ${pkg.version} instead of ${root.version}`);
  if (platformPackages.get(expected) !== root.version) {
    findings.push(`package.json must declare optional dependency ${expected}@${root.version}`);
  }
  if (JSON.stringify(pkg.os) !== JSON.stringify([os]) || JSON.stringify(pkg.cpu) !== JSON.stringify([cpu])) {
    findings.push(`${path} must constrain installation to ${os}/${cpu}`);
  }
  platformPackages.delete(expected);
}
for (const name of platformPackages.keys()) findings.push(`Missing npm platform manifest for ${name}`);
if (findings.length) {
  console.error(findings.join("\n"));
  process.exitCode = 1;
}
