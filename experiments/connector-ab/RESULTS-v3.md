# Connector A/B Results — rerun v3 (catalog-drive)

**Status: phase 1 complete; phases 3–5 pending.** Same bar as [README.md](./README.md) §6.
Product: [D01M83GQ1KP8A0WR3P2S7AG54WQ](../../docs/decisions/D01M83GQ1KP8A0WR3P2S7AG54WQ.md)
(`connector drive --format catalog --out`). Baseline win: [RESULTS-v2.md](./RESULTS-v2.md).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Toolbox cheaper to author and change** | A fewer calls than B and C over phases 1+3 | *pending phase 3* (phase 1 alone: A **22**, B 43, C 29) |
| **C2. Toolbox reuses cheaper** | A fewer calls than B and C in phase 4 | *pending* (v2 rematch: A12 / B18 / C14) |
| **C3. Platforms correct within budget** | Finishing arms fully correct | *phase 1 holds* (all exact) |

## Lead target (this loop)

v2 margins were thin (C1 −4/−2; C2 −6/−2). This loop aims for a **large definitive lead**:
A at least **10 calls** under both rivals on C1 (1+3) **and** on C2 (phase 4), with C3 holding.
If product changes cannot produce that without changing the bar, admit defeat on "large lead"
while keeping the v2 win on record.

Phase 1 already clears the ≥10-call gap vs both rivals (A22 vs B43 / C29).

## Cost

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 | **22** | 43 | 29 |
| Calls phase 3 | | | |
| Calls 1+3 (C1) | | | |
| Calls phase 4 (C2) | | | |
| Calls phase 5 | | | |

## What changed vs v2

`--format catalog --out` removes the hand-rolled drive→entities map script. Phase 1: 36→**22**.

## Artifacts

`experiments/connector-ab/results/v3/`
