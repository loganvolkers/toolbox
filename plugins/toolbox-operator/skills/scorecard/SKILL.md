---
name: scorecard
description: >-
  Prints one scorecard across every entity: level counts, failing rules and evidence. Use when building a work list, and after the server is up. With no name, it lists the scorecards.
---

# toolbox scorecard

1. Pass the scorecard name, or omit it to list names.
2. Narrow with --rule and --outcome, or with --level.
3. Work the evidence list. Do not start from the level counts alone.

```
toolbox scorecard operator-cli --rule has-help --outcome fail -o json
```

Do not edit results to change a count. If it fails, the scorecard name is wrong or the server is down.
