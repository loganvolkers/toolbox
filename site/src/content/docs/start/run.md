---
title: Run Seikika locally
description: Build Seikika from a source checkout and check the reference portal.
sidebar:
  order: 3
---

Build from source to work on Seikika itself. If you only need a CLI for another repository, see [Install Seikika](/start/install/).

## Set up the checkout

Install mise 2026.11.14 or later, Git, and a native linker. The Seikika checkout declares
its development tools in `mise.toml` and its Rust version and components in
`rust-toolchain.toml`. Mise installs Rust through rustup. [Mise installation](https://mise.jdx.dev/installing-mise.html) · [Platform build prerequisites](/start/install/#build-from-source)

From the root of your Seikika checkout, review `mise.toml` and run these commands.

```bash
mise --version
mise trust
mise install
mise ls --current
```

`mise ls --current` lists the selected tools and their config sources. Run `mise install`
again when the checkout changes a tool pin. Use `mise exec --` to run a command with
the checkout's tools, or activate mise to use those tools for the whole shell.
[Mise shell setup](https://mise.jdx.dev/shell-setup.html)

## Build and verify

Run these commands in Bash from the Seikika checkout.

```bash
mise exec -- cargo build --locked
mise exec -- target/debug/seikika validate examples/reference-portal
mise exec -- target/debug/seikika test examples/reference-portal
```

Look for zero validation errors and passing tests. The example configures a catalog with sample ownership, scorecards, and permissions. Its expected outputs give maintainers a way to test changes against the same input.

Follow [Explore a portal](/start/portal/) to run the example, or [Build Seikika](/maintainers/) for other repository checks.
