---
id: D01M7ZD07SE4ADHZP2QKA3XG67E
decided: 2026-11-21T16:30:00Z
---

# R3 failed: Seikika doesn't make an agent cheaper at a standard in one repository

**The rerun that [D01M7Z2ZZ97VZQNTDT4QQHM3HQD](./D01M7Z2ZZ97VZQNTDT4QQHM3HQD.md) set up missed
its bar. After the friction fixes, the Seikika arm (B′) took 510k tokens over the standards
lifecycle, against 445k for bespoke scripts, at equal correctness. R3 is closed as failed.**

1. **What failed:** the claim that an agent spends less to write, change, reuse and read a
   standard with Seikika than with a script it writes itself. B′ matched the scripts after the
   first standard was written (284k against 275k tokens over phases 3 to 5). It beat them on reuse
   in calls and time, but not on tokens. Learning the format cost 55k tokens more than writing a
   script ([experiments/standards-ab](../../experiments/standards-ab/README.md)).
2. **The stop condition applies** ([repo-value.md](../design/repo-value.md) §3, R3): Seikika's
   value for a single repository is not agent efficiency. The plan no longer claims it is, and the
   single-repository case rests on the packaged standards, R2 (findings are true), R4 (the ratchet
   on real pull requests) and R5 (standards travel as packages).
3. **What is kept:** the fixes and packages this round built. They are the packaged standards the
   stop condition points to, and R4 and R5 test them.
4. **Not decided here:** whether repo-level value is proven without R3, and so whether federation
   resumes. That waits for R4's pilot period ([repo-value.md](../design/repo-value.md) §6).

**Status:** ✅

**Where:** [experiments/standards-ab](../../experiments/standards-ab/README.md),
[repo-value.md](../design/repo-value.md) §8
