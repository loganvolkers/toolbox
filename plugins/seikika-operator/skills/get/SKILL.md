---
name: get
description: >-
  Reads one entity from a running portal, including where each field came from. Use when an agent needs the current value of one id, and after the server is up.
---

# seikika get

1. Confirm the server URL. The default is http://127.0.0.1:8080.
2. Pass the entity id. Add --fields when only some fields are needed, and -o json for a machine-readable result.
3. Read source annotations before treating a value as authored. Do not guess an id.

```
seikika get crate:seikika-expr -o json
```

Do not run this without a server. If it fails, the server is down or the id is unknown.
