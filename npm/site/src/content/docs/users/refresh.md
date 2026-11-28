---
title: Refresh an entity
description: Ask a connector to read one entity again and check the resulting catalog snapshot.
sidebar:
  order: 4
---

Refresh an entity after you change its source, such as fixing a broken link in a repository. Seikika asks the connectors that support a targeted read to collect the new facts. It then waits for a catalog snapshot that shows the result.

## Try a refresh in a practice repository

You need the [0.2 source CLI and workspace connector](/start/install/#build-from-source) and the practice repository from [Quickstart](/start/quickstart/). Finish the quickstart so `portal.md` exists and the link in `README.md` works.

In your first terminal, start a local portal from `seikika-practice`. This reads the checkout through the workspace connector.

```bash
seikika serve .seikika --listen 127.0.0.1:8080
```

In a second terminal with Seikika on `PATH`, enter the same practice repository. Check the current document, move its linked file, and ask the connector to read the document again.

```bash
seikika get doc:README.md --url http://127.0.0.1:8080 --as alice --fields brokenLinks -o json
mv portal.md moved-portal.md
seikika refresh doc:README.md --url http://127.0.0.1:8080 --as alice -o json
seikika get doc:README.md --url http://127.0.0.1:8080 --as alice --fields brokenLinks -o json
```

The first read shows an empty `brokenLinks` list. The refresh shows a completed connector command and a new snapshot. The second read lists `portal.md`. Its provenance identifies the workspace connector as the source of the field.

Restore the link and refresh once more.

```bash
mv moved-portal.md portal.md
seikika refresh doc:README.md --url http://127.0.0.1:8080 --as alice
```

Press **Ctrl+C** in the first terminal to stop the portal.

## Refresh a shared catalog

Use your portal's URL and credentials. Refresh only the entity you need and compare the before and after snapshots. To check which connector supplied a field, use `seikika get ENTITY_ID -o json` and inspect `provenance`. Replace `ENTITY_ID` with the entity you are checking.

If Seikika reports `not-refreshable`, ask the operator whether a connector for this entity supports a targeted refresh and is registered. If you lack `can_refresh`, ask the operator to check the portal's [permissions](/operators/access/). A successful refresh can leave a field unchanged when the source has the same value.

See [snapshot reads](/users/api/#snapshots-and-errors) to make a later HTTP read wait for a particular snapshot.
