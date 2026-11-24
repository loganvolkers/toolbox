---
title: Install a release
description: Install the Toolbox CLI for a portal, and the extra setup for writing a connector.
sidebar:
  order: 2
---

From **v0.1.0** onward, install the binaries from GitHub Releases (D90). Current release: **v0.1.1**. Each app on that release has a shell installer and a PowerShell installer; the release notes print the same commands. New to Toolbox? Start at [Quickstart](/toolbox/start/quickstart/).

The installers pick the archive for this machine, check its SHA-256, and place the binaries in `$CARGO_HOME/bin` (`~/.cargo/bin` when `CARGO_HOME` is unset). Open a new shell after installing, or export `PATH` in the current one.

Two installs:

| You are | Install |
|---|---|
| An [operator](/toolbox/operators/) running a portal | The `toolbox` CLI below. Add a released connector binary only when the manifest names that `builtin:` |
| A [package author](/toolbox/authors/connectors/) writing a connector | The CLI, then Rust and the scaffold on [Write a connector](/toolbox/authors/connectors/) |

## Run a portal

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.sh | sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
toolbox --version
```

Windows (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.ps1 | iex"
```

`toolbox validate`, `toolbox test`, `toolbox plan` and `toolbox serve --fixtures` use this binary alone. A portal that only includes packages and fixtures does too.

### Connector binaries the portal runs

When the manifest says `package: builtin:<name>`, `toolbox doctor` looks for `toolbox-connector-<name>` next to `toolbox`, on `PATH`, or in `TOOLBOX_CONNECTORS_DIR`. Install that one app from the same release. The [starter](/toolbox/start/quickstart/) names `builtin:workspace`, so a repo portal adds:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-workspace-installer.sh | sh
```

| Manifest package | Installer |
|---|---|
| `builtin:workspace` | `toolbox-connector-workspace` |
| `builtin:runner` | `toolbox-connector-runner` |
| `builtin:github` | `toolbox-connector-github` |
| `builtin:git-files` | `toolbox-connector-git-files` |
| `builtin:gitlab` | `toolbox-connector-gitlab` |
| `builtin:scim` | `toolbox-connector-scim` |

The URL is the same shape: `…/releases/download/v0.1.1/<installer>-installer.sh`. On Windows, use `<installer>-installer.ps1` with `irm … | iex`.

## Write a connector

Writing a connector is a separate install: the CLI from this page, Rust 1.97 from the scaffold's `rust-toolchain.toml`, and `cargo build`. The steps are on [Write a connector](/toolbox/authors/connectors/). The first-party installers in the table above are for a portal that runs those connectors.

## In GitHub Actions

```yaml
- uses: loganvolkers/toolbox/actions/report@v0.1.1   # or a commit SHA
  with:
    toolbox-version: "0.1.1"
    mode: ci
```

Pin the action ref and the binary version. With `toolbox-version` set, the action downloads the checksum-verified archives for `toolbox`, `toolbox-connector-workspace` and `toolbox-connector-runner`. When `.toolbox/toolbox.lock` is present (git packages or executable connectors), the action runs `toolbox install --frozen` before `report` so the runner cache matches the lock.

## Adopt a portal in another repository

1. Copy [`examples/repo-portal-starter/toolbox.yaml`](https://github.com/loganvolkers/toolbox/blob/main/examples/repo-portal-starter/toolbox.yaml) to `.toolbox/toolbox.yaml`. The quickstart fetches that file from the `v0.1.1` tag.
2. Add the workflow above (see [actions/report README](https://github.com/loganvolkers/toolbox/blob/main/actions/report/README.md)).
3. First run with `mode: update`, commit `.toolbox/results.json`.

## Public repository required for external CI

If `loganvolkers/toolbox` is **private**, another repository's workflow cannot resolve `uses: loganvolkers/toolbox/...` (`repository not found`), and its default `GITHUB_TOKEN` cannot download release assets. Make the repo public (Settings → Change repository visibility → Public), or give each consumer a fine-scoped read token / collaborator access (see [docs/RELEASING.md](https://github.com/loganvolkers/toolbox/blob/main/docs/RELEASING.md)).

## Archives

The release also has a `.tar.xz` (Windows: `.zip`) per app and platform, and a sibling `.sha256`. Unpack the `toolbox*` files onto `PATH` when you want the bytes without the installer. `gh attestation verify <file> --repo loganvolkers/toolbox` checks the build provenance.

## Later channels

Homebrew, npm and aqua are still deferred (D90). The crates carry cargo-binstall metadata and are not published to crates.io, so `cargo binstall` does not resolve them yet.

Building Toolbox itself is documented under [Run Toolbox locally](/toolbox/start/run/).
