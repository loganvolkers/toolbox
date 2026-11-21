---
title: Install a release
description: Install Toolbox v0.1.0 binaries for a repo portal without compiling from source.
sidebar:
  order: 1
---

From **v0.1.0** onward, consumers install attested binaries from GitHub Releases instead of building the Rust workspace (D90).

## What you need for a repo portal

`toolbox report` needs:

| Binary | Role |
|---|---|
| `toolbox` | CLI (`report`, `doctor`, `validate`, …) |
| `toolbox-connector-workspace` | Reads the checkout (README, CODEOWNERS, docs, …) |
| `toolbox-connector-runner` | Optional; only if the portal's runner connector is enabled |

Each is a separate cargo-dist app on the release. One script installs the set used by [actions/report](https://github.com/loganvolkers/toolbox/tree/main/actions/report):

```bash
git clone https://github.com/loganvolkers/toolbox.git
cd toolbox
TOOLBOX_VERSION=0.1.0 ./scripts/install-release.sh
toolbox --version
toolbox doctor examples/repo-portal-starter
toolbox report examples/repo-portal-starter
```

Or install each app's shell installer from the [v0.1.0 release](https://github.com/loganvolkers/toolbox/releases/tag/v0.1.0) into the same `bin` directory (connectors must sit next to `toolbox`, or on `PATH` / `TOOLBOX_CONNECTORS_DIR`).

## In GitHub Actions

```yaml
- uses: loganvolkers/toolbox/actions/report@v0.1.0   # or a commit SHA
  with:
    toolbox-version: "0.1.0"
    mode: ci
```

Pin the action ref and the binary version. With `toolbox-version` set, the action downloads checksum-verified archives instead of compiling.

## Adopt a portal in another repository

1. Copy [`examples/repo-portal-starter/toolbox.yaml`](https://github.com/loganvolkers/toolbox/blob/main/examples/repo-portal-starter/toolbox.yaml) to `.toolbox/toolbox.yaml`.
2. Add the workflow above (see [actions/report README](https://github.com/loganvolkers/toolbox/blob/main/actions/report/README.md)).
3. First run with `mode: update`, commit `.toolbox/results.json`.

## Public repository required for external CI

If `loganvolkers/toolbox` is **private**, another repository's workflow cannot resolve `uses: loganvolkers/toolbox/...` (`repository not found`), and its default `GITHUB_TOKEN` cannot download release assets. Make the repo public (Settings → Change repository visibility → Public), or give each consumer a fine-scoped read token / collaborator access (see [docs/RELEASING.md](https://github.com/loganvolkers/toolbox/blob/main/docs/RELEASING.md)).

Building from source remains documented under [Run Toolbox locally](/toolbox/start/run/).
