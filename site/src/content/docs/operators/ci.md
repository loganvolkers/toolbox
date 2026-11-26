---
title: Gate pull requests
description: Commit a baseline for your repository and run the same standards check locally and in CI.
sidebar:
  order: 3
---

Use a committed baseline to catch a new standards failure in a pull request. You need a Git checkout, the [Seikika CLI and workspace connector](/start/install/), and the starter manifest, tests, and fixtures from the [quickstart](/start/quickstart/#1-create-a-practice-repository).

## Record your starting point

From the root of the repository you want to score, copy the starter's `seikika.yaml`, `tests/`, and `fixtures/` into `.seikika/`. If you already have a portal and baseline on your default branch, bring those files into your checkout before running the check.

```bash
seikika doctor .seikika
seikika validate .seikika
seikika report .seikika --baseline .seikika/results.json --update
git add .seikika
```

Review the staged baseline and commit it with the manifest. It records the starting state, including any current findings. If you include packages by Git ref, run `seikika install .seikika` and commit the lock before running CI.

## Add the GitHub Actions workflow

Save this file as `.github/workflows/seikika.yml`. The action builds the CLI and connectors from the same source revision as the starter. It caches that build between runs.

```yaml
name: Seikika standards
on:
  pull_request:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  standards:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6.1.0
        with:
          persist-credentials: false
      - uses: seikika/seikika/actions/report@c14c9fbb60405f9fea12d86de3d0b32b717755d9
        with:
          mode: ci
```

Change `main` if your default branch has another name. Use the job summary to find the scorecard and new evidence when the job fails. The action also uploads a report. For release binaries and other inputs, see the [action reference](https://github.com/seikika/seikika/tree/main/actions/report).

## Run the gate on another CI system

Install the matching binary and connectors on the runner, check out the repository, and run:

```bash
seikika doctor .seikika
seikika report .seikika --baseline .seikika/results.json --ci
```

If the portal uses a committed package lock, populate the runner cache with `seikika install .seikika --frozen` before `doctor`. Treat a nonzero exit as a failed job. `--ci` checks for regressions and a stale or missing baseline. For an intentional improvement, review the results of `--update` and commit the new baseline. See [scorecards and the ratchet](/concepts/scorecards/) for what counts as a regression.
