---
title: Read the catalog from an agent
description: The same catalog reads are available over MCP, the CLI and the API.
---

The reads are the same on each surface. The person or agent making the call is a [user of a portal](/toolbox/users/).

| Read | What it answers |
|---|---|
| `get` | One entity, by id |
| `query` | Entities of a kind, with a filter |
| `score` | One entity's level, and which rules failed |
| `explain` | The evidence and the fix for a rule |
| `search` | Text across the catalog |

Over MCP those reads are tools. Over the CLI they are subcommands of `toolbox`. Over HTTP they are the portal's API.

## A short loop

1. `score` the entity you are about to change.
2. Read the failing rule's evidence.
3. Change the file the evidence names.
4. Refresh that entity and `score` it again.

The scorecard for this site is the same loop. A page with a broken link fails `links-resolve`, and the evidence is the link target.

Start from [Run Toolbox locally](/toolbox/start/run/) if you do not have a binary yet.
