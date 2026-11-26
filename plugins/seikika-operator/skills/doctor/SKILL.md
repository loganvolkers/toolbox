---
name: doctor
description: >-
  Checks that every in-image connector resolves and every declared tool is at its pinned version. Use when a portal should be runnable, and after install.
---

# seikika doctor

1. Run the command on the portal directory.
2. Read each missing connector or tool. Exit 1 means something is missing.
3. Install what it names, then run it again. Pass --mise to print the mise tools section instead of checking.
4. Read the `connector confinement` line. `UNCONFINED` means this host has no Landlock, so serve
   would start in-image connectors without it; that is a warning, not a failure. `REFUSED` means
   `SEIKIKA_CONNECTOR_CONFINEMENT=required` is set as well, so serve would not start them.

```
seikika doctor portal
seikika doctor portal --mise
```

Do not ignore an exit code of 1. If it fails, a connector binary or a declared tool is absent or at the wrong version.
