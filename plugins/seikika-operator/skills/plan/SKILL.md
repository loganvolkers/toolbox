---
name: plan
description: >-
  Diffs two recomputes of one portal, before and after a manifest change. Use when preparing a portal pull request, and before asking for review of that change.
---

# seikika plan

1. Point --base at the manifest from the base revision. The path argument is the changed portal.
2. Read the Markdown: gates, score changes, authorization diff and declassifications.
3. Leave the plan in the pull request. Do not merge a plan that trips a gate without the owner acknowledging it.

```
seikika plan portal --local --base portal/seikika.yaml
```

Do not drop --base. Without the before manifest the diff is not a plan.
