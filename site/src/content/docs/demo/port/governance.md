---
title: Governance and scorecards
description: Platform-engineer governance cards and Agent Governance Bronze→Gold scorecard, evaluated by Toolbox.
---

### Real scorecard feed

The interactive app’s governance view loads [`report.json`](/toolbox/demo/port/report/report.json) from `toolbox report examples/port-factory-demo`. Open the full [`<toolbox-report>` page](/toolbox/demo/port/report/) for catalog + scorecards with evidence.

### Agent: Release Notes Drafter

External AI Agent · evaluated level **Silver** (Bronze → Silver → Gold)

| Tier | Typical rules | Notes |
|---|---|---|
| Bronze | Owning team, purpose, data classification | Pass |
| Silver | Approved MCP, tools, published skills, agent approved | Pass |
| Gold | Monitoring configured, under budget, gold gate | Short of Gold (monitoring not configured) |

Portal source: [`examples/port-factory-demo/`](https://github.com/loganvolkers/toolbox/tree/cursor/port-factory-catalog-bc64/examples/port-factory-demo). Regenerate with:

```bash
toolbox report examples/port-factory-demo -o web --out site/public/demo/port/report
```

[Interactive governance](/toolbox/demo/port/app/#governance) · [Demo index](/toolbox/demo/port/)
