---
title: Seikika vs Backstage
description: Compare the two portal frameworks and plan a small catalog coexistence pilot.
sidebar:
  order: 2
relationship: replace
peer: Backstage
peer_url: https://backstage.io/
last_verified: 2026-11-26
claim_sources:
  - https://backstage.io/docs/overview/what-is-backstage/
  - https://backstage.io/docs/features/software-catalog/descriptor-format/
  - https://github.com/seikika/seikika/blob/main/experiments/backstage-compare/RESULTS.md
---

Backstage is an open-source framework for composing a portal application with plugins. Seikika is an open-source framework for configuring a catalog, standards, and actions in Git. You can start with Seikika's repository checks while your team keeps using an existing Backstage portal.

## Compare capabilities

| Area | Backstage | Seikika |
|---|---|---|
| Catalog | `catalog-info.yaml`, catalog providers, and a standard entity model | Portal-defined kinds and fields populated by connectors |
| Configuration | A TypeScript application, plugins, and `app-config` | A portal manifest and versioned packages |
| Developer workflows | Software Templates, TechDocs, and plugins | Repository reports, scorecards, and configured self-service actions |
| Integrations | A broad plugin and provider ecosystem | First-party connectors and a protocol for custom connectors |
| Deployment | Build, configure, and run the portal app | Run the CLI for one repo or deploy a shared service with Postgres |

See [Backstage's feature overview](https://backstage.io/docs/overview/what-is-backstage/), [Backstage deployment docs](https://backstage.io/docs/deployment/), and [Build a portal](/operators/).

Choose Backstage when your teams rely on its templates, TechDocs, or particular plugins. Try Seikika when you want a committed baseline for a set of repositories or want to review catalog and standards configuration in Git.

## Pilot alongside an existing catalog

1. Choose one repository. Record its Backstage entity reference, declared owner, and any dependency or lifecycle fields your team uses. Keep Backstage as the source for those values while you evaluate the pilot.
2. Run [Gate pull requests](/operators/ci/) on that repository to add a local standards baseline. This workflow reads the checkout and runs independently of Backstage.
3. If you also need a shared catalog, use the [reference portal mapping](https://github.com/seikika/seikika/blob/main/examples/reference-portal/seikika.yaml). It reads selected `catalog-info.yaml` fields, such as the title, lifecycle, tier, and ownership claim, and combines them with Git and CODEOWNERS facts. Your portal manifest defines the mapping and IDs.
4. Compare the resulting fields and ownership evidence with the Backstage entity. Check how both portals handle a conflicting owner and a user who lacks access. Document which system owns each field before relying on both catalogs.

A full Backstage importer and plugin migration remain planned product work. Existing descriptors can be inputs to a mapping. Backstage templates and permissions need their own evaluation.

## Read the product benchmarks

Seikika's published comparison used Backstage 1.57 and a pinned Seikika build to measure performance and deployment properties. In that environment, Seikika served the tested entity reads faster and used fewer resources. Backstage covered more capabilities. See the [benchmark methods, results, and limitations](https://github.com/seikika/seikika/blob/main/experiments/backstage-compare/RESULTS.md) and the [report's comparison view](/report/#compare).

The [documentation gap analysis](https://github.com/seikika/seikika/blob/main/docs/audits/public-docs-2026-11-26.md) evaluates a different question: how well the public docs help readers finish their work.
