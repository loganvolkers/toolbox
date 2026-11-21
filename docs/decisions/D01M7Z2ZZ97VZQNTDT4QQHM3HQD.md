---
id: D01M7Z2ZZ97VZQNTDT4QQHM3HQD
decided: 2026-11-21T10:30:00Z
---

# R3: fix the measured friction, then rerun the Toolbox arm against a fixed bar

**R3's second round failed as pre-registered ([experiments/standards-ab](../../experiments/standards-ab/README.md)):
both arms were exact, and Toolbox cost more in every phase. Phase 4 (reuse) did not test sharing,
because no package could express the standards, and most of the extra cost was friction with a
named fix. So R3 gets one rerun of the Toolbox arm (B′) after those fixes, with its bar set now.**

1. **Fixes first,** from the round's friction table:
   - a numeric ratchet: a rule's `measure`, compared instead of the evidence count (item 1);
   - a flat list of failures as JSON (`-o failures`, item 3);
   - campaign progress in `report -o json`, with its starting point recorded in the baseline and
     paydown measured from it (items 4 and 5);
   - accepting an intended change to one scorecard (`--update --accept <scorecard>`, item 6);
   - parameterised packages for a file-size and a pattern-count standard, with list and object
     parameters, exemptions with reasons and the ratchet floor for new entities set (items 2, 7, 8);
   - the workspace connector's configuration reference and a starter README (items 11 and 12);
   - semgrep JSON in the runner connector, so an existing semgrep rule can be the source;
   - the small bugs and the slow per-file `git log` (items 9, 15, 18).
2. **Rerun:** fresh agents, same model, same prompts as arm B (with docs pointers updated), same
   truth, gate harness and phase 5 key, on fresh copies at the same commits: phases 1, 3, 4 and 5.
   Phase 4 is the sharing test: the package that holds the Codex standard is included again,
   with OpenFGA's parameters.
3. **Bar:** B′'s total tokens over phases 1, 3, 4 and 5 below arm A's (445k), at equal correctness.
   **If B′ misses it, R3 fails for good:** ship standards as text
   ([repo-value.md](../design/repo-value.md) R3's stop condition).

**Status:** in progress

**Where:** [experiments/standards-ab](../../experiments/standards-ab/README.md),
[repo-value.md](../design/repo-value.md) §8
