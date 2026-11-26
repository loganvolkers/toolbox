---
title: What Seikika supports
description: Choose a repo portal, served portal, federated action, or static report and check feature availability.
sidebar:
  order: 1
---

Choose the way to run Seikika based on what your readers need. A single repository can run checks in CI. A served portal adds a live catalog. A parent portal can offer actions from child repositories, and a static host can serve a saved report.

These capabilities are available in the [0.2 source build](/start/install/#build-from-source). For downloadable builds, check [Releases and upgrades](/start/versions/).

| You need to | Use | Start here |
|---|---|---|
| Score a checkout and catch regressions | A repo portal with `seikika report` and a committed baseline | [Quickstart](/start/quickstart/) |
| Read a current catalog, with permissions for each caller | A served portal and its browser, CLI, HTTP, or Model Context Protocol (MCP) interface | [Explore a portal](/start/portal/) |
| Ask a connector for updated facts about one entity | A targeted refresh and a read on the new snapshot | [Refresh an entity](/users/refresh/) |
| Offer repository actions from one place | A parent portal that discovers, approves, and runs child actions | [Federate repository actions](/operators/federate-actions/) |
| Define a change and check that it worked | An action with inputs, an executor, and tests | [Define a self-service action](/operators/actions/) |
| Share a snapshot that works offline or on a static host | One HTML file, or a snapshot published through Astro | [Generate and host a static report](/operators/publish/) |
| Measure improvement against an existing standard | A campaign with a goal over one or more scorecards | [Track progress and change a standard](/operators/standards/) |
| Record a judgment that needs human or agent review | A rubric and an assessment tied to a specific entity version | [Assess a catalog entity](/operators/assessments/) |

## Planned capabilities

- Action federation is available today. Catalog and score roll-up across children and a static index across their reports are proposed. [Federation design](https://github.com/seikika/seikika/blob/main/docs/design/federation.md)
- The built-in browser interface lists available actions on an entity. Use the CLI, HTTP, or MCP for the supported action operations. Dedicated action and approval pages, shared run ownership across server instances, and resumable executor steps remain planned. [Federated action design](https://github.com/seikika/seikika/blob/main/docs/design/federated-actions.md#72-deferred-with-the-reason)
- Seikika exports a static report page and the data for your own site. Native multi-page generation remains proposed. [Static site and federation proposals](https://github.com/seikika/seikika/blob/main/docs/design/repo-portals.md#5-proposed-a-static-site-and-static-federation)
- Operators can plan against local fixtures. Live remote planning and managed preview environments remain planned. [Change a portal](/operators/portal/)
