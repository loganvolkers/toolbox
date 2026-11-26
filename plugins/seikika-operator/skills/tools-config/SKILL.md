---
name: tools-config
description: >-
  Prints an installer's input from a portal's pinned tools - a mise [tools] table, an aqua.yaml package list, or PyPI requirements. Use when a team installs tools with mise or aqua and wants their config to match the portal's pins.
---

# seikika tools config

1. Pass the portal and `--for mise`, `--for aqua` or `--for pypi`.
2. Put the output in the installer's file: mise.toml's `[tools]`, aqua.yaml's `packages`, or a requirements file.
3. For mise, run `mise lock` so the lockfile records checksums. For aqua, pin the standard registry to a release in place of the placeholder.

```
seikika tools config portal --for mise
```

A note on stderr names each tool without an id for that installer; declare one in `tools.<name>.install`. The pins stay in the portal: regenerate the file rather than editing versions in two places.
