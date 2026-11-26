---
id: D01M801A9QNVCSBBJATYV721TCS
decided: 2026-11-21T22:30:00Z
---

# Finding tools: a deterministic inventory, checked by an agent

**`seikika tools find` lists the tools a repository already uses, from config files, CI and hook
commands, package manifests and installer files, matched against a catalog. For each it gives its
evidence, whether it is in the box, and the slot and kit that would carry it. It also lists the
scripts CI runs. It decides nothing. The `find-tools` agent skill checks each candidate (in use,
version, SARIF command, same findings alone) before anything goes in the box.**

1. **Why not deterministic only:** a stale config looks like a tool in use, and a script can be a
   per-file check or a pass/fail gate. Only reading it tells which.
2. **Why not agent only:** reading every config and workflow is slow and uneven. The inventory
   is quick and repeatable: 1 s over gh-aw's 25 MB of workflows.
3. **The catalog says what was checked:** each SARIF command records where it was run and its
   output read. An empty `checked` tells the agent to run the command first.

**Status:** ✅

**Where:** [docs/design/finding-tools.md](../design/finding-tools.md),
[crates/seikika-cli/src/find.rs](../../crates/seikika-cli/src/find.rs),
[plugins/seikika-operator/skills/find-tools](../../plugins/seikika-operator/skills/find-tools/SKILL.md)
