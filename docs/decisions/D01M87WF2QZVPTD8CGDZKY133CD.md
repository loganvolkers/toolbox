---
id: D01M87WF2QZVPTD8CGDZKY133CD
decided: 2026-11-24T23:40:00Z
---

# A comparison states its verdict rule: behind on any rule, or the majority of rules

**A comparative scorecard may set `compare.verdict`: `anyBehind` (the default, as in
D01M857ZB468ZM2YS6GNSEG6YB9) or `majority`. Under `majority`, the overall standing per peer is
`unknown` if any rule is unknown, else `ahead` when more rules are ahead than behind, `behind`
when more are behind than ahead, and `parity` when they are equal. The derive step records the
rule beside the verdict (`verdictRule`), and the report states the rule the comparison uses
wherever it counts factors.**

1. **Why.** The OpenFGA portal compares OpenFGA with SpiceDB on six factors whose tally was
   pre-registered as a majority of rules (`experiments/openfga-spicedb-compare/README.md`). The
   report's only rule, behind if any rule is behind, read "behind on 4 of 6 factors" where the
   pre-registered result is "ahead on 4 of 6". Three blind design-audit rounds scored that
   severity 3, and the independent adjudication
   ([ADJUDICATION.md](../../experiments/openfga-spicedb-compare/ADJUDICATION.md) §5) called the
   lead number accurate under its rule and a poor summary of a pre-registered A/B.
2. **Why both rules stay.** Behind-on-any suits a comparison used as a ratchet on the subject's
   own portal, where any loss matters; Toolbox for Toolbox keeps it against Backstage. Majority
   suits an A/B whose authors fixed their tally before the run. The rule is part of the
   comparison's definition, so it is set beside `subject` and `peers` and is reviewed like any
   other bar.
3. **Unknown rules.** Under `majority`, one unknown rule makes the factor unknown, because the
   count is not complete. Under `anyBehind`, a rule behind still decides first, as before.
4. **What does not change.** Rule standings, tolerances, the ratchet (peers stay out of the
   baseline) and validation are as in D01M857ZB468ZM2YS6GNSEG6YB9. A scorecard without
   `verdict` derives exactly what it did.
5. **The box or a tool.** The box: it is how the report counts what the tools measured.
