---
title: Change a portal
description: Validate the manifest, test it against fixtures, and plan the diff before you deploy.
sidebar:
  order: 2
---

An operator changes a portal in the repository that holds its manifest. The wider job, scoring a repository and serving the catalog, is [Build a portal](/toolbox/operators/). The loop is the same shape as any other change: check it, test it, look at the diff, then deploy.

```bash
target/debug/toolbox validate path/to/portal
target/debug/toolbox test path/to/portal
target/debug/toolbox plan --local --base path/to/portal
```

`validate` checks the manifest. `test` runs the portal's expectations against its fixtures, so a rule means the same thing tomorrow as it does today. `plan --local` recomputes the catalog twice and prints the difference.

## What belongs in the manifest

Kinds, fields, the map from connector records, and the scorecard rules. Thresholds live here. A connector that starts failing entities because a number changed in code is the wrong shape: raise the number in the manifest, and the plan shows who drops a level.

## What this site is not

This site documents Toolbox. It is not a running portal, and it is not the one-file HTML report a repository can publish from `toolbox report`. A portal's own UI is served by `toolbox serve`.
