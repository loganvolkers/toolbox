import { defineConfig } from "astro/config";
import toolboxReport from "./integration.mjs";

export default defineConfig({
  integrations: [toolboxReport({ data: new URL("./fixtures/starter-empty.json", import.meta.url), path: "/" })],
  build: { inlineStylesheets: "never" },
});
