# Design audit: one report, three portals, 2026-11-24 (loop 3)

*Status: record of the first design loop run under the revised design-review stage
([D01M87PSM9YQGGW3YQ5X4XCJP63](../decisions/D01M87PSM9YQGGW3YQ5X4XCJP63.md)): story probes first,
captures from Storybook, a blind three-auditor panel scored by median, and two reference portals
plus the starter. About the box: how the report shows what the tools found, whichever portal it
serves.*

ROUND_SUMMARY

## Scope

| Item | Value |
|---|---|
| Surface | `<toolbox-report>` from `report-ui/src`, rendered by Storybook stories over the fixtures in `report-ui/fixtures/` |
| Portals | OpenFGA (`examples/openfga-portal` over OpenFGA v1.23.0, compared with SpiceDB v1.57.1), Toolbox for Toolbox (`portal/`, compared with Backstage), the repo portal starter, and the starter before its first run |
| Stories | 22 page stories: 12 OpenFGA, 6 Toolbox, 1 starter, 3 no-data |
| Viewports | 1280×800 and 390×844, light and dark, plus a full-page desktop light shot |
| Gate before auditors | `npm --prefix report-ui run probe`: axe (WCAG 2.0–2.2 A/AA), horizontal overflow at 390 px and script errors on 53 stories × 4 views |
| Auditors | Three fresh agents per round, each blind to source, diffs, earlier scores and each other, with [the rubric](../../plugins/toolbox-operator/skills/design-audit/references/rubric.md) and the tasks below. A dimension's score is the panel median |
| Stop rule | Two consecutive passing rounds with no rise in the targeted medians, or every targeted median at 4. A round passes when the probes pass, no severity 3–4 finding is open and no targeted median falls |

The tasks were written before the first capture and kept for every round:

| Task | Reader | Question |
|---|---|---|
| T1 | OpenFGA maintainer, new to the report | Which line of work is furthest from its goal, and what one fix would move it most? |
| T2 | Architect choosing between OpenFGA and SpiceDB | Where is OpenFGA ahead, where behind, how was it measured, and what is not measured yet? |
| T3 | Go contributor to OpenFGA | Which Go file is furthest over the size limit, what is the limit, and what should be done? |
| T4 | Toolbox maintainer (regression guard) | Is `toolbox-cli` at the top level on every scorecard that rates it? If not, which rule holds it back? |
| T5 | Engineer adopting the starter, first run | The portal shows no data yet. What should I do next, and is anything on the page misleading? |
| T6 | Reader comparing portals | Do tracks, scorecards, campaigns and comparisons read the same way on all three portals? |

## Scores (panel medians)

ROUND_TABLE

## Round 1: what the panel found

The probes passed 212 of 212 checks before any auditor looked. All three auditors then found the
same three severity-4 problems, none of which the Toolbox-only loops had surfaced, because Toolbox's
own portal has data, measured comparisons and a repository standard:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The starter before its first run read "On track", with a green 0%, a gold medal on "0 scorecards with entities at none" and "✓ clear" in the sidebar; no page said what to do (A, B, C) | Box | A first-run hero ("Not scored yet") with the next command; tiles with an empty denominator are hidden; scorecards with nothing scored say "not scored", and an empty scorecard says what has not been collected and how to collect it |
| The unmeasured OpenFGA comparison said "not measured yet" in its headline but "✓ Not behind on any rule", "0 of 4 rules at parity" over full bars, and "What would close it" on every rule (A, B, C) | Box | Every section agrees: factor cards say "Not measured: none of N rules has values" over an empty bar, the behind section says "Nothing to compare yet", the tally shows dashes, "What would close it" appears only where a product is behind |
| Products were titled by machine id ("product:openfga vs product:spicedb", truncated to "product:ope…") (A, B, C) | Box | A product with no record yet falls back to its short name, not its id |
| Only Toolbox's portal had a status hero; OpenFGA and the starter opened on a "Fleet health" card holding one run-on sentence (A, B, C) | Box | Every portal opens on a status hero: a status word ("Needs attention", "On track", "Not scored yet") and the sentence behind it; the repository-level hero stays where a repository standard exists |
| "238 docs · 100% at the top level" beside "1 at none" (A, B, C) | Box | A share never rounds to 100% while one entity is short, nor to 0% while one is there |
| "Gold" stood for every scorecard's top level: "265 gold" in the catalog beside "265 within-limit" on the scorecard (A, B, C) | Box | Where one scorecard rates a kind, the catalog and the entity page use that scorecard's level names; the overview tile says "top level" |
| T3 needed a scan of 21 unsorted rows, three of them just "check.go" (A, B, C) | Box | A rule that measures how far off each entity is lists the furthest first, with the full path |
| The OpenFGA sidebar listed `docs` and `repo-hygiene` where the starter said "Docs with working links" and "Repository basics" (A, B, C) | Portal | `examples/openfga-portal/standards/repo.yaml` titles them as the starter does |
| 'rule "Within limit" is fail', "counted from the start the baseline records" (A, C) | Box and packages | "fails “Within limit”"; five packages now say "counted from the starting point saved in the baseline" |
| The starter's scorecards page said "grouped by track" over "Not in a track" (A, C) | Box | The sentence appears only when the portal has tracks |
| Four tabs wrapped to leave "Scorecards" alone on a second row on a phone (A, B, C) | Box | Phone tabs form an even grid (four as two by two) |

One finding was declined: "Toolbox's maintainers chose the factors" on the OpenFGA track (C) is that
comparison's own description and names its actual authors, as the finding asked.

Every fix but one was in the box, and every one generalises: the report now has a first-run state,
an unmeasured-comparison state and per-scorecard level names that no single portal would have
forced. That is what the second reference portal was for.

ROUND_2

## Sources

- Rubric and stop rule: [design-audit SKILL.md](../../plugins/toolbox-operator/skills/design-audit/SKILL.md), [design-audit.md](../design/design-audit.md).
- Stories and fixtures: [report-ui/README.md](../../report-ui/README.md).
- The loop this one follows: [toolbox-report-2026-11-24-2.md](./toolbox-report-2026-11-24-2.md) and its process audit, [design-loop-efficiency-2026-11-24.md](./design-loop-efficiency-2026-11-24.md).
