---
title: Semgrep standards
description: Turn Semgrep findings and inline suppressions into file-level scorecard results.
---

Use `semgrep` when you already have a Semgrep rule or ruleset. The package runs Semgrep and counts findings per file, including those suppressed with `nosemgrep`. A baseline can catch an increase in findings or suppressions.

## Select a ruleset

You need Seikika 0.2, the [Runner connector](/connectors/runner/), a Semgrep binary, and a rule file or directory in your checkout. Replace the example path with the location of your rules and narrow `target` if needed. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/semgrep@5818e032cac3cefec1fe9f6a2a4559bd410174cf
    with:
      config: .semgrep/rules
      target: src
```

The package accepts one config path and one target. Use the [SARIF package](/packages/sarif/) to run a Semgrep command with several separate config flags or targets. Set `countSuppressed: true` if you want the SARIF scorecard to count inline suppressions.

## Verify the findings

Run `seikika report .seikika -o failures` from the checkout root. Look for findings on the `semgrep` scorecard. A file without findings can disappear from the report when Semgrep emits no results for it.

If an expected finding is missing, run Semgrep with the configured rules and target. Check the rule's own path filters. Moving a rule file can change the finding names held by your baseline, so review that diff when moving rules. [Parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/semgrep/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/semgrep/tests)
