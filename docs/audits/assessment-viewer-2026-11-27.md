# Assessment viewer design audit

The served portal and the published report are maintainer SDLC work. The viewer belongs to Seikika. Rubric definitions, documentation links, decisions, and assessment records belong to each portal.

## Reader tasks

| Task | Reader | Completion |
|---|---|---|
| T1 | Seikika maintainer | Find the published assessments from the report landing page and identify the rubric used to assess seikika-expr. |
| T2 | Standards maintainer | Open the Assessment quality rubric, read its criteria and find its supporting documentation. |
| T3 | Assessment author | Find the quality results for the seikika-expr Crate design assessment and distinguish the subject verdict, scorecard checks and the independent review. |
| T4 | Operator | Find a recent assessment, name its verdict and date, and give its time when recorded. |
| T5 | OpenFGA maintainer | Determine whether the published OpenFGA report has assessments or rubrics. |

The baseline was the existing Overview, Catalog, catalog kind and Scorecards views in the Seikika and OpenFGA Storybook fixtures. The new views use an assessment subset generated from the current Seikika report. The presentation of a recorded time uses a test timestamp with an explicit offset. Hosted portal stories cover the same content in the served portal shell. Each audit used desktop at 1280 × 800 and mobile at 390 × 844, in light and dark mode. Three independent auditors per round received only the rendered captures, metrics, tasks and [rubric](../../plugins/seikika-operator/skills/design-audit/references/rubric.md).

## Independent scores

Scores follow the rubric order U1–U10 and J1–J4. S is success, P is partial, and F is fail. Each panel median is calculated per dimension.

| Evidence | Auditor | U1–U10 | J1–J4 | T1–T5 |
|---|---|---|---|---|
| Baseline | A | 1 2 3 2 3 3 2 2 3 2 | 3 2 2 2 | S P P F S |
| Baseline | B | 2 2 2 2 3 3 2 2 3 2 | 2 1 3 2 | S P P F S |
| Baseline | C | 1 2 2 2 3 3 2 2 3 2 | 2 1 3 2 | P P F F S |
| **Baseline median** | | **1 2 2 2 3 3 2 2 3 2** | **2 1 3 2** | **S P P F S** |
| Round 1 | A | 3 2 2 3 3 3 3 3 3 3 | 3 2 2 2 | S S P S S |
| Round 1 | B | 3 2 2 3 3 3 3 3 3 2 | 2 2 2 2 | S S P S S |
| Round 1 | C | 3 2 2 2 3 3 3 3 3 2 | 2 2 2 2 | S S P S S |
| **Round 1 median** | | **3 2 2 3 3 3 3 3 3 2** | **2 2 2 2** | **S S P S S** |
| Round 2 | A | 3 3 2 3 3 3 3 2 3 3 | 3 2 2 3 | S S S S S |
| Round 2 | B | 3 2 3 3 3 3 3 3 3 3 | 3 2 2 3 | S S S S S |
| Round 2 | C | 4 3 2 3 3 3 3 2 3 3 | 3 2 2 3 | S S S S S |
| **Round 2 median** | | **3 3 2 3 3 3 3 2 3 3** | **3 2 2 3** | **S S S S S** |

## Findings and fixes

| Severity | Evidence | Change |
|---|---|---|
| 4 | Baseline Overview and Catalog exposed assessment counts and aggregate levels. They gave readers no dated result. | Add Assessments to both navigation surfaces and sort published results by their recorded dates. Show the rubric and verdict beside the time when available. |
| 3 | Baseline Catalog placed assessment and rubric entries under Standards and judgement. Readers had to combine Catalog and Scorecards to identify a rubric. | Add a rubric index, criteria, guidance, and documentation links. Link each assessment to its rubric and each rubric back to its runs. |
| 3 | Round 1 run detail showed Gold and Bronze under the same heading and described a review as stale below them. The report and portal on mobile showed criterion prose first. | Put the subject verdict, scorecard checks, and recorded review in separately labeled cells near the title. Show the recorded review verdict and date alongside its stale state and show whether the same assessor wrote it. Keep detailed checks and evidence in disclosures. |
| 2 | The baseline OpenFGA Catalog required readers to infer both empty inventories. | Show the number of published runs and rubrics and give the empty state the scope of its report snapshot. |
| 2 | Round 1 hosted screenshots counted four small targets in the list and five in the detail. The current section looked like a regular navigation link. | Give the portal navigation links a minimum target height and mark Assessments as current. |
| 2 | Review runs in Round 1 led with long catalog IDs. Rubric documentation was below the mobile fold. | Lead with readable subject names and link the backing documentation beside the rubric description. Keep the full identifiers searchable. |
| 3 | Round 2 readers identified the historical Pass and its staleness. The heading still suggested independent sign-off even though the reviewer was the same assessor. | Lead with the current review status. Put the dated historical verdict and the same-assessor explanation beneath it. Give reviews of assessments their own recorded scope. |
| 2 | Round 2 readers matched the scorecard names and pass counts by comparing the summary with the rule list. Returning from details could lose the active filters. | Repeat the official scorecard name with the level and count above the fold. Link the first failed requirement for the next level when the result identifies one. Put filters into the run-list URL and verify restoration with browser Back. |

## Measured changes

The baseline passed 70 stories across 280 browser views. Round 1 passed 79 stories across 316 views and Round 2 passed 83 stories across 332 views. Capture metrics had zero serious axe findings and zero mobile overflow in all three sets. In Round 1 the recent runs list used 0% monospace text. From Round 1 to Round 2, desktop run detail fell from 2,280 to 1,495 px in the report and from 2,057 to 1,372 px in the hosted portal. The hosted list and detail went from four and five small targets to zero in both. The baseline had no dedicated run detail.

The panel prioritized visible passed-criterion counts and direct links to evidence. It deferred additional charts because the verdict, scorecard level, and independent review answer separate reader questions and already fit in the summary.
