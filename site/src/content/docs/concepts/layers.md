---
title: Four SDLC layers
description: See what operators, package authors, users, and maintainers build with Seikika.
---

Several groups work with Seikika. Each group owns a different part of the software development lifecycle (SDLC). One person can work in more than one group.

| Role | What you build | Start here |
|---|---|---|
| Operator | A portal with data sources, rules, and permissions | [Build a portal](/operators/) |
| Package author | Reusable standards, tests, and connectors | [Publish a standard](/authors/) |
| User | An app, script, check, or agent that uses the portal | [Use a portal](/users/) |
| Maintainer | The Seikika CLI, server, first-party connectors, and docs | [Build Seikika](/maintainers/) |

Maintainers release Seikika. Package authors publish standards against a known version. Operators include packages when they configure their portals. Users read the catalog and report gaps, such as a missing field or a rule that needs better evidence.

## What each role can do

| Role | Available in current source | Planned |
|---|---|---|
| Operator | Validate, test, plan against fixtures, serve a catalog, and configure actions | Live remote planning, managed preview environments, and a separate `seikika verify` command |
| Package author | Test, share by Git ref, and install locked packages | An OCI registry mirror |
| User | Read data through the CLI, HTTP, or MCP. Preview, request, and verify configured actions. | A generated client, service identities, and a deprecation feed |
| Maintainer | Build, test, and score Seikika. Check the public docs as part of CI. | See the [project roadmap](https://github.com/seikika/seikika/blob/main/docs/plan.md) for later work. |

Check [Releases and upgrades](/start/versions/) for the version that matches your work.
