import { isAbsolute, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vue from "@astrojs/vue";

const entry = fileURLToPath(new URL("./src/report.astro", import.meta.url));
const defaultLayout = fileURLToPath(new URL("./src/layouts/DefaultLayout.astro", import.meta.url));
const virtual = "virtual:seikika-report";

export default function seikikaReport({ data, path = "/report", aliases = [], layout } = {}) {
  if (!data) throw new TypeError("seikikaReport requires a report JSON file");
  const localPath = (file) => file instanceof URL ? fileURLToPath(file) : isAbsolute(file) ? file : resolve(file);
  const dataFile = localPath(data);
  const layoutFile = layout ? localPath(layout) : defaultLayout;
  const routes = [...new Set([path, ...aliases])];

  return {
    name: "@seikika/astro-report",
    hooks: {
      "astro:config:setup": ({ config, injectRoute, updateConfig }) => {
        for (const pattern of routes) injectRoute({ pattern, entrypoint: entry });
        const integrations = config.integrations.some(integration => integration.name === "@astrojs/vue") ? [] : [vue()];
        updateConfig({
          integrations,
          vite: {
            resolve: { alias: {
              "@astrojs/vue/client.js": fileURLToPath(import.meta.resolve("@astrojs/vue/client.js")),
              "@astrojs/vue/server.js": fileURLToPath(import.meta.resolve("@astrojs/vue/server.js")),
            } },
            plugins: [{
              name: "seikika-report-options",
              resolveId(id) { if (id === virtual) return "\0" + virtual; },
              load(id) {
                if (id !== "\0" + virtual) return;
                this.addWatchFile(dataFile);
                return "export { default as Layout } from " + JSON.stringify(layoutFile) +
                  "; export const dataFile = " + JSON.stringify(dataFile) + ";";
              },
            }],
          },
        });
      },
    },
  };
}
