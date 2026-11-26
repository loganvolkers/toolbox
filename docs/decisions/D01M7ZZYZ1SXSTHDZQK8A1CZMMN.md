---
id: D01M7ZZYZ1SXSTHDZQK8A1CZMMN
decided: 2026-11-21T21:00:00Z
---

# Root causes are a core value: every regression says what caused it

**The ratchet doesn't stop at "something got worse". Every regression carries its root cause, as
a chain that ends at something a person or an agent can act on: a file, a line, a rule's evidence,
and the command that reproduces it with the tool alone. An aggregate's regression (a repository
level) names the constituent scorecard that fell and the change below it that made it fall.**

1. **The chain:**
   - aggregate (a scorecard that reads `cards`);
   - constituent scorecard whose level fell, with its levels before and after;
   - the entity changes now at that lower level: new below the floor, a rule that started
     failing, evidence that grew;
   - the rule's evidence now;
   - the record it came from, or the tool's `command` (take-it-out,
     D01M7ZRAJT47W11X5X3FEJ6QTX8).
2. **Everywhere the ratchet speaks:** the summary (indented under each regression), `--against`,
   the `--verdict` JSON (`detail` and nested `causes`), and the report action's annotations.
3. **Why it is core:** a gate that fails without a cause teaches people to update the baseline
   blindly. The first internal use of the box showed it: a new generated doc failed one docs rule,
   and the repository level fell. The ratchet reported the fall, but listed its cause only as a
   change (`seikika-and-tools.md`, `experiments/box-ab` §1).

**Status:** ✅

**Where:** [docs/design/root-causes.md](../design/root-causes.md),
[crates/seikika-cli/src/causes.rs](../../crates/seikika-cli/src/causes.rs)
