---
name: assess-check
description: >-
  Validates a portal's assessment files and lists, per rubric, which entities are fresh, stale or missing. Use when checking assessments for drift, and before trusting a committed judgement.
---

# seikika assess check

1. Run the command on the portal. Skip the same connectors the portal's CI skips.
2. Read fresh, stale and missing. Fresh means the subject version and any recorded rubric hash match. Stale means at least one version changed. Missing means no file.
3. Brief stale entities for the review you are performing. Routine CI reports staleness. Use
   `--require-fresh` for the release preflight or an explicitly high-risk change.

```
seikika assess check .seikika --skip runner,github-actions
```

Do not delete an assessment to turn stale into missing. Invalid files always fail, including assessments missing a verdict for a rubric criterion.

For a campaign, use `--queue --campaign NAME --rubric RUBRIC --with-briefs` to collect a bounded
worklist and its briefs in one pass. Choose a next action with `--action`. Hand off independent
reviews when the queue names the original assessor, and use the reported assessment path for updates.
Run the check again with `--entity ID --rubric RUBRIC --require-fresh` after updating that subject.
Use `--run-manifest PATH` to retain version, connector and output evidence for the verification run.
