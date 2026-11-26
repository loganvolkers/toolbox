---
title: Run Seikika locally
description: Build Seikika from a source checkout and check the reference portal.
sidebar:
  order: 3
---

Build from source to work on Seikika itself. If you only need a CLI for another repository, see [Install Seikika](/start/install/).

## Build and verify

You need Git, Rust through rustup, and a native linker. See the [source build prerequisites](/start/install/#build-from-source) for your platform. From the root of your Seikika checkout, run:

```bash
cargo build --locked
target/debug/seikika validate examples/reference-portal
target/debug/seikika test examples/reference-portal
```

Look for zero validation errors and passing tests. The example configures a catalog with sample ownership, scorecards, and permissions. Its expected outputs give maintainers a way to test changes against the same input.

Follow [Explore a portal](/start/portal/) to run the example, or [Build Seikika](/maintainers/) for other repository checks.
