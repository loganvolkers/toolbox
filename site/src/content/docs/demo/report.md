---
title: This repository, scored
description: The Seikika catalog and scorecard report, published as an Astro integration.
---

[Browse the report](/report/) to see every derived entity, scorecard results, evidence, and fixes. [Browse Storybook](/storybook/) for the report and hosted portal stories.

The docs build imports `@seikika/astro-report`. To publish your own report, generate a snapshot and pass its path to the integration in your Astro configuration.

```bash
seikika report portal -o web --out generated
```

```js
import { defineConfig } from 'astro/config';
import seikikaReport from '@seikika/astro-report';

export default defineConfig({
  integrations: [seikikaReport({ data: new URL('./generated/report.json', import.meta.url) })],
});
```

The integration publishes the report at `/report/` by default and accepts an Astro layout for your navigation. Publish a snapshot approved for the site's readers. The same command writes a standalone `index.html` for offline use.
