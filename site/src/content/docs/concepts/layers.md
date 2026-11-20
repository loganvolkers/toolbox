---
title: Three SDLC layers
description: Maintainer, operator and user are three different pieces of software, with three different loops.
---

Toolbox is built in three layers. Naming the layer keeps a standard from being applied to the wrong person.

| Layer | Loop |
|---|---|
| Maintainer SDLC | Change this repository, run its checks, and keep the scorecards from regressing |
| Operator SDLC | Change a portal: validate, test, plan, deploy, verify |
| User SDLC | Build an app or an agent on a portal's published API, MCP and CLI |

The maintainer layer is what this repository's scorecards cover today: crates, documents, skills, workflows, and the pages of this site. The operator layer is the loop a platform team uses to change a portal without editing Toolbox. The user layer is the contract an application depends on, versioned and stable.

A rule belongs to one layer. A document that says "SDLC" without naming the layer fails the docs scorecard, because an agent cannot tell which loop the sentence is about.

Read [how to change a portal](/toolbox/operators/portal/) for the operator loop, and [how an agent reads the catalog](/toolbox/agents/reads/) for the user loop.
