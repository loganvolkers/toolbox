---
name: campaign
description: >-
  Prints one campaign's progress over existing scorecards: percent meeting the
  goal, milestones, and entities that still have gaps. Use when checking a
  time-bound focus area, and after the server is up. With no name, it lists the
  campaigns.
---

# toolbox campaign

1. Pass the campaign name, or omit it to list names and progress.
2. Work the entities that are not meeting the goal; each gap names a scorecard
   include (`atLeast` or a rule).
3. Fix the underlying scorecard outcomes; do not edit campaign progress.

```
toolbox campaign skill-silver -o json
```

Campaigns are declared in the manifest (`tracks` / `campaigns`). They compose
scorecards; they do not define new checks. If it fails, the campaign name is
wrong or the server is down. Without a server, pass `--portal <dir>` (and
`--baseline <file>` for the paydown since each campaign's recorded start):
`toolbox campaign --portal .toolbox --baseline .toolbox/results.json`.
