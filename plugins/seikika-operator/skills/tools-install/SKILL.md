---
name: tools-install
description: >-
  Installs a portal's pinned tools (zizmor, actionlint, ruff) by running an installer: mise, uv, pipx or pip. Use when seikika doctor reports a missing or wrong tool, or to set up CI or a new machine from the portal's own pins.
---

# seikika tools install

1. Run `seikika tools install <portal> --dry-run` and read which installer each tool would use. `auto` takes the first of mise, uv, pipx and pip that is installed and that the tool declares an id for (`tools.<name>.install`).
2. Pass `--via <installer>` to choose one. Seikika runs that installer with the pinned version; it downloads nothing itself.
3. Run `seikika doctor <portal>` afterwards. It checks every tool against its pin.

```
seikika tools install portal --dry-run
seikika tools install portal --via uv
```

A tool marked SKIP declares no id for any available installer: add one to its `install` map, or install it another way at the pinned version. aqua installs from its own aqua.yaml, which `seikika tools config --for aqua` prints.
