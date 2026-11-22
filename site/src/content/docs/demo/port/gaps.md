---
title: Gaps versus Port
description: Honest list of Port demo capabilities this Toolbox docs rebuild does not fake.
---

| Port capability | This demo | Why |
|---|---|---|
| Live self-service actions | Not executed | Toolbox M5 self-service actions remain paused |
| Widget builder / private pages | Not rebuilt | Docs demo uses fixed MDX + one static app |
| Jira / PagerDuty / Slack / GitHub live data | Catalog YAML only | Docs site has no SaaS credentials; entities are declared under `examples/port-factory-demo/catalog/` |
| Classic software catalog / API dashboard | Partial / pending | Deep links still incomplete in research; not faked here |
| Federated multi-portal import | Not wired | Operator federation is separate work |
| Persona “view as” admin banner | Omitted | Marketing/demo chrome, not a product requirement |
| Live Port SaaS tenant | Docs + local portal | This is a docs-site rebuild, not a hosted Port tenant |
| Home KPI strip / stage counts | Still fixture numbers | Aggregates over report entities are not yet the home cards (stages remain illustrative) |
| Risky skills / ROI persona cards | Fixtures only | No Skill/ROI kinds in the factory portal yet |

### What is real Toolbox now

- Catalog kinds `FactoryLine`, `HitlTask`, and `Agent` in [`examples/port-factory-demo/`](https://github.com/loganvolkers/toolbox/tree/cursor/port-factory-catalog-bc64/examples/port-factory-demo)
- `agent-governance` Bronze→Gold scorecard evaluated by `toolbox report` / `toolbox test`
- Committed report feed at [`/toolbox/demo/port/report/`](/toolbox/demo/port/report/) (`report.json`); the interactive app’s **governance** tab reads those scores

What we still do **not** claim: live action runs, SaaS sync, or Port widget builder.

[Demo index](/toolbox/demo/port/) · Live Port: [demo.getport.io](https://demo.getport.io/) · Self-serve (Port): [demo.getport.io/self-serve](https://demo.getport.io/self-serve)
