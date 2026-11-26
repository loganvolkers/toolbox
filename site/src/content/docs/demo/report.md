---
title: This repository, scored
description: The Seikika catalog and scorecard report, published as an Astro integration.
---

[Browse the report](/report/) to see every derived entity, scorecard results, evidence, and fixes. [Browse Storybook](/storybook/) for the report and hosted portal stories.

To publish a report on your own Astro site, install [the report integration](/authors/npm/) and generate a snapshot. From your Astro project, run the following command. Replace `portal` with your portal directory if needed.

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

Build the Astro site and open `/report/` to check that it shows your catalog. Review the snapshot for sensitive data before publishing it. The report command also writes a standalone `index.html` in `generated/` that you can open offline.
