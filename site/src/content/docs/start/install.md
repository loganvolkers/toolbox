---
title: Install Seikika
description: Choose a current release or build the CLI and connector binaries from source.
sidebar:
  order: 2
---

Install the Seikika CLI to validate, test, and serve a portal with fixtures. Install the workspace connector as well to score a repository with the [starter manifest](/start/quickstart/).

## Run a portal

Check [GitHub Releases](https://github.com/seikika/seikika/releases) for a release with
`seikika-cli` binaries. These docs use the command names and manifest format introduced in
v0.2.0. The 0.1 releases use Toolbox names and a different manifest header. Until v0.2.0 is
available, [build from source](#build-from-source).

Install mise 2026.11.14 or later and activate it in your shell. Run the following to install
the CLI for your user. [Mise installation](https://mise.jdx.dev/installing-mise.html) ·
[Shell setup](https://mise.jdx.dev/shell-setup.html)

```bash
mise use --global 'github:seikika/seikika[matching=seikika-cli-]@0.2.0'
seikika --version
```

To pin tools for a repo portal, copy the [starter mise config](https://github.com/seikika/seikika/blob/main/examples/repo-portal-starter/mise.example.toml)
to the root of your Git checkout as `mise.toml`, or merge its aliases and versions into an
existing config. It pins the CLI and workspace connector. Review the config, then run
`mise trust` and `mise install` in that checkout.

The release also supplies shell and PowerShell installers, archives, and checksum files.
Use matching CLI and connector versions.

## Install a connector

If the manifest contains `package: builtin:NAME`, install the `seikika-connector-NAME`
executable. The starter mise config above includes the workspace connector for the tutorial.
Use its connector entry as a template for other connector aliases. Seikika looks beside the CLI, in
`SEIKIKA_CONNECTORS_DIR`, and on `PATH`. Use `seikika doctor PATH_TO_PORTAL` to check
what resolves. Pass the directory containing `seikika.yaml`.

## Build from source

Install [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git), [Rust through rustup](https://www.rust-lang.org/tools/install), and a native linker. On Linux, install a C compiler and build tools, such as `build-essential` on Debian or Ubuntu. On macOS, install the Xcode Command Line Tools. On Windows, use a Rust-supported C++ toolchain and run the shell examples in a Unix-compatible environment such as WSL.

From a directory where you can create a checkout, run these commands in Bash. The revision matches the starter and CI example in the quickstart.

```bash
git clone https://github.com/seikika/seikika.git seikika-source
cd seikika-source
git checkout --detach c14c9fbb60405f9fea12d86de3d0b32b717755d9
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
