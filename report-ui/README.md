# @toolbox-dev/astro-report

An Astro integration for publishing the box's catalog and scorecard snapshots. It takes the `report.json` produced by `toolbox report -o web --out <directory>`, renders the report in Vue, and shares the portal's presentation components. GitHub Pages and the Toolbox CLI build the same Astro integration.

```js
// astro.config.mjs
import { defineConfig } from 'astro/config';
import toolboxReport from '@toolbox-dev/astro-report';

export default defineConfig({
  integrations: [
    toolboxReport({
      data: new URL('./generated/report.json', import.meta.url),
      path: '/report',
    }),
  ],
});
```

Install the integration in an Astro project with Vue. The integration adds the Astro Vue renderer when the project has not already configured it. Use the optional `layout` URL to supply your own Astro document template with a `title` prop and a default slot. The package exports `@toolbox-dev/astro-report/default-layout.astro` for reuse. `aliases` can preserve existing report URLs, including links to an entity, scorecard, or rule.

Every reader can inspect a published snapshot. Use report data approved for the site's audience. A live Toolbox portal shapes its reads using the viewer's authorization.

From the repository root:

- `pnpm build` refreshes the Rust-embedded Astro portal and offline report.
- `pnpm check` checks the shared components, hosted pages, report, and package catalog.
- `pnpm storybook` opens the shared component and page stories.
- `pnpm storybook:build` builds the static Storybook.
- `pnpm storybook:probe` checks all stories for runtime errors, mobile overflow, and serious accessibility violations across light and dark themes.
- `pnpm docs:build` includes the report and Storybook in the docs artifact.

Fixtures in `fixtures/` come from Toolbox for Toolbox, the OpenFGA portal, and the repository starter. Run `pnpm --filter @toolbox-dev/astro-report fixtures` to regenerate them with the pinned Toolbox tooling.
