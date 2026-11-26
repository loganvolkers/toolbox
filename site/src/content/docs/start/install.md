---
title: Install Seikika
description: Choose a current release or build the CLI and connector binaries from source.
sidebar:
  order: 2
---

Install the Seikika CLI to validate, test, and serve a portal with fixtures. Install the workspace connector as well to score a repository with the [starter manifest](/start/quickstart/).

## Run a portal

Check [GitHub Releases](https://github.com/seikika/seikika/releases) for a release with assets named `seikika-cli-installer`. These docs use the **Seikika** command names and manifest format introduced in 0.2. The 0.1 releases use **Toolbox** names and a different manifest header. If the release you need is unavailable, [build from source](#build-from-source).

1. Open the release you want and choose its installer for your system. The shell installer ends in `.sh` and the PowerShell installer ends in `.ps1`. The release notes list supported platforms and commands.
2. Run the downloaded installer. It selects the archive for your platform, verifies its checksum, and adds the executable to the installer directory. Open a new terminal if the installer updated your path.
3. Check the installed version.

   ```bash
   seikika --version
   ```

The [release assets](https://github.com/seikika/seikika/releases) also include archives and checksums for manual installation. Use matching CLI and connector versions.

## Install a connector

If the manifest contains `package: builtin:NAME`, install the `seikika-connector-NAME` executable. For example, the repo starter needs `seikika-connector-workspace`. Seikika looks beside the CLI, in `SEIKIKA_CONNECTORS_DIR`, and on `PATH`. Use `seikika doctor PATH_TO_PORTAL` to check what resolves, where `PATH_TO_PORTAL` is the directory containing `seikika.yaml`.

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
