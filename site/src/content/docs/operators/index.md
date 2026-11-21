---
title: Build a portal
description: A platform team writes a portal in code, scores what it covers, and serves that catalog to people and agents.
sidebar:
  order: 1
---

An operator builds and runs a portal. The portal is the catalog of what the organisation runs, the standards those things are held to, and the reads an agent can make. The manifest lives in a repository. Toolbox is the engine that validates it, scores it and serves it.

The operator SDLC is that loop: author the manifest, validate it, test the rules, plan a change, then deploy and watch it. Several of those steps run today. Preview environments, an image build and `toolbox verify` are specified for a later milestone.

## What you produce

| Artefact | What it is |
|---|---|
| `toolbox.yaml` | Kinds, fields, the map from connector records, and the scorecard rules. Thresholds live here |
| Fixtures and `tests/*.test.yaml` | The cases that pin what each rule means |
| Connectors | The programs that read GitHub, a checkout, a directory, or a system you connect yourself |
| A running service | `toolbox serve`, with Postgres when the catalog has to survive a restart |

A change to a standard is a change to this repository's manifest. The plan prints who would drop a level before the change merges.

## Score a repository on every push

The smallest portal scores one checkout and needs no server. Copy the starter at `examples/repo-portal-starter` to `.toolbox/toolbox.yaml` and add the report action. The starter's `repo-hygiene` scorecard asks for a README, a CODEOWNERS file that starts with a catch-all team rule, and a LICENSE. Its `docs` scorecard checks that relative links in markdown resolve.

```bash
toolbox report examples/repo-portal-starter
toolbox test examples/repo-portal-starter
```

`report` reads the checkout, recomputes, and prints each scorecard with the evidence for rules that fail. With `--baseline .toolbox/results.json`, a pull request fails when a rule starts failing or a failing rule's evidence grows. `--update` records an improvement so the committed file stays current. `--ci` also fails when that file is stale.

The first baseline is `toolbox report .toolbox --baseline .toolbox/results.json --update`, or a manual run of the action with `mode: update`. After that, `mode: ci` is the pull-request check.

## Change a standard

[Change a portal](/toolbox/operators/portal/) is the inner loop: `validate`, `test`, then `plan --local --base`. `validate` catches schema, reference and CEL errors. `test` runs the expectations over fixtures. `plan` recomputes twice over the same fixtures and prints the difference in entities, scores and access.

```bash
toolbox validate path/to/portal
toolbox test path/to/portal
toolbox plan path/to/portal --local --base path/to/previous/toolbox.yaml
```

`plan --remote`, which diffs a candidate manifest against the live catalog, is specified and not in the binary yet. Until then the plan is against fixtures.

## Serve it

```bash
toolbox serve examples/reference-portal --fixtures
```

That listens on `127.0.0.1:8080`. The same process serves the HTTP API, MCP at `/mcp`, and the UI at `/`. `--fixtures` loads the portal's fixtures into memory; the data is gone when the process exits. Pass `--database-url` (or set `DATABASE_URL`) to use Postgres.

`toolbox push path/to/portal` sends a portal's fixtures to a server that is already running. People and agents then use the reads on [Use a portal](/toolbox/users/).

## Where the loop stands

| Step | What you run | State |
|---|---|---|
| Author the manifest | `toolbox.yaml`, or a package [included](/toolbox/authors/) from someone else | Built. A TypeScript builder exists as a prototype |
| Validate | `toolbox validate` | Built |
| Test the rules | `toolbox test` | Built |
| Plan | `toolbox plan --local --base` | Built for fixtures. A plan against live data is later |
| Serve | `toolbox serve`, `toolbox push` | Built |
| Preview, deploy, verify | One image, rollback by image, `toolbox verify` | Specified. No image pipeline has been published, and `toolbox verify` is not in the binary |

`toolbox connector test` runs the conformance kit against a connector binary. A connector in this binary is `builtin:<name>`. A connector you run yourself is remote, or an executable `toolbox install` pins by digest. Writing that connector for other portals is the [package author](/toolbox/authors/) loop.
