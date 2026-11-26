---
name: serve
description: >-
  Runs the Seikika server: the snapshot loop and the REST API. Use when a local portal should answer read commands, and when an in-memory run with fixtures is enough.
---

# seikika serve

1. Start the server on the portal directory. Without a database URL the store is in memory.
2. Pass --fixtures to load the portal's fixtures at start.
3. To keep it running in the background, add `--detach`: it returns once the server answers and
   prints its URL, pid, log, approval key and where run records are kept. `seikika serve <portal> --stop` stops it; start it
   again the same way. It needs no terminal multiplexer or `nohup`.
4. Stop the process to discard an in-memory store. Do not expect that store to survive a restart.
   Action runs do survive one: they are kept in the database, or without one under
   `$SEIKIKA_STATE_DIR/served/<portal>/runs`. A run whose executor the restart cut off is failed.

5. On a loopback address, people sign in with `--as` on each request. Anywhere else, set
   `SEIKIKA_AUTH_MODE` (`seikika serve --help` lists the modes).
6. A portal with `federation.children` keeps a clone of each child under `$SEIKIKA_STATE_DIR`
   and reads it again before it lists, previews or runs an action. It signs approvals with a key
   it keeps there too; `--detach` and `seikika run --url <server>` print it, for children to pin.
7. Each in-image connector starts in its own Landlock domain. Where the host has no Landlock,
   `/readyz` counts the connectors that started unconfined, and `/v1/connectors` names them. Set
   `SEIKIKA_CONNECTOR_CONFINEMENT=required` to refuse to start them instead.

```
seikika serve portal --fixtures
seikika serve org --listen 127.0.0.1:8080 --detach
seikika serve org --stop
seikika serve portal --database-url "$DATABASE_URL" --listen 127.0.0.1:8080
```

Do not point this at a production database from a laptop without meaning to. If it fails to bind, another process holds the port.
