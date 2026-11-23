---
id: D01M831P5QWJQ4A2FT85N5MXJQF
decided: 2026-11-23T03:00:00Z
---

# Meta gold means well-formed, not good; reviews are independent and findings come first

**The deterministic meta scorecards (`meta-rule`, `meta-scorecard`, `meta-kind`, `meta-field`,
`meta-connector`) measure the form of a standard: tested, explained, decided on fixtures,
described. They stay as the ratchet's floor for new standards, and their gold is called
"well-formed", never "good". Quality is judged by assessments (D79, D80), which stay advisory. Three
changes make those assessments count:**
1. every `review-*` scorecard puts an entity with a current failing review at `none`, so open
   findings come first;
2. a review written by the assessment's own assessor doesn't count;
3. the campaign `close-review-findings` works the findings off.

1. **Why:** 0 of 19 meta-gold rules and scorecards that agents reviewed passed review. A random,
   adversarial sample of 16 failed in full, each `hard-to-game` failure with a concrete gaming route.
   Two failing reviews had gone unacted for three days, and one of three assessment reviews was
   self-review ([meta-standards-review.md](../design/meta-standards-review.md) §2).
2. **Goodhart, applied:** the deterministic checks are targets, so they must stay checks on form.
   Judgement must not gate the ratchet, because a review goes stale on change, and a failing review
   would then punish the reviewer.
3. **Also changed:**
   - blank descriptions and guidance don't count;
   - fixture ladders need a real level;
   - a rule is tested only by an expectation on its own scorecard.
4. **Withdrawn:** the prediction that a standard is promoted to `mandatory` at meta gold
   ([scorecards-for-scorecards.md](../design/scorecards-for-scorecards.md) §7). Promotion needs a
   current, independent, passing review and, when there is one, an agent trial.

**Status:** ✅ amends D71, D79 and D80

**Where:** [docs/design/meta-standards-review.md](../design/meta-standards-review.md),
`portal/standards/{meta,assessments,crates,sdlc}.yaml`, `portal/campaigns.yaml`
