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

Save this file as `.github/workflows/seikika.yml`. Replace both `ACTION_SHA` placeholders with
the same full commit SHA that includes the actions. You can use the commit for a
[Seikika release](https://github.com/seikika/seikika/releases) once it includes them.
Choose a published CLI version that supports your manifest and use that version when updating
the baseline locally.

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
      - uses: seikika/seikika/actions/setup@ACTION_SHA
        with:
          seikika-version: "0.2.1"
      - uses: seikika/seikika/actions/ratchet@ACTION_SHA
```

Change `main` if your default branch has another name. Use the job summary to find the scorecard
and evidence when the job fails. Make the `standards` job a required check if you want it to
block merges.

Choose the steps that fit your workflow.

| Goal | Steps |
|---|---|
| Validate a manifest or test its rules | Keep setup and run `seikika validate .seikika` or `seikika test .seikika`. Install locked packages first if the manifest uses them. |
| Enforce an existing baseline | Keep the ratchet step. Use its default `ci` mode to require an up-to-date baseline, or `mode: check` to block regressions while allowing unrecorded improvements. |
| Browse the current state without a baseline | Use [build-portal](/operators/publish/#host-the-report-on-github-pages) in the workflow that produces the static site. |
| Maintain an existing one-step workflow | Use [report](/reference/contracts/#github-actions) for its baseline modes, Markdown and HTML uploads, and cached source build. |

Use a job with `contents: read` on pull requests. The manifest and its connectors run with the
job's permissions. Keep deployment credentials in a separate job that runs from a trusted ref.
[GitHub Actions inputs and outputs](/reference/contracts/#github-actions)

## Run the gate on another CI system

Install the matching binary and connectors on the runner, check out the repository, and run:

```bash
seikika doctor .seikika
seikika report .seikika --baseline .seikika/results.json --ci
```

If the portal uses a committed package lock, populate the runner cache with `seikika install .seikika --frozen` before `doctor`. Treat a nonzero exit as a failed job. `--ci` checks for regressions and a stale or missing baseline. For an intentional improvement, review the results of `--update` and commit the new baseline. See [scorecards and the ratchet](/concepts/scorecards/) for what counts as a regression.
