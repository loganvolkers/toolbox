// Bundles src/ into the one file `toolbox report` embeds (crates/toolbox-cli/src/toolbox-report.js).
// `--check` fails when the committed file differs from a fresh bundle.
import { build } from "esbuild";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const out = here("../../crates/toolbox-cli/src/toolbox-report.js");
const header = readFileSync(here("../src/index.js"), "utf8")
  .split("\n")
  .filter((l) => l.startsWith("//"))
  .join("\n");

const result = await build({
  entryPoints: [here("../src/index.js")],
  bundle: true,
  format: "iife",
  target: "es2020",
  charset: "utf8",
  legalComments: "none",
  write: false,
  banner: { js: header + "\n// Generated from report-ui/src by `npm run build`; edit the sources, not this file." },
});
const js = result.outputFiles[0].text;

if (process.argv.includes("--check")) {
  if (readFileSync(out, "utf8") !== js) {
    console.error(out + " is stale: run `npm run build` in report-ui and commit it.");
    process.exit(1);
  }
  console.log("toolbox-report.js matches report-ui/src");
} else {
  writeFileSync(out, js);
  console.log("wrote " + out + " (" + js.split("\n").length + " lines)");
}
