---
title: Change a portal
description: Validate the manifest, test it against fixtures, and plan the diff before you deploy.
sidebar:
  order: 2
---

An operator changes a portal in the repository that holds its manifest. The wider job, scoring a repository and serving the catalog, is [Build a portal](/operators/). The loop is the same shape as any other change: check it, test it, look at the diff, then deploy.

```bash
target/debug/seikika validate path/to/portal
target/debug/seikika test path/to/portal
target/debug/seikika plan --local --base path/to/portal
```

`validate` checks the manifest. `test` runs the portal's expectations against its fixtures, so a rule means the same thing tomorrow as it does today. `plan --local` recomputes the catalog twice and prints the difference.

## What belongs in the manifest

Kinds, fields, the map from connector records, and the scorecard rules. Thresholds live here. A connector that starts failing entities because a number changed in code is the wrong shape: raise the number in the manifest, and the plan shows who drops a level.

## What this site is not

This site documents Seikika. It is not a running portal, and it is not the one-file HTML report a repository can publish from `seikika report`. A portal's own UI is served by `seikika serve`.
