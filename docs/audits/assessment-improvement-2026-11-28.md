# Seikika assessment improvement retrospective

Maintainer SDLC, repository-first. This case study examines an assessment campaign and the fixes it prompted. The [benchmark design](assessment-improvement-benchmark.md) defines a separate comparative test.

## Executive summary

Seikika helped turn an open-ended maintainer cleanup into a sequence of inspectable changes. The Meets campaign went from **7 of 374 to 96 of 374** passing, an increase of **89 checks and 23.8 percentage points**. The final local portal test run passed **168 tests**. The strongest result was the combination of source-backed judgments with executable changes. For example, the observability scorecard now checks that both required alert paths are declared and that the collector test script is invoked in CI. The checkpoint used the original checkout. The subsequent integration with main verified **92 of 375** assessments in the migrated portal. [PR #183](https://github.com/seikika/seikika/pull/183)

The 89-check gain is a campaign outcome, and a causal claim about agent productivity requires a comparative run. At the checkpoint, 185 assessments were stale, 82 fresh assessments failed, eight fresh assessments were unsure, and three had no review. The separate close-review-findings campaign ended at 62/326, the same count as the first snapshot. Standard-review coverage moved from 7/326 to 6/326. Regular reporting should show these dimensions together. A higher evidence score can mean a review of an assessment is sound even when the assessment itself identifies work still to do.

The main changes to prioritize are an incremental, reason-coded freshness queue, a practical way to route reviews to independent reviewers, and a verifiable run manifest that links the candidate, report, assessment inputs, and tool coverage. The next evaluation should use an external correctness oracle and measure costs across a handoff and a subsequent change. This single campaign provides the hypotheses and failure cases for that experiment.

Earlier audits cover PR assessment policy and how readers navigate published evidence. This case study covers the experience of running repeated maintenance cycles against the repository itself. [Agentic assessment audit](agentic-assessments-2026-11-26.md) · [Assessment viewer audit](assessment-viewer-2026-11-27.md)

## What ran

The request was to substantially improve the Meets assessments and verify progress. The user then paused campaign work and requested a PR, this retrospective, raw evidence, and a repeatable comparison design. The checkpoint starts from commit `237fe887` and lands at `0467ff2d`. It changes 250 files, with 5,788 insertions and 7,637 deletions, including 178 assessment-review files. Added files include vendored agent-plugin schemas with their license and an assessment-quality review of maintainer design review. The patch and file-level counts are in the raw archive.

The workflow iterated through campaign report, scorecard or rubric criterion, source and test inspection, implementation, assessment brief, source-backed review, fixture tests, and report/ratchet verification. The retained commands and logs center on `seikika report`, `seikika assess brief`, `seikika assess check`, `seikika test`, Cargo tests and Clippy, YAML formatting, and generated standard/baseline checks. Working notes and scripts preserve review inputs and repeated operations. Use of a shell helper or a template is evidence of workflow overhead. The resulting assessment still requires a criterion-by-criterion reading of the cited sources.

| Workstream | Representative changes and checks |
| --- | --- |
| Standards and fixtures | CLI skills, crate/schema checks, documentation and repository level, lifecycle stages, observability; fixture regressions for the branches the rules should distinguish |
| Evidence production | Workspace extraction for schema consumers and documented facts, connector and report evidence, assessment metadata, API/CLI behavior and telemetry tests |
| Maintainer guidance | Corrected how to run the ratchet and use supported design, package, and lifecycle commands |
| Assessment reviews | Reopened original judgments against source, recorded reasons and suggestions where warranted, retained failing or unsure results, preserved reviewer attribution |
| Distribution | Regenerated the standard and baseline, committed the checkpoint and opened PR #183 |

The record distinguishes the repository’s own agent skills, which were assessment subjects, from skills invoked by this session. The Cloud Environment Runtime skill was applied during the GitHub handoff. The visible continuation and saved artifacts corroborate tool families, generated outputs and repeated loops. An exhaustive list of skill invocations or individual tool calls requires the session export.

## Checkpoint results

| Assessment-quality state | Count |
| --- | ---: |
| Fresh pass | 96 |
| Fresh fail | 82 |
| Fresh unsure | 8 |
| Stale with prior pass | 50 |
| Stale with prior fail | 116 |
| Stale with prior unsure | 19 |
| Missing | 3 |
| Total | 374 |

| Assessment subject rubric | Fresh pass | Fresh fail | Fresh unsure | Stale | Missing |
| --- | ---: | ---: | ---: | ---: | ---: |
| Crate design | 5 | 3 | 0 | 9 | 0 |
| Rule quality | 74 | 65 | 8 | 126 | 0 |
| Scorecard quality | 10 | 10 | 0 | 34 | 0 |
| Lifecycle stage quality | 7 | 4 | 0 | 16 | 0 |
| Testing contract quality | 0 | 0 | 0 | 0 | 3 |

All seven initial passes still passed at the checkpoint. Rule quality accounted for 273 of the 374 assessments and 126 of the 185 stale reviews. The three missing reviews concern testing contracts.

The stale prior verdicts are historical observations. A refresh should reopen the evidence and may change the judgment. The three missing cases were identified during the session as needing a reviewer independent of the Codex-authored originals. Distinct reviewer metadata helps surface self-review, while operational independence still depends on assignment and provenance.

The final checkpoint ratchet run logged no regressions against its selected baseline. A prior retained ratchet run explicitly marked 118 outcomes unknown and excluded them from comparison. The final campaign report and assessment check used `--skip runner,github-actions`; coverage for those connectors requires a separate run. Several other campaigns gained coverage between the first and last report, so their changed denominators also reflect available inputs. CI security moved from 0/18 to 18/18 in those observations. The CLI coverage and skill campaigns stayed at 35/35 and 39/44.

## Effort and visibility

The earliest comparable report was written at 9:59 PM Arizona time on November 27, 2026. The last was written at 12:08 AM Arizona time on November 28. The observed UTC span is about 2 h 10 min. These are file modification times. They cover an observed work interval containing execution and idle time. Total session wall time, active agent time, and time attributable to Seikika are separate quantities. The timeline includes 58 report snapshots. Fifty-seven use the checkpoint population; one 20-entity fixture run is labeled separately. The observed snapshots include retries and a temporary reversal. They do not count all tool calls.

Exact input, cached, output and reasoning token totals, billable cost, turn durations, and the complete conversation/tool trace are unavailable in the accessible workspace. This report draws on the visible steering and continuation context, committed patch, raw artifacts, and PR observations. It does not substitute file size, output characters, or snapshot count for token billing. The evidence archive preserves reports, checks, logs, briefs and review-generation scripts selected from task artifacts, with paths, sizes, timestamps and SHA-256 hashes. It excludes credentials, configuration and unavailable host telemetry. This checked-in evidence contains the projected campaign rows and timestamps.

## Where the loop performed well

- A subject hash and rubric identity made freshness an explicit input to the verdict.
- Assessment briefs and cited reasons preserved a path from the campaign status back to the subject and underlying sources.
- Targeted fixtures made fixes independently checkable. For alert declarations and CI script wiring, the negative fixtures are especially useful because the pass result alone cannot expose a path-only check.
- The ratchet caught regressions at report time. The separate campaigns exposed limits of the headline evidence count.
- Local incremental verification allowed multiple kinds of repairs in the same session without treating every outstanding judgment as a reason to stop.

## Friction and priorities

| Observation | Proposed change | How to tell it helped |
| --- | --- | --- |
| A change can invalidate review evidence through subject or path changes, and stale verdicts dominate the remainder | Explain which hash input changed and why. Emit the affected subjects, dependencies and scope so an agent can work a stable queue | Fewer reviews and less time per correctly understood change; zero accepted stale verdicts |
| Fresh fail and unsure results mix implementation gaps with unsupported or insufficient evidence | Expose separate queues for repair, evidence collection, and independent adjudication, with criterion-level links and deterministic exit checks | Independent oracle confirms more real repairs per run and fewer false closures |
| Running with skipped collectors and cached inputs is useful locally, but comparisons can mix coverage | Record commit/tree hash, connector success/skip/error, cache state, binary/rubric versions, and output hashes in each report | Reproduce a checkpoint and distinguish coverage gain from implementation gain |
| Templates and temporary scripts accumulated across the session | Provide structured assessment brief/check output, a bounded pending worklist and a direct transition from a finding to the smallest relevant verification | Lower finish-to-verify time and less output read at equal correctness |
| Review independence needs a separate actor | Record reviewer provenance and assign changes via a queue with a stable handoff payload | Fewer stranded originals; count self-review attempts independently |
| PR handoff hit a blocked sandbox proxy, GitHub private-email protection, and an upstream portal move | Verify remote base and author privacy before packaging, and use stable IDs with explicit migration/key recertification | Fewer interrupted handoffs; migrated evidence can be replayed with known invalidations |

The trial merge against GitHub main `4e0c66f1` found 186 unmerged paths including 177 assessment files. Main moved the self-portal to `.seikika` and also changed scanner behavior, schema discovery and package paths. The initial trial merge was aborted to preserve the paused checkpoint. The subsequent integration at `924743da` reconciled code and fixtures with main and compared each of the 96 previously passing review subjects with its migrated version. Ten hashes already matched. Eighty-two required a new hash because repository paths and commands had moved. Four source assessments had other edits and stayed stale. The live migrated report had 92 of 375 passes, zero invalid assessment files, and a clean ratchet. This is evidence of how a path move affects the review loop, and it is a separate population from the original checkpoint. [PR #183](https://github.com/seikika/seikika/pull/183)

## Reproducing the checkpoint

Use the checkpoint commit and its pinned development tools in the repository root. The report uses the original self-portal path. Current main uses the migrated path.

```sh
seikika test portal
seikika assess check portal --skip runner,github-actions -o json
seikika report portal --skip runner,github-actions --baseline portal/results.json -o json
```

The report also depends on connector inputs and the evaluation date. The archived observations preserve the exact returned campaign and assessment states. The portal test was rerun after the checkpoint commit and passed 168/168.

## Evidence

- The [checkpoint timeline](evidence/assessment-improvement-2026-11-28/timeline.csv) contains each retained campaign observation and identifies the fixture-only run.
- The [entity outcomes](evidence/assessment-improvement-2026-11-28/outcomes.csv) give the matching entity IDs, campaign verdicts and review states at baseline, checkpoint and integration.
- The [session chronology](evidence/assessment-improvement-2026-11-28/chronology.md) records the work sequence, data provenance and available instrumentation.
- Git preserves the [checkpoint patch](https://github.com/seikika/seikika/commit/0467ff2d6753db0dc2fdaed7cdeb8ce7e8d0b9a5) and the [integration changes](https://github.com/seikika/seikika/pull/183). The session archive also retains the original reports, logs, notes and assessment briefs.

The [framework validation follow-up](evidence/assessment-improvement-2026-11-28/validation.md) records two repair and verification loops and the observed cost of preparing a scoped brief.
