---
title: Meta scoring
description: Check whether a portal’s rules, fixtures, kinds, connectors and reviews support its standards.
---

Run the same checks on a portal’s own definitions that it runs on the work it catalogs. The package
generates catalog entities from the manifest, reads the portal’s fixture assertions and tracks the
review evidence behind its scorecards.

## Add the package

Use a current Seikika source build with the Workspace and Runner connectors. From a repository that
has a `.seikika/seikika.yaml` manifest, copy the package into your portal.

```sh
seikika vendor /path/to/seikika/packages/meta-scoring --portal .seikika
```

```yaml
include:
  - package: packages/meta-scoring
    with:
      owner: group:platform
```

## Verify a standard

Run `seikika validate .seikika`, `seikika test .seikika` and `seikika report .seikika -o failures`
from the repository root. The `meta-rule` card checks fixture coverage. The strongest evidence
includes a passing fixture and a failing fixture, with assertions that require each outcome.

Use `seikika assess brief <id> --portal .seikika` to review whether a rule measures the intended
behavior. Record the verdict with its sources, then run `seikika assess check .seikika`. Reviews
remain advisory, and a failing verdict stays visible when the definition changes. [Manage assessments](/operators/assessments/)
