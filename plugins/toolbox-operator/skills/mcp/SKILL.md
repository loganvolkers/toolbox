---
name: mcp
description: >-
  Serves the portal's MCP endpoint over stdio. Use when a harness should call the portal's tools through a local process, and the server is already running.
---

# toolbox mcp

1. Point TOOLBOX_URL at the running server, or pass --url.
2. Leave the process attached to the harness. It speaks MCP on stdin and stdout.
3. Stop the process to disconnect. Do not parse its stdout as a one-shot JSON report.

```
toolbox mcp --url http://127.0.0.1:8080
```

Do not run this in a terminal and type at it. If it fails, the server URL is wrong.
