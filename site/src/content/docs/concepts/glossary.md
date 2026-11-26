---
title: Glossary
description: Look up terms used across the Seikika docs.
sidebar:
  order: 5
---

| Term | Meaning |
|---|---|
| Action | A change you can request through a portal, with declared inputs, effects, and any approvals |
| Baseline | A saved report used to check future results for regressions |
| Catalog | A collection of entities and their relationships |
| Connector | A program that reads a source and reports records to Seikika |
| Entity | A catalog item with a kind, an ID, and fields |
| Evidence | The facts or findings that explain a rule result |
| Fixture | Sample input used to test or explore a portal |
| Kind | An entity type such as `Repository` or `Doc` |
| Manifest | The `seikika.yaml` configuration and files it includes |
| MCP | Model Context Protocol, a way for agent clients to discover and call portal tools |
| Package | Versioned configuration and tests another portal can include |
| Portal | A configured catalog and the standards and actions associated with it |
| Ratchet | A rule comparison that catches regressions against a baseline |
| Repo portal | A portal configuration you run in a repository with `seikika report` |
| Scorecard | Rules for one kind, with levels that show which requirements an entity meets |
| Snapshot | A published catalog state identified by an ID |
| Unknown | A rule result for a check that could not finish, such as a missing tool |

See [The catalog](/concepts/catalog/), [Scorecards](/concepts/scorecards/), and [the four software development lifecycles](/concepts/layers/) for examples.
