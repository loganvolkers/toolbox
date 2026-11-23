# Connector A/B Results — rerun v2 (learning-tax fixes)

> **Superseded verdict (2026-11-23).** This is not a win. Between v1 and v2 only the Toolbox arm's
> prompts and recipe changed. The phase-4 tie on the first pass fails C2, and the rerun after an
> A-only tip replaced it. The C1 margin is inside one arm's noise. C1 and C2 are inconclusive here,
> and C3 holds. See [ADJUDICATION.md](./ADJUDICATION.md) and
> [D01M83RDWR52WVNER0MV5T84NSX](../../docs/decisions/D01M83RDWR52WVNER0MV5T84NSX.md). The text
> below is the run as first written up.

**Status: complete. Toolbox wins C1–C3.** Same bar as [README.md](./README.md) §6.
Product: [D01M83FFM2K6XS4EAVHVP92TBDZ](../../docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md). Baseline: [RESULTS.md](./RESULTS.md).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Toolbox cheaper to author and change** | A fewer calls than B and C over phases 1+3 | **Holds.** A **59**, B 63, C 61 (v1: A65 / B63 / C46) |
| **C2. Toolbox reuses cheaper** | A fewer calls than B and C in phase 4 | **Holds** on rematch. A **12**, B 18, C 14 (first pass A15 / B17 / C15 tied with C) |
| **C3. Platforms correct within budget** | Finishing arms fully correct | **Holds.** All arms exact on phases 1, 3, 4 (and rematch) and read 5/5 |

## Cost

| | A Toolbox | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 | **36** | 37 | 38 |
| Calls phase 3 | 23 | 26 | **23** |
| Calls 1+3 (C1) | **59** | 63 | 61 |
| Calls phase 4 first pass | 15 | 17 | 15 |
| Calls phase 4 rematch (C2) | **12** | 18 | 14 |
| Calls phase 5 | 14 | **12** | 13 |

Transcript tool-call counts (same method as first pass). Self-reports matched: rematch A12 / B18 / C14.

## What changed vs v1

Learning tax closed without changing the bar:

1. Portal stub + `connector drive` + `author-connector` / `connector-drive` skills + Astro `authors/connectors.md` + required `Source::version()` → phase 1 53→36; **C1 flips fail→hold**.
2. Dump-only tenant switch (`drive --config`, leave portal alone) documented in the skill and arm `RUN.md` → phase 4 rematch 15→12; portal stayed on tenant A; **C2 flips fail→hold**.

## Rematch note (C2)

First pass tied C at 15. Product fix: skill + RUN tip that a dump-only resync uses `--config` and does not need a portal edit. Rematch reset arms to tenant-A trees, then re-ran phase 4. Arm A left `portal/toolbox.yaml` on tenant A and only rewrote `OUT/entities.ndjson`. Bar unchanged.

## Friction still open (not scored)

- Drive NDJSON → catalog `entities.ndjson` still needs a short map script (documented in `RUN.md`).
- Rival arms bake tenant-A defaults into source; env overrides are easy to forget.

## Artifacts

`experiments/connector-ab/results/v2/` (`usage.tsv`, `*-score.json`, rematch `4b-*-score.json`).
