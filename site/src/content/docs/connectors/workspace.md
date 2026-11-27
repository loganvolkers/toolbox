---
title: Workspace connector
description: Read files and repository metadata from a checkout to score or catalog them.
sidebar:
  order: 5
---

Use Workspace to collect facts from a checkout, such as source file sizes, workflow settings, and Cargo workspace members. These facts can feed a repo portal's scorecards. Use the [Runner connector](/connectors/runner/) when you need results from a command.

## Read matching files

You need the Seikika CLI and `seikika-connector-workspace` from the same 0.2 source revision. Start in a checkout with a portal manifest and at least one Markdown file under `docs/`. [Install a connector](/start/install/#build-from-source) · [Repo portal quickstart](/start/quickstart/)

Add this entry to the `connectors` list in `seikika.yaml`. The `**` pattern includes nested directories. Set `crates: false` to read files without running Cargo metadata.

```yaml
connectors:
  - name: workspace
    package: builtin:workspace
    runtime: in-image
    config:
      root: .
      crates: false
      files: ["docs/**/*.md"]
    records: [file]
```

From the checkout root, read the records. If your manifest is in `.seikika/`, pass `--portal .seikika` in place of `--portal .`.

```bash
seikika connector records --portal . --name workspace
```

Look for a `file` record with an external ID such as `file:docs/setup.md`. Its data includes the path and line count. The connector can resync a full set of records and refresh by external ID.

## Choose other inputs

Set `docs` to Markdown file patterns for `doc` records with links and other document facts. Set `repo: true` to collect facts about the README, license, and repository settings. Leave Cargo metadata enabled to get a `crate` record per workspace member. The reference lists every option, including dependency, workflow, and public site records. [Workspace configuration and record reference](https://github.com/seikika/seikika/blob/main/docs/reference/workspace-connector.md)

Workspace reads the local tree for these checks. A parent portal can also fetch repositories configured for federation and read them as `portal` records. [Federate repository actions](/operators/federate-actions/)

Set `githubOrg` to score an organization from GitHub's default branches with a single parent scorecard. [Organization scorecards](/demo/organization/)

## Troubleshoot missing records

- If no files match, check your working directory, `root`, and any `exclude` patterns. Patterns are relative to `root`.
- If Cargo metadata fails on a checkout without a Cargo workspace, set `crates: false`. On a Rust project, check that Cargo is on the path and that `cargo metadata` runs in the checkout.
- If your report lacks entities, check that your manifest declares the emitted kinds and maps their fields. [Portal manifest reference](/operators/manifest/)
