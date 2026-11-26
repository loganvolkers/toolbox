---
name: actions
description: >-
  Lists the actions available on one entity of a running server. Use when an agent might change an entity through the portal, and after the server is up. To declare a new action, see the add-action skill.
---

# seikika actions

1. Pass the entity id.
2. Read each action's name and input before calling one.
3. Stop if the list is empty. Do not invent an action the command did not return.

```
seikika actions crate:seikika-expr -o json
```

Do not run an action from this command. It only lists them. If it fails, the server is down or the id is unknown.
