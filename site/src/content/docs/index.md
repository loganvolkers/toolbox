---
title: Seikika
description: Score a repository or serve a software catalog that developers and agents can use.
---

Seikika is an open-source framework for building an internal developer portal in code. Use it to track software and ownership, see which standards a repository meets, and offer self-service actions. You can start by scoring one repository with the command-line interface (CLI), then run a portal that developers and agents can query.

Bring data into your portal from GitHub, GitLab, local tools, or an identity provider. [Browse connectors](/connectors/) · [Set up GitHub](/connectors/github/)

## Start with one task

| If you want to | Start here |
|---|---|
| Score a repository and catch regressions in CI | [Quickstart](/start/quickstart/) |
| Start a local portal and look up a service | [Explore a portal](/start/portal/) |
| Configure, deploy, or maintain a portal | [Build a portal](/operators/) |
| Offer actions from several repositories | [Federate repository actions](/operators/federate-actions/) |
| Publish a report on a static host | [Generate and host a static report](/operators/publish/) |
| Share standards or write a connector | [Publish a standard](/authors/) |
| Use a portal from a script, an app, or an agent | [Use a portal](/users/) |
| Change Seikika or contribute to the docs | [Build Seikika](/maintainers/) |

In a repo portal, a **scorecard** groups rules such as “include a README.” The report shows which rules failed, what Seikika found, and what you can change. Save those results in Git to catch regressions in continuous integration (CI).

A running portal serves a **catalog** of software, teams, and their relationships. You can query it from the web interface, the CLI, the HTTP API, or Model Context Protocol (MCP).

## Explore further

- [Install Seikika](/start/install/) or check [what Seikika supports](/concepts/capabilities/).
- [Browse this repository's report](/demo/report/) to see scores, evidence, and fixes.
- [Compare Seikika with Backstage and other tools](/compare/) and see how [developer experience benchmarks](/concepts/dx-benchmarks/) are run.
- [Find a command](/start/cli/), [look up a term](/concepts/glossary/), or [troubleshoot a problem](/start/troubleshooting/).
