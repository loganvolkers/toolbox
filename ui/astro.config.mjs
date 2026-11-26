import { defineConfig } from "astro/config";
import vue from "@astrojs/vue";

export default defineConfig({
  integrations: [vue()],
  outDir: "../crates/seikika-api/ui-dist",
  build: { assets: "assets", inlineStylesheets: "never" },
});
