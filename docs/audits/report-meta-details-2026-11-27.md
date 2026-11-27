# Report detail navigation audit

This maintainer change covers the report used in repository portals and the central portal. Both
entry paths now give the same operational detail and catalog evidence for the selected item.

| Task | Reader | Completion |
| --- | --- | --- |
| 1 | Seikika standards maintainer | Enter Agent skills from Catalog and Scorecards. Find the lowest level among the skills and identify the scorecards evaluating Agent skills itself. |
| 2 | Seikika rule reviewer | Find Has example from the Rule catalog and from Agent skills. Read its check and the fix for a failure. |
| 3 | OpenFGA operator | Find Go files within 800 lines from Catalog and Campaigns. Read its target and progress. |

Three independent auditors compared rendered states at 1280 × 800 and 390 × 844 in light and
dark mode. The captures established what each destination showed. The browser probes exercised
the catalog links, legacy entity URLs, keyboard focus at the scorecard shortcut, and the expanded
rule results and relationships. [Scoring rubric](../../plugins/seikika-operator/skills/design-audit/references/rubric.md)

## Independent scores

Scores follow the order of the usability and joy dimensions in the rubric. Each auditor rated all
three baseline tasks partial and all three tasks in the shared views successful from the rendered
destinations.

| Evidence | Auditor | U1–U10 | J1–J4 |
| --- | --- | --- | --- |
| Baseline | 1 | 2 2 3 3 4 2 1 2 3 3 | 3 1 3 2 |
| Baseline | 2 | 2 2 3 3 3 2 1 3 3 3 | 3 2 3 3 |
| Baseline | 3 | 2 2 2 2 4 2 1 3 3 3 | 3 1 3 2 |
| **Baseline median** | | **2 2 3 3 4 2 1 3 3 3** | **3 1 3 2** |
| Shared views | 1 | 3 3 3 3 4 3 3 3 3 2 | 3 1 3 3 |
| Shared views | 2 | 3 3 3 3 3 3 3 3 3 2 | 3 2 3 3 |
| Shared views | 3 | 3 3 3 3 4 3 3 3 3 2 | 3 2 3 3 |
| **Shared views median** | | **3 3 3 3 4 3 3 3 3 2** | **3 2 3 3** |
| Confirmation | 1 | 3 3 3 3 4 4 4 3 2 3 | 3 2 4 3 |
| Confirmation | 2 | 4 3 3 4 4 4 3 4 3 3 | 3 2 4 3 |
| Confirmation | 3 | 3 3 3 3 4 3 3 3 2 3 | 3 3 3 3 |
| **Confirmation median** | | **3 3 3 3 4 4 3 3 2 3** | **3 2 4 3** |

| Severity | Evidence | Change |
| --- | --- | --- |
| 4 | Campaigns showed the OpenFGA target and 93% attainment. Opening the Campaign kind in Catalog ended at a missing-kind page. | List report campaigns under Campaign in the catalog and open the same progress page from both lists. |
| 4 | The Has example catalog detail showed how the portal reviews the rule. Its own check and remediation were available through Scorecards. | Use the rule detail for both entries and keep the quality scores with its catalog information. |
| 3 | Agent skills showed the skills distribution in Scorecards and the scorecard's quality review in Catalog. Readers had to locate both separately. | Combine the views and preserve the distinct scorecard and entity score scopes. |
| 3 | The first shared view put the quality review in a long section reached by a generic Catalog details shortcut. The extra material reduced the density and responsiveness score. | Name the shortcut and destination How this scorecard is evaluated. Show the evaluating scorecard names together and let readers expand rule results and catalog fields when needed. |
| 3 | The confirmation panel found a target-size violation on closely spaced links to evaluating scorecards. | Give each evaluator link at least 28 px of height and add 6 px between rows. |

All three confirmation readers found both evaluating scorecards from the shortcut and completed the
rule and campaign tasks. Collapsing the supporting material reduced the desktop Agent skills page
height from 5,068 to 3,630 px before the link spacing adjustment. All captures fit at 390 px.
After the spacing adjustment, all 340 Storybook views passed, including the accessibility checks.

The design keeps campaign progress and rule outcomes as the primary content. Catalog information
links back to its kind, and the exact item and report tab are the same from either entry path.
