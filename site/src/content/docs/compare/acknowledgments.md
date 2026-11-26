---
title: Acknowledgments
description: Projects Seikika learned from — Betterer, Backstage, OpenSSF Scorecard, Port Ocean, and peers.
sidebar:
  order: 10
relationship: acknowledgments
last_verified: 2026-11-23
claim_sources:
  - https://github.com/phenomnomnominal/betterer
  - https://backstage.io/
  - https://github.com/ossf/scorecard
  - https://github.com/port-labs/ocean
---

Short thanks for the projects that shaped Seikika. Comparison pages use the same copy so gratitude does not drift into sales language.

## Betterer

We learned the ratchet from [Betterer](https://github.com/phenomnomnominal/betterer): commit the results, treat change as better / same / worse, fail CI when the file is stale, and review improvements like a snapshot. Seikika `report --baseline` carries those ideas into scorecard levels and fleet reporting, and can still read `.betterer.results` so teams that already use Betterer keep their enforcement.

See [Seikika ratchets vs Betterer](/compare/betterer/). Design notes: [ratchets.md](https://github.com/seikika/seikika/blob/main/docs/design/ratchets.md).

## Backstage

[Backstage](https://backstage.io/) defined the internal developer portal as a product category: a software catalog next to the code, docs-like-code, and a plugin ecosystem that proved platforms can grow in the open. Seikika keeps the catalog envelope so `catalog-info.yaml` stays portable, and builds the agent-first, config-as-code shape we wanted next.

See [Seikika vs Backstage](/compare/backstage/). Research notes: [research.md](https://github.com/seikika/seikika/blob/main/docs/research.md).

## OpenSSF Scorecard

[OpenSSF Scorecard](https://github.com/ossf/scorecard) made software supply-chain scoring accessible: one tool, public checks, and scores that work for a single repository and for a fleet. Seikika carries Scorecard unchanged, ratchets its outcomes in portal context, and treats it as the template for opinionated standards in other domains.

See [Seikika and OpenSSF Scorecard](/compare/openssf-scorecard/). Design notes: [openssf-template.md](https://github.com/seikika/seikika/blob/main/docs/design/openssf-template.md).

## Port Ocean

[Port Ocean](https://github.com/port-labs/ocean) showed how to sync messy third-party APIs into a catalog with a small integration contract and declarative mappings. We study that connector thinking without adopting a database-held portal config: Seikika keeps configuration in code and carries tools rather than wrapping them.

See [Seikika vs Port](/compare/port/). Review notes: [ocean.md](https://github.com/seikika/seikika/blob/main/docs/ocean.md).

## Others we cite

When a compare page leans on a standard slot or stewardship model, credit travels with the citation:

- **SARIF** — the findings slot tools already speak; Seikika prefers standard slots over custom extractors ([seikika-and-tools](https://github.com/seikika/seikika/blob/main/docs/design/seikika-and-tools.md)).
- **CNCF** — Backstage's stewardship path showed portal software can live as open infrastructure.

## Last verified

**2026-11-23.** Shared paragraphs live under `site/src/compare-snippets/` (outside the published content tree, so they are not scored as site pages) so hub, peer pages, and this page stay aligned.
