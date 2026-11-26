---
title: Write a connector
description: Author a stdio connector with the Rust SDK, a portal stub, connector test, and connector drive.
sidebar:
  order: 2
---

A connector is a process that speaks Seikika's control channel and emits records of named kinds. Package authors ship connectors that other portals include. Operators wire a shipped or published connector into a running portal. This page is the package-author path. An operator who runs builtins and included packages follows [Install a release](/start/install/) and [Build a portal](/operators/).

## Install

1. [Install the `seikika` CLI for your user](/start/install/#install-the-cli-for-your-user) and [activate mise in your shell](https://mise.jdx.dev/shell-setup.html), or use the [source build](/start/run/#try-a-repo-portal-from-source).
2. Open the scaffold in a checkout. Its `rust-toolchain.toml` pins Rust 1.97, which the SDK needs.

```bash
git clone --depth 1 https://github.com/seikika/seikika.git
cd seikika/examples/connector-scaffold
cargo build
```

The clone is the scaffold source
([`examples/connector-scaffold/`](https://github.com/seikika/seikika/tree/main/examples/connector-scaffold)).
`cargo build` produces your connector. A test portal that names a `builtin:` also needs that connector binary on `PATH`.

Agents should follow the `author-connector` skill in
[`plugins/seikika-operator/skills/author-connector/`](https://github.com/seikika/seikika/tree/main/plugins/seikika-operator/skills/author-connector).

## Loop

1. Copy the scaffold (keep `rust-toolchain.toml`; the SDK needs Rust 1.97 / edition 2024).
2. Implement `Source`: `name`, **`version` from your crate's `CARGO_PKG_VERSION`**, `capabilities`, `handle`. Call `run_stdio`.
3. Read config only from `SEIKIKA_CONNECTOR_CONFIG` JSON.
4. Keep a minimal `portal/seikika.yaml` that declares the connector and its `records`.
5. Prove the protocol, then dump records without inventing stdio JSON:

```bash
cargo build
seikika connector test --name example --portal portal -- target/debug/example-connector
seikika connector drive --kinds example-item --format catalog --out OUT/entities.ndjson \
  --config '{"seed":"alpha"}' \
  -- target/debug/example-connector
```

`connector test` needs the portal. `connector drive` does not: it starts the binary, sends one `resync`, and with `--format catalog` writes sorted `{"kind","id","data"}` lines (`id` = `externalId` with a leading `{kind}:` stripped). Default `--format records` still prints protocol dumps (`kind`/`externalId`/`data`).

## Secrets

Put logical placeholders in example manifests (`${secret:…}`). Operators map them. See [Secrets in connector config](/authors/secrets/).

The scaffold uses a workspace path dependency. See the [Rust release guide](https://github.com/seikika/seikika/blob/main/docs/RELEASING.md#rust-crates-on-cratesio) for the crates.io release.

## What is still missing

| Gap | State |
|---|---|
| SDKs for languages other than Rust | Protocol is language-agnostic; only the Rust SDK ships today |
| `seikika connector new` generator | Scaffold directory is the substitute |

Design detail: [connector-control.md](https://github.com/seikika/seikika/blob/main/docs/design/connector-control.md). Decision that closed the learning-tax gaps from the connector A/B: [D01M83FFM2K6XS4EAVHVP92TBDZ](https://github.com/seikika/seikika/blob/main/docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md).
