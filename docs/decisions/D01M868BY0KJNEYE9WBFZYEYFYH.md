---
id: D01M868BY0KJNEYE9WBFZYEYFYH
decided: 2026-11-24T07:47:16Z
---

# Standards say why, kinds come in groups, and every result says whether a tool or an agent decided it

**Every scorecard and rule may carry a `rationale` (why it exists: the risk or cost it guards
against) and `links` (the long story elsewhere), and Toolbox for Toolbox (T4T) requires both
through `explains-why` on `meta-rule` (bronze) and `meta-scorecard` (silver). Manifests may group
kinds with a top-level `kindGroups`. Each rule is classed *measured* or *judged* from what its
check reads, and the report shows the class next to every rule and result. The report's Tracks,
Catalog and Compare views are built on these fields.** It extends the report redesign of
D01M85MK7FCRXH2TRMRKXTNA5GT and was judged by the same blind design audit
([toolbox-report-2026-11-24-2.md](../audits/toolbox-report-2026-11-24-2.md)).

1. **Rationale and links, not an encyclopedia.** A fix says what to do and a description says
   what a standard covers; neither says why anyone should meet it. `rationale` is one short
   paragraph. Anything longer goes behind `links` (`title`, `url`), which tracks may carry too.
   A rule without its own links shows its scorecard's. Both fields are part of the subject hash,
   so changing them makes the quality assessments of that rule or scorecard stale.
2. **Kind groups.** `kindGroups` is a list of `{name, title, description, kinds}`. Validation
   rejects unknown kinds and a kind in two groups. Kinds left out are shown under "Other kinds". T4T
   uses six groups: Code, Agent and user surfaces, Docs and examples, CI and supply chain,
   Standards and judgement, and SDLC and evidence.
3. **Measured or judged.** A rule is *judged* when its check reads, directly or through computed
   fields, a relation to entities mapped from `assessment` records (D79). Its outcome moves when
   an assessor's verdict does. Every other rule is *measured*: a tool, connector or script read
   the fact, and the same input gives the same result. The class is computed
   (`crates/toolbox-cli/src/judged.rs`), never declared, so it cannot drift from the check. The
   report JSON, `STANDARD.md` and `describe` carry it as `basis`.
4. **Tracks everywhere.** Every T4T scorecard is on a track. A package scorecard takes its track
   from a package parameter. A parameter whose default is `null` is optional, so a package can
   leave the track to the portal that includes it.
5. **Comparisons explain themselves.** A comparison carries each product's facts and, per rule,
   the fields it reads with their descriptions. The Compare view can then show what was measured
   and how without the reader opening the benchmark's docs. It says that factors are tallied,
   not weighted.
6. **Contract.** Everything here is additive. The report JSON gains `kindGroups`, `kinds` (each
   kind's description), `rationale`, `links` and `basis` on scorecards and rules, `links` on
   tracks, `facts`, `reads` and `fields` on comparisons, and `unassessed` (per scorecard with
   judged rules, the scored entities no assessment points at, whose judged passes mean nothing
   was looked at). Existing fields and deep links are unchanged.
7. **Reopen when** a judged rule is found that the reading of assessment relations misclasses,
   or readers on the audit's tasks still need the benchmark docs to understand a comparison.

## Where

- [report-ux.md](../design/report-ux.md), [scorecards-for-scorecards.md](../design/scorecards-for-scorecards.md)
- `schema/manifest.schema.json`, `crates/toolbox-model/src/validate.rs`, `crates/toolbox-cli/src/judged.rs`, `crates/toolbox-cli/src/report.rs`,
  `crates/toolbox-cli/src/toolbox-report.js`
- `portal/toolbox.yaml` (`kindGroups`), `portal/standards/meta.yaml` (`explains-why`)
