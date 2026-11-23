---
id: D01M83G9F4EEGN1GTP14HEYB8B6
decided: 2026-11-23T08:00:00Z
---

# Close the review findings: safety rules in the floor, claims checked against facts, citations that resolve

**The `close-review-findings` campaign changed the standards its independent reviews failed,
rather than the reviews:**
1. Credential and hidden-instruction rules move from gold to bronze in all ten scorecards.
2. Claims are checked against facts:
   - a concurrency group must vary by ref and cancel in progress;
   - `continue-on-error` must have its outcome read;
   - a test must assert;
   - a stage's commands must resolve and its checks must run in CI;
   - a stage with no doc is missing whatever its status says.
3. An assessment's citations must resolve in the repository.
4. The meta scorecards gain the gold and silver rules the reviews asked for:
   - review cards: a stale failing review stays open, and gold needs a cited review;
   - meta-rule: evidence that tracks the check, and a fix that names a place;
   - meta-scorecard: described, and fixtures at every level;
   - meta-field: computed CEL within budget;
   - meta-connector: a map must read its record.

**Why:** a fresh, independent reviewer failed every one of the 16 meta-gold entities it
sampled, each with a concrete gaming route
([meta-standards-review.md](../design/meta-standards-review.md) §2). Fixing the review to agree
would have been the silent agreement the re-evaluation warned about.

**Thresholds** are justified in [thresholds.md](../design/thresholds.md). Each is measured on this
repository or labelled as a choice.

**Status:** ✅ amends D70, D72, D73, D87 and D01M831P5QWJQ4A2FT85N5MXJQF

**Where:** [docs/design/thresholds.md](../design/thresholds.md), [docs/design/meta-standards-review.md](../design/meta-standards-review.md) §6
