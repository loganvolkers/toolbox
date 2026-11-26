---
title: Run Seikika locally
description: Build the seikika binary and validate the reference portal.
sidebar:
  order: 3
---

Prefer a [released install](/start/install/) for another repository. Build from source when you are changing Seikika itself. The binary is `target/debug/seikika`.

```bash
cargo build
target/debug/seikika validate examples/reference-portal
```

`validate` checks a portal manifest. The reference portal under `examples/reference-portal` is the anchor for v1: its expected outputs are the oracle.

## Score this repository

Seikika also scores itself. From a built checkout:

```bash
target/debug/seikika report portal --skip runner,github-actions
```

That report reads the catalog in `portal/` and prints each scorecard with the evidence for rules that fail. It does not need Postgres.

## What you get

| Surface | What it is |
|---|---|
| Catalog | Entities (crates, docs, services, scorecards) and the relations between them |
| Scorecards | Levels with evidence and a fix, ratcheted in CI |
| API, MCP, CLI, UI | The same reads, for people and for agents |

Next: [what a catalog holds](/concepts/catalog/).
