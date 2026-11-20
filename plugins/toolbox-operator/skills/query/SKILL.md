---
name: query
description: >-
  Lists entities of one kind that match a CEL filter over entity and scores. Use when an agent needs a set of entities, and after the server is up.
---

# toolbox query

1. Name the kind with --kind. Kinds are the manifest names, such as Crate or Command.
2. Add --filter as CEL. It keeps matches. Add --fields and -o json to limit the payload.
3. Page the work from the result. Do not filter in the client what the CEL filter can express.

```
toolbox query --kind Crate --filter "entity.maxFnLines > 80" -o json
```

Do not invent a kind name. If it fails, the kind is not in the portal or the filter does not type-check.
