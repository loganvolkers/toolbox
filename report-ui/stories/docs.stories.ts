import { h } from "vue";
import PageFrame from "@seikika/ui/components/PageFrame.vue";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
import CliReference from "../../site/src/components/CliReference.vue";

import NpmPackageCatalog from "../../site/src/components/NpmPackageCatalog.vue";
import packages from "../../site/src/data/npm-packages.json";

export default { title: "Documentation" };

export const CliCommands = {
  render: () => ({
    setup: () => () => h(PageFrame, {}, { default: () => [
      h(PageHeader, { title: "CLI reference" }),
      h(CliReference, { catalog: {
        commands: [{
          name: "get", slug: "get", about: "Read a catalog entity.", mcpTool: "get_entity", structuredOutput: true,
          args: [{ name: "entity", long: null, positional: true, described: true }],
          usage: "seikika get <ENTITY>", helpText: "Usage: seikika get <ENTITY>",
        }, { name: "validate", slug: "validate", about: "Validate a portal.", mcpTool: null,
          args: [], structuredOutput: false }],
        mcpTools: [{ name: "get_entity", description: "Read a catalog entity.", cliCommands: ["get"] }],
      } }),
    ] }),
  }),
};


export const PublicNpmPackages = {
  render: () => ({
    setup: () => () => h(PageFrame, {}, { default: () => [
      h(PageHeader, { title: "Public npm packages" }),
      h(NpmPackageCatalog, { packages }),
    ] }),
  }),
};
