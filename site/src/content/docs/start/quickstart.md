---
title: Quickstart
description: Install a release, score a repository with the starter portal, and ratchet the baseline in CI.
sidebar:
  order: 1
---

Score a repository with the CLI, a checkout, and a committed baseline. This path uses a local connector. Prefer this path before choosing an [operator](/operators/), [user](/users/), or [maintainer](/maintainers/) loop.

## 1. Install the binaries

The starter uses the `seikika` CLI and the `seikika-connector-workspace` binary. When **v0.2.0** is listed on [GitHub Releases](https://github.com/seikika/seikika/releases), install [mise](https://mise.jdx.dev/installing-mise.html) 2026.11.14 or later and run these commands from the repository you want to score. Before the release, follow the [source setup](/start/run/#try-a-repo-portal-from-source), then continue at [Score once](#3-score-once).

If the repository already has a mise config, merge the tool aliases and pins from the [starter config](https://github.com/seikika/seikika/blob/v0.2.0/examples/repo-portal-starter/mise.example.toml). To start a new config, copy the file.

```bash
curl -fsSL https://raw.githubusercontent.com/seikika/seikika/v0.2.0/examples/repo-portal-starter/mise.example.toml -o mise.toml
```

Install and check the CLI and connector.

```sh
mise trust
mise install
mise exec -- seikika --version
```

For PowerShell, use `curl.exe` for the downloads. For other `builtin:` connectors, see [Install a release](/start/install/).

## 2. Copy the starter portal

In the repository you want to score:

```bash
mkdir -p .seikika
curl -fsSL https://raw.githubusercontent.com/seikika/seikika/v0.2.0/examples/repo-portal-starter/seikika.yaml -o .seikika/seikika.yaml
```

In PowerShell, create the directory with `New-Item -ItemType Directory -Force .seikika`.

Or copy `examples/repo-portal-starter/seikika.yaml` from a local checkout. The starter’s `repo-hygiene` and `docs` scorecards ask for a README, CODEOWNERS, LICENSE, and markdown links that resolve.

## 3. Score once

```bash
mise exec -- seikika doctor .seikika
mise exec -- seikika report .seikika
```

`report` prints each scorecard, failing rules, evidence, and a fix. `doctor` checks that connectors resolve and declared tools are present.

## 4. Commit a baseline

```bash
mise exec -- seikika report .seikika --baseline .seikika/results.json --update
git add .seikika
git commit -m "Add Seikika repo portal and baseline"
```

If you used mise to install the release, commit your `mise.toml` alongside the baseline so contributors use the same versions.

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
