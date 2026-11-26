import type { AstroIntegration } from "astro";

interface SeikikaReportOptions {
  /** File produced by seikika report -o web. Publish only data your readers may see. */
  data: string | URL;
  /** Defaults to /report. */
  path?: string;
  aliases?: string[];
  /** An Astro document layout with a default slot. */
  layout?: string | URL;
}

export default function seikikaReport(options: SeikikaReportOptions): AstroIntegration;
