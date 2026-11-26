---
id: D01M82E93XAPNE0WFXEPJZ59WG4
decided: 2026-11-22T22:00:00Z
---

# Carry OpenSSF Scorecard instead of reimplementing it; OpenSSF is the template for other domains

**Seikika doesn't define its own scorecard where an established standard covers the domain. It
carries the standard and its tool unchanged, and adds only what the box adds: the ratchet,
campaigns, root causes and the command that reproduces each result. OpenSSF Scorecard is the
first, through `packages/openssf-scorecard` (Scorecard 5.7.0, `--local`, a `scorecard` fitting in
the runner). Seikika for Seikika includes it and retires its own versions of SAST and
Security-Policy (`maintainer-supply-chain`'s `code-scanning` and `security-policy`, from D85).**

1. **Why not keep our own rules:** they were weaker copies of Scorecard's checks. For example,
   Scorecard's Security-Policy also asks for linked content. Two scores for one question would
   compete, which is what Seikika is not for.
2. **What stays:** `dependabot` checks each ecosystem, which Scorecard's
   Dependency-Update-Tool doesn't. `workflows-have-owners` has no `--local` equivalent.
   `code-scanning-pinned` stays until Pinned-Dependencies reaches 10, because it holds the
   level the repository's gold depends on.
3. **The template:** the parts of an opinionated standard (facts, judgements, documentation, tests,
   distribution) map onto Seikika's packages, so others can build OpenSSF-like standards for
   observability, quality, correctness or architecture conformance
   ([openssf-template.md](../design/openssf-template.md) §3).

**Status:** ✅ amends D85

**Where:** [docs/design/openssf-template.md](../design/openssf-template.md),
[packages/openssf-scorecard](../../packages/openssf-scorecard/README.md)
