---
title: This repository, scored
description: The Toolbox-for-Toolbox report, produced by toolbox report and published with this site.
---

This is the report Toolbox writes about itself. The page is the HTML from `toolbox report`, for this commit, not a picture of it. It is the maintainer SDLC applied to this repository: the same scorecards CI ratchets, with the evidence and the fix on each failing rule.

[Open the report](/toolbox/report/)

CI checks the published file against a fresh run, byte for byte:

```bash
target/debug/toolbox report portal --skip runner,github-actions -o html --out site/public/report/index.html
```

The test runner and the GitHub API are skipped, the same way the fast ratchet skips them. A rule that needs those runs says that it has no run data.

<iframe src="/toolbox/report/" title="Toolbox report for this repository" style="width: 100%; height: 80vh; border: 1px solid var(--sl-color-gray-5); border-radius: 8px;"></iframe>
