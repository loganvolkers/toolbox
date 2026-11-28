---
title: Seikika and the linter/formatter ecosystem
description: Use results from linters and formatters in Seikika scorecards.
sidebar:
  order: 9
relationship: complement
peer: Linters and formatters
last_verified: 2026-11-26
claim_sources:
  - https://github.com/seikika/seikika/blob/main/docs/design/seikika-and-tools.md
---

Linters and formatters such as ESLint, Ruff, and rustfmt keep their own commands and settings. Use Seikika when you want to report results from several tools in one place or check them against a shared baseline.

1. Configure the rules in each tool's own configuration file.
2. Pin the tool version in your Seikika portal. Run `seikika doctor` to check that it is installed.
3. Choose a package that understands the tool's output. For example, the [SARIF package](https://github.com/seikika/seikika/tree/main/packages/sarif) reads Static Analysis Results Interchange Format, a common format for linter findings.
4. Check the [scorecard results](/concepts/scorecards/) for evidence. Run the tool on its own when you need its original output or a code fix.

Use [Gate pull requests](/operators/ci/) to apply a baseline after the tool runs reliably. See [the tool integration design](https://github.com/seikika/seikika/blob/main/docs/design/seikika-and-tools.md) for when to add a mapping or connector.
