---
title: CLI reference
description: Primary toolbox commands and the MCP tools that share the same authorization path.
sidebar:
  order: 4
---

The `toolbox` binary is the operator and user surface. Catalog **reads** are the same over the CLI, MCP, and HTTP: one authorization path, one snapshot model. This page indexes what ships today. It is not a generated OpenAPI catalog — for request shapes, run `toolbox <command> --help` or attach to MCP and inspect the tools.

Install: [Quickstart](/toolbox/start/quickstart/) or [Install a release](/toolbox/start/install/).

## Portal loop

| Command | What it does |
|---|---|
| `toolbox validate [path]` | Manifest schema, references, CEL, fixtures |
| `toolbox test [path]` | Expectation files over fixtures |
| `toolbox plan [path] --local --base <manifest>` | Diff two manifests over the same fixtures |
| `toolbox report [path]` | Score a repo portal; `--baseline` / `--update` / `--ci` for the ratchet |
| `toolbox serve [path]` | HTTP API, UI at `/`, MCP at `/mcp` (`--fixtures` or `--database-url`) |
| `toolbox push [path]` | Push fixtures into a running server |
| `toolbox derive [path]` | Recompute from fixtures; optional `--out` for reference files |
| `toolbox doctor [path]` | Connectors resolve; tools at pinned versions (`--mise` prints pins) |
| `toolbox install [path]` | Pin/fetch git packages and executable connectors (`--frozen` for CI) |
| `toolbox vendor <spec> --portal <path>` | Copy a package into the portal for offline use |
| `toolbox describe portal <paths…>` | Emit portal/scorecard facts as records |

`plan --remote` and `toolbox verify` are specified and not in the binary yet.

## Catalog reads

Point these at a running portal (`--url`, `TOOLBOX_URL`) with a token (`--token` / `TOOLBOX_TOKEN`) or, on a dev server, `--as <user>`.

| CLI | MCP tool | Answers |
|---|---|---|
| `get <id>` | `get` | One entity and field provenance |
| `query --kind …` | `query` | Entities of a kind (optional CEL `--filter`) |
| `search <text>` | `search` | Entities by name or title |
| `graph <id>` | `graph` | Neighbours (`--depth` 1–3) |
| `owner <id>` | `owner_of` | Ownership and CODEOWNERS evidence |
| `explain <id>` | `explain` | Field (`--field`) or rule (`--rule scorecard/rule`) |
| `score <id>` | `scorecard_status` | Levels and rule outcomes for one entity |
| `scorecard [name]` | `scorecard_report` | One scorecard across entities (a work list) |
| `campaign [name]` | `campaign_report` | Campaign progress (`--portal` for repo portals) |
| `refresh <id>` | `refresh_entity` | Ask connectors to re-read; wait for snapshot |
| `actions <id>` | `list_actions` | Actions available on an entity (running them is paused) |

Output: `-o md` (default), `json`, or `yaml` where supported. Snapshot ids on responses support `min_snapshot` for “at least this snapshot” reads — see [Use a portal](/toolbox/users/).

## MCP

```bash
toolbox mcp --url http://127.0.0.1:8080
# or attach an MCP client to http://127.0.0.1:8080/mcp on a running serve
```

Agent loop: [Read the catalog from an agent](/toolbox/agents/reads/).

## Packages, connectors, tools

| Command | What it does |
|---|---|
| `toolbox connector test --name … --portal … -- <program>` | Conformance kit against a connector binary |
| `toolbox connector drive --kinds … -- <program>` | One resync; print records (no portal required) |
| `toolbox connector records --name … --portal …` | What a portal connector emitted |
| `toolbox tools install` / `toolbox tools config` | Install or print pinned tool installer input |
| `toolbox assess brief` / `check` / `submit` | Rubric briefs and assessment files (Toolbox calls no model) |

Authoring a connector: [Write a connector](/toolbox/authors/connectors/).

## Tokens and meta

| Command | What it does |
|---|---|
| `toolbox token create` | Personal access token for a portal (hashed in Postgres) |
| `toolbox describe` | CLI / built-in MCP contract snapshots |

Service identities for jobs are specified and not built; CI today uses a person’s token or a repo-portal `report` with no server.
