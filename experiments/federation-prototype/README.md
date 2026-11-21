# Federation prototype

**Question:** Can Toolbox federate standards down and results up—across enterprise →
org → leaf → monorepo subgraph—without a parallel product, and still catalogue
services/components/libraries (not only repos)?

**Answer (prototype):** yes, as a docs-as-code vertical slice. `compose.py`
resolves package includes, rejects weakenings, merges catalogs, and rolls child
reports into a parent adoption view. Production federation (D68 F1–F5) stays
paused until repo-level value is proven; this experiment does not change T4T
ratchets.

Design (project store): federation prototype design (concerns 1–5, §7 campaigns).
In-repo design: [docs/design/federation.md](../../docs/design/federation.md) (D68).

## Layout

| Path | Role |
|---|---|
| [federation.yaml](federation.yaml) | Tree: enterprise → org-payments → repo-payments-api → checkout; mobile-app observe-only |
| [enterprise/](enterprise/) | Governance package: CodeQL, Prettier, TS, JUnit, lockfile, Ubuntu, rulesets, Agents.md, Lefthook + campaigns |
| [org-payments/](org-payments/) | Org package that extends with `payments-idempotency` and tightens Prettier |
| [repo-payments-api/](repo-payments-api/) | Leaf portal fixtures: includes, multi-entity catalog, sample `results.json` |
| [repo-payments-api/packages/checkout/](repo-payments-api/packages/checkout/) | Monorepo subgraph + grass-roots experimental package |
| [mobile-app/](mobile-app/) | Observe-only leaf (Expo/RN facts; no mobile standard pushed) |
| [testdata/weaken/](testdata/weaken/) | Fixture that raises `prettierPrintWidth` and must fail |
| [compose.py](compose.py) | Applicator CLI |
| [tests/test_compose.py](tests/test_compose.py) | Push / extend / weaken / results-up / catalog tests |

Fixtures live under `experiments/`, not discoverable `packages/`, so they do not
inflate T4T package counts or weaken yamark / D79.

## Run

```bash
# From repo root (PyYAML required; jsonschema optional for Draft 2020-12 check)
python3 experiments/federation-prototype/compose.py check
python3 experiments/federation-prototype/compose.py weaken
python3 experiments/federation-prototype/compose.py report -o /tmp/federation-report.json
python3 -m unittest experiments.federation-prototype.tests.test_compose
# or:
python3 experiments/federation-prototype/tests/test_compose.py
```

## What the tests prove

| Flow | Proof |
|---|---|
| **Push** | Leaf compose includes enterprise scorecards (`codeql`, …) and kinds Repository/Service/Component/Library |
| **Extend** | Org adds `payments-idempotency`; checkout publishes experimental `cart-latency` |
| **Weaken rejected** | `prettierPrintWidth: 120` against default 100 fails with `weakens` |
| **Results up** | Parent report lists child package digests, campaign rollup, grass-roots package |
| **Observe-only** | `mobile-app` contributes stack facts without a mandatory mobile scorecard |
| **Catalog** | Merged entities include Service, Component, Library (and repos) |

## §7 campaigns: modeled vs stubbed

| Campaign | In fixtures | Live automation |
|---|---|---|
| CodeQL zero findings | scorecard + campaign + report measure | No CodeQL runner |
| Prettier | scorecard + tighten param | No formatter install |
| TypeScript ratchet | scorecard | No file scan |
| JUnit 4→5 EOY | library scorecard + campaign `end` | OpenRewrite stub only |
| Actions lockfile 60d | campaign + `enforceAt` | No GHE settings API |
| Ubuntu docker hosts | advisory scorecard | No host inventory |
| Branch protection → rulesets | scorecard | No GitHub API |
| Agents.md / no `npx skills` | scorecard | No plugin manager install |
| Husky → Lefthook | scorecard + campaign | No migrator |
| Mobile observe-only | `observeOnly` leaf + report | No standard pushed (intentional) |

## Operators

`federation.yaml` marks `enterprise` / `org-payments` as **governance** and
`repo-payments-api` / `checkout` / `mobile-app` as **leaf**, matching the design's
central vs leaf operator split.
