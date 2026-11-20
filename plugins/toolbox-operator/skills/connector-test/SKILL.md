---
name: connector-test
description: >-
  Runs the connector conformance kit against a connector binary over stdio. Use when a connector crate changed, and before calling that connector done.
---

# toolbox connector test

1. Pass --name as the connector's name in the manifest and --portal as the portal directory.
2. Put the connector program after --.
3. Read which checks failed. Do not skip a failing check by changing the kit.

```
toolbox connector test --name workspace --portal portal -- target/debug/toolbox-connector-workspace
```

Do not omit the -- before the program. If it fails, the binary is missing or a conformance check failed.
