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
| Round 3 | A | 3 3 3 3 4 3 3 3 3 3 | 3 2 3 3 | S S S S S |
| Round 3 | B | 4 3 3 3 4 3 3 3 3 3 | 3 2 3 3 | S S S S S |
| Round 3 | C | 3 3 3 3 3 3 3 3 3 3 | 2 2 3 3 | S S S S S |
| **Round 3 median** | | **3 3 3 3 4 3 3 3 3 3** | **3 2 3 3** | **S S S S S** |
| Round 4 | A | 3 3 3 3 4 3 3 4 3 3 | 3 2 3 3 | S S S S S |
| Round 4 | B | 3 3 3 3 3 4 3 4 3 3 | 2 2 3 3 | S S S S S |
| Round 4 | C | 3 3 2 3 4 3 3 4 3 3 | 3 2 3 3 | S S S S S |
| **Round 4 median** | | **3 3 3 3 4 3 3 4 3 3** | **3 2 3 3** | **S S S S S** |
| Round 5 | A | 4 3 3 3 3 4 3 4 3 3 | 3 2 3 3 | S S S S S |
| Round 5 | B | 3 3 3 3 4 3 3 4 3 3 | 3 2 3 3 | S S S S S |
| Round 5 | C | 3 3 3 3 4 3 3 4 3 3 | 3 2 2 3 | S S S S S |
| **Round 5 median** | | **3 3 3 3 4 3 3 4 3 3** | **3 2 3 3** | **S S S S S** |
| Round 6 | A | 3 3 2 3 4 3 3 4 3 3 | 2 1 3 3 | S S S S S |
| Round 6 | B | 4 3 3 3 4 3 3 4 3 3 | 3 2 3 3 | S S S S S |
| Round 6 | C | 3 3 3 3 4 3 3 4 3 3 | 3 1 3 3 | S S S S S |
| **Round 6 median** | | **3 3 3 3 4 3 3 4 3 3** | **3 1 3 3** | **S S S S S** |
| Round 7 | A | 3 3 3 3 4 3 3 4 3 3 | 3 2 3 3 | S S S S S |
| Round 7 | B | 4 3 3 3 4 3 3 4 3 3 | 3 2 3 3 | S S S S S |
| Round 7 | C | 4 3 3 3 4 4 3 4 3 3 | 3 2 3 3 | S S S S S |
| **Round 7 median** | | **4 3 3 3 4 3 3 4 3 3** | **3 2 3 3** | **S S S S S** |

Rounds 6 and 7 used the same release captures with independent panels. The release brief initially implied that readers had to distinguish all three results on the list alone. The auditors confirmed their T3 outcomes against the original task, which follows the named run into detail. All six completed all five tasks, with no severity 3 or 4 findings. The medians for recognition, hierarchy, consistency, and honest states held across the two confirmation panels. The change in U1 and J2 between those panels came from different judgments of the same captures.

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
| 3 | The published Pages smoke check reproduced report navigation overflow at 320 px after the new tab was added. | Give the report navigation two columns on narrow phones. The public organization smoke check passed at 320 and 390 px with the pinned browser and the generated organization report. |
| 3 | Round 3 found the stale and same-assessor context on the original run. Opening the linked review on its own lost both qualifiers. | Label review rows and distinguish their recorded verdict. Carry the known review qualifiers onto the detail and link back to the reviewed assessment. |
| 3 | The confirmation panels for rounds 4 and 5 found the same remaining issue. The direct review title and a green Pass drew attention ahead of the historical qualification, and the list lost that qualification entirely. | Lead with Historical review in the title and list. Give stale and same-assessor qualifications visual priority and render the recorded verdict in a neutral badge. Let the reviewed verdict and its context occupy the full width on mobile. |

## Measured changes

The baseline passed 70 stories across 280 browser views. Round 1 passed 79 stories across 316 views and Round 2 passed 83 stories across 332 views. Capture metrics had zero serious axe findings and zero mobile overflow in all three sets. In Round 1 the recent runs list used 0% monospace text. From Round 1 to Round 2, desktop run detail fell from 2,280 to 1,495 px in the report and from 2,057 to 1,372 px in the hosted portal. The hosted list and detail went from four and five small targets to zero in both. The baseline had no dedicated run detail.

The release passed 84 stories across 336 browser views. The 12 release captures had zero serious axe findings, zero targets under 24 px, and zero horizontal overflow at 390 px. Browser probes separately followed review and rubric links, used Enter to expand evidence, verified that browser Back restored search, and matched the timestamp in the list and detail.

The panel prioritized visible passed-criterion counts and direct links to evidence. It deferred additional charts because the verdict, scorecard level, and independent review answer separate reader questions and already fit in the summary.

The release panels' remaining findings were minor. Some scorecard rule names require opening the rule to read the requirement, and long titles wrap in the two-column summary on mobile. Reviewers also suggested adding current review status to subject rows for faster fleet triage.
