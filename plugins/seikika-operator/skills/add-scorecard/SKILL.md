---
name: add-scorecard
description: Add or change a Seikika scorecard, its CEL rules, levels, waivers or visibility (summary vs details facets and declassification). Use when asked to create standards, maturity levels, production readiness checks or scorecard rules in a Seikika portal.
metadata:
  status: pre-release
---

# Add a scorecard

1. **Write the fixture tests first.** For every rule, add at least one passing and one failing entity
   under `tests/fixtures/`. Add `skip` and `unknown` cases where the rule uses `applies_if` or
   `stale_after`.
2. **Define the scorecard** in the owning module:
   - `owner`
   - `appliesTo` (a kind or a CEL predicate)
   - `levels`
   - each rule's `check` (typed CEL), `fix` text and optional `remediation` action
3. **Visibility:**
   - Rule outcomes inherit the facet labels of the data they read.
   - If outcomes computed from restricted details must be visible to everyone, add an explicit
     `declassify: { outcome: "summary" }`, and expect reviewers to ask why.
   - Summaries over fewer than the minimum cardinality are suppressed automatically.
4. **Run** `validate` and `test`. Then run `plan` and check the "Scorecards" and "Declassifications"
   sections: how many entities change level, and who will be notified.
