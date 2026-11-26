---
title: Install a release
description: Install the Seikika CLI for a portal, and the extra setup for writing a connector.
sidebar:
  order: 2
---

Mise installs Seikika's prebuilt binaries from GitHub Releases (D90). These examples pin **v0.2.0**, the first release under the Seikika name. Until that tag appears on [GitHub Releases](https://github.com/seikika/seikika/releases), [build from source](/start/run/#try-a-repo-portal-from-source).

## Run a portal

Install [mise](https://mise.jdx.dev/installing-mise.html) 2026.11.14 or later. Add these entries to `mise.toml` at the root of your portal repository.

```toml
[tool_alias]
seikika = "github:seikika/seikika"

[tools]
seikika = { version = "0.2.0", matching = "seikika-cli-" }
```

Run these commands in that repository. Mise selects and verifies the release archive for your platform. Seikika releases Linux and macOS binaries for x86-64 and ARM64, and Windows binaries for x86-64.

```sh
mise trust
mise install
mise exec -- seikika --version
```

`seikika validate`, `seikika test`, `seikika plan` and `seikika serve --fixtures` use this binary alone. The [quickstart](/start/quickstart/) provides a config with both the CLI and the workspace connector for a repo portal.

### Connector binaries the portal runs

When the manifest says `package: builtin:<name>`, add `seikika-connector-<name>` to both tables in your config. For example, `builtin:runner` adds the following entries.

```toml
[tool_alias]
seikika-connector-runner = "github:seikika/seikika"

[tools]
seikika-connector-runner = { version = "0.2.0", matching = "seikika-connector-runner-" }
```

Use the same pattern for `workspace`, `github`, `git-files`, `gitlab` and `scim`. Keep the CLI and connector versions in sync. Run `mise install` after editing, then `mise exec -- seikika doctor .seikika` if your portal is in `.seikika/`. Mise puts each installed connector on the command's `PATH`.

### Install the CLI for your user

After installing mise, add the CLI to your global config to use it across projects.

```sh
mise use --global 'github:seikika/seikika[matching=seikika-cli-]@0.2.0'
mise exec -- seikika --version
```

## Write a connector

Install the CLI for your user and use the Rust toolchain pinned in the scaffold. See [Write a connector](/authors/connectors/).

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

## Archives and standalone installers

Each release also provides shell and PowerShell installers and a `.tar.xz` (Windows: `.zip`) archive per app and platform. If you unpack an archive manually, check the sibling `.sha256` and put the binaries on `PATH`. `gh attestation verify <file> --repo seikika/seikika` checks the build provenance.

## Package registries

The [Rust release guide](https://github.com/seikika/seikika/blob/main/docs/RELEASING.md#rust-crates-on-cratesio) covers the separate crates.io release. The [npm catalog](/authors/npm/) covers the frontend packages.

Building Seikika itself is documented under [Run Seikika locally](/start/run/).
