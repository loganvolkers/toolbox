---
name: graph
description: >-
  Lists the neighbours of one entity. Use when an agent needs relations around an id, and after the server is up.
---

# toolbox graph

1. Pass the entity id. --depth is 1, 2 or 3.
2. Narrow the walk with --relation (one relation name, e.g. dependsOn) and --direction (in, out or both; default both).
3. Add -o json and read the edges before walking further.
4. Stop at the depth asked for. Do not follow every neighbour recursively in the client.

```
toolbox graph crate:toolbox-expr --depth 1 -o json
```

Everything that depends on a crate, up to three hops away, in one call:

```
toolbox graph crate:toolbox-expr --relation dependsOn --direction in --depth 3 -o json
```

Do not set depth above 3. If it fails, the id is unknown or the server is down.
