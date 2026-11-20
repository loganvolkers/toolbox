---
name: push
description: >-
  Sends a portal's fixtures to a running server through the ingest API and waits for the snapshot that contains them. Use when a local server should hold the portal's fixture data.
---

# toolbox push

1. Start the server first, and note its URL.
2. Run the command against that URL. It sends one batch per connector, then native values.
3. Read the snapshot id it prints. Do not assume the data is visible before that snapshot.

```
toolbox push portal --url http://127.0.0.1:8080
```

Do not push fixtures at a server that already holds real catalog data. If it fails, the server is down or the URL is wrong.
