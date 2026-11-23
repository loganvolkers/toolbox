---
title: Toolbox and DX Fabric
description: How Toolbox relates to DX and DX Fabric — adjacent engineering intelligence and context, not an IDP replacement claim.
sidebar:
  order: 8
relationship: complement
peer: DX Fabric
peer_url: https://docs.getdx.com/fabric/overview/
last_verified: 2026-11-23
claim_sources:
  - https://docs.getdx.com/fabric/overview/
  - https://getdx.com/
  - https://github.com/loganvolkers/toolbox/blob/main/docs/research.md
---

This is **not a replacement**. [DX](https://getdx.com/) (getdx.com; Atlassian announced an acquisition in 2025) is a developer-intelligence platform: experience measurement, productivity analytics, and a large connector surface. [DX Fabric](https://docs.getdx.com/fabric/overview/) is the part of that platform that offers a software catalog, SQL-powered scorecards, and self-service workflows as a **context layer** for teams and AI agents — including alongside portals such as Backstage. Toolbox is an open-source, config-as-code portal and tool box. Overlap on catalog and scorecards does not make either product a drop-in substitute for the other.

(Toolbox's own [DX Benchmarks](/toolbox/concepts/dx-benchmarks/) are unrelated naming: pre-registered agent A/Bs in this repository, not the DX product.)

## Quick glance

| | DX / DX Fabric | Toolbox |
|---|---|---|
| Category | Commercial developer intelligence; Fabric = catalog + SQL scorecards + self-service context | Open-source agent-first IDP / portal framework |
| Deploy | Vendor-hosted SaaS (Atlassian family) | Self-host: one image + Postgres |
| Config locus | Product UI, CLI/API; imports from Backstage or Compass | Manifest and packages in git |
| Scorecards | SQL against Fabric catalog / data-cloud tables | CEL over connector facts; `report --baseline` ratchets |
| Measurement / DORA / surveys | Core DX product strength | Not the job — ingest facts; do not compete on analytics |
| Agent surface | DX MCP and product context for agents | API, MCP, CLI, UI on the portal |
| Who operates it | DX / Atlassian operates the control plane | Your team operates the portal |

## What each owns

- **DX** owns developer-experience measurement, engineering productivity analytics, and the connector lake that feeds them.
- **Fabric** owns a live catalog, SQL scorecards, and self-service actions as context for humans and agents — DX positions this as foundation beside a portal UI, not as “rip out Backstage.”
- **Toolbox** owns an OSS portal you configure in code: packages, connectors, standards, ratchets, and agent-readable reads. It is a [box for tools](https://github.com/loganvolkers/toolbox/blob/main/docs/design/toolbox-and-tools.md), not a measurement suite.

## When each fits

### Choose DX / Fabric when

- You need surveys, DORA-style metrics, or org-wide developer-experience analytics as a product.
- You want a vendor-operated context layer (catalog + SQL scorecards + self-service) that plugs into an existing portal such as Backstage.
- Your organization already standardizes on DX (or the Atlassian suite that includes it) for intelligence and AI-enablement workflows.

### Choose Toolbox when

- You want an open-source, self-hosted portal whose config, scorecards, and baselines live in git.
- You need CI ratchets and package kits over connector facts agents read via MCP/CLI — without adopting a SaaS intelligence control plane.
- You are assembling the portal itself, not only measuring developers against one.

### Use both when

Teams can keep DX for measurement and Fabric context while running Toolbox for repo-local / fleet ratchets and config-as-code portal ops. Be explicit which system is authoritative for which rule; dual catalogs without a sync story create two sources of truth.

## How Toolbox relates (without wrapping DX)

- **Ingest, do not replace.** Metrics and DORA-style analytics stay DX's territory. Toolbox should eventually carry such facts as connector inputs when a team wants them in portal scorecards — not reimplement the analytics product ([research.md](https://github.com/loganvolkers/toolbox/blob/main/docs/research.md)).
- **Different expression default.** Fabric scorecards use SQL over catalog tables (strong for aggregate, cross-entity checks). Toolbox uses CEL first; SQL-shaped aggregate rules are a stated future lesson from DX, not a claim of parity today.
- **No Fabric importer.** There is no DX → Toolbox migration path shipping in v1. Coexistence is the honest posture.

## When to use DX alone

- You only need developer-experience or productivity measurement.
- Fabric's catalog and SQL scorecards already satisfy context needs inside DX / Backstage, and you are not looking for a self-hosted OSS portal.
- You are evaluating Atlassian Compass + DX together inside that vendor's stack.

## What we learned

DX showed that **SQL is the most agent-familiar language for aggregate, cross-entity checks** (for example “percent of services on the latest base image”), and that **measurement products and portals are different jobs**. Fabric's catalog + scorecards + self-service packaging is worth studying as a context layer that can sit beside a portal rather than claiming to replace every IDP. Toolbox credits that separation: compete on the OSS portal and ratchet loop; ingest intelligence facts; leave DX's analytics core alone.

We are not adding DX to [Acknowledgments](/toolbox/compare/acknowledgments/) yet — the lesson is recorded here and in research until a stronger product debt is owed.

## Last verified

**2026-11-23**, against [docs/research.md](https://github.com/loganvolkers/toolbox/blob/main/docs/research.md) (DX / Fabric section), [Fabric overview](https://docs.getdx.com/fabric/overview/), and [getdx.com](https://getdx.com/). Re-check those primary sources when describing product features; omit undated adoption or pricing claims. Atlassian acquisition status may change product packaging — update the fact rows when official docs move.
