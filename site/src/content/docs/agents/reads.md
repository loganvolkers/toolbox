---
title: Read the catalog from an agent
description: The same catalog reads are available over MCP, the CLI and the API.
sidebar:
  order: 1
---

The reads are the same on each surface. The person or agent making the call is a [user of a portal](/toolbox/users/). Command names, flags, and MCP tool names are on the [CLI reference](/toolbox/start/cli/) (built from the live CLI at site build time).

| Read | What it answers |
|---|---|
| `get` | One entity, by id |
| `query` | Entities of a kind, with a filter |
| `score` | One entity's level, and which rules failed |
| `explain` | The evidence and the fix for a rule |
| `search` | Text across the catalog |
| `owner` | Who owns an entity and why |
| `graph` | Neighbours of an entity |
| `scorecard` | One scorecard across entities |
| `campaign` | Progress on a campaign |

Over MCP those reads are tools. Over the CLI they are subcommands of `toolbox`. Over HTTP they are the portal's API under `/v1`. Authorization is shared: a field you cannot see comes back hidden.

## Connect

Serve a portal, then attach:

```bash
toolbox serve examples/reference-portal --fixtures
# MCP HTTP: http://127.0.0.1:8080/mcp
# or stdio bridge:
toolbox mcp --url http://127.0.0.1:8080 --as alice
```

On a deployed portal, pass a personal access token (`TOOLBOX_TOKEN` / `--token`) instead of `--as`. Service identities for jobs are specified and not built.

## A short loop

1. `score` the entity you are about to change.
2. Read the failing rule's evidence (`explain --rule scorecard/rule`).
3. Change the file the evidence names.
4. Refresh that entity and `score` it again.

The scorecard for this site is the same loop. A page with a broken link fails `links-resolve`, and the evidence is the link target.

Nouns: [Glossary](/toolbox/concepts/glossary/). First local score without a server: [Quickstart](/toolbox/start/quickstart/). Build from source: [Run Toolbox locally](/toolbox/start/run/).
