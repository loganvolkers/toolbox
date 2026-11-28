---
title: Seikika and DX Fabric
description: Compare DX Fabric catalog and scorecards with Seikika portal and repository workflows.
sidebar:
  order: 8
relationship: complement
peer: DX Fabric
peer_url: https://docs.getdx.com/fabric/overview/
last_verified: 2026-11-26
claim_sources:
  - https://docs.getdx.com/fabric/overview/
  - https://getdx.com/
  - https://github.com/seikika/seikika/blob/main/docs/research.md
---

DX is an engineering intelligence platform. DX Fabric includes a catalog, SQL scorecards, and self-service workflows. Seikika provides a code-configured portal that can run standards checks over data from connectors and tools.

| Need | Where to look |
|---|---|
| Surveys or engineering productivity analytics | DX |
| A hosted catalog with SQL checks | DX Fabric |
| Local repository checks and a committed CI baseline | Seikika |
| A portal you configure, deploy, and operate | Seikika |

If you use both, decide which system owns each catalog field and standard. Evaluate a connector against the specific data you need to share. DX Fabric's SQL examples are particularly relevant when a rule spans several catalog entities.

See [DX Fabric](https://docs.getdx.com/fabric/overview/), [DX Scorecards](https://docs.getdx.com/scorecards/overview/), and [Seikika scorecards](/concepts/scorecards/). Seikika's [developer experience benchmarks](/concepts/dx-benchmarks/) are a research program run in this repository.
