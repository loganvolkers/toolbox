---
title: Connector directory
description: Find a Seikika connector, see what it reads, and follow its setup guide.
sidebar:
  order: 1
---

Connect your portal to source control, local tools, or an identity provider. A **connector** sends records, such as repository details or group membership. Your portal's manifest defines how those records become catalog entities and scorecard inputs.

The Seikika project maintains the connectors listed here. These guides use the 0.2 source. Build the CLI and the connector binary from the same revision. [Install a connector](/start/install/#install-a-connector) · [Check release availability](/start/versions/)

## Source control

| Connector | What it reads |
|---|---|
| [GitHub](/connectors/github/) | Repositories, code owners, app installations, and GitHub Actions run statistics |
| [GitLab](/connectors/gitlab/) | Groups, direct group members, and projects within a group |

## Files and tools

| Connector | What it reads |
|---|---|
| [Git files](/connectors/git-files/) | Backstage catalog descriptors and selected file paths |
| [Workspace](/connectors/workspace/) | Files, repository settings, and workspace metadata |
| [Runner](/connectors/runner/) | Output from tests, linters, and other commands in a checkout |

## Identity

| Connector | What it reads |
|---|---|
| [SCIM](/connectors/scim/) | User and group changes that an identity provider sends |

Seikika starts the source control and local tool connectors as child processes. This is the `in-image` runtime. You run SCIM as a separate service with the `remote` runtime.

To use connector records in your catalog, declare their kinds under `connectors` and add mappings under `kinds`. For a source that requires credentials, use environment variables or secret references in the manifest. [Portal manifest reference](/operators/manifest/) · [Connector secrets](/operators/secrets/)

To add a new data source, follow the connector author guide. For a connector that ships with Seikika, add a guide to this directory so it passes the public guide rule. [Write a connector](/authors/connectors/) · [First-party connector scorecard](https://github.com/seikika/seikika/blob/main/docs/design/connector-standards.md)
