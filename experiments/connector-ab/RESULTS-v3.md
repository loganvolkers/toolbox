# Connector A/B Results — rerun v3 (catalog-drive)

> **Superseded verdict (2026-11-23).** "Holds" below is not a verdict that stands. The round
> followed a product format made "to widen connector-ab lead", which emits the harness's own NDJSON
> shape, and only the Seikika arm's recipe names it. The C1 margin over Backstage is 1 call, while
> Backstage's own C1 total moved by 21 across rounds. Its prompts and scaffold did not change in
> those rounds. Only the shared `FORGE.md` gained notes, and every arm got them. C1 and C2 are inconclusive here, and C3 holds. The phase-3 first pass that failed was
> Seikika and Ocean, not Backstage. See [ADJUDICATION.md](./ADJUDICATION.md) and
> [D01M83RDWR52WVNER0MV5T84NSX](../../docs/decisions/D01M83RDWR52WVNER0MV5T84NSX.md). The text below
> is the run as first written up.

**Status: complete.** Same bar as [README.md](./README.md) §6.
Product: [D01M83GQ1KP8A0WR3P2S7AG54WQ](../../docs/decisions/D01M83GQ1KP8A0WR3P2S7AG54WQ.md),
[D01M83H3HBH2SC205MX3ANRZYWN](../../docs/decisions/D01M83H3HBH2SC205MX3ANRZYWN.md).
Baseline: [RESULTS-v2.md](./RESULTS-v2.md).

## Against the bar (§6)

| Hypothesis | Holds if | Result |
|---|---|---|
| **C1. Seikika cheaper to author and change** | A fewer calls than B and C over phases 1+3 | **Holds.** A **39**, B 57, C 40 (v2: 59/63/61) |
| **C2. Seikika reuses cheaper** | A fewer calls than B and C in phase 4 | **Holds.** A **9**, B 12, C 16 (v2 rematch: 12/18/14) |
| **C3. Platforms correct within budget** | Finishing arms fully correct | **Holds.** Phases 1, 3, 4 exact (phase 3 after C3 rematch); read 5/5 |

## Large-lead target (this loop)

Target was A ≥10 under both rivals on C1 **and** C2. **Missed.**
- C1 margin vs C: only **1** call (39 vs 40); vs B: 18.
- C2 margin vs B: 3; vs C: 7.

Seikika still wins the pre-registered bar; the "large definitive lead" goal is not met.
Further product changes that would cut phase-3 change cost enough for ≥10 on C1 without
gaming the bar are not obvious after catalog-drive + composite-id fixes — **admit defeat
on large lead**, keep the v3 win on record.

## Cost

| | A Seikika | B Ocean | C Backstage |
|---|---|---|---|
| Calls phase 1 | **22** | 43 | 29 |
| Calls phase 3 (rematch) | 17 | 14 | **11** |
| Calls 1+3 (C1) | **39** | 57 | 40 |
| Calls phase 4 (C2) | **9** | 12 | 16 |
| Calls phase 5 | 13 | **10** | 11 |

## What changed vs v2

1. `connector drive --format catalog --out` → phase 1 36→22; phase 4 12→9.
2. Composite catalog-id guidance + numeric-externalId fallback → phase-3 C3 rematch.

## Artifacts

`experiments/connector-ab/results/v3/`
