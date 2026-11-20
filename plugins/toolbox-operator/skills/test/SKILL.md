---
name: test
description: >-
  Recomputes a portal from its fixtures and runs tests/*.test.yaml. Use when scorecard rules, fixtures or expectations changed, and after validate has passed.
---

# toolbox test

1. Validate the portal first, so a schema error is not reported as a failed expectation.
2. Run the command on the portal directory. Each expectation prints pass or fail.
3. Fix the rule, the fixture or the expectation that failed, then run the command again.

```
toolbox test portal
```

Do not weaken an expectation to make it pass. If it fails, the check or the fixture is wrong.
