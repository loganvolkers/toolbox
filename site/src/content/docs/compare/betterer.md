---
title: Seikika ratchets vs Betterer
description: When to use Seikika report ratchets or Betterer for preventing quality regressions.
sidebar:
  order: 6
relationship: replace
peer: Betterer
peer_url: https://github.com/phenomnomnominal/betterer
last_verified: 2026-11-26
claim_sources:
  - https://github.com/phenomnomnominal/betterer
  - https://github.com/seikika/seikika/blob/main/docs/design/ratchets.md
---

Betterer checks a repository against saved results and catches regressions. Seikika applies the same idea to scores and evidence from different sources.

| Area | Betterer | Seikika |
|---|---|---|
| Checks | Tests in a JavaScript or TypeScript project | Scorecards over connector records and tool outputs |
| Baseline | `.betterer.results` | A JSON results file, such as `.seikika/results.json` |
| CI | Checks for regressions and changes to expected results | `report --baseline … --ci` checks for regressions and baseline staleness |

If your project already uses Betterer, keep its tests in CI. Seikika's workspace connector can read `.betterer.results` for reporting. Use [Gate pull requests](/operators/ci/) when you want one baseline for several tools or a standard shared across projects.

Seikika learned the committed-baseline workflow from Betterer. See [Running Betterer](https://phenomnomnominal.github.io/betterer/docs/running-betterer/) and [Seikika's ratchet design](https://github.com/seikika/seikika/blob/main/docs/design/ratchets.md).
