---
title: Write a connector
description: Author a stdio connector with the Rust SDK, a portal stub, connector test, and connector drive.
sidebar:
  order: 2
---

A connector is a process that speaks Toolbox's control channel and emits records of named kinds. Package authors ship connectors that other portals include. Operators wire them into a running portal. This page is the package-author path.

The public docs used to say there was no scaffold. There is one now: copy
[`examples/connector-scaffold/`](https://github.com/loganvolkers/toolbox/tree/main/examples/connector-scaffold).
Agents should follow the `author-connector` skill in
[`plugins/toolbox-operator/skills/author-connector/`](https://github.com/loganvolkers/toolbox/tree/main/plugins/toolbox-operator/skills/author-connector).

## Loop

1. Copy the scaffold (keep `rust-toolchain.toml`; the SDK needs Rust 1.97 / edition 2024).
2. Implement `Source`: `name`, **`version` from your crate's `CARGO_PKG_VERSION`**, `capabilities`, `handle`. Call `run_stdio`.
3. Read config only from `TOOLBOX_CONNECTOR_CONFIG` JSON.
4. Keep a minimal `portal/toolbox.yaml` that declares the connector and its `records`.
5. Prove the protocol, then dump records without inventing stdio JSON:

```bash
cargo build
toolbox connector test --name example --portal portal -- target/debug/example-connector
toolbox connector drive --kinds example-item \
  --config '{"seed":"alpha"}' \
  -- target/debug/example-connector
```

`connector test` needs the portal. `connector drive` does not: it starts the binary, sends one `resync`, and prints `{"kind","externalId","data"}` lines.

## Secrets

Put logical placeholders in example manifests (`${secret:…}`). Operators map them. See [Secrets in connector config](/toolbox/authors/secrets/).

## What is still missing

| Gap | State |
|---|---|
| Published `toolbox-connect` crate on crates.io | Not yet; path-dep or git dep against a Toolbox release |
| SDKs for languages other than Rust | Protocol is language-agnostic; only the Rust SDK ships today |
| `toolbox connector new` generator | Scaffold directory is the substitute |

Design detail: [connector-control.md](https://github.com/loganvolkers/toolbox/blob/main/docs/design/connector-control.md). Decision that closed the learning-tax gaps from the connector A/B: [D01M83FFM2K6XS4EAVHVP92TBDZ](https://github.com/loganvolkers/toolbox/blob/main/docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md).
