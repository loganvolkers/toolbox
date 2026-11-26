---
title: Quickstart
description: Install a release, score a repository with the starter portal, and ratchet the baseline in CI.
sidebar:
  order: 1
---

Score a repository with the CLI, a checkout, and a committed baseline. This path uses a local connector. Prefer this path before choosing an [operator](/operators/), [user](/users/), or [maintainer](/maintainers/) loop.

## 1. Install the binaries

The starter uses the `seikika` CLI and the `seikika-connector-workspace` binary. Use the installers when **v0.2.0** is listed on [GitHub Releases](https://github.com/seikika/seikika/releases). Before that, follow the [source setup](/start/run/#try-a-repo-portal-from-source) to build both binaries and copy the starter, then continue at [Score once](#3-score-once).

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/seikika/seikika/releases/download/v0.2.0/seikika-cli-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/seikika/seikika/releases/download/v0.2.0/seikika-connector-workspace-installer.sh | sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
seikika --version
```

Windows, and any other `builtin:` your manifest names: [Install a release](/start/install/). Writing a connector: [Write a connector](/authors/connectors/). Building Seikika itself: [Run Seikika locally](/start/run/).

## 2. Copy the starter portal

In the repository you want to score:

```bash
mkdir -p .seikika
curl -fsSL \
  https://raw.githubusercontent.com/seikika/seikika/v0.2.0/examples/repo-portal-starter/seikika.yaml \
  -o .seikika/seikika.yaml
```

Or copy `examples/repo-portal-starter/seikika.yaml` from a local checkout. The starter’s `repo-hygiene` and `docs` scorecards ask for a README, CODEOWNERS, LICENSE, and markdown links that resolve.

## 3. Score once

```bash
seikika doctor .seikika
seikika report .seikika
```

`report` prints each scorecard, failing rules, evidence, and a fix. `doctor` checks that connectors resolve and declared tools are present.

## 4. Commit a baseline

```bash
seikika report .seikika --baseline .seikika/results.json --update
git add .seikika
git commit -m "Add Seikika repo portal and baseline"
```

Later runs with the same `--baseline` fail when a rule newly fails or evidence grows. `--ci` also fails when the file is stale after an improvement. That ratchet is the same idea as Betterer’s results file — see [Scorecards](/concepts/scorecards/) and [Seikika ratchets vs Betterer](/compare/betterer/).

## 5. Gate pull requests

Once the v0.2.0 release is available, use the matching action ref and binaries in CI.

```yaml
- uses: seikika/seikika/actions/report@v0.2.0
  with:
    seikika-version: "0.2.0"
    mode: ci
```

Use `mode: update` once when you intentionally tighten the baseline. Pin the action ref and the binary version together ([Install a release](/start/install/)).

## What you just proved

| Surface | What happened |
|---|---|
| Repo portal | Standards as data in `.seikika/seikika.yaml`, no server |
| Scorecards | Levels with evidence and a fix |
| Ratchet | Committed `.seikika/results.json` that CI compares |

## Next steps

| If you are… | Go to |
|---|---|
| Changing the portal or serving the catalog | [Build a portal](/operators/) |
| Writing a connector other portals run | [Write a connector](/authors/connectors/) |
| Reading owners and scores from scripts or agents | [Use a portal](/users/) · [Agents](/agents/reads/) |
| Publishing a package other portals include | [Publish a standard](/authors/) |
| Changing Seikika itself | [Build Seikika](/maintainers/) |
| Comparing to other IDPs and tools | [Seikika vs others](/compare/) |

Command map: [CLI reference](/start/cli/).
