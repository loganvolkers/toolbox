---
title: The catalog
description: Follow a connector record through the manifest to a catalog entity and scorecard.
---

A catalog is the collection of software, people, and other things a portal tracks. Seikika calls each item an **entity**. Each entity has an ID, a kind, and fields.

## How a record becomes a score

1. A connector reads a source such as a Git checkout or an API. In the [repo starter](/start/quickstart/), the workspace connector examines Markdown files and reports broken links.
2. The manifest maps each Markdown record to a `Doc` entity. The README becomes `doc:README.md`, with a `brokenLinks` field.
3. The `docs` scorecard checks that `brokenLinks` is empty. If it contains `notes.md`, the report marks the rule as failing and shows that path as evidence.

Connectors collect facts. [The manifest](/operators/manifest/) defines what the portal does with those facts, including rules and relationships. A portal can also map a repository's ownership claim to a team, then show whether other records support the claim. [Explore a portal](/start/portal/) walks through that example.

Each page on this site becomes a `SitePage` entity in Seikika's own catalog. You can see its results in [this repository's report](/demo/report/). See the [glossary](/concepts/glossary/) for more terms.
