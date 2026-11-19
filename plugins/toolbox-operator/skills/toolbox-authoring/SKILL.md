---
name: toolbox-authoring
description: The core workflow for changing a Toolbox internal developer portal repo (kinds, connectors, scorecards, actions, permissions, views). Use whenever editing a repo that contains Toolbox.yaml or toolbox.lock, or when asked to add or change anything in a Toolbox portal.
compatibility: Requires the toolbox CLI pinned in the portal repo's toolbox.lock. Pre-release skeleton; commands describe the planned CLI.
metadata:
  status: pre-release
---

# Toolbox authoring loop

Work test-first. Pass the portal repo root as `workspace` to every `toolbox-dev` MCP tool.

1. **Specify:** add or update the property ids the change affects in `spec/properties.md`.
2. **Red:** write failing tests first:
   - fixtures under `tests/`
   - `.fga.yaml` for permission changes
   - dry-run snapshots for actions
3. **Implement** in the owning module (`modules/<name>/`). Every new definition needs an `owner`.
4. **Check:** run `validate`, then `test`. Use `explain_error` on any diagnostic; errors include a
   suggested fix.
5. **Simulate:** `simulate` with seeds for the module. Replay any failing seed with `--seed`.
6. **Plan:** run `plan` against production and read it with the `review-plan` skill. Note any tripped
   gates and any declassifications.
7. **PR:** open a pull request with the plan summary. Never merge it yourself. Review is routed to the
   owners of every touched definition.

Never put secrets in code; use `tb.secret("vault:…")`. Never weaken a test to make it pass.
