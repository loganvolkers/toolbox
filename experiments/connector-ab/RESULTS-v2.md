# Connector A/B Results — rerun v2 (learning-tax fixes)

**Status: in progress** (phases 1+3 complete; 4–5 running). Same bar as [README.md](./README.md) §6.
Product: [D01M83FFM2K6XS4EAVHVP92TBDZ](../../docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md). Baseline: [RESULTS.md](./RESULTS.md).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Toolbox cheaper to author and change** | A fewer calls than B and C over phases 1+3 | **Holds.** A **59**, B 63, C 61 (v1: A65 / B63 / C46) |
| **C2. Toolbox reuses cheaper** | A fewer calls than B and C in phase 4 | *pending* |
| **C3. Platforms correct within budget** | Finishing arms fully correct | *pending* (1+3 all pass) |

## Cost so far

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 | **36** | 37 | 38 |
| Calls phase 3 | **23** | 26 | 23 |
| Calls 1+3 (C1) | **59** | 63 | 61 |

Phase 1 learning tax closed: 53→36 on Toolbox after scaffold/portal/`connector drive`/skills/docs.
