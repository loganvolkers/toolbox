import { defineConfig } from "astro/config";
import seikikaReport from "./integration.mjs";

export default defineConfig({
  integrations: [seikikaReport({ data: new URL("./fixtures/starter-empty.json", import.meta.url), path: "/" })],
  build: { inlineStylesheets: "never" },
});
