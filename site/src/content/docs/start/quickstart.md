---
title: Quickstart
description: Create a portal, browse connectors and standards packages, and catch a broken link.
sidebar:
  order: 1
---

Run `seikika init` to create a portal and browse the available connectors and standards packages.
For the practice exercise, put a CLI with `init` and the matching Workspace connector on `PATH` in
the same terminal. If your release predates `init`, use the [source build](/start/install/#build-from-source).

## 1. Create a portal

Start in a disposable directory so you can edit the practice files freely.

```bash
mkdir seikika-practice && cd seikika-practice
seikika init
```

Press Enter at each prompt to keep the defaults. The wizard writes `.seikika/seikika.yaml` with the Workspace connector and a Markdown link check. Use `seikika init --yes` to accept the same defaults in a script.

For your own portal, pick connectors and packages by number or name. The wizard asks for settings such as the GitHub organization or the files to check. Follow the printed connector guides to install binaries and set any required credentials. If you selected a standards package, run the `seikika install` command the wizard prints to fetch it and write `seikika.lock`. [Connector directory](/connectors/) · [Standards package directory](/packages/)

## 2. Check the setup and record a baseline

Create a working Markdown link, check the setup, and save the first report.

```bash
printf '# Practice\n\nRead the [portal notes](portal.md).\n' > README.md
printf '# Portal notes\n' > portal.md
seikika validate .seikika
seikika report .seikika --baseline .seikika/results.json --update
```

`validate` checks the manifest. The report writes `.seikika/results.json`, your **baseline** of scores and findings. If the report cannot run Workspace, check that the connector binary is available with `seikika doctor .seikika`. [Install a connector](/start/install/#install-a-connector)

## 3. Break a link and catch the regression

Move the file that `README.md` links to and rerun the check.

```bash
mv portal.md moved-portal.md
seikika report .seikika --baseline .seikika/results.json --ci
```

The command exits with status 1. Look for the `docs` scorecard and the `links-resolve` rule for `README.md`. The report names `portal.md` in the evidence.

## 4. Fix the link and verify the result

```bash
mv moved-portal.md portal.md
seikika report .seikika --baseline .seikika/results.json --ci
```

The check now exits with status 0. You restored the link and kept the original baseline. Seikika calls this comparison a **ratchet** because the baseline can record improvements and catch regressions.

## Next steps

Start your portal locally with `seikika serve .seikika`. To adopt repository checks in CI, follow the pull request guide. [Explore a running catalog](/start/portal/) · [Gate pull requests](/operators/ci/)
