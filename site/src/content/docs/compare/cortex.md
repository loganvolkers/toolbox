---
title: Toolbox vs Cortex
description: When to choose Toolbox or Cortex for scorecards and engineering intelligence.
sidebar:
  order: 4
relationship: replace
peer: Cortex
peer_url: https://www.cortex.io/
last_verified: 2026-11-23
claim_sources:
  - https://github.com/loganvolkers/toolbox/blob/main/docs/research.md
---

Both projects help organizations define what “good” looks like for services and drive improvement. [Cortex](https://www.cortex.io/) popularized scorecards inside commercial IDP / engineering-intelligence products, with CQL rules and optional scorecards-as-code. Toolbox puts scorecards, evidence, and Betterer-style ratchets in an open-source portal you configure in git.

## Quick glance

| | Cortex | Toolbox |
|---|---|---|
| License / deploy | Commercial SaaS | Open-source; self-host image + Postgres |
| Scorecard language | CQL (and product UI) | CEL over connector facts |
| Scorecards as code | YAML under `.cortex/`; UI lock when GitOps is on | Manifest and packages in git always |
| Catalog entities | `cortex.yaml` and product catalog | Manifest kinds + connector records |
| Ratchet / baseline | Product workflows and initiatives | `toolbox report --baseline` (committed results) |
| Agent surface | MCP (reads + CQL); Context Graph positioning | API, MCP, CLI, UI |
| Who operates it | Cortex operates the control plane | Your team operates the portal |

## What we share

- Progressive levels: an entity earns a level when the rules for that level pass.
- Interest in expressing standards as code, not only as dashboard clicks.
- An agent-readable view of ownership, links, and policy-shaped facts.
- The idea that scorecards are how a platform talks about quality.

## Where they differ

### Scorecards in a SaaS vs portal-as-artifact

Cortex's strength is a productized scorecard and intelligence experience. Toolbox's strength is a portal and report that live as artifacts in your repositories — the same loop as the code they score.

### Expression language

CQL is powerful inside Cortex. Toolbox prefers CEL so agents and humans share a common expression language already used elsewhere in cloud-native tooling. That is a design choice, not a claim that CQL is inadequate.

### Ratchets

Toolbox's baseline file records levels and evidence so CI can say better / same / worse. That design explicitly credits Betterer ([ratchets](/toolbox/compare/betterer/)). Compare product initiative features against that mechanism on their own docs when you evaluate Cortex.

## Good for / not for

### Choose Cortex when

- You want a commercial engineering-intelligence suite with scorecards as a product feature.
- Your organization already standardizes on Cortex's catalog and CQL.
- You need the vendor's integrations and UI more than a self-hosted config-as-code portal.

### Choose Toolbox when

- You want open-source scorecards tied to git-reviewed portal config.
- You need CI ratchets and campaigns over the same facts agents read via MCP/CLI.
- You are assembling a portal from packages and connectors rather than adopting a SaaS IDP.

## Coexistence

No Cortex → Toolbox importer ships today. Teams can keep Cortex for intelligence product needs while using Toolbox for repo-local ratchets, or the reverse, as long as they are honest about two sources of truth. Dual-running without a sync story needs clear ownership of which system is authoritative for which rule.

## What we learned

Cortex showed that scorecards belong in the IDP conversation, not only in scattered spreadsheets — including progressive levels and taking GitOps seriously enough to lock UI edits when code is the source. Toolbox carries that product lesson into config-as-code scorecards and [Acknowledgments](/toolbox/compare/acknowledgments/) for the wider peer set.

## Last verified

**2026-11-23**, against [docs/research.md](https://github.com/loganvolkers/toolbox/blob/main/docs/research.md) (Cortex public docs cited there for CQL, scorecards-as-code, and MCP). Re-check [Cortex docs](https://docs.cortex.io/) when describing product features; omit undated adoption statistics.
