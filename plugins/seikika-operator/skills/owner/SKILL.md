---
name: owner
description: >-
  Says who owns an entity and why, with CODEOWNERS evidence. Use when a change needs an owner, and after the server is up.
---

# seikika owner

1. Pass the entity id.
2. Read the owners and the CODEOWNERS rule that matched.
3. Ask that owner. Do not invent an owner the command did not return.

```
seikika owner command:validate -o json
```

Do not treat a missing owner as permission to merge. If it fails, the server is down or the id is unknown.
