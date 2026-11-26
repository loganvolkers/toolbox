# @seikika/ui

The box's shared presentation components for Astro and Vue. Both Seikika portal builds use these components. Import the theme and individual Vue components into an Astro application with the Astro Vue integration.

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

Use the named slots on `PageFrame`, `PageHeader`, `EntityPage`, and `ScorecardPage` to customize their content. Set design tokens on `.seikika-ui`. Import `SectionCard`, `LevelBadge`, and `DataValue` for consistent presentation within those slots.

Run `pnpm check` and `pnpm storybook` from the repository root.
