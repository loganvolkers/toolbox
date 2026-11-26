---
title: Use a portal
description: Look up software, check ownership and scores, and build on a portal's catalog.
sidebar:
  order: 1
---

Use the catalog to find software, check who owns it, and understand why it received a score. Your operator supplies the portal URL and tells you how to sign in. To practice with sample data, start with [Explore a portal](/start/portal/).

## Choose a way to read the catalog

| You want to | Use |
|---|---|
| Browse the catalog as a person | The portal URL in a browser |
| Check an owner, a score, or the evidence behind a rule | The [CLI reference](/start/cli/) |
| Build a script or dashboard | The [HTTP API reference](/users/api/) |
| Connect an agent | [Read the catalog from an agent](/agents/reads/) |
| Request a change the portal offers | [Test and run an action](/users/actions/) |

## Read an owner or score

For a token-based portal, set `SEIKIKA_URL` in your shell and load `SEIKIKA_TOKEN` from your secret store as your operator directs. Replace `repository:checkout-web` below with your entity's ID. In the local tutorial, use the loopback URL and add `--as bob` to each command.

```bash
seikika owner repository:checkout-web
seikika score repository:checkout-web -o json
seikika explain repository:checkout-web --rule ownership/branch-protected
```

`owner` explains the effective owner and any evidence behind it. `score` returns the levels for one entity. `explain` can show the evidence and fix for one rule. If a rule is hidden, your client may receive a redaction marker. Handle that marker separately from a missing field or a failing rule.

## Build on the results

A script can compare a level against your team's minimum and fail a build if the level is too low. Define how the script handles `unknown`, skipped, and redacted results. Use your portal's real level names. The fixture catalog uses levels such as `bronze` and `silver`.

The catalog changes as connectors report new facts and operators update the manifest. Record the snapshot ID if you need to know which catalog version answered a request. Use `min_snapshot` on HTTP endpoints that accept it when your read needs to see a particular update. See the [HTTP API reference](/users/api/#snapshots-and-errors) for response and retry details.
