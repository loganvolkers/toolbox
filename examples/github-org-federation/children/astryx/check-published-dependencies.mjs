import {
  alignmentProblems,
  canaryOnlyNames,
  publishedPackages,
  releaseVersions,
} from "../scripts/npm/published-manifests.mjs";

const root = process.cwd();
const packages = publishedPackages(root);
const versions = releaseVersions(root);
const canaryOnly = canaryOnlyNames(root);
const findings = packages.flatMap(({ dir, manifest }) =>
  alignmentProblems(manifest, versions, canaryOnly)
    .map(problem => `${dir}/package.json: ${problem}`));
if (packages.length === 0) findings.push("No published package manifests were found");
const pathsByName = new Map();
for (const { dir, manifest } of packages) {
  const paths = pathsByName.get(manifest.name) ?? [];
  paths.push(`${dir}/package.json`);
  pathsByName.set(manifest.name, paths);
}
for (const [name, paths] of pathsByName) {
  if (paths.length > 1) findings.push(`Duplicate published name ${name}: ${paths.join(", ")}`);
}
if (findings.length) {
  console.error(findings.join("\n"));
  process.exitCode = 1;
}
