---
title: Seikika vs OpsLevel
description: When to choose Seikika or OpsLevel for catalog rubrics, scorecards, and campaigns.
sidebar:
  order: 5
relationship: replace
peer: OpsLevel
peer_url: https://www.opslevel.com/
last_verified: 2026-11-23
claim_sources:
  - https://github.com/seikika/seikika/blob/main/docs/research.md
---

Both projects help platform teams publish a service catalog, define quality rubrics, and run campaigns to raise the floor. [OpsLevel](https://www.opslevel.com/) does that as a commercial product with shared levels, checks, and time-boxed campaigns. Seikika does it with standards packages, scorecards, and `seikika report` baselines in an open-source portal.

## Quick glance

| | OpsLevel | Seikika |
|---|---|---|
| License / deploy | Commercial SaaS | Open-source; self-host image + Postgres |
| Quality model | Rubrics: levels × categories × checks | Scorecards with levels and rules |
| Campaigns | Time-boxed pushes; convertible with checks | Campaigns with recorded start and paydown |
| Component as code | `opslevel.yml` for components and ownership | Manifest + git descriptors / connectors |
| Push-based facts | Custom event checks (JSON to an endpoint) | Connector records and standard tool slots |
| Agent surface | OpsLevel MCP (as documented by OpsLevel) | API, MCP, CLI, UI |
| Who operates it | OpsLevel operates the control plane | Your team operates the portal |

## What we share

- Global, comparable levels so scorecards mean the same thing across teams.
- Campaigns (initiatives) as a first-class way to improve checks over time.
- Components and ownership described as code where possible.
- The need for facts that arrive from events and integrations, not only from git.

## Where they differ

### Product suite vs standards packages

OpsLevel optimizes for a hosted catalog and rubric experience. Seikika publishes reusable standards as packages other portals `include`, and scores this repository with the same machinery ([scorecards](/concepts/scorecards/)).

### Ratchet artifact

Seikika commits a baseline report and fails CI on regressions or staleness. Evaluate OpsLevel's campaign and check workflows on OpsLevel's docs when you need their product behavior; this page does not restate vendor UI detail.

## Good for / not for

### Choose OpsLevel when

- You want a commercial catalog + rubric + campaign product with vendor support.
- Your organization already runs OpsLevel checks and `opslevel.yml` as the source of truth.
- Hosted workflows matter more than self-hosting a config-as-code portal.

### Choose Seikika when

- You want open-source scorecards and campaigns reviewed in git.
- Package authors should publish standards other portals reuse.
- Agents and CI should share one catalog contract with a committed ratchet.

## Coexistence

No OpsLevel importer ships in Seikika today. Coexistence means picking an authoritative system per concern (ownership, a given check, campaign tracking) until a sync exists. Document that choice; do not assume silent reconciliation.

## What we learned

OpsLevel's rubric and campaign model underlined two needs Seikika keeps: shared levels for comparable scorecards, and time-bounded improvement work with a recorded starting point. Push-based facts (custom events) remain a connector problem worth solving without requiring teams to abandon git-reviewed standards.

More on [Acknowledgments](/compare/acknowledgments/).

## Last verified

**2026-11-23**, against [docs/research.md](https://github.com/seikika/seikika/blob/main/docs/research.md) (OpsLevel docs cited there for rubrics, campaigns, and custom event checks). Re-check [OpsLevel docs](https://docs.opslevel.com/) for product behavior; do not use undated competitive blog claims.
