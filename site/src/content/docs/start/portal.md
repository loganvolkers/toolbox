---
title: Explore a portal
description: Open a local catalog, check repository ownership, and explain a scorecard result.
sidebar:
  order: 5
---

Run a portal with practice data and find out why a repository has no verified owner. You need a Bash terminal, a [Seikika source checkout and CLI](/start/install/#build-from-source), and the [OpenFGA server binary](https://github.com/openfga/openfga/releases/tag/v1.22.1). The example uses OpenFGA to show the same permission checks a deployed portal can apply.

## 1. Start the portal

From the Seikika source directory, set `SEIKIKA_OPENFGA_BIN` to the absolute path of your OpenFGA binary. Then run:

```bash
seikika serve examples/reference-portal --fixtures --listen 127.0.0.1:8080
```

Keep this terminal open. The server uses sample data called **fixtures** and stores the catalog in memory. The address accepts connections from your machine only.

Open [the local portal](http://127.0.0.1:8080/) in your browser and choose a fixture identity if the interface asks. You can also check that the catalog is ready from a second terminal:

```bash
curl -fsS http://127.0.0.1:8080/readyz
```

Look for `"ready":true` and a snapshot identifier. If the server fails to start, see [portal troubleshooting](/start/troubleshooting/#running-a-portal).

## 2. Check an ownership claim

In the second terminal, make sure `seikika` is on `PATH`. Query the catalog as Alice:

```bash
seikika owner repository:ledger --url http://127.0.0.1:8080 --as alice
```

The result says the ownership claim is unverified. The repository's descriptor names `group:payments`, but its CODEOWNERS rule for the descriptor names `@acme/platform`. The portal shows the mismatch so the team can decide which file to fix.

## 3. Explain a score

Ask why Bob's repository reached bronze for ownership.

```bash
seikika score repository:checkout-web --url http://127.0.0.1:8080 --as bob
seikika explain repository:checkout-web --rule ownership/branch-protected \
  --url http://127.0.0.1:8080 --as bob
```

The explanation shows `defaultBranchProtected: false`. Branch protection is a silver rule in this portal. The failed rule keeps the repository at bronze.

Press **Ctrl+C** in the first terminal to stop the portal. Continue with the [HTTP API reference](/users/api/), [MCP guide](/agents/reads/), or [deployment guide](/operators/deploy/).
