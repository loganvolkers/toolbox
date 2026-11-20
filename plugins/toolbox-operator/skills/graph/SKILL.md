---
name: graph
description: >-
  Lists the neighbours of one entity. Use when an agent needs relations around an id, and after the server is up.
---

# toolbox graph

1. Pass the entity id. --depth is 1, 2 or 3.
2. Add -o json and read the edges before walking further.
3. Stop at the depth asked for. Do not follow every neighbour recursively in the client.

```
toolbox graph crate:toolbox-expr --depth 1 -o json
```

Do not set depth above 3. If it fails, the id is unknown or the server is down.
