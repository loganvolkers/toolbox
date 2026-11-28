---
title: Shell script standards
description: Ratchet ShellCheck findings and shfmt formatting and run shell behavior tests with Bats.
---

The `shell` package checks each selected shell source for ShellCheck warnings
and errors and for shfmt formatting with two spaces and indented case arms.
Bats runs as a separate CI step and checks script behavior.

## Configure the package

Choose a Git ref that contains `packages/shell`, and copy
`packages/shell/check-shell.py` from that same ref to `tools/check-shell.py` in
your repository. Use Git pathspecs to select your maintained shell sources.

```yaml
include:
  - package: github:seikika/seikika/packages/shell@<ref>
    with:
      command: [python3, tools/check-shell.py, "scripts/*.sh"]
```

From your checkout root, follow the package install steps and run
`seikika report .seikika --all`. Look for the **Shell script quality** scorecard
and confirm that it lists the scripts you selected. Failed records include
the ShellCheck finding or the shfmt command to apply the standard.
[Pin and install a package](/packages/#pin-and-install)

## Gate behavior in CI

Use the Seikika ratchet for lint and formatting. Run these commands in a
required CI job with the same shfmt and Bats versions shown by
`seikika doctor .seikika`.

```sh
shfmt -i 2 -ci -d tests/shell/*.bats
bats tests/shell
```

Add regression tests for behavior changes using temporary directories and
local substitutes for external dependencies. Assert both exit status and
observable output or side effects. [Package source and standard](https://github.com/seikika/seikika/tree/main/packages/shell)
