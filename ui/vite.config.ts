import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Fixed output names: the server embeds dist/ (crates/toolbox-api/src/ui.rs).
export default defineConfig({
  plugins: [react()],
  base: "/",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: { output: { entryFileNames: "assets/app.js", assetFileNames: "assets/app[extname]", chunkFileNames: "assets/[name].js" } },
  },
});
