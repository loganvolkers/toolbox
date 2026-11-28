import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, mkdir, open, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const checker = fileURLToPath(new URL("./check-coupling.mjs", import.meta.url));

test("coupling evidence and blocking mode agree on clean and forbidden dependencies", async () => {
  const root = await mkdtemp(join(tmpdir(), "seikika-coupling-"));
  const write = async (rel, text) => {
    const path = join(root, rel);
    await mkdir(dirname(path), { recursive: true });
    await writeFile(path, typeof text === "string" ? text : JSON.stringify(text));
  };
  const run = async (...args) => {
    const stdoutFile = join(root, "stdout");
    const stderrFile = join(root, "stderr");
    const stdout = await open(stdoutFile, "w");
    const stderr = await open(stderrFile, "w");
    try {
      const code = await new Promise((resolve, reject) => {
        const child = spawn(process.execPath, [checker, `--root=${root}`, ...args],
          { stdio: ["ignore", stdout.fd, stderr.fd] });
        child.once("error", reject);
        child.once("close", resolve);
      });
      return { code, stdout: await readFile(stdoutFile, "utf8"), stderr: await readFile(stderrFile, "utf8") };
    } finally {
      await Promise.all([stdout.close(), stderr.close()]);
    }
  };
  const records = async () => (await run("--ndjson")).stdout.trim().split("\n").map(line => JSON.parse(line).data);
  const crate = (name, dependencies = "") => `[package]\nname = "${name}"\nversion = "1.0.0"\n${dependencies}\n`;
  const a = { name: "@fixture/a", exports: { "./ui": "./src/index.ts" } };
  const b = { name: "@fixture/b", dependencies: { "@fixture/a": "workspace:*" } };
  try {
    await write("Cargo.toml", '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n');
    for (const name of ["a", "b"]) {
      await write(`crates/${name}/Cargo.toml`, crate(name));
      await write(`crates/${name}/src/lib.rs`, "");
    }
    await write("npm/a/package.json", a);
    await write("npm/b/package.json", b);
    await write("npm/a/src/index.ts", "export const value = 42;");
    await write("npm/b/src/screen.vue", '<script setup lang="ts">import { value } from "@fixture/a/ui";</script><template>{{ value }}</template>');
    await write("npm/b/tsconfig.base.json", { compilerOptions: { paths: { "~local/*": ["./src/*"] } } });
    await write("npm/b/tsconfig.json", { extends: "./tsconfig.base.json" });
    await write("npm/b/src/local.astro", '---\nimport Screen from "~local/screen.vue";\n---\n<Screen />');
    await write("npm/a/integration.mjs", [
      'export default { vite: { resolve: { alias: {',
      '"@astrojs/vue/client.js": fileURLToPath(import.meta.resolve("@astrojs/vue/client.js")),',
      '} } } };',
    ].join("\n"));
    const clean = await run();
    assert.equal(clean.code, 0, clean.stderr);
    assert.match(clean.stdout, /4 modules, 1 internal edges, 0 findings/);

    await write("crates/a/Cargo.toml", crate("a", '[dev-dependencies]\nb = { path = "../b" }'));
    await write("crates/b/Cargo.toml", crate("b", '[target.\'cfg(unix)\'.dependencies]\na = { package = "a", path = "../a" }'));
    await write("crates/a/src/lib.rs", 'const DATA: &str = include_str!("../../../.seikika/seikika.yaml");');
    await write("npm/a/package.json", { ...a, dependencies: { "@fixture/b": "workspace:*" } });
    await write("npm/b/src/private.astro", [
      "---",
      'import value from "../../a/src/index";',
      'import catalog from "../../../examples/catalog.json";',
      'import unused from "@fixture/a/private";',
      "---",
      '<a href="#scorecard/s4s-only">Report</a>',
      '<a href="https://seikika.com/demo/organization/">Organization</a>',
      '<a href="https://demo.seikika.com/">Demo</a>',
    ].join("\n"));
    await write("examples/catalog.json", {});
    await write("examples/script.js", "export default 42;");
    await write("npm/b/tsconfig.json", {
      extends: "./tsconfig.base.json",
      compilerOptions: { paths: {
        "~local/*": ["./src/*"],
        "~deployment/*": ["../../examples/*"],
        "~private/*": ["../a/src/*"],
      } },
    });
    await write("npm/b/src/aliases.astro", [
      "---",
      'import catalog from "~deployment/catalog.json";',
      'import privateValue from "~private/index";',
      "---",
      "<!-- A comment with a script-like tag has no dependency. <script>import '~private/ignored-comment';</script> -->",
      '🍕<script>import missing from "~private/missing";</script>',
      '<script src="~deployment/script.js"></script>',
    ].join("\n"));
    await write("npm/b/astro.config.mjs", 'export default { vite: { resolve: { alias: { sneaky: "../../examples" } } } };');
    const result = await records();
    assert.deepEqual(result.find(item => item.name === "a").cycles, ["a -> b -> a"]);
    assert.match(result.find(item => item.name === "a").boundaries[0], /includes \.seikika\/seikika.yaml/);
    const checked = result.find(item => item.name === "@fixture/b");
    assert.deepEqual(checked.cycles, ["@fixture/b -> @fixture/a -> @fixture/b"]);
    assert.equal(checked.entrypoints.length, 5);
    assert.ok(checked.entrypoints.some(finding => finding.includes("defines Vite resolve.alias")));
    assert.ok(checked.entrypoints.some(finding => finding.includes("bypasses the package exports: ~private/index")));
    assert.ok(checked.entrypoints.some(finding => finding.includes("unresolved path alias: ~private/missing")));
    assert.ok(checked.boundaries.some(finding => finding.includes("imports deployment data: ~deployment/catalog.json")));
    assert.ok(checked.boundaries.some(finding => finding.includes("imports deployment data: ~deployment/script.js")));
    assert.ok(checked.boundaries.some(finding => finding.includes("imports deployment data: ../../../examples/catalog.json")));
    assert.deepEqual(checked.deploymentData.map(finding => finding.split("link: ")[1]), [
      "#scorecard/s4s-only", "https://demo.seikika.com/", "https://seikika.com/demo/organization/",
    ]);
    const rejected = await run();
    assert.equal(rejected.code, 1, rejected.stderr);
    assert.match(rejected.stderr, /bypasses the package exports/);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
