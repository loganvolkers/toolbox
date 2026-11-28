# Astro and Vue portal migration design audit

The box's static catalog and hosted entity page share the same Vue overview. The report retains
its standards summaries, detail routes, and evidence. A service page exposes ownership and
operational links even when the snapshot contains no score for it.

## Scope and tasks

Three independent auditors reviewed each set of rendered captures using the
[design-audit rubric](../../plugins/seikika-operator/skills/design-audit/references/rubric.md).
The baseline used the embedded report at `a1e568d`. The comparison used the Vue stories with
the same Seikika-for-Seikika and OpenFGA snapshots. The Checkout example adds service metadata
to demonstrate the shared overview. Views were captured at 1280 and 390 pixels in light and dark.

| Reader | Task | Baseline | Vue report |
|---|---|---|---|
| Engineering director | Find the repository level, blocker, and investigation path | Success | Success |
| Platform owner | Find the group with most scorecards at none, and the detail filters | Success with effort | Success |
| OpenFGA entity owner | Find passed checks, the next fix, and evidence | Partial | Partial |
| Checkout owner | Find the owning team and Grafana without scorecards | Fail | Success |

The selected OpenFGA entity has one passing measured check and needs no fix. The capture shows the
evidence entry point. It cannot establish how an unmeasured check reads on that entity.

## Scores

Rows follow U1–U10 and J1–J4 in rubric order. Column medians use the three independent scores.

| Capture | Reviewer | U1 | U2 | U3 | U4 | U5 | U6 | U7 | U8 | U9 | U10 | J1 | J2 | J3 | J4 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Baseline | 1 | 3 | 3 | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 1 | 3 | 2 |
| Baseline | 2 | 2 | 3 | 3 | 2 | 3 | 3 | 2 | 2 | 2 | 2 | 3 | 2 | 3 | 3 |
| Baseline | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 1 | 3 | 3 |
| Baseline | Median | 3 | 3 | 3 | 2 | 3 | 3 | 2 | 3 | 2 | 3 | 3 | 1 | 3 | 3 |
| Vue report | 1 | 3 | 3 | 3 | 3 | 3 | 3 | 2 | 3 | 3 | 3 | 3 | 1 | 4 | 3 |
| Vue report | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 0 | 4 | 3 |
| Vue report | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 3 | 3 | 3 | 2 | 4 | 3 |
| Vue report | Median | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 1 | 4 | 3 |

The five common capture routes had no serious axe findings and no horizontal overflow at 390 pixels
in either version. Stills show visible controls and reading order. They do not measure latency or
confirm interaction.

## Findings and fixes

| Severity | Finding | Change |
|---|---|---|
| 4 | An unscored Checkout did not surface ownership or Grafana. | Render the shared entity overview before scorecard evidence. |
| 3 | The index made readers find the highest group count by scanning the list. | Add a plain-language count and a button that focuses the relevant group. |
| 3 | The mobile scorecard hid its filter shortcut below explanatory content. | Place the shortcut immediately after the scorecard verdict. |
| 3 | The search placeholder was low contrast and served as the only visible label. | Use the shared accessible text color and add a visible label to the index search. |
| 3 | The OpenFGA passing rule looked like a status pill. | Explain that opening the check shows evidence. An unmeasured OpenFGA entity remains outside the captured sample. |

The report keeps the existing progress and attainment views so readers can still see what passes,
what holds a level, and where to investigate. Short reports now fill the theme background across the
viewport. The [browser smoke test](../../npm/report-ui/scripts/smoke.mjs) exercises the two navigation
shortcuts, keyboard tab movement, and the Pages URLs. The
[Storybook probe](../../npm/report-ui/scripts/probe.mjs) checks the complete story collection.
