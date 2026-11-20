---
title: The catalog
description: Entities, kinds and connectors. Facts live in the connector. Rules live in the manifest.
---

A portal's catalog is a set of entities. Each entity has a kind, an id and fields. A **connector** reports facts from a system of record: a checkout, GitHub, a directory of skills. The portal manifest maps those records onto entities.

The split is the point. The connector says what is true (`this file has no title`, `this workflow uses an unpinned action`). The manifest says what is required.

## Kinds you will see

| Kind | A typical entity |
|---|---|
| Crate | One Cargo package in the workspace |
| Doc | One markdown document in the repository |
| Scorecard | A ladder of rules for one kind |
| Site page | One page of this site |

Relations come from the same facts. A document that names a command is linked to that command, so an agent can walk from a failing score to the file that should change.

## This site is in the catalog

Each page under `site/src/content/docs` is a `site-page` entity. Adding a page adds an entity. The [scorecard](/toolbox/concepts/scorecards/) then says whether that page is titled, linked, and reachable from the sidebar.
