---
name: assess-brief
description: >-
  Prints the rubric, the entity's facts, the subject hash and the assessment file path for one entity. Use when asked to assess, re-assess, or judge a Toolbox entity, and when a review says an entity is unassessed or stale.
---

# toolbox assess brief

1. Run the command with the entity id and --portal. Skip connectors the portal's own check skips, such as runner and github-actions.
2. Read the sources the facts point at, then judge each criterion pass, fail or unsure. Cite a file, a decision or a number in the reason, and add a suggestion for fail or unsure.
3. Write the assessment to the path the brief names, with subjectHash copied exactly and assessor.name set to the harness. Then check the file.

```
toolbox assess brief rule:toolbox/maintainer-crate/has-tests --portal portal --skip runner,github-actions
```

Do not edit subjectHash to match a new entity. If the brief's hash differs from the file, the assessment is stale and must be judged again. Do not copy another assessor's verdicts.
