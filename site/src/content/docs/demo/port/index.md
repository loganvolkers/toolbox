---
title: Software factory demo
description: A Port-inspired AI Software Factory demo rebuilt on Toolbox concepts, hosted in this docs site with honest gaps called out.
---

This is a **Toolbox** rebuild of the public [Port demo](https://demo.getport.io/) (no login). It is not affiliated with Port. Factory lines, HITL tasks, and agents are real catalog entities in [`examples/port-factory-demo/`](https://github.com/loganvolkers/toolbox/tree/cursor/port-factory-catalog-bc64/examples/port-factory-demo); Agent Governance is a real Bronze→Gold scorecard. Home KPI numbers and some persona cards remain fixtures — see [gaps](/toolbox/demo/port/gaps/).

## Open the interactive demo

[Launch the factory demo UI](/toolbox/demo/port/app/) — home KPIs, factory lines catalog, entity detail, human-in-the-loop, governance, and agent scorecards.

## Pages in this section

- [Factory lines catalog](/toolbox/demo/port/catalog/) — tabular catalog of delivery lines
- [Feature Delivery Line](/toolbox/demo/port/entity/) — entity overview + stages
- [My work (HITL)](/toolbox/demo/port/my-work/) — tasks waiting on a person
- [Governance & scorecards](/toolbox/demo/port/governance/) — agent risk + Agent Governance ratchet
- [Honest gaps](/toolbox/demo/port/gaps/) — what Port has that this demo does not fake

## Research

Project research and live screenshots live in Context: Port demo research (store `docs/port-demo-research.md`).

## How it maps to Toolbox

| Port surface | Here |
|---|---|
| Factory Lines catalog | `FactoryLine` entities + table UI; IDs in [report](/toolbox/demo/port/report/) |
| HITL / My work | `HitlTask` entities |
| Agent Governance (Bronze→Gold) | `agent-governance` scorecard via `toolbox report` |
| Home / persona dashboards | Partial: KPI strip still fixture numbers |
| Self-service actions | Documented gap (M5 paused) |
| Live SaaS integrations | Out of scope for docs demo |
