import type { StorybookConfig } from "@storybook/vue3-vite";
import vue from "@vitejs/plugin-vue";
import { mergeConfig } from "vite";

export default {
  framework: { name: "@storybook/vue3-vite", options: { docgen: false } },
  stories: ["../stories/**/*.stories.ts"],
  core: { disableTelemetry: true },
  viteFinal: config => mergeConfig(config, { plugins: [vue()], resolve: { dedupe: ["vue"] } }),
} satisfies StorybookConfig;
