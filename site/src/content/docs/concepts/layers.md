---
title: Four SDLC layers
description: Maintainer, package author, operator and user are four different pieces of software, with four different loops.
---

Toolbox has four groups of builders. Naming the layer keeps a standard, a command and a review from being applied to the wrong person.

| Layer | Who | What they build | The page |
|---|---|---|---|
| Maintainer SDLC | People changing this repository | Toolbox: the binary, the connectors it ships, the schemas, this site | [Build Toolbox](/toolbox/maintainers/) |
| Package author SDLC | Anyone publishing standards for other portals | A package: scorecards, fixtures, a connector, the tools those rules need | [Publish a standard](/toolbox/authors/) |
| Operator SDLC | A platform team | A portal: manifest, connectors, authorization, a running service | [Build a portal](/toolbox/operators/) |
| User SDLC | A developer or an agent | An app, a script, a check or an agent on that portal's API, MCP and CLI | [Use a portal](/toolbox/users/) |

The loops meet at the edges. Maintainers release Toolbox. Package authors pin a Toolbox version and ship a package. Operators include that package and deploy a portal. Users call the portal. Feedback, a failing scorecard, a missing field, comes back up the same chain.

A sentence that says "SDLC" and names no layer is ambiguous, and the docs scorecard fails it. The four names above are the ones to use.

## What each loop can rely on

| Layer | In the binary today | Specified, and not in the binary yet |
|---|---|---|
| Maintainer SDLC | Build, test, the reference portal as the oracle, the ratchet on this repository | A tagged release others can install |
| Package author SDLC | A package directory, `include`, `toolbox test`, `toolbox install` pinning a git tag by content hash | A release archive, attestations, a fleet rollout. The registry is an open question |
| Operator SDLC | `validate`, `test`, `plan --local`, `serve`, `push`, `connector test`, `report` | A plan against live data, preview environments, `toolbox verify` |
| User SDLC | The catalog reads on CLI, MCP and HTTP, personal access tokens, snapshot ids | Typed clients, service identities, a deprecation feed. Action runs are paused |

The reference portal sits on the boundary. Maintainers use it as the acceptance oracle. Its scenarios are written as user tasks: who owns a repository, why a level is redacted, refresh this, install this app.
