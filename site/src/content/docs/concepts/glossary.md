---
title: Glossary
description: Short definitions of terms used on this site. Portal nouns first, then scorecards and the box.
sidebar:
  order: 5
---

Product-specific words. Read this before answering questions about Seikika from these pages. The maintainer design glossary in the repository is longer; this list is what public docs assume.

## Portal and people

| Term | Meaning |
|---|---|
| **Portal** | One organisation’s Seikika deployment: kinds, connectors, scorecards, and the service that serves them |
| **Repo portal** | A portal run by `seikika report` in one repository, without a server |
| **Manifest** | The portal’s config (`seikika.yaml` and includes): kinds, mappings, rules, thresholds |
| **Operator** | Builds and runs a portal ([Build a portal](/operators/)) |
| **Package author** | Publishes scorecards, connectors, or kits others include ([Publish a standard](/authors/)) |
| **User** | Reads a running portal over CLI, MCP, or HTTP ([Use a portal](/users/)) |
| **Maintainer** | Changes Seikika itself ([Build Seikika](/maintainers/)) |
| **SDLC layers** | Those four loops. Name the layer; “the SDLC” alone is ambiguous ([Four SDLC layers](/concepts/layers/)) |

## Catalog

| Term | Meaning |
|---|---|
| **Entity** | One catalog object with a kind, id, and fields |
| **Kind** | An entity type declared in the manifest (for example Repository, Doc, Scorecard) |
| **Connector** | A program that reports facts from a system of record; the manifest maps records onto entities |
| **Package** | Versioned fragments (kinds, scorecards, fixtures, tests) another portal `include`s |
| **Snapshot** | A consistent catalog read; responses carry a snapshot id for `min_snapshot` |

## Scorecards and ratchets

| Term | Meaning |
|---|---|
| **Scorecard** | A ladder of rules on one kind; level = highest bar where every rule at that level and below passes |
| **Evidence** | The items a failing rule names (what to fix) |
| **Unknown** | A rule that could not run (missing tool or skipped connector). Not a pass; not a ratchet regression |
| **Ratchet / baseline** | A committed results file; a newly failing rule or growing evidence is a regression ([Scorecards](/concepts/scorecards/)) |
| **Report action** | `actions/report`: GitHub Action that runs `seikika report` with the ratchet in one step |

## The box and the tools

| Term | Meaning |
|---|---|
| **Box** | What Seikika is for: pin, run, remember, and report tools together — not a wrapper that replaces them |
| **Tool** | A program with its own CLI and config that still works taken out of the box (ruff, zizmor, clippy, …) |
| **Slot** | A standard output shape the box reads without per-tool code (SARIF, NDJSON, JUnit) |
| **DX Benchmark** | A pre-registered agent A/B with a published bar and losses ([DX Benchmarks](/concepts/dx-benchmarks/)). Unrelated to the DX / DX Fabric product |

## See also

- [The catalog](/concepts/catalog/)
- [Seikika vs others](/compare/)
- Maintainer glossary: [`docs/glossary.md`](https://github.com/seikika/seikika/blob/main/docs/glossary.md)
