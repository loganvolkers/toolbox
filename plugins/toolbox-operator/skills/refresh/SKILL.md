---
name: refresh
description: >-
  Asks the connectors behind an entity to re-read it and waits for the fresh snapshot. Use when a source changed and a running portal should catch up.
---

# toolbox refresh

1. Pass the entity id of the thing that changed.
2. Wait for the command to print the new snapshot.
3. Read the entity again with get. Do not refresh in a loop when the command returns an error.

```
toolbox refresh crate:toolbox-expr
```

Do not refresh every entity to answer one question. If it fails, the connector or the server rejected the id.
