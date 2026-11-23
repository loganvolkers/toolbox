---
title: Quickstart
description: Install a release, score a repository with the starter portal, and ratchet the baseline in CI.
sidebar:
  order: 1
---

Fifteen minutes to a scored repository. No Postgres and no portal server — only the CLI, a checkout, and a committed baseline. Prefer this path before choosing an [operator](/toolbox/operators/), [user](/toolbox/users/), or [maintainer](/toolbox/maintainers/) loop.

## 1. Install the binaries

Current release: **v0.1.1**. You need `toolbox` and `toolbox-connector-workspace` on `PATH` (the runner connector is optional for the starter).

```bash
git clone https://github.com/loganvolkers/toolbox.git
cd toolbox
TOOLBOX_VERSION=0.1.1 ./scripts/install-release.sh
toolbox --version
```

Details and GitHub Actions pins: [Install a release](/toolbox/start/install/). Building from source: [Run Toolbox locally](/toolbox/start/run/).

## 2. Copy the starter portal

In the repository you want to score (not only this clone):

```bash
mkdir -p .toolbox
curl -fsSL \
  https://raw.githubusercontent.com/loganvolkers/toolbox/v0.1.1/examples/repo-portal-starter/toolbox.yaml \
  -o .toolbox/toolbox.yaml
```

Or copy `examples/repo-portal-starter/toolbox.yaml` from a local checkout. The starter’s `repo-hygiene` and `docs` scorecards ask for a README, CODEOWNERS, LICENSE, and markdown links that resolve.

## 3. Score once

```bash
toolbox doctor .toolbox
toolbox report .toolbox
```

`report` prints each scorecard, failing rules, evidence, and a fix. `doctor` checks that connectors resolve and declared tools are present.

## 4. Commit a baseline

```bash
toolbox report .toolbox --baseline .toolbox/results.json --update
git add .toolbox
git commit -m "Add Toolbox repo portal and baseline"
```

Later runs with the same `--baseline` fail when a rule newly fails or evidence grows. `--ci` also fails when the file is stale after an improvement. That ratchet is the same idea as Betterer’s results file — see [Scorecards](/toolbox/concepts/scorecards/) and [Toolbox ratchets vs Betterer](/toolbox/compare/betterer/).

## 5. Gate pull requests

```yaml
- uses: loganvolkers/toolbox/actions/report@v0.1.1
  with:
    toolbox-version: "0.1.1"
    mode: ci
```

Use `mode: update` once when you intentionally tighten the baseline. Pin the action ref and the binary version together ([Install a release](/toolbox/start/install/)).

## What you just proved

| Surface | What happened |
|---|---|
| Repo portal | Standards as data in `.toolbox/toolbox.yaml`, no server |
| Scorecards | Levels with evidence and a fix |
| Ratchet | Committed `.toolbox/results.json` that CI compares |

## Next steps

| If you are… | Go to |
|---|---|
| Extending standards, connectors, or a served catalog | [Build a portal](/toolbox/operators/) |
| Reading owners and scores from scripts or agents | [Use a portal](/toolbox/users/) · [Agents](/toolbox/agents/reads/) |
| Publishing a package other portals include | [Publish a standard](/toolbox/authors/) |
| Changing Toolbox itself | [Build Toolbox](/toolbox/maintainers/) |
| Comparing to other IDPs and tools | [Toolbox vs others](/toolbox/compare/) |

Command map: [CLI reference](/toolbox/start/cli/).
