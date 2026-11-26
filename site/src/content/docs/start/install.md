---
title: Install a release
description: Install the Seikika CLI for a portal, and the extra setup for writing a connector.
sidebar:
  order: 2
---

Install the Seikika binaries from GitHub Releases (D90). These commands use **v0.2.0**, the first release under the Seikika name. Check [GitHub Releases](https://github.com/seikika/seikika/releases) for that tag. Until it is available, [build from source](/start/run/#try-a-repo-portal-from-source). For a first repo portal, follow [Quickstart](/start/quickstart/).

The installers pick the archive for this machine, check its SHA-256, and place the binaries in `$CARGO_HOME/bin` (`~/.cargo/bin` when `CARGO_HOME` is unset). Open a new shell after installing, or export `PATH` in the current one.

Two installs:

| You are | Install |
|---|---|
| An [operator](/operators/) running a portal | The `seikika` CLI below. Add a released connector binary only when the manifest names that `builtin:` |
| A [package author](/authors/connectors/) writing a connector | The CLI, then Rust and the scaffold on [Write a connector](/authors/connectors/) |

## Run a portal

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/seikika/seikika/releases/download/v0.2.0/seikika-cli-installer.sh | sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
seikika --version
```

Windows (PowerShell):

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/seikika/seikika/releases/download/v0.2.0/seikika-cli-installer.ps1 | iex"
```

`seikika validate`, `seikika test`, `seikika plan` and `seikika serve --fixtures` use this binary alone. A portal that only includes packages and fixtures does too.

### Connector binaries the portal runs

When the manifest says `package: builtin:<name>`, `seikika doctor` looks for `seikika-connector-<name>` next to `seikika`, on `PATH`, or in `SEIKIKA_CONNECTORS_DIR`. Install that one app from the same release. The [starter](/start/quickstart/) names `builtin:workspace`, so a repo portal adds:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/seikika/seikika/releases/download/v0.2.0/seikika-connector-workspace-installer.sh | sh
```

| Manifest package | Installer |
|---|---|
| `builtin:workspace` | `seikika-connector-workspace` |
| `builtin:runner` | `seikika-connector-runner` |
| `builtin:github` | `seikika-connector-github` |
| `builtin:git-files` | `seikika-connector-git-files` |
| `builtin:gitlab` | `seikika-connector-gitlab` |
| `builtin:scim` | `seikika-connector-scim` |

The URL is the same shape: `…/releases/download/v0.2.0/<installer>-installer.sh`. On Windows, use `<installer>-installer.ps1` with `irm … | iex`.

## Write a connector

Writing a connector is a separate install: the CLI from this page, Rust 1.97 from the scaffold's `rust-toolchain.toml`, and `cargo build`. The steps are on [Write a connector](/authors/connectors/). The first-party installers in the table above are for a portal that runs those connectors.

## In GitHub Actions

```yaml
- uses: seikika/seikika/actions/report@v0.2.0   # or a commit SHA
  with:
    seikika-version: "0.2.0"
    mode: ci
```

Pin the action ref and the binary version. With `seikika-version` set, the action downloads the checksum-verified archives for `seikika`, `seikika-connector-workspace` and `seikika-connector-runner`. When `.seikika/seikika.lock` is present (git packages or executable connectors), the action runs `seikika install --frozen` before `report` so the runner cache matches the lock.

## Adopt a portal in another repository

1. Copy [`examples/repo-portal-starter/seikika.yaml`](https://github.com/seikika/seikika/blob/main/examples/repo-portal-starter/seikika.yaml) to `.seikika/seikika.yaml`. The quickstart fetches that file from the `v0.2.0` tag.
2. Add the workflow above (see [actions/report README](https://github.com/seikika/seikika/blob/main/actions/report/README.md)).
3. First run with `mode: update`, commit `.seikika/results.json`.

## Archives

The release also has a `.tar.xz` (Windows: `.zip`) per app and platform, and a sibling `.sha256`. Unpack the `seikika*` files onto `PATH` when you want the bytes without the installer. `gh attestation verify <file> --repo seikika/seikika` checks the build provenance.

## Package registries

The [Rust release guide](https://github.com/seikika/seikika/blob/main/docs/RELEASING.md#rust-crates-on-cratesio) covers the separate crates.io release. The [npm catalog](/authors/npm/) covers the frontend packages.

Building Seikika itself is documented under [Run Seikika locally](/start/run/).
