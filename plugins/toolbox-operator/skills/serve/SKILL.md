---
name: serve
description: >-
  Runs the Toolbox server: the snapshot loop and the REST API. Use when a local portal should answer read commands, and when an in-memory run with fixtures is enough.
---

# toolbox serve

1. Start the server on the portal directory. Without a database URL the store is in memory.
2. Pass --fixtures to load the portal's fixtures at start.
3. Stop the process to discard an in-memory store. Do not expect that store to survive a restart.
   Action runs do survive one: they are kept in the database, or without one under
   `$TOOLBOX_STATE_DIR/served/<portal>/runs`. A run whose executor the restart cut off is failed.

```
toolbox serve portal --fixtures
toolbox serve portal --database-url "$DATABASE_URL" --listen 127.0.0.1:8080
```

Do not point this at a production database from a laptop without meaning to. If it fails to bind, another process holds the port.
