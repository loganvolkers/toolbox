---
id: D01M85G2YXV2VRTZWQJ33MHXCKN
decided: 2026-11-24T00:42:56Z
---

# Protocol 2's cost rule becomes an exact rank test with five runs per arm, from the next frozen round

**A cost hypothesis holds or fails on an exact two-sided Mann-Whitney test between the two arms'
runs, at α = 0.05 after a Holm correction over the round's registered cost hypotheses, and the
direction of the medians says which. Otherwise it is inconclusive. A round that registers a cost
hypothesis runs at least five times per arm, and its README states the smallest effect the design
detects. Medians, ranges and *W* are still published. This applies to rounds frozen after this
decision; every label already published stands as registered.** This amends P2 of
D01M83RDWQ5EVKY5SFHDWAD5HFH. The other rules, the catalog fields and the scorecard stay as they are.

1. **Why.** The *W* rule (medians differ by more than the wider arm's range) almost never gives a
   false result, and it almost never finds a real one. At five runs per arm it detects a
   one-standard-deviation effect 4% of the time and a two-standard-deviation effect 23% of the
   time (simulation, normal data). In graphify-ab, every Graphify run on Toolbox made fewer calls
   and read less than every baseline run (exact p = 0.008 on each), and the rule called both
   inconclusive. The red team found this ([graphify-red-team.md](../graphify-red-team.md) R1), and
   the designer's re-check confirmed it
   ([graphify-red-team-response.md](../graphify-red-team-response.md) §1).
2. **The rule.**

   | Part | Requirement |
   |---|---|
   | Test | Exact two-sided Mann-Whitney U on the per-run measure, between the arms the hypothesis names |
   | Correction | Holm over the round's registered cost hypotheses, at α = 0.05. With five runs per arm, the smallest p is 0.008, so only complete separation passes and at most six cost hypotheses can hold; register fewer, or run more |
   | Runs | At least five per arm per phase when a cost hypothesis is registered. Three runs per arm can't reach p < 0.05 at all. Phases with only correctness or guard hypotheses keep P2's minimum of three |
   | Sensitivity | The README states the smallest effect, in standard deviations of a dry run or an earlier round, that the design detects at least four times in five. A smaller claimed effect needs more runs |
   | Reporting | Every run, the minimum, median and maximum, *W*, U and p |

3. **It cuts both ways.** Read post hoc, the rank test would have given Graphify two cost wins on
   Toolbox and Toolbox two more against reviewdog, and it would have erased Toolbox's round-1 loss
   to reviewdog in gate-repair-ab, which ran three runs per arm
   ([graphify-red-team-response.md](../graphify-red-team-response.md) §4). Re-labelling past
   rounds under a rule chosen after seeing them is what P5 forbids, so they keep their labels, and
   the post-hoc readings are published beside them as post hoc.
4. **What it does not change.** P1 and P3–P7, pre-registration, publishing losses, and the
   catalog's fields. *W* stays in every write-up, so old and new rounds can be compared.

**Status:** ✅

**Where:** [docs/design/dx-benchmarks.md](../design/dx-benchmarks.md) §4.1 (P2) and L23
