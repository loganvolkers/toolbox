---
title: Seikika and OpenSSF Scorecard
description: How Seikika carries OpenSSF Scorecard — it does not replace the security scoring tool.
sidebar:
  order: 7
relationship: complement
peer: OpenSSF Scorecard
peer_url: https://github.com/ossf/scorecard
last_verified: 2026-11-23
claim_sources:
  - https://github.com/ossf/scorecard
  - https://github.com/seikika/seikika/blob/main/docs/design/openssf-template.md
---

This is **not a replacement**. [OpenSSF Scorecard](https://github.com/ossf/scorecard) remains the security scoring tool: its checks, scores, and documentation stay OpenSSF's. Seikika pins, runs, remembers, and ratchets those outcomes in portal context.

## Quick glance

| | OpenSSF Scorecard | Seikika |
|---|---|---|
| Owns | Probes, checks, 0–10 scores, public docs | Portal catalog, packages, baselines, campaigns |
| Runs as | CLI, GitHub Action, API, `--local` | Carries Scorecard via the `openssf-scorecard` kit |
| Config locus | Scorecard's own flags and policies | Portal manifest `include` + tool pins |
| Over time | Snapshot scores (API history elsewhere) | `report --baseline` ratchet and campaigns |
| Scope | Software supply-chain security | Any standard the box can carry; Scorecard is the template |

## How Seikika carries Scorecard

- **Pin and run** the Scorecard binary as a tool in the box (same command you can run alone).
- **Remember** scores in the portal report and fail CI when a check drops (`maintainer-openssf` in this repository).
- **Campaign** toward higher scores without inventing parallel security checks.
- **Template** other domains: opinionated standards should look like Scorecard — one tool, public judgements, evidence — while Seikika supplies the box ([openssf-template.md](https://github.com/seikika/seikika/blob/main/docs/design/openssf-template.md)).

DX Benchmarks include a security-scorecards arm that compares agent-built scorecards with OpenSSF Scorecard; see [DX Benchmarks](/concepts/dx-benchmarks/).

## When to use Scorecard alone

- You only need supply-chain scores for one repository, with Scorecard's CLI or Action.
- You are contributing checks or probes upstream — that work belongs in OpenSSF, not in a portal wrapper.
- You want the raw Scorecard JSON or SARIF without a catalog or ratchet yet.

## What we learned

[OpenSSF Scorecard](https://github.com/ossf/scorecard) made software supply-chain scoring accessible: one tool, public checks, and scores that work for a single repository and for a fleet. Seikika carries Scorecard unchanged, ratchets its outcomes in portal context, and treats it as the template for opinionated standards in other domains.

More on [Acknowledgments](/compare/acknowledgments/).

## Last verified

**2026-11-23**, against [docs/design/openssf-template.md](https://github.com/seikika/seikika/blob/main/docs/design/openssf-template.md) and the OpenSSF Scorecard repository. Re-check kit pins and Scorecard release notes when upgrading the carried binary.
