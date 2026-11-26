---
title: OpenSSF Scorecard
description: Run OpenSSF supply-chain checks on tracked files and catch drops from your baseline.
---

Use `openssf-scorecard` to track Open Source Security Foundation (OpenSSF) Scorecard checks in your repo portal. Each result keeps the upstream score, reason, and check documentation. Seikika records a baseline so a later score drop shows up in review.

## Add the local checks

You need a Git checkout with tracked files, Seikika 0.2, the [Runner connector](/connectors/runner/), and the Scorecard binary version shown by `seikika doctor .seikika`. The default command runs Scorecard's local mode on an export of the tracked files. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/openssf-scorecard@5818e032cac3cefec1fe9f6a2a4559bd410174cf
```

The default connector and scorecard name is `openssf`. Use the `command` parameter if you want to select a subset of the checks. Keep its JSON output format so the connector can parse it.

## Verify the results

From the checkout root, run `seikika report .seikika --all`. Open the `openssf` scorecard and confirm that it has one entity per reported check. Each entity includes a command to reproduce its result. Checks that need live repository data can be inconclusive in local mode.

If the package has no results, check that the binary is available and that the files you expect are tracked. Run the check's recorded command to compare results with the upstream tool. [Parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/openssf-scorecard/README.md) · [OpenSSF checks](https://github.com/ossf/scorecard/blob/main/docs/checks.md)
