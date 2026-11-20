---
name: assess-check
description: >-
  Validates a portal's assessment files and lists, per rubric, which entities are fresh, stale or missing. Use when checking assessments for drift, and before trusting a committed judgement.
---

# toolbox assess check

1. Run the command on the portal. Skip the same connectors the portal's CI skips.
2. Read fresh, stale and missing. Fresh means the subject hash still matches. Stale means the entity changed after the assessment. Missing means no file.
3. Brief each stale entity again and judge it. Do not treat a stale file as current.

```
toolbox assess check portal --skip runner,github-actions
```

Do not delete an assessment to turn stale into missing. If it fails, a file does not match the assessment schema or its rubric.
