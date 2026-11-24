---
name: actions
description: >-
  Lists the actions available on one entity of a running server. Use when an agent might change an entity through the portal, and after the server is up. To add an action, use the add-action skill instead.
---

# toolbox actions

1. Pass the entity id.
2. Read each action's name and input before calling one.
3. Stop if the list is empty. Do not invent an action the command did not return.

```
toolbox actions crate:toolbox-expr -o json
```

Do not run an action from this command. It only lists them. If it fails, the server is down or the id is unknown.
