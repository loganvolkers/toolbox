---
title: This repository, scored
description: The Toolbox-for-Toolbox report, produced by toolbox report and hosted in this site's own navigation.
---

This is the report Toolbox writes about itself. It is the maintainer SDLC applied to this repository: the same scorecards CI ratchets, with the evidence and the fix on each failing rule. The scorecards are the custom element `<toolbox-report>`. This site fills its `header`, `nav`, and `footer` slots; the element renders the scorecards in its own shadow tree and does not concatenate the site's markup into the report.

[Open the report in the site's navigation](/toolbox/report-hosted/)

[Open the report on its own](/toolbox/report/)

The standalone page is one file. A host that wants its own chrome uses the directory form, then writes a page around the element:

```html
<script src="/report/toolbox-report.js"></script>
<toolbox-report src="/report/report.json">
  <header slot="header"><a href="/">Docs</a></header>
  <nav slot="nav"><a href="/standards">Standards</a></nav>
  <footer slot="footer"><p>Published from CI.</p></footer>
</toolbox-report>
```

`toolbox report -o web` writes `toolbox-report.js`, `report.json`, and `index.html`. CI checks those three files against a fresh run, byte for byte:

```bash
target/debug/toolbox report portal --skip runner,github-actions -o web --out site/public/report
```

The test runner and the GitHub API are skipped, the same way the fast ratchet skips them. A rule that needs those runs says that it has no run data.

<iframe src="/toolbox/report-hosted/" title="Toolbox report with this site's header and footer" style="width: 100%; height: 80vh; border: 1px solid var(--sl-color-gray-5); border-radius: 8px;"></iframe>
