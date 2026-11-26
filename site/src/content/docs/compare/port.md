---
title: Seikika vs Port
description: When to choose Seikika or Port for a developer portal and integration catalog.
sidebar:
  order: 3
relationship: replace
peer: Port
peer_url: https://www.port.io/
last_verified: 2026-11-23
claim_sources:
  - https://github.com/port-labs/ocean
  - https://github.com/seikika/seikika/blob/main/docs/research.md
  - https://github.com/seikika/seikika/blob/main/docs/ocean.md
---

Both projects give platform teams a catalog of services, scorecards, and ways to drive change. [Port](https://www.port.io/) is a mature commercial SaaS portal with blueprints, rich integrations, and productized AI surfaces. Seikika is open-source infrastructure: portal configuration in git, one self-hosted image, and no hosted no-code suite.

## Quick glance

| | Port | Seikika |
|---|---|---|
| License / deploy | Commercial SaaS (Ocean OSS for integrations) | Open-source; self-host image + Postgres |
| Config locus | Portal config in Port's database; Terraform/Pulumi as clients | Manifest and packages in git only |
| Data model | User-defined blueprints, mirror/calculated properties | Kinds and fields in the portal manifest |
| Integrations | Ocean (Python) and a large catalog of syncs | First-party connectors; packages carry tools |
| Scorecards | Per-blueprint rules and levels in product | CEL rules, levels, baselines, campaigns |
| Agent surface | Context Lake / remote MCP and product AI features | API, MCP, CLI, UI over versioned contracts |
| Who operates it | Port operates the control plane | Your team operates the portal deployment |

## What we share

- A desire for typed catalog entities, ownership, and relations.
- Scorecards with levels teams can work toward.
- Integrations that pull messy API data into a catalog.
- An agent-facing interface over portal context.

## Where they differ

### SaaS product vs config-as-code OSS

Port optimizes for a hosted control plane and UI-driven portal building. Seikika keeps configuration in the repository so `validate` → `test` → `plan` → deploy stays a normal engineering loop. Terraform providers for Port exist; they still talk to DB-held config, so drift is possible ([research.md](https://github.com/seikika/seikika/blob/main/docs/research.md)).

### Ocean lessons, not Ocean lock-in

We studied [Port Ocean](https://github.com/port-labs/ocean) for connector and mapping ideas ([ocean.md](https://github.com/seikika/seikika/blob/main/docs/ocean.md)). Seikika does not adopt Ocean as its connector framework: Ocean is coupled to Port's API and mapping storage. The lesson is the contract shape, not the Python runtime.

### Not a no-code suite

Seikika does not claim Port's product depth in UI builders, approval workflows, or the breadth of commercial integrations. Self-service actions in Seikika remain paused pending design review; do not treat them as parity.

## Good for / not for

### Choose Port when

- You want a hosted portal with a polished UI and a long list of managed integrations.
- Non-engineers need to edit portal structure in a product UI without living in git.
- You are buying a commercial platform and support relationship.

### Choose Seikika when

- You need an open-source, self-hosted portal whose entire config is reviewable in pull requests.
- Agents and CI should consume stable API/MCP/CLI contracts from day one.
- You want scorecard ratchets and tool kits in the same repo loop as the catalog.

## Coexistence

There is no Seikika “import from Port” product path yet. Honest coexistence today means: keep Port where the SaaS fit is right, or export/sync facts through whatever APIs and git artifacts you already own. A migration importer is **not built**.

## What we learned

[Port Ocean](https://github.com/port-labs/ocean) showed how to sync messy third-party APIs into a catalog with a small integration contract and declarative mappings. We study that connector thinking without adopting a database-held portal config: Seikika keeps configuration in code and carries tools rather than wrapping them.

Port's blueprint expressiveness (mirror, calculated, and aggregated properties) is worth matching *in code* over time; that is admiration of the model, not a claim of feature parity today.

More on [Acknowledgments](/compare/acknowledgments/).

## Last verified

**2026-11-23**, against [docs/research.md](https://github.com/seikika/seikika/blob/main/docs/research.md) (Terraform provider schemas and Ocean sources) and [docs/ocean.md](https://github.com/seikika/seikika/blob/main/docs/ocean.md). Vendor marketing pages were not used as load-bearing evidence. Re-check Ocean and Port docs when describing integrations or MCP tools.
