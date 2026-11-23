# Connector A/B Results — rerun v3 (catalog-drive)

**Status: phases 1+3 complete; phase 4–5 pending.** Same bar as [README.md](./README.md) §6.
Product: [D01M83GQ1KP8A0WR3P2S7AG54WQ](../../docs/decisions/D01M83GQ1KP8A0WR3P2S7AG54WQ.md),
[D01M83H3HBH2SC205MX3ANRZYWN](../../docs/decisions/D01M83H3HBH2SC205MX3ANRZYWN.md).
Baseline: [RESULTS-v2.md](./RESULTS-v2.md).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Toolbox cheaper to author and change** | A fewer calls than B and C over phases 1+3 | **Holds** — A **39**, B 57, C 40 (v2: 59/63/61). Lead vs C is only 1 calls — **not** the ≥10 large-lead target |
| **C2. Toolbox reuses cheaper** | A fewer calls than B and C in phase 4 | *pending* |
| **C3. Platforms correct within budget** | Finishing arms fully correct | **Holds** on phases 1 and 3 (after C3 rematch for component ids) |

## Lead target (this loop)

Aim: A ≥10 under both rivals on C1 and C2. Phase 1 alone cleared that; phase 3 rematch
costs left C1 margin vs C at 1. Continue phase 4; if large lead still
misses after C2, ship another product fix or admit defeat on "large lead".

## Cost

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 | **22** | 43 | 29 |
| Calls phase 3 (rematch) | 17 | 14 | **11** |
| Calls 1+3 (C1) | **39** | 57 | 40 |
| Calls phase 4 (C2) | | | |
| Calls phase 5 | | | |

## What changed vs v2

1. `connector drive --format catalog --out` → phase 1 36→22.
2. Composite catalog-id guidance + numeric-externalId fallback → phase-3 C3 rematch.

First phase-3 passes discarded (numeric component ids on all arms).

## Artifacts

`experiments/connector-ab/results/v3/`
