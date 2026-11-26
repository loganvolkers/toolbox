---
name: score
description: >-
  Prints scorecard levels and rule outcomes for one entity. Use when an agent needs the levels of one id, and after the server is up.
---

# seikika score

1. Pass the entity id.
2. Add -o json and read failing rules before the level.
3. Follow the fix on a failing rule. Do not raise a level by skipping the entity.

```
seikika score command:validate -o json
```

Do not confuse this with scorecard, which is one scorecard across every entity. If it fails, the server is down or the id is unknown.
