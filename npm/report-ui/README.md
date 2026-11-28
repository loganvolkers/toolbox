# @seikika/astro-report

Publish a Seikika catalog and scorecard snapshot in an Astro site. In your Astro project, install `@seikika/astro-report` and its Astro and Vue peers. Generate a snapshot with `seikika report <portal> -o web --out generated`, then add the integration to your Astro configuration.

```js
// astro.config.mjs
import { defineConfig } from 'astro/config';
import seikikaReport from '@seikika/astro-report';

export default defineConfig({
  integrations: [
    seikikaReport({
      data: new URL('./generated/report.json', import.meta.url),
      path: '/report',
    }),
  ],
});
```

Build or start your site and open `/report`. You should see the catalog and scorecards from the snapshot. Every reader can inspect the data embedded in the page. Publish only a snapshot approved for the site's audience.

## API reference

### `@seikika/astro-report`

Call `seikikaReport(options)` and add the returned integration to `astro.config.mjs`. It installs the Astro Vue renderer if the project has none.

| Option | Behavior |
|---|---|
| `data` | Required path or file URL for `report.json`. Use a file URL to resolve from your Astro configuration. A relative string resolves from the process working directory. |
| `path` | Astro route pattern for the report. Defaults to `/report`. |
| `aliases` | Extra Astro route patterns that serve the same snapshot. Defaults to `[]`. Use aliases when moving a report and keeping existing URLs. |
| `layout` | Path or file URL for an Astro layout with a `title` prop and default slot. Defaults to the bundled document layout. |

Missing `data` or an invalid file URL fails when configuring the integration. If Astro cannot read or parse the file while rendering the report, check that `data` points to the `report.json` from `seikika report -o web`.

### `@seikika/astro-report/page.astro`

The report page renders the snapshot and mounts the Vue report in the browser. The integration registers this page at `path` and `aliases`. To render it from your own Astro route, configure the integration first. The page uses the layout and snapshot provided by the integration and takes no props or slots.

### `@seikika/astro-report/default-layout.astro`

Use this export to wrap content in the report's default Astro document layout. The optional `title` prop sets the browser tab title and defaults to `Seikika report`.

| Slot | Content |
|---|---|
| `default` | Page content inside the themed document body. |

### `@seikika/astro-report/app`

Mount the Vue report app on the client when a host supplies the snapshot as its `data` prop.
Use `initialRoute` to initialize a route without a hash. Hosts with live APIs can pass
`requestAction` and `filterCatalog` callbacks. Set `embedded` when the host provides the header.

| Slot | Content |
|---|---|
| `tools` | Host controls with the current report route. |
| `pending` | Loading state when the `pending` prop is true. |
| `extra-view` | Host content for an `extraViews` route, with its route and report data. |
| `entity-tools` | Entity controls with its ID, source, and snapshot. |
| `overview` | Extra overview content with the report data and summary. |

### `@seikika/astro-report/types`

Use type-only imports for `ReportData`, the snapshot accepted by the app, and
`CatalogFilter`, the callback for filtering against a live snapshot.

## Develop the package

From the repository root:

- `pnpm build` refreshes the Rust-embedded Astro portal and offline report.
- `pnpm check` checks the shared components, hosted pages, report, and package catalog.
- `pnpm storybook` opens the shared component and page stories.
- `pnpm storybook:build` builds the static Storybook.
- `pnpm storybook:probe` checks all stories for runtime errors, mobile overflow, and serious accessibility violations across light and dark themes.
- `pnpm storybook:probe -- --contracts-only` checks the campaign hatch and unmeasured comparison in the same views.
- `pnpm docs:build` includes the report and Storybook in the docs artifact.

Fixtures in `fixtures/` come from Seikika for Seikika, the OpenFGA portal, and the repository starter. Run `pnpm --filter @seikika/astro-report fixtures` to regenerate them with the pinned Seikika tooling.
