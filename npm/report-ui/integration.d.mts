import type { AstroIntegration } from "astro";

/** Files and routes for publishing one static Seikika report snapshot. */
interface SeikikaReportOptions {
  /** Path or file URL for `report.json` from `seikika report -o web`. Strings resolve from the working directory. */
  data: string | URL;
  /** Astro route pattern where the report is mounted. Defaults to `/report`. */
  path?: string;
  /** Additional Astro route patterns that serve the same snapshot. Defaults to an empty list. */
  aliases?: string[];
  /** Path or file URL for an Astro document layout that accepts `title` and renders the default slot. */
  layout?: string | URL;
}

/**
 * Mounts a Seikika report snapshot in an Astro site.
 *
 * Add the returned integration to `astro.config.mjs`. The integration adds Astro's Vue
 * renderer if the site has none. It reads the snapshot when rendering the report and
 * embeds it in the page, so every reader of that page can inspect the full snapshot.
 *
 * @param options - Snapshot location, report routes, and optional document layout.
 * @throws TypeError if `data` is missing or a URL is not a valid local file URL.
 *
 * @example
 * ```js
 * import { defineConfig } from "astro/config";
 * import seikikaReport from "@seikika/astro-report";
 *
 * export default defineConfig({
 *   integrations: [seikikaReport({ data: new URL("./report.json", import.meta.url) })],
 * });
 * ```
 */
export default function seikikaReport(options: SeikikaReportOptions): AstroIntegration;
