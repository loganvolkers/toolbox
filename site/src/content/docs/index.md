---
title: Toolbox
description: An agent-first internal developer portal, configured in code.
---

Toolbox is an open-source framework for an internal developer portal. A portal is a catalog of what your organisation runs, the standards those things are held to, and the actions an agent can take. You configure the portal in code. The same catalog is exposed over an API, an MCP server, a CLI and a UI.

## Who you are

Four groups build with Toolbox. Each group has its own loop, and a command belongs to one of them.

| You | You are here to | Start |
|---|---|---|
| Platform team | Build and run a portal | [Build a portal](/toolbox/operators/) |
| Standards author | Publish scorecards other portals include | [Publish a standard](/toolbox/authors/) |
| Developer or agent | Read the catalog and build on it | [Use a portal](/toolbox/users/) |
| Toolbox maintainer | Change this repository | [Build Toolbox](/toolbox/maintainers/) |

[The four SDLC layers](/toolbox/concepts/layers/) is how those loops meet, and which parts run today.

## Measured, not claimed

Toolbox benchmarks its own developer experience. Coding agents do real work on open-source repositories, once with Toolbox and once with the alternatives: hand-written glue, a checklist, an agent building from scratch, or OpenSSF Scorecard. The bar is set before each run, and the losses are published. See [DX Benchmarks](/toolbox/concepts/dx-benchmarks/).

## Start

- [Quickstart](/toolbox/start/quickstart/) — install, score a repo, ratchet in CI
- [Install a release](/toolbox/start/install/)
- [Run Toolbox locally](/toolbox/start/run/)
- [CLI reference](/toolbox/start/cli/)
- [Glossary](/toolbox/concepts/glossary/)
- [The catalog](/toolbox/concepts/catalog/)
- [Scorecards](/toolbox/concepts/scorecards/)
- [DX Benchmarks](/toolbox/concepts/dx-benchmarks/)
- [Toolbox vs others](/toolbox/compare/)
- [This repository, scored](/toolbox/demo/report/)

The source of this site is in the [Toolbox repository](https://github.com/loganvolkers/toolbox). The pages you are reading are catalogued and scored with the rest of the repository.
