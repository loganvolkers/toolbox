---
title: Seikika vs Backstage
description: When to choose Seikika or Backstage for an internal developer portal.
sidebar:
  order: 2
relationship: replace
peer: Backstage
peer_url: https://backstage.io/
last_verified: 2026-11-23
claim_sources:
  - https://github.com/backstage/backstage
  - https://github.com/seikika/seikika/blob/main/docs/research.md
  - https://github.com/seikika/seikika/blob/main/experiments/backstage-compare/RESULTS.md
---

Both projects help an organization publish a software catalog, standards, and the surfaces people and agents use to work with them. [Backstage](https://backstage.io/) created that category as a scaffolded TypeScript application with a rich plugin ecosystem. Seikika is a config-as-code portal aimed at agents first, shipped as a versioned dependency rather than an app you own the scaffold of.

## Quick glance

| | Backstage | Seikika |
|---|---|---|
| License / deploy | Apache-2.0; you run the scaffolded app | Open-source; one container image + Postgres |
| Config locus | Code in the adopter's app + `app-config` | Portal manifest and packages in git |
| Catalog descriptors | `catalog-info.yaml` (Kubernetes-like envelope) | Same envelope for git-sourced entities; connectors supply facts |
| Agent surface | Actions registry over MCP; growing AI kinds | API, MCP, CLI, and UI over one catalog contract |
| Scorecards / ratchets | Via plugins (for example Tech Insights) and distributions | First-class scorecards, evidence, `report` ratchet |
| Who operates it | Platform team owns the TypeScript app and upgrades | Platform team operates the image and the portal repo |
| Extension model | In-process TypeScript plugins | Connectors, packages, standard tool slots (SARIF, and more) |

## What we share

- A software catalog next to the code, with entity refs and ownership.
- Docs-like-code as a first-class idea.
- The belief that portals should be extensible and useful to both humans and automation.
- Interest in MCP and agent-facing actions with clear read vs write intent.

## Where they differ

### Config-as-code vs owning the scaffold

Backstage adopters compose plugins inside an application they generate and upgrade. Seikika treats the portal as configuration and packages you validate, plan, and deploy — the binary upgrades like a dependency. That is the core product bet, not a slight against Backstage's ecosystem.

### Agent-first contracts

Seikika exposes the same catalog over API, MCP, CLI, and UI from the start. Backstage has added MCP and AI-oriented catalog kinds; both projects are moving, and primary docs are the place to verify current surfaces.

### Box for tools

Scorecards and lint findings in Seikika prefer carrying existing tools and standard outputs rather than re-implementing checks inside the portal process. See [the box and the tools](https://github.com/seikika/seikika/blob/main/docs/design/seikika-and-tools.md).

## Measured, beyond DX

We measured both products with scripts, no agent in the loop, on one VM: Seikika's release build with OpenFGA authorizing every read (each decision checked once per snapshot, then cached), against Backstage 1.57 from `@backstage/create-app` 0.9.4, built as its Dockerfile does and run with its production config. The comparative scorecards in this repository's own report score the result; see the **Compare** tab of [this repository, scored](/report-hosted/#compare). Round 3, at 10,000 components:

| Factor | Seikika | Backstage | Standing |
|---|---|---|---|
| Performance | get 56.5k req/s, owner list 3,790 req/s, ready 0.9 s, catalog readable 2.2 s | get 4.2k req/s, owner list 474 req/s, ready 3.3 s, catalog readable 26.6 s | ahead |
| Footprint | 89 MB idle, 166 MB loaded, 121 MB installed, 2 adopter files | 452 MB idle, 806 MB loaded, 710 MB installed, 44 adopter files | ahead |
| Security | owner-only fields; connectors are separate processes, but a same-user connector can still read OpenFGA's credentials | per-entity permissions; plugins share one process (its threat model) | ahead (2 rules), parity on 6 |
| Supply chain | 605 packages, 5 known vulnerabilities (0 high), 100% provenance | 2,791 packages, 34 known (16 high), 0% provenance | ahead |
| Change safety | offline validate and plan; 0 adopter lines per upgrade | 47 template lines per release; Knex migrations to roll back | ahead |
| Org fit | repo portals without a server; no federation yet | one backend per catalog; no federation | ahead |
| Agent surface | MCP by default, 6.5 KB tool list | MCP by default, 20.5 KB tool list | ahead |
| Extensibility | connectors in any language, no rebuild | TypeScript plugins, rebuild the app | ahead |
| Capabilities | catalog, ownership, graph, scorecards, permissions, MCP | all of those, plus TechDocs, search, scaffolder, API docs, Kubernetes, notifications, and a community of 2,826 contributors | **behind** |

Both products listen on every interface as shipped. Backstage wins on capabilities and community, and that loss is published, not waived. The method, every round (including a first round Seikika lost on filtered lists, and a harness defect that measured Backstage's lists on empty responses), the threats to validity and an independent adjudication are in [the results](https://github.com/seikika/seikika/blob/main/experiments/backstage-compare/RESULTS.md).

## Good for / not for

### Choose Backstage (or Roadie / RHDH) when

- You already run Backstage and have invested in plugins, TechDocs, and the scaffolder.
- Your platform team is staffed to own a TypeScript portal application and its upgrade cadence.
- You need a specific community plugin that has no Seikika equivalent yet.
- A hosted Backstage distribution (Roadie, Red Hat Developer Hub, Spotify Portal, and similar) matches how you buy and operate software.

### Choose Seikika when

- You want a greenfield portal configured only in code, with agents as a primary audience.
- You want scorecards and CI ratchets without standing up a separate insights plugin stack.
- You prefer a single deployable image plus Postgres over scaffolding and maintaining an app monorepo.

## Coexistence

Seikika is designed to **read** Backstage-shaped `catalog-info.yaml` so descriptors stay portable ([research.md](https://github.com/seikika/seikika/blob/main/docs/research.md), plan entity envelope). A full migration importer and a “Backstage plugin that speaks Seikika” are product work — **not built** as of this page. Running both during a transition is a reasonable path; ripping and replacing overnight is not required.

## What we learned

[Backstage](https://backstage.io/) defined the internal developer portal as a product category: a software catalog next to the code, docs-like-code, and a plugin ecosystem that proved platforms can grow in the open. Seikika keeps the catalog envelope so `catalog-info.yaml` stays portable, and builds the agent-first, config-as-code shape we wanted next.

More on [Acknowledgments](/compare/acknowledgments/).

## Last verified

**2026-11-23**, against [docs/research.md](https://github.com/seikika/seikika/blob/main/docs/research.md) (Backstage docs read from `backstage/backstage`) and [backstage.io](https://backstage.io/). Re-check upstream docs when citing plugin or MCP details; do not copy undated third-party staffing statistics.
