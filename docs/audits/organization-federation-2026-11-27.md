# Organization federation report audit

The organization report lets readers identify 92 audited public workflows, trace their scores to the organization standard, and follow the remediation campaign and track. The audit used the public organization snapshot and the OpenFGA reference portal. Chromium captured desktop and 390 px mobile views in both color schemes.

## Reader tasks

| Reader | Task | Outcome in both panels |
|---|---|---|
| Engineering lead | Find the number and provenance of audited workflows in the full catalog and grouped by repository. | All three auditors found 92 workflows across eight repositories with workflow files. They found the zizmor scorecard and the repository on the catalog rows. |
| Maintainer | Find the zizmor campaign target and its associated security track. | All three found a 100% gold target, 30 of 92 workflows meeting the goal, and the linked Workflow security track. |
| OpenFGA reviewer | Find a Go file's level and evidence. | All three found `assets.go` under Go file, with `within-limit` and one passing check. |

## Independent scores

Each panel rated ten usability dimensions and four joy dimensions from 0 to 4. The columns follow the [audit rubric](../../plugins/seikika-operator/skills/design-audit/references/rubric.md).

| Panel | Auditor | U1 | U2 | U3 | U4 | U5 | U6 | U7 | U8 | U9 | U10 | J1 | J2 | J3 | J4 |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Initial | 1 | 2 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 2 | 3 | 2 | 3 | 2 |
| Initial | 2 | 2 | 2 | 2 | 2 | 3 | 3 | 2 | 3 | 2 | 2 | 2 | 2 | 3 | 2 |
| Initial | 3 | 2 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 2 | 3 | 2 | 3 | 2 |
| Initial | Median | 2 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 2 | 3 | 2 | 3 | 2 |
| Revised | 1 | 3 | 2 | 2 | 3 | 3 | 3 | 2 | 3 | 2 | 2 | 3 | 0 | 3 | 2 |
| Revised | 2 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 3 | 2 | 2 | 3 | 2 | 3 | 2 |
| Revised | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 2 | 4 | 2 | 2 | 3 | 2 | 3 | 2 |
| Revised | Median | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 3 | 2 | 2 | 3 | 2 | 3 | 2 |

## Measured changes

| Measure | Initial | Revised |
|---|---:|---:|
| Grouped catalog desktop page height | 12,106 px | 1,538 px |
| Campaign desktop page height | 23,673 px | 1,584 px |
| Full catalog desktop page height | 11,834 px | 11,202 px |
| Serious axe findings per organization route | 1 | 0 |
| Mobile viewport overflow | 0 px | 0 px |
| Grouped catalog text in monospace | 0.4% | 0% |
| Grouped catalog distinct font sizes and hues | 11 sizes, 4 hues | 10 sizes, 4 hues |
| Grouped catalog words above desktop fold | 306 | 446 |

## Findings and fixes

- **Severity 3.** The initial catalog repeated the catalog index above workflow controls. Campaign entries overlapped at desktop width. The revised catalog opens the selected kind directly, the campaign lists workflow identity above evidence, and grouped repositories and campaign evidence open on demand.
- **Severity 3.** The initial Starlight search button's accessible name omitted its visible keyboard shortcut. The docs component now exposes the visible name and shortcut. The revised captures report zero serious axe findings on the five organization routes.
- **Severity 3.** The revised mobile captures showed that the sticky docs header hid the first report tab row. The organization page now supplies an inset for deep-link scrolling and uses the report tabs as its section navigation. The smoke test checks the tab row against the sticky header at 390 px.
- **Severity 3.** The Pages browser run found grouped catalog overflow. The pinned browser reproduced the source selector extending the page to 377 px on a 320 px viewport. The selector now fits its container, and the smoke test covers 320 and 390 px.
- **Severity 3.** The revised full catalog still repeated a long scorecard label for every workflow. The catalog now names the standard and level counts above the rows, loads 40 rows at a time, and keeps every repository group available. The smoke test follows pagination to the full count and checks grouped coverage against the snapshot.
- **Severity 2.** Captures identified small targets, mainly inline links, and the generic OpenFGA group title. The organization catalog breadcrumb now has a 24 px target. The audit did not measure interaction latency or contrast separately from axe.
