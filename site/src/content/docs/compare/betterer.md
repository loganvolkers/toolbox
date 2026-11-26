---
title: Seikika ratchets vs Betterer
description: When to use Seikika report ratchets or Betterer for preventing quality regressions.
sidebar:
  order: 6
relationship: replace
peer: Betterer
peer_url: https://github.com/phenomnomnominal/betterer
last_verified: 2026-11-23
claim_sources:
  - https://github.com/phenomnomnominal/betterer
  - https://github.com/seikika/seikika/blob/main/docs/design/ratchets.md
---

Both projects stop quality from sliding backward. [Betterer](https://github.com/phenomnomnominal/betterer) pioneered committed results and better / same / worse for TypeScript repositories. Seikika applies the same ideas to scorecard outcomes — with owners, levels, and a path to fleet views — and still reads Betterer's results file.

## Quick glance

| | Betterer | Seikika |
|---|---|---|
| License / form | MIT, Node CLI | Open-source CLI + portal (`report --baseline`) |
| What is ratcheted | Test values and per-file issues | Scorecard levels, evidence counts, optional numeric measures |
| Results file | `.betterer.results` (committed) | Baseline JSON (for example `portal/results.json`) |
| Outcomes | better / same / worse | better / same / worse, plus levels and `unknown` |
| CI staleness | `ci` mode fails when the file drifts | `--ci` fails on regressions and a stale baseline |
| Owners and levels | Not in the results model | Scorecards carry owners, bronze → gold, campaigns |
| Ecosystem | TypeScript / JavaScript tests in Node | Any fact a connector or tool slot can supply |
| Fleet view | Per repository | Local today; fleet roll-up designed with federation |

## What we share

- A committed snapshot of “how bad is it,” reviewed like test output.
- Improvement is allowed without waiting for a clean pass.
- CI should fail when someone forgets to commit an improvement, not only when things get worse.
- Teams should be able to keep an existing ratchet while adopting something broader.

## Where they differ

### Job boundary

Betterer is excellent at ratcheting *one repository's* Betterer tests. Seikika ratchets *scorecard* outcomes that can span languages, tools, and (with federation) many repositories. That is a different job, built on Betterer's ideas rather than a fork of its runtime.

### Integrate, don't erase

The `workspace` connector can parse `.betterer.results`. Teams that already enforce with Betterer keep that enforcement; Seikika reports the state into portal scorecards. See [ratchets.md](https://github.com/seikika/seikika/blob/main/docs/design/ratchets.md).

### Box, not wrapper

Linters and formatters stay their own tools. Seikika carries them and ratchets what their findings mean for a standard. Betterer remains the right Node-native ratchet when that is all you need.

## Good for / not for

### Choose Betterer when

- The repository is TypeScript / JavaScript and Betterer already fits the workflow.
- You want Betterer's editor and test UX for custom constraints in Node.
- A fleet scorecard model would be overhead for a single team's ratchet.

### Choose Seikika ratchets when

- You need owners, levels, and campaigns on top of “don't get worse.”
- Findings come from many tools or languages, not only Betterer tests.
- You want the same baseline shape in CI today and in a portal catalog later.

## Coexistence

Use both: keep `.betterer.results` as the team’s enforcement, and let Seikika read it for roll-up. A migration that deletes Betterer is optional, not required. Full “replace Betterer everywhere” importers are not a product goal; reading the results file is.

## What we learned

We learned the ratchet from [Betterer](https://github.com/phenomnomnominal/betterer): commit the results, treat change as better / same / worse, fail CI when the file is stale, and review improvements like a snapshot. Seikika `report --baseline` carries those ideas into scorecard levels and fleet reporting, and can still read `.betterer.results` so teams that already use Betterer keep their enforcement.

More on [Acknowledgments](/compare/acknowledgments/).

## Last verified

**2026-11-23**, against [docs/design/ratchets.md](https://github.com/seikika/seikika/blob/main/docs/design/ratchets.md) and the Betterer project's public documentation and repository. Re-check those sources when either side changes the results format or CI modes.
