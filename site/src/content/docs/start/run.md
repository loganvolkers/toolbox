---
title: Run Seikika locally
description: Build the seikika binary and validate the reference portal.
sidebar:
  order: 3
---

Build from source to try the current checkout or contribute to Seikika. Install [mise](https://mise.jdx.dev/installing-mise.html) 2026.11.14 or later, then run these commands in the Seikika checkout. Mise reads the Rust version and components from `rust-toolchain.toml`.

```bash
mise trust
mise install
mise exec -- cargo build --locked
mise exec -- target/debug/seikika validate examples/reference-portal
```

`validate` checks a portal manifest. The reference portal under `examples/reference-portal` is the anchor for v1: its expected outputs are the oracle.

## Try a repo portal from source

Build the CLI and the starter's workspace connector. Set `SEIKIKA_SRC` from the clone so
you can use the same binaries and manifest from a different checkout.

```bash
git clone https://github.com/seikika/seikika.git
cd seikika
export SEIKIKA_SRC="$PWD"
mise trust
mise exec rust -- cargo build --locked -p seikika-cli -p seikika-connector-workspace
export PATH="$SEIKIKA_SRC/target/debug:$PATH"
```

In the Git repository you want to score, copy the starter and continue at
[Score once](/start/quickstart/#3-score-once).

```bash
mkdir -p .seikika
cp "$SEIKIKA_SRC/examples/repo-portal-starter/seikika.yaml" .seikika/seikika.yaml
```

## Score this repository

Seikika also scores itself. From a built checkout:

```bash
mise exec -- target/debug/seikika report portal --skip runner,github-actions
```

That report reads the catalog in `portal/` and prints each scorecard with the evidence for rules that fail. It does not need Postgres.

## What you get

| Surface | What it is |
|---|---|
| Catalog | Entities (crates, docs, services, scorecards) and the relations between them |
| Scorecards | Levels with evidence and a fix, ratcheted in CI |
| API, MCP, CLI, UI | The same reads, for people and for agents |

Next: [what a catalog holds](/concepts/catalog/).
