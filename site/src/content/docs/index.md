---
title: Toolbox
description: An agent-first internal developer portal, configured in code.
---

Toolbox is an open-source framework for an internal developer portal. A portal is a catalog of what your organisation runs, the standards those things are held to, and the actions an agent can take. You configure the portal in code. The same catalog is exposed over an API, an MCP server, a CLI and a UI.

## Three SDLC layers

Toolbox names three software-development layers, so a sentence about "the SDLC" says which one.

| Layer | Who | What they build |
|---|---|---|
| Maintainer SDLC | The people building Toolbox | This repository |
| Operator SDLC | A platform team | A portal |
| User SDLC | A developer or an agent | An app on that portal |

[The three layers](/toolbox/concepts/layers/) is the longer version.

## Start

- [Run Toolbox locally](/toolbox/start/run/)
- [The catalog](/toolbox/concepts/catalog/)
- [Scorecards](/toolbox/concepts/scorecards/)
- [Change a portal](/toolbox/operators/portal/)
- [Read the catalog from an agent](/toolbox/agents/reads/)
- [This repository, scored](/toolbox/demo/report/)

The source of this site is in the [Toolbox repository](https://github.com/loganvolkers/toolbox). The pages you are reading are catalogued and scored with the rest of the repository.
