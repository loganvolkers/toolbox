---
id: D01M8BEC7ZX7TFBTSC47W1XQVV4
decided: 2026-11-26T08:08:29Z
---

# Publish the Rust workspace on crates.io

Publish the CLI, connector SDK, connectors, and the libraries they depend on under their
existing `seikika-*` crate names. Keep `seikika-durable` private while it is a placeholder.
The workspace uses one version. In-workspace dependencies pin that version so a published
CLI resolves the matching protocols and components.

Place the JSON Schemas in `seikika-model` and the generated hosted UI in `seikika-api`.
The crate archives contain everything their Rust source embeds. The hosted portal and CLI
still consume those same files in a workspace checkout.

Verify the crate archives with Cargo before publishing. Use an authenticated first publish
to establish ownership, then register crates.io Trusted Publishing for subsequent releases.
See [the release guide](../RELEASING.md#rust-crates-on-cratesio).
