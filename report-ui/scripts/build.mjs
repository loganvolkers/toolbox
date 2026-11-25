// Bundles src/ into the one file `toolbox report` embeds (crates/toolbox-cli/src/toolbox-report.js).
// `--check` fails when the committed file differs from a fresh bundle. Type checking is `tsc`'s job;
// this script only bundles, so the npm scripts run `tsc` first.
import { build } from "vite";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const here = (p) => fileURLToPath(new URL(p, import.meta.url));
const entry = here("../src/index.ts");
const out = here("../../crates/toolbox-cli/src/toolbox-report.js");
const header = readFileSync(entry, "utf8")
  .split("\n")
  .filter((l) => l.startsWith("//"))
  .join("\n");

// Inline config, not vite.config.ts: Storybook's Vite builder reads that file, and the bundle and
// the stories need different settings.
const [result] = [].concat(
  await build({
    configFile: false,
    publicDir: false,
    logLevel: "warn",
    build: {
      write: false,
      target: "es2020",
      minify: false,
      lib: { entry, formats: ["iife"], name: "ToolboxReportBundle", fileName: () => "toolbox-report.js" },
      rollupOptions: {
        output: { banner: header + "\n// Generated from report-ui/src by `npm run build`; edit the sources, not this file." },
      },
    },
  }),
);
const js = result.output.find((o) => o.type === "chunk").code;

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
