---
title: Use a portal
description: A developer or an agent reads owners, scores and evidence over the CLI, MCP or the API.
sidebar:
  order: 1
---

A user builds on a portal someone else runs. The manifest, the scorecards and the deploy belong to the [operator](/toolbox/operators/). You ask the catalog questions, and you build scripts, checks and agents on the answers.

The same reads are on the CLI, on MCP and on the HTTP API, and the same authorization applies to each. A field you are not allowed to see comes back as hidden, with the relation that would reveal it.

## Who owns this?

Ownership is a fact with evidence. In the reference portal, `repository:ledger` claims `group:payments` in its catalog file, and CODEOWNERS names a different team. The claim stays unverified, the ownership level is `none`, and the detail is visible to portal admins.

Run the reference portal from its fixtures:

```bash
toolbox serve examples/reference-portal --fixtures
toolbox owner repository:ledger --as alice
toolbox score repository:ledger --as alice
```

`--as` is the dev-server switch for acting as a user. A deployed portal expects a personal access token: `toolbox token create --user alice --database-url "$DATABASE_URL"`, then `TOOLBOX_TOKEN` or `--token` on the read. The token is stored as a hash in that portal's Postgres. Service identities, so a job has its own principal, are specified and not built. A CI job today uses a person's token.

`owner` answers who, and why. `explain` answers one rule: the evidence and the fix.

## Why is this below the bar?

`ownership` on `repository:checkout-web` stops at bronze: the default branch is unprotected, and that rule is silver. App hygiene is a separate scorecard on the same repository.

```bash
toolbox score repository:checkout-web --as bob
toolbox explain repository:checkout-web --rule ownership/branch-protected --as bob
toolbox scorecard ownership --level none --as alice
```

`score` is one entity, every scorecard, every rule. `scorecard` is one scorecard across entities, which is the work list. `explain --rule` takes `scorecard/rule`. On `ownership`, the level is public and the rule detail is for the owner. A caller outside that relation gets the level and a marker naming the relation, which is the same fact an app has to handle.

The levels in this repository's own standards are `none`, `bronze`, `silver` and `gold`. An entity's level is the highest one where every rule at that level and below passes. A skipped scorecard, such as an archived repository, is not a failure.

The read names and the short loop an agent follows are on [Read the catalog from an agent](/toolbox/agents/reads/).

## Fail a check when the level drops

A CI job can refuse a deploy when a scorecard is below bronze. Point it at the portal and read JSON:

```bash
TOOLBOX_TOKEN=… toolbox score repository:ledger -o json
```

The script owns the comparison. The token is a personal access token for the portal. Toolbox returns the level, the rule outcomes and, where the caller is allowed to see them, the evidence. Pin the read to a snapshot when the job must see the write it just made: responses carry a snapshot id, and a later read can pass `min_snapshot`.

`toolbox query` lists a kind with a CEL filter over `entity` and `scores`. `toolbox search` finds entities by name. `toolbox get` is one entity and the provenance of its fields.

## Build on the contract

| App | Surface | What works today |
|---|---|---|
| An agent in a harness | MCP at `/mcp`, or `toolbox mcp` over stdio | The catalog reads, with the caller's authorization |
| A script or a CI check | CLI `-o json`, or HTTP with a personal access token | The same reads. The script decides pass and fail |
| A dashboard | HTTP `/v1` | The same reads. A generated typed client is specified and not built |
| Something that changes a system | Actions | Listing actions works. Running one, with preview and a confirm token, is paused |

Two promises already hold for anything you build. Paths and problem codes under `/v1` stay stable within that version. Snapshot ids are monotonic per portal, and `min_snapshot` means "at least this snapshot". Portal-defined kinds and fields can change when an operator edits the manifest. `toolbox plan` is how that operator sees the diff, including removed fields and MCP tools, before the change ships. A deprecation feed for users is specified and not built.

Start from [Run Toolbox locally](/toolbox/start/run/) if you do not have a binary yet. The reference portal is the fixture catalog the commands above read.
