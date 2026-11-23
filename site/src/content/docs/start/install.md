---
title: Install a release
description: Install Toolbox v0.1.1 from the GitHub Release installers, without cloning the repository.
sidebar:
  order: 2
---

From **v0.1.0** onward, install the binaries from GitHub Releases (D90). Current release: **v0.1.1**. Each app on that release has a shell installer and a PowerShell installer; the release notes print the same commands. New to Toolbox? Start at [Quickstart](/toolbox/start/quickstart/).

The installers pick the archive for this machine, check its SHA-256, and place the binaries in `$CARGO_HOME/bin` (`~/.cargo/bin` when `CARGO_HOME` is unset). That is cargo-dist's default install path, so the binaries sit on `PATH` next to other tools installed the same way. Open a new shell after installing, or export `PATH` in the current one.

## What a repo portal needs

`toolbox report` needs these binaries. A connector has to sit next to `toolbox`, on `PATH`, or in `TOOLBOX_CONNECTORS_DIR`.

| Installer | Binary | When |
|---|---|---|
| `toolbox-cli` | `toolbox` | Always |
| `toolbox-connector-workspace` | `toolbox-connector-workspace` | The portal uses `builtin:workspace` (the starter does) |
| `toolbox-connector-runner` | `toolbox-connector-runner` | The portal enables the runner connector |

### macOS and Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-workspace-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-runner-installer.sh | sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
toolbox --version
```

Skip the runner installer when the portal does not enable that connector. `toolbox doctor <portal>` reports a missing connector binary.

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-workspace-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-runner-installer.ps1 | iex"
```

## Other connectors

GitHub, git-files and SCIM are separate apps on the same release. Use the same URL with `toolbox-connector-github`, `toolbox-connector-git-files` or `toolbox-connector-scim` in place of `toolbox-cli`.

## Archives

The release also has a `.tar.xz` (Windows: `.zip`) per platform and a sibling `.sha256`. Unpack the `toolbox*` files onto `PATH` when you want the bytes without the installer. `gh attestation verify <file> --repo loganvolkers/toolbox` checks the build provenance.

## In GitHub Actions

```yaml
- uses: loganvolkers/toolbox/actions/report@v0.1.1   # or a commit SHA
  with:
    toolbox-version: "0.1.1"
    mode: ci
```

Pin the action ref and the binary version. With `toolbox-version` set, the action downloads checksum-verified archives. When `.toolbox/toolbox.lock` is present (git packages or executable connectors), the action runs `toolbox install --frozen` before `report` so the runner cache matches the lock.

## Adopt a portal in another repository

1. Copy [`examples/repo-portal-starter/toolbox.yaml`](https://github.com/loganvolkers/toolbox/blob/main/examples/repo-portal-starter/toolbox.yaml) to `.toolbox/toolbox.yaml`. The quickstart fetches that file from the `v0.1.1` tag.
2. Add the workflow above (see [actions/report README](https://github.com/loganvolkers/toolbox/blob/main/actions/report/README.md)).
3. First run with `mode: update`, commit `.toolbox/results.json`.

## Public repository required for external CI

If `loganvolkers/toolbox` is **private**, another repository's workflow cannot resolve `uses: loganvolkers/toolbox/...` (`repository not found`), and its default `GITHUB_TOKEN` cannot download release assets. Make the repo public (Settings → Change repository visibility → Public), or give each consumer a fine-scoped read token / collaborator access (see [docs/RELEASING.md](https://github.com/loganvolkers/toolbox/blob/main/docs/RELEASING.md)).

## Later channels

Homebrew, npm and aqua are still deferred (D90). The crates carry cargo-binstall metadata and are not published to crates.io, so `cargo binstall` does not resolve them yet.

Building Toolbox itself is documented under [Run Toolbox locally](/toolbox/start/run/).
