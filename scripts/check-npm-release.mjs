import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const packages = ["@seikika/ui", "@seikika/astro-report"];
const registry = "https://registry.npmjs.org";
const provenanceType = "https://slsa.dev/provenance/v1";

function packedTarball(directory, name) {
  const prefix = name.slice(1).replace("/", "-") + "-";
  const matches = readdirSync(directory).filter(file => file.startsWith(prefix) && file.endsWith(".tgz"));
  assert.equal(matches.length, 1, `Expected one packed tarball for ${name} in ${directory}`);
  return resolve(directory, matches[0]);
}

function checkProvenance(npm, installed) {
  const audit = JSON.parse(npm("audit", "signatures", "--json", "--include-attestations"));
  for (const [name, version] of Object.entries(installed)) {
    const entry = audit.verified.find(pkg => pkg.name === name && pkg.version === version);
    const provenance = entry?.attestationBundles?.find(bundle => bundle.predicateType === provenanceType);
    assert(provenance, `Missing verified npm provenance for ${name}@${version}`);
    const statement = JSON.parse(Buffer.from(provenance.bundle.dsseEnvelope.payload, "base64").toString());
    const workflow = statement.predicate.buildDefinition.externalParameters.workflow;
    assert.equal(workflow.repository, "https://github.com/seikika/seikika", `Publisher repository for ${name}`);
    assert.equal(workflow.path, ".github/workflows/publish-npm.yml", `Publishing workflow for ${name}`);
    assert.equal(workflow.ref, "refs/heads/main", `Publishing ref for ${name}`);
    console.log(`Verified npm provenance for ${name}@${version}`);
  }
}

const [mode, ...args] = process.argv.slice(2);
const packed = mode === "packed" && args.length === 1;
const published = mode === "registry" && args.length === 2 &&
  args.every(version => version === "latest" || /^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.+-]+)?$/.test(version));
assert(packed || published, "Usage: check-npm-release.mjs packed <tarball-dir> | registry <ui-version|latest> <astro-report-version|latest>");

const specs = packages.map((name, index) => packed ? packedTarball(args[0], name) : `${name}@${args[index]}`);
const work = mkdtempSync(join(tmpdir(), "seikika-npm-smoke-"));
try {
  writeFileSync(join(work, "package.json"), JSON.stringify({ name: "seikika-npm-smoke", private: true, type: "module" }));
  writeFileSync(join(work, ".npmrc"), "");
  const env = {
    ...process.env,
    ASTRO_TELEMETRY_DISABLED: "1",
    XDG_CONFIG_HOME: work,
    npm_config_cache: join(work, "npm-cache"),
    npm_config_userconfig: join(work, ".npmrc"),
    npm_config_globalconfig: join(work, "global.npmrc"),
  };
  delete env.NODE_AUTH_TOKEN;
  delete env.NPM_TOKEN;
  const options = { cwd: work, env, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 };
  const npm = (...args) => execFileSync("npm", [...args, "--registry", registry], options);

  console.log(npm("install", "--ignore-scripts", "--no-audit", "--no-fund", "--save-exact", ...specs, "astro@7", "vue@3.5"));
  const lockfile = JSON.parse(readFileSync(join(work, "package-lock.json"), "utf8"));
  const installed = Object.fromEntries(packages.map((name, index) => {
    const path = `node_modules/${name}`;
    const { version } = lockfile.packages[path];
    if (published && args[index] !== "latest") assert.equal(version, args[index], `Installed version of ${name}`);
    assert.deepEqual(
      Object.keys(lockfile.packages).filter(key => key.endsWith(path)),
      [path],
      `Expected the consumer and integration to use the same installed ${name}`,
    );
    return [name, version];
  }));
  if (published) checkProvenance(npm, installed);

  mkdirSync(join(work, "src/pages"), { recursive: true });
  copyFileSync(new URL("../npm/report-ui/fixtures/starter-empty.json", import.meta.url), join(work, "report.json"));
  writeFileSync(join(work, "astro.config.mjs"), `
import { defineConfig } from "astro/config";
import seikikaReport from "@seikika/astro-report";

export default defineConfig({
  integrations: [seikikaReport({ data: new URL("./report.json", import.meta.url), path: "/standards" })],
});
`);
  writeFileSync(join(work, "src/pages/index.astro"), `---
import "@seikika/ui/theme.css";
import * as entity from "@seikika/ui/entity";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
---
<PageHeader title="Seikika npm consumer" description={Object.keys(entity).length + " entity helpers"} />
`);
  execFileSync(process.execPath, ["node_modules/astro/bin/astro.mjs", "build"], { ...options, stdio: "inherit" });
  const page = readFileSync(join(work, "dist/index.html"), "utf8").replace(/<!--[\s\S]*?-->/g, "");
  assert.match(page, /<h1>Seikika npm consumer<\/h1>/);
  assert.match(page, /[1-9][0-9]* entity helpers/);
  const report = readFileSync(join(work, "dist/standards/index.html"), "utf8");
  assert.match(report, /id="seikika-report-data"/);
  assert.match(report, /Repository standards/);
  console.log(`Consumer build passed for ${Object.entries(installed).map(([name, version]) => `${name}@${version}`).join(", ")}`);
} finally {
  rmSync(work, { recursive: true, force: true });
}
