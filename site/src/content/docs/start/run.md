---
title: Run Toolbox locally
description: Build the toolbox binary and validate the reference portal.
---

Toolbox is a Rust workspace. The binary is `target/debug/toolbox`.

```bash
cargo build
target/debug/toolbox validate examples/reference-portal
```

`validate` checks a portal manifest. The reference portal under `examples/reference-portal` is the anchor for v1: its expected outputs are the oracle.

## Score this repository

Toolbox also scores itself. From a built checkout:

```bash
target/debug/toolbox report portal --skip runner,github-actions
```

That report reads the catalog in `portal/` and prints each scorecard with the evidence for rules that fail. It does not need Postgres.

## What you get

| Surface | What it is |
|---|---|
| Catalog | Entities (crates, docs, services, scorecards) and the relations between them |
| Scorecards | Levels with evidence and a fix, ratcheted in CI |
| API, MCP, CLI, UI | The same reads, for people and for agents |

Next: [what a catalog holds](/toolbox/concepts/catalog/).
