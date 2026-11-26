---
title: SARIF standards
description: Turn findings from a SARIF reporting tool into a scorecard and track them in CI.
---

Use `sarif` to carry results from a tool that emits Static Analysis Results Interchange Format (SARIF) 2.1.0. The package reports each file's findings and the command that produced them, so you can reproduce a result with the tool alone.

## Configure a tool

You need Seikika 0.2, the [Runner connector](/connectors/runner/), and a tool with SARIF output. This example uses Ruff and scans Python files. The final command argument expands to matching files. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/sarif@5818e032cac3cefec1fe9f6a2a4559bd410174cf
    with:
      name: py-lint
      kind: PythonFile
      tool: ruff
      version: "0.14.0"
      command: [ruff, check, --output-format, sarif, --exit-zero, "{files}"]
      files: ["**/*.py"]
```

Give each tool a unique `name` and `kind` in the portal. Configure the command to write only SARIF to standard output. Keep the scanner's own settings in your repository.

## Verify the findings

Use `seikika report .seikika --all` from the checkout root and look for `py-lint`. Use `-o failures` to inspect failing file records with rule names and line numbers. Run the command recorded for one file to compare the result with Ruff's output.

If the scorecard has no files, check the glob and run the command with a matching path. If the tool reports a version mismatch, run `seikika doctor .seikika` to see the expected version. [Parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/sarif/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/sarif/tests)
