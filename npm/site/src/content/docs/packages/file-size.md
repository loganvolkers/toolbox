---
title: File size
description: Limit the number of lines in selected files and catch new or growing violations in CI.
---

Use `file-size` to set a line limit for files you want to keep small. For example, check production TypeScript files while excluding tests. The package reports each file's line count, the number of lines over the limit, and a fix.

## Configure the limit

You need Seikika 0.2 and the [Workspace connector](/connectors/workspace/). Add this entry to your portal manifest. Change `files` to the paths you want to measure and set `limit` to the largest acceptable line count. Patterns are relative to the checkout root. [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/file-size@5818e032cac3cefec1fe9f6a2a4559bd410174cf
    with:
      files: ["src/**/*.ts"]
      exclude: ["**/*.test.ts"]
      limit: 400
```

Use `exempt` to record a reason for a file that should still be measured but can exceed the limit. To apply two different limits, give each include a distinct `name` and PascalCase `kind`.

## Verify the check

From the checkout root, run `seikika report .seikika --all` after installation. Look for the `file-size` scorecard and check the line count of one matching file. At the default scorecard name, `file-size-zero` tracks the files and lines over the limit. A saved baseline lets CI flag a new violation or a file that grows further over the limit.

If the scorecard has no files, check the working directory and the `files` and `exclude` patterns. [Parameter reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/file-size/README.md) · [Test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/file-size/tests)
