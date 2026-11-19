---
name: review-plan
description: Read and summarise `toolbox plan` output for a portal change. Covers model migrations, mapping and identity changes, score deltas, authorization diff, declassifications, notification volume, MCP and agent-plugin surface changes, and tripped gates. Use before opening or reviewing a Toolbox portal pull request.
metadata:
  status: pre-release
---

# Review a plan

Summarise these sections, in this order, for the PR description:

1. **Blocking items:** tripped gates (deletion ratio, authz loss, operator access, notification volume,
   agent-surface removals, missing owners) and the team that must acknowledge each one.
2. **Model changes** and any required migrations.
3. **Authorization diff and declassifications:** who gains or loses access, and what becomes visible
   more widely.
4. **Scorecards:** entities changing level, and new rules.
5. **Agent surface:** MCP tools added, changed or removed, and changes to the generated user agent
   plugin.
6. **Notifications:** estimated new issues and updates.

Never acknowledge a gate on the user's behalf. Explain it, and name who can.
