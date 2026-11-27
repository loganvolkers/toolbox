---
name: init
description: >-
  Creates a new Seikika portal manifest with first-party connectors and standards packages.
  Use when setting up a repository portal or starting a shared catalog.
---

# seikika init

1. Run `seikika init` from the checkout root. Pass a directory to change the destination from `.seikika`.
2. Choose connectors and standards packages by number or name. Follow the prompts for each selection. `--yes` accepts the Workspace default with the Markdown link check.
3. Provide provider credentials through the environment named in the connector guide. Review `seikika.yaml` and follow the printed guide for each selected dependency.
4. Run `seikika install` on the portal if you selected packages. Commit `seikika.lock` with the manifest. Run `seikika validate` and `seikika doctor` against the new directory.

```bash
seikika init
seikika init portal --yes
```

The command exits with status 2 when input ends before setup finishes or a manifest already exists. Edit an existing portal in place using its normal operator workflow.

Treat a shared portal's repository as privileged infrastructure. Anyone with read access can see its configuration and plan output.
