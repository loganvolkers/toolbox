---
title: Pattern count
description: Record where a regular expression matches files and stop pull requests from adding matches.
---

Use `pattern-count` to find a text pattern, such as a deprecated function name. The package reports each match's line number. Use the [Semgrep package](/packages/semgrep/) when you need to match the structure of code.

## Choose a pattern

You need Seikika 0.2 and the [Workspace connector](/connectors/workspace/). Add this entry to your portal manifest. Adapt the file patterns and the regular expression to your repository. Single quotes preserve backslashes in the YAML pattern. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/pattern-count@5818e032cac3cefec1fe9f6a2a4559bd410174cf
    with:
      files: ["src/**/*.ts"]
      pattern: '\bconsole\.(log|debug)\('
      fix: Use the project logger.
```

The package uses Rust regular expression syntax. You can add `exclude` globs for paths to skip and `exempt` entries with reasons for specific files. [Regular expression syntax](https://docs.rs/regex/latest/regex/#syntax)

## Verify the matches

From the checkout root, run `seikika report .seikika -o failures`. The `pattern-count` scorecard reports failing files with matching line numbers and the fix you set. In a disposable checkout, add one matching line and check that the report lists it. Save a baseline to catch new matches in CI.

If a known match is missing, check the file glob and quote the expression in YAML. Review the pattern on representative code before making it a CI gate. [Parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/pattern-count/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/pattern-count/tests)
