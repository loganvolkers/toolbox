---
id: D01M8CRS50G5086P5DCFQN636BA
decided: 2026-11-26
---

# Give every standards package a public adoption guide

Generate the public package directory from the first-party `seikika-package.yaml` paths and their published landing pages. Derive the guide route from the package directory name. Each guide explains the package's purpose, prerequisites, configuration, and how to verify the result. The existing Git distribution and lockfile workflow supply version pins.

The workspace connector emits one `standard-package` record for every matching package manifest. The `maintainer-standard-package` scorecard requires a public guide at bronze. This gives every new package a documentation check even when its guide is missing. The site scorecard and documentation style guide cover the pages.

**Status** Implemented.

## Sources

- [Package directory audit](../audits/package-directory.md)
- [Standards package scorecard](../../portal/standards/standard-packages.yaml)
