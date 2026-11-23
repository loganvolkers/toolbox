---
title: Toolbox and the linter/formatter ecosystem
description: How Toolbox carries ESLint, ruff, clippy, actionlint, zizmor, and peers — without replacing them.
sidebar:
  order: 8
relationship: complement
peer: Linters and formatters
last_verified: 2026-11-23
claim_sources:
  - https://github.com/loganvolkers/toolbox/blob/main/docs/design/toolbox-and-tools.md
---

This is **not a replacement**. ESLint, ruff, clippy, rustfmt, actionlint, zizmor, and the rest of the linter and formatter ecosystem remain the tools that check and rewrite code. Toolbox is the *box*: it pins them, runs them together, and remembers what they found ([toolbox-and-tools](https://github.com/loganvolkers/toolbox/blob/main/docs/design/toolbox-and-tools.md)).

## Quick glance

| | Linter / formatter | Toolbox |
|---|---|---|
| Owns | Rules, config file, CLI, editor integration | Which tools, versions, how they run, what findings mean for a standard |
| Output | Tool-native or SARIF / similar | Reads standard slots (SARIF, NDJSON, …) and mappings into entities |
| Config | `.eslintrc`, `ruff.toml`, `clippy` flags, … | `tools:` pins and package kits; tool config stays the tool's |
| Alone | Fully useful without a portal | Useful as `report` / doctor; not a substitute for the tool's CLI |

## How Toolbox standardizes and carries them

- **Take it out.** The command the box runs is a command you can run; configuration lives in the tool's own file when possible.
- **Standard slots first.** Prefer SARIF (and similar) so a new tool fits without a custom extractor.
- **Pin and doctor.** Declared versions and `toolbox doctor` so missing or wrong tools are visible.
- **Ratchet the meaning.** Scorecards and baselines decide levels and regressions; they do not rewrite the tool's findings.
- **Kits.** Packages can ship a set of tools, pins, and scorecards (for example GitHub Actions lint kits) for other portals to include.

## When to use the tool alone

- You are fixing a single repository in an editor or a one-off CI job.
- You are developing a new lint rule — that belongs in ESLint, clippy, semgrep, or another tool community.
- You need the tool's native UX (auto-fix, IDE extensions) without a catalog.

## Last verified

**2026-11-23**, against [docs/design/toolbox-and-tools.md](https://github.com/loganvolkers/toolbox/blob/main/docs/design/toolbox-and-tools.md). Re-check package READMEs and tool pins when naming specific versions in portal config.
