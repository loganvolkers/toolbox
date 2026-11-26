---
title: Build a portal
description: A platform team writes a portal in code, scores what it covers, and serves that catalog to people and agents.
sidebar:
  order: 1
---

An operator builds and runs a portal. The portal is the catalog of what the organisation runs, the standards those things are held to, and the reads an agent can make. The manifest lives in a repository. Seikika is the engine that validates it, scores it and serves it.

The operator SDLC is that loop: author the manifest, validate it, test the rules, plan a change, then deploy and watch it. Several of those steps run today. Preview environments, an image build and `seikika verify` are specified for a later milestone.

## What you produce

| Artefact | What it is |
|---|---|
| `seikika.yaml` | Kinds, fields, the map from connector records, and the scorecard rules. Thresholds live here |
| Fixtures and `tests/*.test.yaml` | The cases that pin what each rule means |
| Connectors | Optional. A portal names a shipped `builtin:` or a package someone else published. Writing a new connector is [Write a connector](/authors/connectors/) |
| A running service | `seikika serve`, with Postgres when the catalog has to survive a restart |

A change to a standard is a change to this repository's manifest. The plan prints who would drop a level before the change merges.

## Score a repository on every push

The smallest portal scores one checkout and needs no server. The numbered path is [Quickstart](/start/quickstart/). Copy the starter at `examples/repo-portal-starter` to `.seikika/seikika.yaml` and add the report action. The starter's `repo-hygiene` scorecard asks for a README, a CODEOWNERS file that starts with a catch-all team rule, and a LICENSE. Its `docs` scorecard checks that relative links in markdown resolve.

```bash
seikika report examples/repo-portal-starter
seikika test examples/repo-portal-starter
```

`report` reads the checkout, recomputes, and prints each scorecard with the evidence for rules that fail. With `--baseline .seikika/results.json`, a pull request fails when a rule starts failing or a failing rule's evidence grows. The output is then only the verdict: what got worse, with new evidence marked `+` (add `-o md` to see every scorecard). `--update` records an improvement so the committed file stays current. `--ci` also fails when that file is stale.

The first baseline is `seikika report .seikika --baseline .seikika/results.json --update`, or a manual run of the action with `mode: update`. After that, `mode: ci` is the pull-request check.

## Change a standard

[Change a portal](/operators/portal/) is the inner loop: `validate`, `test`, then `plan --local --base`. `validate` catches schema, reference and CEL errors. `test` runs the expectations over fixtures. `plan` recomputes twice over the same fixtures and prints the difference in entities, scores and access.

```bash
seikika validate path/to/portal
seikika test path/to/portal
seikika plan path/to/portal --local --base path/to/previous/seikika.yaml
```

`plan --remote`, which diffs a candidate manifest against the live catalog, is specified and not in the binary yet. Until then the plan is against fixtures.

## Serve it

```bash
seikika serve examples/reference-portal --fixtures
```

With no address set, that listens on `127.0.0.1:8080`. Set `PORT` and the same binary binds `0.0.0.0:$PORT`, which is the contract a container host injects; `--listen` or `SEIKIKA_LISTEN` still wins when you set one. The same process serves the HTTP API, MCP at `/mcp`, and the UI at `/`. `--fixtures` loads the portal's fixtures into memory; the data is gone when the process exits. Pass `--database-url` (or set `DATABASE_URL`) to use Postgres.

A portal that reads a real Git server instead of fixtures is `examples/acme-live`: one image, Postgres, and connector URLs for the GitHub API and a SCIM feed. `examples/acme-live/deploy/score.yaml` is the workload, and `examples/acme-live/deploy/chart` is the Helm chart.

`seikika push path/to/portal` sends a portal's fixtures to a server that is already running. People and agents then use the reads on [Use a portal](/users/).

## Where the loop stands

| Step | What you run | State |
|---|---|---|
| Author the manifest | `seikika.yaml`, or a package [included](/authors/) from someone else | Built. A TypeScript builder exists as a prototype |
| Validate | `seikika validate` | Built |
| Test the rules | `seikika test` | Built |
| Plan | `seikika plan --local --base` | Built for fixtures. A plan against live data is later |
| Serve | `seikika serve`, `seikika push` | Built |
| Preview, deploy, verify | One image, rollback by image, `seikika verify` | Specified. No image pipeline has been published, and `seikika verify` is not in the binary |

`seikika connector test` runs the conformance kit against a connector binary. A connector in this binary is `builtin:<name>`. A connector you run yourself is remote, or an executable `seikika install` pins by digest. Writing that connector for other portals is the [package author](/authors/) loop.
