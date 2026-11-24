---
id: D01M857ZB468ZM2YS6GNSEG6YB9
decided: 2026-11-23T23:30:00Z
---

# Comparative scorecards: one scorecard scores a subject and its peers, and says where the subject stands

**A scorecard may name a subject and its peers: `compare: {subject: <entity>, peers: [<entity>,
…]}`. Each of its rules may give a numeric `value` (CEL), whether `better` is `lower` or
`higher`, a `unit`, and a relative `tolerance`. The derive step then records, per rule and per
peer, the subject's standing: `ahead`, `parity`, `behind` or `unknown`. It also records an
overall standing per peer: `behind` if any rule is behind, then `unknown` if any rule is, then
`ahead` if any rule is, else `parity`. `toolbox report` renders this as a table in markdown and
as a Compare view in the web report. The peers are not this portal's debt: they are left out of
the ratchet baseline and the health views.**

1. **Why.** The Backstage comparison
   ([backstage-comparison.md](../design/backstage-comparison.md)) needs a durable, re-runnable
   record of where Toolbox stands, factor by factor. The record must sit in the
   Toolbox-for-Toolbox report the docs site publishes. A scorecard already says what good looks
   like, with a check, a level and a fix. Scoring a rival with the same rules makes the
   comparison as auditable as any other standard. No separate comparison format, page or tool is
   needed.
2. **How a rule's standing is decided.**
   - When both sides have a `value`, the values decide. Within `tolerance × max(|a|, |b|)` of each
     other is `parity`. Otherwise the better side, by `better`, wins.
   - When either side has no value, the outcomes decide: `pass` beats `waived`, which beats `fail`.
   - An unscored entity, or an `unknown` outcome, makes the standing `unknown`. A missing peer is
     never `parity`.
   - `value` is evaluated for every in-scope entity, pass or fail. `measure` stays what it was:
     the size of a failure, for the ratchet.
3. **Where the products come from.** Products are ordinary entities. For Toolbox for Toolbox,
   they are `Product` records that `experiments/backstage-compare/` writes into
   `portal/comparisons/`, read through a workspace `data:` kind. A product is a snapshot of a
   measured run, with its date and the harness's commit, so a comparison is re-run and not
   hand-edited.
4. **The ratchet.** The subject's rows stay in the baseline, so a rule the subject passed and
   now fails is a regression, as in any scorecard. Peers' rows are excluded, because a re-run
   that found the rival slower is not an improvement in this portal.
5. **Validation.** `value` is type-checked as CEL. `better` and `value` require each other, and
   `tolerance` and `unit` require `value` (the JSON Schema's `dependentRequired`). A peer that
   repeats, or that is the subject, is refused (`xref.compare`). A subject or peer that is not
   in the catalog is a derive diagnostic (`compare.missing`).
6. **The box or a tool.** This is the box: it reports what tools measured. The measurements come
   from tools (`oha`, `/proc`, `du`, `osv-scanner`), run by the harness.
