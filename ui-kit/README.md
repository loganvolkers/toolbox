# @seikika/ui

Build entity and scorecard pages with the Seikika portal's shared Vue components. Use a build tool that handles TypeScript and Vue single-file components. In Astro, configure the Astro Vue integration before importing the components. Import the theme once and wrap the content in `.seikika-ui`, or use `PageFrame` to supply the wrapper.

```astro
---
import EntityPage from '@seikika/ui/components/EntityPage.vue';
import { entityOverview } from '@seikika/ui/entity';
import '@seikika/ui/theme.css';

const entity = entityOverview('service:checkout', 'Service', {
  title: 'Checkout',
  owner: 'group:payments',
  grafanaUrl: 'https://grafana.example.com/d/checkout',
});
---
<div class="seikika-ui">
  <EntityPage {entity}>
    <p>Operational details go here.</p>
  </EntityPage>
</div>
```

The example renders a Checkout page with its owner and Grafana link. Editors with Vue language support show prop descriptions at the point of use. See the component stories for examples of custom slots. [Storybook](https://seikika.com/storybook/)

## API reference

### `@seikika/ui/entity`

Import `entityOverview` to prepare `EntityOverview` data from catalog fields. The helper uses the ID as the title when no title, display name, or name is available. It keeps the first link for each URL and filters links to HTTP and HTTPS URLs without credentials. The module exports `EntityOverview` and `EntityLink` as TypeScript types.

Import `entityHref` to map an entity ID to the hosted portal's `/e/` route. For example, `entityHref('service:checkout')` returns `/e/service%3Acheckout`. Supply your own resolver for a different route or base path.

### `@seikika/ui/components/EntityPage.vue`

Pass an `entity` from `entityOverview`. Optionally pass an `ownerHref` to link the owner, a `snapshot` ID to show its version, and a `heading` for the built-in header. The heading defaults to `h1`.

| Slot | Content |
|---|---|
| `breadcrumbs` | Navigation before the header. |
| `header` | Replacement for the built-in header. |
| `meta` | Replacement for the ID, kind, and snapshot line in the built-in header. |
| `header-actions` | Controls in the built-in header. |
| `default` | Main content after the owner and link list. |
| `scorecards` | Scorecards after the main content. |
| `actions` | Action controls after the scorecards. |

### `@seikika/ui/components/ScorecardPage.vue`

Pass a `title` and optional `description` to show a scorecard with the shared page header. Set `heading` to `h2` when the surrounding page supplies its own `h1`.

| Slot | Content |
|---|---|
| `breadcrumbs` | Navigation before the header. |
| `header` | Replacement for the built-in header. |
| `meta` | Metadata in the built-in header. |
| `header-actions` | Controls in the built-in header. |
| `default` | Scorecard details. |

### `@seikika/ui/components/PageFrame.vue`

Use `PageFrame` for a themed page with a main landmark. Supply `label` if the main landmark needs an accessible name.

| Slot | Content |
|---|---|
| `navigation` | Navigation before the main landmark. |
| `default` | Content inside the main landmark. |
| `footer` | Content after the main landmark. |

### `@seikika/ui/components/PageHeader.vue`

Pass a `title` and optional `description`. The `heading` defaults to `h1` and also accepts `h2`.

| Slot | Content |
|---|---|
| `title` | Replacement for the title text, inside the heading element. |
| `meta` | Context below the title and description. |
| `actions` | Controls beside the heading. |

### `@seikika/ui/components/SectionCard.vue`

Pass a `title` to group related content under a heading. The `heading` defaults to `h2` and also accepts `h3` or `h4`.

| Slot | Content |
|---|---|
| `title` | Replacement for the title text, inside the heading element. |
| `default` | Content below the heading. |

### `@seikika/ui/components/LevelBadge.vue`

Pass a `level`, such as `bronze` or `gold`, to display it using the theme's level colors. `null` and `undefined` display `none`. Object values use `DataValue`, including redaction markers.

### `@seikika/ui/components/DataValue.vue`

Pass a `value` from a catalog field. The component shows nullish values as a dash, array items separated by commas, and objects as JSON. A redaction marker displays its available visibility or reason text. Supply `entityHref` to link entity references. Omit it to display those references as text.

### `@seikika/ui/theme.css`

Import the stylesheet once. Override custom properties on `.seikika-ui` after the default stylesheet. The defaults follow the reader's light or dark color preference.

| Tokens | Controls |
|---|---|
| `--bg`, `--fg`, `--muted`, `--faint` | Page background, main text, secondary text, and subdued details. |
| `--line`, `--card`, `--tint`, `--shadow` | Borders, card surfaces, tinted surfaces, and elevation. |
| `--accent`, `--accent-bg` | Links and focus outlines, plus their tinted background. |
| `--pass`, `--fail`, `--unknown` | Outcome colors. `--ok`, `--bad`, and `--warn` default to the same colors, respectively. |
| `--none`, `--bronze`, `--silver`, `--gold`, `--skipped` | Scorecard level and skipped-state fills. |
| `--none-ink`, `--bronze-ink`, `--silver-ink`, `--gold-ink`, `--skipped-ink` | Text colors for those states. |
| `--judged`, `--judged-bg`, `--lock` | Judgment foreground and background, plus the lock color, which defaults to `--judged`. |

## Develop the package

Run `pnpm check` and `pnpm storybook` from the repository root.
