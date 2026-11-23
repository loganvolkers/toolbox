---
title: Toolbox vs Backstage
description: When to choose Toolbox or Backstage for an internal developer portal.
sidebar:
  order: 2
relationship: replace
peer: Backstage
peer_url: https://backstage.io/
last_verified: 2026-11-23
claim_sources:
  - https://github.com/backstage/backstage
  - https://github.com/loganvolkers/toolbox/blob/main/docs/research.md
---

Both projects help an organization publish a software catalog, standards, and the surfaces people and agents use to work with them. [Backstage](https://backstage.io/) created that category as a scaffolded TypeScript application with a rich plugin ecosystem. Toolbox is a config-as-code portal aimed at agents first, shipped as a versioned dependency rather than an app you own the scaffold of.

## Quick glance

| | Backstage | Toolbox |
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

Backstage adopters compose plugins inside an application they generate and upgrade. Toolbox treats the portal as configuration and packages you validate, plan, and deploy — the binary upgrades like a dependency. That is the core product bet, not a slight against Backstage's ecosystem.

### Agent-first contracts

Toolbox exposes the same catalog over API, MCP, CLI, and UI from the start. Backstage has added MCP and AI-oriented catalog kinds; both projects are moving, and primary docs are the place to verify current surfaces.

### Box for tools

Scorecards and lint findings in Toolbox prefer carrying existing tools and standard outputs rather than re-implementing checks inside the portal process. See [the box and the tools](https://github.com/loganvolkers/toolbox/blob/main/docs/design/toolbox-and-tools.md).

## Good for / not for

### Choose Backstage (or Roadie / RHDH) when

- You already run Backstage and have invested in plugins, TechDocs, and the scaffolder.
- Your platform team is staffed to own a TypeScript portal application and its upgrade cadence.
- You need a specific community plugin that has no Toolbox equivalent yet.
- A hosted Backstage distribution (Roadie, Red Hat Developer Hub, Spotify Portal, and similar) matches how you buy and operate software.

### Choose Toolbox when

- You want a greenfield portal configured only in code, with agents as a primary audience.
- You want scorecards and CI ratchets without standing up a separate insights plugin stack.
- You prefer a single deployable image plus Postgres over scaffolding and maintaining an app monorepo.

## Coexistence

Toolbox is designed to **read** Backstage-shaped `catalog-info.yaml` so descriptors stay portable ([research.md](https://github.com/loganvolkers/toolbox/blob/main/docs/research.md), plan entity envelope). A full migration importer and a “Backstage plugin that speaks Toolbox” are product work — **not built** as of this page. Running both during a transition is a reasonable path; ripping and replacing overnight is not required.

## What we learned

[Backstage](https://backstage.io/) defined the internal developer portal as a product category: a software catalog next to the code, docs-like-code, and a plugin ecosystem that proved platforms can grow in the open. Toolbox keeps the catalog envelope so `catalog-info.yaml` stays portable, and builds the agent-first, config-as-code shape we wanted next.

More on [Acknowledgments](/toolbox/compare/acknowledgments/).

## Last verified

**2026-11-23**, against [docs/research.md](https://github.com/loganvolkers/toolbox/blob/main/docs/research.md) (Backstage docs read from `backstage/backstage`) and [backstage.io](https://backstage.io/). Re-check upstream docs when citing plugin or MCP details; do not copy undated third-party staffing statistics.
