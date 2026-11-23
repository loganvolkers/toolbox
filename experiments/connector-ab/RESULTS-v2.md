# Connector A/B Results — rerun v2 (learning-tax fixes)

**Status: run complete for C1–C3; C2 rematch in flight.** Same bar as [README.md](./README.md) §6.
Product: [D01M83FFM2K6XS4EAVHVP92TBDZ](../../docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md). Baseline: [RESULTS.md](./RESULTS.md).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Toolbox cheaper to author and change** | A fewer calls than B and C over phases 1+3 | **Holds.** A **59**, B 63, C 61 (v1: A65 / B63 / C46) |
| **C2. Toolbox reuses cheaper** | A fewer calls than B and C in phase 4 | **Fails** on first pass (A15 / B17 / C15 — tied with C). Rematch after dump-only tenant-switch guidance: *in flight* |
| **C3. Platforms correct within budget** | Finishing arms fully correct | **Holds.** All arms exact on phases 1, 3, 4 and read 5/5 |

## Cost

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 | **36** | 37 | 38 |
| Calls phase 3 | 23 | 26 | **23** |
| Calls 1+3 (C1) | **59** | 63 | 61 |
| Calls phase 4 (C2 first pass) | **15** | 17 | **15** |
| Calls phase 5 | *see usage.tsv* | | |

## What changed vs v1

Learning tax closed: Toolbox phase 1 53→36 after portal stub, `connector drive`, `author-connector` skill, Astro docs, required `Source::version()`. C1 flips fail→hold without changing the bar.

## Artifacts

`experiments/connector-ab/results/v2/`
