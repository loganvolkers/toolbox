---
title: Read the catalog from an agent
description: Connect an MCP client, discover its tools, and read a scorecard with the caller's permissions.
sidebar:
  order: 1
---

Connect an agent to a running portal with Model Context Protocol (MCP). Start with [Explore a portal](/start/portal/) for fixture data or ask your operator for the shared portal URL and credentials.

## Connect a client

Use the client settings for a Streamable HTTP server. Set the URL to the portal's `/mcp` endpoint. A token-based portal expects `Authorization: Bearer …`. Use your client's secret store for the token. For local fixture use, set `x-seikika-user` to the fixture name, such as `bob`.

If your client uses standard input and output (stdio), configure it to run the CLI as a subprocess. For the loopback example, use this command and arguments:

```bash
seikika mcp --url http://127.0.0.1:8080 --as bob
```

For a token-based portal, load `SEIKIKA_TOKEN` into the subprocess environment and set `--url` to the shared portal's HTTPS root. The CLI connects to `/mcp` for you.

## Discover the tools and read a score

Ask your client to list tools. It should show names such as `get`, `owner_of`, `scorecard_status`, and `explain`. Look up their input fields and CLI equivalents in the [MCP tool reference](/reference/mcp/).

Call `scorecard_status` with the following arguments on the fixture portal:

```json
{
  "id": "repository:checkout-web",
  "scorecard": "ownership"
}
```

Look for a bronze ownership level. As Bob, you can use `explain` with `id` set to `repository:checkout-web` and `rule` set to `ownership/branch-protected` to see why the next level failed. The portal applies the same permissions to MCP and HTTP reads. A response can show the level and redact rule details.

If you need a newer source value, ask the portal to [refresh one entity](/users/refresh/). For writes, use `list_actions` and `preview_action` to see what the portal offers and review the effects. `run_action` requires the preview token. Use `get_run` to check the result. Pass approval requests to an eligible person. See [Test and run an action](/users/actions/).
