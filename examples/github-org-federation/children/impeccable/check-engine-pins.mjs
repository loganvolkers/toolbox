import { readFileSync } from "node:fs";

const expectedAssets = [
  "impeccable-darwin-arm64",
  "impeccable-darwin-x64",
  "impeccable-linux-arm64",
  "impeccable-linux-x64",
  "impeccable-windows-x64.exe",
];
const pin = readFileSync("ENGINE_VERSION", "utf8").trim();
const launcherPin = readFileSync("skill/scripts/VERSION", "utf8").trim();
const checksumPath = "skill/scripts/engine-checksums.txt";
const rows = readFileSync(checksumPath, "utf8").split("\n")
  .map(line => line.trim())
  .filter(line => line && !line.startsWith("#"))
  .map(line => line.split(/\s+/));

const findings = [];
if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(pin)) {
  findings.push("ENGINE_VERSION must contain a released major.minor.patch version.");
}
if (pin !== launcherPin) {
  findings.push(`skill/scripts/VERSION is ${launcherPin}, but ENGINE_VERSION is ${pin}.`);
}
for (const asset of expectedAssets) {
  const matching = rows.filter(([, name]) => name === asset);
  if (matching.length !== 1 || matching[0].length !== 3 ||
      matching[0][0] !== pin || !/^[a-f0-9]{64}$/.test(matching[0][2]) || /^0+$/.test(matching[0][2])) {
    findings.push(`${checksumPath} requires one nonzero SHA-256 entry for ${asset} at ${pin}.`);
  }
}
if (rows.length !== expectedAssets.length) {
  findings.push(`${checksumPath} has ${rows.length} asset rows. Expected ${expectedAssets.length}.`);
}
if (findings.length) {
  console.error(findings.join("\n"));
  process.exitCode = 1;
}
