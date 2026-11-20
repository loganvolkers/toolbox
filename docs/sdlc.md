# SDLC in Toolbox: Four Layers

*Status: 2026-11-20 (decisions D63 and D87, amending D34/G2). Whenever a Toolbox document says
"SDLC", it means one of these four layers and says which. D87 added the package author SDLC.*

Toolbox has four groups of builders. Each needs its own plan → build → test → deploy → monitor
loop, at the standard of ordinary software engineering, and each is worked mostly by agents in
harnesses such as Claude Code.

| Layer | Who | What they build | Where it's designed | State today |
|---|---|---|---|---|
| **1. Maintainer SDLC** | People and agents working on *this repository* | Toolbox itself: the Rust workspace, connectors, the UI, schemas, docs | [design/maintainer-sdlc.md](./design/maintainer-sdlc.md), with [developer-velocity.md](./design/developer-velocity.md) and [verification.md](./design/verification.md) | **In daily use:** pinned toolchain, reference-model differential tests, parity and acceptance scripts, CI, scale budget |
| **2. Package author SDLC** (D87) | Anyone publishing packages, connectors or standards for others: platform teams, vendors, and us for our own standards | *Packages*: standards as data, connector binaries or WASM components, and the tools they require | [design/package-author-sdlc.md](./design/package-author-sdlc.md), [design/packages.md](./design/packages.md) | **Mostly missing:** connectors outside this repository can only be remote, no package format, and no released Toolbox to test against |
| **3. Operator SDLC** | Platform teams and their agents running a portal | A *portal*: manifest, authorization model, connectors, actions, UI bundles, deployed as one image | [design/sdlc.md](./design/sdlc.md) | **Partly built:** `validate`, `test`, `plan --local`, `derive`, `serve`, `push`, `connector test`; remote plan, previews, build, deploy and verify arrive with M6 |
| **4. User SDLC** | Developers and agents *using* a portal | *Apps* powered by Toolbox context and actions: agents, scripts, dashboards, internal tools, CI checks | [design/user-sdlc.md](./design/user-sdlc.md) | **Foundations built:** MCP, REST, CLI, PATs, snapshot ids for reproducible reads; typed clients, the generated user plugin and action runs are still to come |

## Why separate them

- **Different artefacts, different contracts.** Maintainers change Toolbox, so their contract is the
  [v1 spec](./v1-spec.md) and the reference model. Operators change a portal, so theirs is the manifest
  schema and `toolbox plan`. Users build on a portal, so theirs is the API, the MCP tools and their
  stability.
- **Different speeds that must not leak.** A maintainer's inner loop can take seconds; an operator's
  must too (charter goal G2). Neither may break a user's app without a versioned, announced change.
- **Different testing data.** Maintainers test against the reference portal's fixtures and
  generated worlds. Operators test their own fixtures and their own authorization model. Users test
  against a dev or preview portal and pin snapshot ids.

## How the layers connect

```
Layer 1 (maintainers)     ── releases toolbox, image, SDKs, schemas ────▶  Layers 2 and 3
Layer 2 (package authors) ── distributes packages: standards, connectors ─▶  Layer 3 (operators)
Layer 3 (operators)       ── deploys a portal: API, MCP, UI, actions ─────▶  Layer 4 (users)
Layer 4 (users)           ── feedback, needs, failing scorecards ─────────▶  Layers 3, 2, 1

Package authors fork from the maintainer loop at *test*: they test one package against released
Toolbox versions and platforms, then distribute it, instead of merging into one repository.
```

The reference portal ([examples/reference-portal](../examples/reference-portal/README.md)) sits on
the boundary between the layers. It is a layer-2 portal that layer 1 uses as its acceptance oracle,
and its acceptance scenarios are written as layer-3 tasks ("who owns X?", "refresh this", "install
this app").

## Toolbox scores its own layers

[Toolbox for Toolbox](./design/toolbox-for-toolbox.md) (D65) turns these layers into scorecards on
this repository: `maintainer-crate` and `maintainer-docs` (layer 1), `operator-cli` (layer 3), and
`user-cli` and `user-mcp` (layer 4). The layers themselves are catalogued too (D87): `portal/sdlc/`
declares every stage of every layer (exit criterion, commands, checks, status, gaps), scored by
`sdlc-stage` and `sdlc-layer` and reviewed by agents against `sdlc-stage-quality`. Run `scripts/self-portal.sh report`, or point an agent at
`scripts/self-portal.sh serve`.

## Rules for documents

1. Name the layer: "maintainer SDLC", "package author SDLC", "operator SDLC" or "user SDLC", never bare "SDLC".
2. A capability that serves several layers says how it serves each (for example, `toolbox test`
   is a layer-2 tool that layer 1 also uses on the reference portal).
3. Each layer document keeps an **implemented vs. planned** table, so agents can tell what they can
   rely on today.
