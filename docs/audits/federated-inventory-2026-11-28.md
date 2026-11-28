# Federated inventory report design audit

The audit used the organization federation example with a real Seikika for Seikika artifact and the OpenFGA reference story. Chromium captured the overview, repository list and detail, tracks, campaigns, scorecards, assessments, and Compare at desktop and 390 px in both color schemes. Each panel used the [design-audit rubric](../../plugins/seikika-operator/skills/design-audit/references/rubric.md) and saw rendered captures and metrics only.

## Reader tasks

| Reader | Answer to locate |
| --- | --- |
| Platform lead | Identify the publication that supplied the framework standards and cite its artifact SHA-256. |
| Repository owner | Open the single `seikika/seikika` repository. Read the parent hygiene score and child repository level and find the fields each source provided. |
| Standards reviewer | Find a child track, campaign, and scorecard with the Seikika for Seikika source label. |
| Evidence reviewer | Identify who published a child assessment and which products the comparison covers. |
| OpenFGA reviewer | Find a Go file’s level and the measurement behind its line-limit rule. |

## Independent scores

| Round | Auditor | U1 | U2 | U3 | U4 | U5 | U6 | U7 | U8 | U9 | U10 | J1 | J2 | J3 | J4 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Initial | 1 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 0 | 2 | 3 | 3 | 2 | 3 | 2 |
| Initial | 2 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 0 | 2 | 3 | 3 | 1 | 3 | 2 |
| Initial | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 1 | 2 | 3 | 3 | 0 | 3 | 3 |
| Initial | Median | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 0 | 2 | 3 | 3 | 1 | 3 | 2 |
| Revised | 1 | 3 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 1 | 3 | 2 |
| Revised | 2 | 3 | 2 | 3 | 3 | 3 | 2 | 2 | 3 | 2 | 3 | 3 | 2 | 3 | 2 |
| Revised | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 3 | 4 | 2 | 2 | 3 | 2 | 3 | 3 |
| Revised | Median | 3 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 2 | 3 | 2 |

## Initial findings and changes

- An empty campaign appeared on target and an unmeasured track showed a top-level achievement. Campaign results now distinguish absent measurements from scored progress, and unscored tracks report how many scorecards await results. Recommendations derived from missing checks withhold a predicted gain.
- The source disclosure hid the framework artifact fingerprint. The overview now lists report sources independently of the repository groups and exposes the full SHA-256 in the disclosure.
- The repository detail repeated generic link and field labels. The inventory and child observations now have separate field headings, the two outgoing links have specific labels, and the headline identifies the lowest published level.
- Chromium identified a target-size violation on the track link in the repository detail. The entity’s campaign and track links now have a minimum target height.
- The panel prioritized traceable source citations, specific next steps with substantiated gains, and more visible evidence controls. The revision preserves the artifact fingerprint and clarifies whether a gain has been measured.

## Revised findings and changes

- All three reviewers distinguished the organization’s repository results from the child’s scores, and found OpenFGA’s `tuples.go` measurement of 2,629 lines against the 800-line rule. The source register exposed the full child fingerprint. Reviewers requested a more explicit artifact label and publication roles on assessment rows. Those labels now appear in the overview and the assessment list and detail.
- The revision retained a serious target-size issue on the first track link in the repository’s context card. The link and its list row now reserve 28 px of vertical space.
- Scoped scorecard rows now show their own denominators. Catalog-wide kind counts appear with a scorecard caption only when the scorecard covers the whole kind.
