---
id: D01M87QHJWYKGNH0182XQ5NKHMB
decided: 2026-11-24T21:31:44Z
---

# Protocol 2's rank-test cost rule takes effect when it reached main, so actions-ab's rounds stand under *W*

**D01M85G2YXV2VRTZWQJ33MHXCKN's cost rule (an exact rank test, five runs per arm) applies to
rounds frozen after it reached `main` in #83, on 2026-11-24T21:27:39Z. It does not apply from its
`decided` time.** Rounds frozen before then are decided by the *W* rule, as registered, including
every actions-ab round up to round 8. This only sets the effective time; the rule itself is
unchanged.

1. **Why.** The decision was written on a branch at 00:42Z. actions-ab was built in parallel on
   another branch and frozen later the same day under the *W* rule, with three runs per arm,
   before the decision was on `main`. Its designer couldn't have followed a rule that wasn't on
   `main`. Applying the rule from its `decided` time would make round 8's registered H2 pass
   non-compliant after the fact. At three runs per arm, a two-sided rank test can't reach 0.05,
   so that pass would become inconclusive without any new evidence. P3 fixes a round's harness
   and bar at its frozen commit, and D01M85G2YXV2VRTZWQJ33MHXCKN already keeps published labels
   as registered. A round frozen before the rule reached `main` is the same case.
2. **The cost is visible.** actions-ab's H2 and H3 labels were decided by a rule that detects a
   two-standard-deviation effect less often than the rank test does (L23). Its README and
   catalog entry already publish every run, so a reader can apply the rank test post hoc, as
   graphify-red-team-response.md §4 did for the earlier benchmarks.
3. **For later decisions.** A protocol rule takes effect when it reaches `main`, not when its
   file says it was decided.

**Status:** ✅

**Where:** [docs/design/dx-benchmarks.md](../design/dx-benchmarks.md) header and P2;
amends the effective time of D01M85G2YXV2VRTZWQJ33MHXCKN.
