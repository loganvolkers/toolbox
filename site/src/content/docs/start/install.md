---
title: Install Seikika
description: Choose a current release or build the CLI and connector binaries from source.
sidebar:
  order: 2
---

Install the Seikika CLI to create, validate, test, and serve a portal. Install the Workspace connector as well to score a repository in the [quickstart](/start/quickstart/).

## Run a portal

Check [GitHub Releases](https://github.com/seikika/seikika/releases) for a release with
`seikika-cli` binaries. These docs use the command names and manifest format introduced in
v0.2.0. The 0.1 releases use Toolbox names and a different manifest header. Until v0.2.1 is
available, [build from source](#build-from-source). Use the source build for the quickstart
if your release predates `seikika init`.

Install mise 2026.11.14 or later. Use a repository config to share the CLI and connector
versions with your team. Use the global install when you want the CLI available across
projects. [Mise installation](https://mise.jdx.dev/installing-mise.html)

The release targets Linux (glibc) and macOS on x86-64 and Arm64, and Windows on x86-64.

### Install for a repo portal

Work from the root of the repository you want to score. If it already has a mise config,
merge the entries from the starter into its `[tool_alias]` and `[tools]` tables. If it has
no mise config, download the example as `mise.toml`. [Starter mise config](https://github.com/seikika/seikika/blob/main/examples/repo-portal-starter/mise.example.toml)

```bash
curl -fsSL https://raw.githubusercontent.com/seikika/seikika/main/examples/repo-portal-starter/mise.example.toml -o mise.toml
```

Use `curl.exe` in PowerShell. Review the config before you trust it. Each alias points to
the Seikika release repository, and each `matching` value selects that app's archive.

```sh
mise trust
mise install
mise exec -- seikika --version
mise which seikika-connector-workspace
```

The version should match the CLI pin in the config, and `mise which` should print the path
to the workspace connector. Commit the config to share both pins. Once your portal manifest
is in `.seikika/`, check that the CLI can find its connector.

```sh
mise exec -- seikika doctor .seikika
```

Look for `ok` next to the workspace connector. `mise exec --` makes the project's tools
available for one command. To use plain `seikika` commands, activate mise in your shell.
[Mise shell setup](https://mise.jdx.dev/shell-setup.html)

### Install the CLI globally

Use a terminal outside a project with its own Seikika pin to verify your global install.

```sh
mise use --global 'github:seikika/seikika[matching=seikika-cli-]@0.2.1'
mise exec -- seikika --version
```

The release also supplies shell and PowerShell installers, archives, and checksum files.

## Install a connector

Choose a connector and check its prerequisites and record kinds in the [connector directory](/connectors/).

If the manifest contains `package: builtin:NAME`, install the `seikika-connector-NAME`
executable. The starter mise config above includes the workspace connector for the tutorial.
For `builtin:runner`, add these entries to the corresponding tables in your config. Use the
same version as the CLI.

```toml
[tool_alias]
seikika-connector-runner = "github:seikika/seikika"

[tools]
seikika-connector-runner = { version = "0.2.1", matching = "seikika-connector-runner-" }
```

Review the updated config, run `mise trust` and `mise install`, then run
`mise exec -- seikika doctor .seikika`. Use the directory containing `seikika.yaml` if
your manifest lives elsewhere. Mise puts the installed connectors on the command's `PATH`.
Seikika also looks beside the CLI and in `SEIKIKA_CONNECTORS_DIR`.

## Build from source

Install Git, Rust through rustup, a native linker, and the Node.js and pnpm versions pinned by the checkout. On Linux, install a C compiler and build tools, such as `build-essential` on Debian or Ubuntu. On macOS, install the Xcode Command Line Tools. On Windows, use a Rust-supported C++ toolchain and run the shell examples in a Unix-compatible environment such as WSL. [Git setup](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git) · [Rust setup](https://www.rust-lang.org/tools/install) · [Pinned development tools](/start/run/#set-up-the-checkout)

From a directory where you can create a checkout, run these commands in Bash.

```bash
git clone https://github.com/seikika/seikika.git seikika-source
cd seikika-source
pnpm install --frozen-lockfile
pnpm build
cargo build --locked -p seikika-cli -p seikika-connector-workspace
export PATH="$PWD/target/debug:$PATH"
seikika --version
```

Rustup selects the toolchain pinned by `rust-toolchain.toml`. The last command prints the CLI version. The path change applies to this shell. If you open another terminal, add the checkout's absolute `target/debug` path again.

To build a different first-party connector, run `cargo build --locked -p seikika-connector-NAME` in the source checkout, replacing `NAME` with the connector name. For details about changing Seikika itself, see [Run Seikika locally](/start/run/).

## Version and setup checks

- See the [release and upgrade guide](/start/versions/) for compatibility, release notes, and rollback checks.
- See [Gate pull requests](/operators/ci/) for the action configuration and the CLI-only workflow.
- If a setup command fails, check [installation and scoring problems](/start/troubleshooting/#installation-and-scoring).
