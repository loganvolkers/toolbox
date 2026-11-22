# Software factory demo portal

A real Toolbox catalog + **Agent Governance** scorecard for the docs demo at
[`/toolbox/demo/port/`](../../site/src/content/docs/demo/port/). Entity facts live in
`catalog/*.yaml` (workspace `data:` records). The scorecard evaluates those entities with CEL —
same ratchet shape as Port’s Bronze→Gold agent governance surface.

| Kind | Source | Scored by |
|---|---|---|
| `FactoryLine` | `catalog/factory-lines.yaml` | Catalog only (no scorecard yet) |
| `HitlTask` | `catalog/hitl-tasks.yaml` | Catalog only (HITL queue) |
| `Agent` | `catalog/agents.yaml` | `agent-governance` (bronze → gold) |

**Agent Governance rules**

| Level | Rules |
|---|---|
| Bronze | owning team, purpose, data classification |
| Silver | approved MCP, approved tools, published skills, agent approved |
| Gold | monitoring configured, under budget, gold gate |

Release Notes Drafter reaches **silver** (monitoring not configured; gold gate open) — matching the Port demo shape.

## Commands (from the repository root)

```bash
toolbox validate examples/port-factory-demo
toolbox test examples/port-factory-demo
toolbox report examples/port-factory-demo
toolbox report examples/port-factory-demo --baseline examples/port-factory-demo/results.json
toolbox report examples/port-factory-demo -o web --out site/public/demo/port/report
```

Regenerate fixtures after editing catalog YAML:

```bash
toolbox connector records --portal examples/port-factory-demo --name workspace \
  > examples/port-factory-demo/fixtures/workspace.ndjson
```

**Box or tool:** this example is the **box** (kinds, scorecard, report). Catalog YAML is demo data,
not a live Jira/Slack connector.
