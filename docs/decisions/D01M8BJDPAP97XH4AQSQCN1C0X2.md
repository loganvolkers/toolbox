---
id: D01M8BJDPAP97XH4AQSQCN1C0X2
decided: 2026-11-26T09:19:11Z
---

# Use mise for maintainer tools and operator installs

Maintainers install the checkout's tools with mise. Keep the Rust version and components in
`rust-toolchain.toml`, use the Node and Python series from CI, and align the CLI tool pins with
the checks that use them. Keep application dependencies in the existing lockfiles.

For the operator path in [D90](D01M7WVNEYT0000000000000000.md), use mise's GitHub backend with
one alias per cargo-dist app. The aliases let mise install the CLI and the selected connectors
from one release and expose all installed binaries on `PATH`. Start with v0.2.0, whose archive
names use `seikika-`. The first-party config can ship in the release tag.

See the [maintainer setup](../design/maintainer-sdlc.md#5-local-prerequisites) and the
[operator install guide](../../site/src/content/docs/start/install.md). [Mise Rust toolchains](https://mise.jdx.dev/lang/rust.html), [GitHub backend](https://mise.jdx.dev/dev-tools/backends/github.html).
