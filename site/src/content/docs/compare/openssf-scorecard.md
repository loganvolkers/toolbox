---
title: Seikika and OpenSSF Scorecard
description: Use OpenSSF Scorecard results in a Seikika portal.
sidebar:
  order: 7
relationship: complement
peer: OpenSSF Scorecard
peer_url: https://github.com/ossf/scorecard
last_verified: 2026-11-26
claim_sources:
  - https://github.com/ossf/scorecard
  - https://github.com/seikika/seikika/blob/main/docs/design/openssf-template.md
---

OpenSSF Scorecard checks the supply chain security practices of a repository. Use its CLI or GitHub Action for a standalone score. Use Seikika's package when you want to include those results in a broader set of standards.

The Seikika package runs the pinned Scorecard executable. Scorecard owns the checks and their 0–10 results. The package maps those results into the portal so Seikika can report evidence, track levels, and catch changes against a baseline.

- Read [OpenSSF's check documentation](https://github.com/ossf/scorecard/blob/main/docs/checks.md) to understand what a check measures and how to improve a score.
- Read [the Seikika package README](https://github.com/seikika/seikika/tree/main/packages/openssf-scorecard) for the current pin, configuration, and tool prerequisites.
- See [Publish a standard](/authors/) to include a package in a portal.

Seikika uses Scorecard's published checks and per-repository results as a model for reusable standards. See [Acknowledgments](/compare/acknowledgments/).
