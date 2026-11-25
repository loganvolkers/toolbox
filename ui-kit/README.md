# @toolbox-dev/ui

The box's shared presentation components for Astro and Vue. Both Toolbox portal builds use these components. Import the theme and individual Vue components into an Astro application with the Astro Vue integration.

```astro
---
import EntityPage from '@toolbox-dev/ui/components/EntityPage.vue';
import { entityOverview } from '@toolbox-dev/ui/entity';
import '@toolbox-dev/ui/theme.css';

const entity = entityOverview('service:checkout', 'Service', {
  title: 'Checkout',
  owner: 'group:payments',
  grafanaUrl: 'https://grafana.example.com/d/checkout',
});
---
<div class="toolbox-ui">
  <EntityPage {entity}>
    <p>Operational details go here.</p>
  </EntityPage>
</div>
```

Use the named slots on `PageFrame`, `PageHeader`, `EntityPage`, and `ScorecardPage` to customize their content. Set design tokens on `.toolbox-ui`. Import `SectionCard`, `LevelBadge`, and `DataValue` for consistent presentation within those slots.

Run `pnpm check` and `pnpm storybook` from the repository root.
