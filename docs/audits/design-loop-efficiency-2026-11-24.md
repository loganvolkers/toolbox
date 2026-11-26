# Inner-loop audit: 36 rounds of design review on the hosted report, 2026-11-24

*Status: audit of the maintainer SDLC's design-review stage, as run on the hosted report's second
design loop ([seikika-report-2026-11-24-2.md](./seikika-report-2026-11-24-2.md)). Its
recommendation is decided in [D01M87PSM9YQGGW3YQ5X4XCJP63](../decisions/D01M87PSM9YQGGW3YQ5X4XCJP63.md).
About the box: how the maintainers change the way it reports.*

## Conclusions

1. **Fourteen rounds earned the result, and twenty-two more measured noise.** Rounds 1 to 14
   (about 6 hours) took the report from 1 of 6 reader tasks to 6 of 6, and usability from 23 to
   34 of 40. Over rounds 15 to 36 (about 7 hours), usability averaged 34.2 with a standard deviation
   of 1.6. A dimension fell in 14% of round-to-round comparisons and rose in another 14%.
2. **The stop rule could not pass.** The rule was that no usability dimension falls and the
   targeted ones rise. With ten dimensions that each fall 14% of the time by chance, a round with no
   fall happens with probability 0.86¹⁰ ≈ 0.22. Five of the 22 plateau rounds had no fall, which is
   what chance predicts (4.9). The loop kept fixing whatever the latest auditor disliked, so its
   cost was set by the auditor's noise rather than by the report.
3. **Fixes broke things that no check could see.** Round 35's fix for hatched campaign bars never
   rendered, because a rule of equal specificity later in the stylesheet won. Only round 36's blind
   auditor found it, 17 minutes later, as a severity-3 finding. Defects like this are mechanical:
   rendering one component in its states and checking it takes seconds.
4. **The monolith is a real cost, but not the main one.** `seikika-report.js` grew from 2,058 to
   5,632 lines during the loop and was touched by 61 of the loop's 161 commits. It has one global
   stylesheet in the shadow root, no linter, no types and no component tests, and the specificity
   bug above came from that. Editing speed was not the bottleneck, though. A fix took minutes, while
   the loop spent its hours waiting on, and reacting to, a noisy signal.
5. **One portal hid one portal's assumptions.** Every task, route and fixture in 36 rounds was
   Seikika's own portal. The first render of a second portal, an OpenFGA repository, showed three
   defects in the first minute (see *Second-portal check*).

## Hypotheses

| Hypothesis | Verdict | Evidence |
|---|---|---|
| A standard web framework such as Vue would speed up development | **Not supported as stated.** Split the file into modules and bundle them. If a framework is wanted, it is React, which D31 already chose, adopted one view at a time. | Edit-to-preview took 10 s: a 1.6 s build and an 8.5 s report. That is not where the hours went. Vue would add a second framework next to the React minimal UI and `@seikika/ui`, against the sprawl budget (D45). The custom-element contract (D01M7X0RYD20000000000000000) is framework-agnostic, so modules keep it intact. |
| Storybook stories for pages and components would speed up the loop | **Supported**, for the reason the loop lacked: fixtures and fast checks | Each story is a fixture an agent can render, capture and probe in seconds, without regenerating a portal. The round 35 to 36 defect would have failed a story probe. Storybook 10 is already the default kit (D31, D01M7YNZFP68W0DR24PEA6T3FVM). |
| A second reference portal (OpenFGA) would show the design is not fitted to one portal | **Supported** | Three defects appeared on its first render, none visible on Seikika's own portal, and two of them are in the box (see *Second-portal check*). |

## Where the time went

| Step | Per round | Over 36 rounds |
|---|---|---|
| Capture (14 routes × 2 viewports × 2 schemes, axe, metrics) | 53 s | about 30 min |
| Rebuild and regenerate the report after an edit | 1.6 s build + 8.5 s report | small |
| Blind auditor (fresh agent, capture plus live page) | about 9 min | about 5.5 h |
| Fixes, checks and recording the round | about 8 min | about 5 h |
| **Round** | **about 17 min** | **about 13 h** (07:50 to 20:45) |

Most of the recording time went to keeping a 37-column table by hand. Round 14's usability
total was written as 35 instead of 34, and two placeholders reached `main` unfilled. About 80
throwaway probe and recorder scripts were written in `/tmp`, each round redoing what the skill's
`capture.cjs` does not: hover and focus checks, overflow per element, contrast on one node.

## The noise, measured

| Dimension | Changes after round 14 (of 22) |
|---|---|
| U9 Accessibility | 13 |
| U8 Honest states | 10 |
| U10 Density and responsiveness | 9 |
| U3 Recognition, U6 Consistency | 8 each |
| U4, U5, U7 | 4 each |
| U1 Answer first | 2 |
| U2 Reader's language | 0 |

The joy total stayed between 12 and 14 throughout. U2 and J4 were 3 in every audit from round 1 on, so
no fix moved them, and the auditors never asked for one that would. The dimensions that moved most (U9,
U8, U10) are those an auditor scores from whichever screen it happens to look at last, and they
are also the ones a mechanical probe can measure: contrast, target size, overflow, and states named
in fixtures.

## Second-portal check

[examples/openfga-portal](../../examples/openfga-portal/README.md) scores an OpenFGA v1.23.0
checkout with generic packages only. Its first render, before any comparison was measured:

| Finding | Where it lives |
|---|---|
| "Against SpiceDB: ahead on 0, behind on 0 of 6 factors … never behind", with no products measured | The box: the track ring and overview line treat an empty comparison as a tie |
| The file-size and pattern-count scorecards sit under "Not on a track" | The box: those two packages take no `track` parameter, unlike `sarif` and `github-actions` |
| staticcheck's scorecard read 0%: it held only files with findings, so every file it held failed | The portal's config: the `sarif` package needs `files` to give clean files a record |

## Recommendation

Adopted as [D01M87PSM9YQGGW3YQ5X4XCJP63](../decisions/D01M87PSM9YQGGW3YQ5X4XCJP63.md):

1. **Modules, not a framework switch.** Split the report into ES modules under `report-ui/`, with
   one module per view and per shared component. esbuild bundles them into the committed
   `seikika-report.js`, so the binary still embeds one file, and CI fails when the bundle is stale.
   The modules later moved to TypeScript bundled by Vite, on the same pins as `ui/`
   ([D01M880QMWD7Z7FTGKJT99AHWBX](../decisions/D01M880QMWD7Z7FTGKJT99AHWBX.md)); the embedded
   file and the freshness check are unchanged.
2. **Stories over fixtures.** Write Storybook 10 stories for every page and shared component, over
   committed report fixtures: Seikika's own portal, the OpenFGA portal, and states the two lack
   (a comparison not measured yet, a portal with no data).
3. **Probes in CI.** A story probe renders every story at 1280 and 390 px and fails on serious axe
   violations (WCAG 2.2 AA, which includes 24 px targets) and horizontal overflow. It replaces most per-round scripts.
4. **A noise-aware stop rule.** A round passes when its probes pass, no severity-3 or 4 finding is
   open, and the median of a small panel of blind auditors does not fall on any dimension the
   round targeted. The loop stops when two rounds in a row move no targeted dimension.
5. **Two reference portals.** Every design-review round renders Seikika's own portal and the
   OpenFGA portal, and the reader tasks include at least one about the second.

## Measured after the change

| Step | Before (loop 2) | After |
|---|---|---|
| See one page in one state after an edit | 1.6 s build + 8.5 s report for the one portal | Storybook dev server hot reload; a static build of all 53 stories takes 1.5 s |
| Mechanical checks per round | 53 s capture of 14 routes on one portal, then the auditor | 33 s probe of 212 checks (53 stories × 4 views) on three portals and two synthetic states, in CI on every push |
| States an auditor can see | Whatever S4S's data happened to contain | Fixtures for a comparison not measured yet and a portal with no data, beside the real ones |
| Defects found mechanically | None by a probe; WCAG 2.2 target size was found by an auditor in round 7 | 46 of 212 checks failed on the first probe: a WCAG 2.5.3 name mismatch on all three portals, an unnamed empty bar, and sideways scroll on two pages of the second and third portals only. All fixed in one commit |

The second portal kept paying for itself after the first render: it also showed that the
`github-actions` package named its scorecards with Seikika's own `maintainer-` prefix (fixed in
0.3.0 with a `prefix` parameter) and that the starter's two scorecards had no titles.

## Sources

- [seikika-report-2026-11-24-2.md](./seikika-report-2026-11-24-2.md): the scores, findings and
  fixes of all 36 rounds; the statistics above are computed from its table.
- `git log` of `cursor/report-ux-loop-2-8dda` (07:50 to 21:13 UTC, 161 commits; squash-merged as
  b15820e): commit counts and the file's growth.
- [design-audit.md](../design/design-audit.md) and the
  [`design-audit` skill](../../plugins/seikika-operator/skills/design-audit/SKILL.md): the loop
  and its stop rule as run.
- [maintainer-sdlc.md](../design/maintainer-sdlc.md): the check pyramid this stage joins.
- Decisions D31 (`D01M7TCJ32P0000000000000000`), D45, `D01M7X0RYD20000000000000000` and
  `D01M7YNZFP68W0DR24PEA6T3FVM`, as cited above.
