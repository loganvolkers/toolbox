---
title: This repository, scored
description: The Toolbox catalog and scorecard report, published as an Astro integration.
---

[Browse the report](/toolbox/report/) to see every derived entity, scorecard results, evidence, and fixes. [Browse Storybook](/toolbox/storybook/) for the report and hosted portal stories.

The docs build imports `@toolbox-dev/astro-report`. To publish your own report, generate a snapshot and pass its path to the integration in your Astro configuration.

```bash
toolbox report portal -o web --out generated
```

```js
import { defineConfig } from 'astro/config';
import toolboxReport from '@toolbox-dev/astro-report';

export default defineConfig({
  integrations: [toolboxReport({ data: new URL('./generated/report.json', import.meta.url) })],
});
```

The integration publishes the report at `/report/` by default and accepts an Astro layout for your navigation. Publish a snapshot approved for the site's readers. The same command writes a standalone `index.html` for offline use.
