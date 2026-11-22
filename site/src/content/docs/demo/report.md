---
title: This repository, scored
description: The Toolbox-for-Toolbox report, produced by toolbox report and hosted in this site's own navigation.
---

This is the report Toolbox writes about itself. It is the maintainer SDLC applied to this repository: fleet health (Overview), every derived entity (Catalog), and every scorecard with pass and fail outcomes, evidence, and fixes. The page is the custom element `<toolbox-report>`. This site fills its `header`, `nav`, and `footer` slots; the element renders those modes in its own shadow tree and does not concatenate the site's markup into the report.

[Open the report in the site's navigation](/toolbox/report-hosted/)

Related: the [Software factory demo](/toolbox/demo/port/) rebuilds Port’s public demo shapes (catalog, entity, governance) with fixtures and honest gaps.

The pages build also publishes a standalone one-file page at `/toolbox/report/` (generated into `site/public/report`, not committed). A host that wants its own chrome uses the directory form, then writes a page around the element:

```html
<script src="/report/toolbox-report.js"></script>
<toolbox-report src="/report/report.json">
  <header slot="header"><a href="/">Docs</a></header>
  <nav slot="nav"><a href="/standards">Standards</a></nav>
  <footer slot="footer"><p>Published from CI.</p></footer>
</toolbox-report>
```

`toolbox report -o web` writes `toolbox-report.js`, `report.json`, and `index.html`. The site's build generates them for each commit, with the pinned zizmor and actionlint, so the page always shows the commit it was built from. Locally, prefer `scripts/regen-site-report.sh` (installs those same pins, runs `doctor`, then the command below):

```bash
target/debug/toolbox report portal --skip runner,github-actions -o web --out site/public/report
```

The test runner and the GitHub API are skipped, the same way the fast ratchet skips them. A rule that needs those runs is unknown, and it names the connector that was skipped.

<iframe src="/toolbox/report-hosted/" title="Toolbox report with this site's header and footer" style="width: 100%; height: 80vh; border: 1px solid var(--sl-color-gray-5); border-radius: 8px;"></iframe>
