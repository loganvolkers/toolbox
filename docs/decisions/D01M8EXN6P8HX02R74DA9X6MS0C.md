---
id: D01M8EXN6P8HX02R74DA9X6MS0C
decided: 2026-11-27T16:33:15Z
---

# Generate embedded UI assets before building Rust

Source builds use the pinned Node.js and pnpm toolchain to generate the hosted portal and offline
report before Cargo compiles the server or CLI. Git ignores both output directories. The Rust
binary embeds the assets and can run and generate reports without a JavaScript toolchain.

The binary release builders generate the assets with the frozen frontend lockfile. The two
Rust crates that embed them include the generated files in their crates.io archives. The
publishing workflow packages and verifies the workspace, then passes those assets to the
publishing job. Cargo packaging allows these untracked inputs.

This supersedes the committed-bundle freshness gate in D01M880QMWD7Z7FTGKJT99AHWBX. The Astro
and Vue architecture in D01M8A5518E6GYEY9EG8Z7FPXK2 continues to apply. See the
[maintainer build checks](../design/maintainer-sdlc.md) and [release workflow](../RELEASING.md).
