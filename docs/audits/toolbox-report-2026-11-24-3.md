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

The second portal also found a box defect that no auditor could see. Its `fetch.sh` clones OpenFGA
into `examples/openfga-portal/checkout/`, which git ignores, and the workspace connector walked every
file on disk: Toolbox for Toolbox scored OpenFGA's `.github/instructions/*.md` as its own agent docs,
and the repository level fell to none in a developer's checkout while CI's fresh clone stayed at
bronze. The walk now lists the files git sees (tracked, or untracked and not ignored) and falls back
to the disk outside a work tree, so fixtures and `results.json --update` built locally match CI.

## Round 2: what the panel found

The probes again passed 212 of 212. Medians rose on four of the five targeted dimensions: Honesty
(U8) from 1 to 2, Clarity (U2) and Orientation (U1) from 2 to 3. The starter's first run went from
fail to success for all three auditors. The round did not pass, because severity 3 and 4 findings
were still open:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The starter's tile said "0 of 240 scored entities are at the top level" beside a gold medal, over "239 docs · 99% at the top level". The tile counted only entities at gold, and the starter's ladder stops at bronze (A, B, C) | Box | The tile counts entities at the top level of their own scorecard, and its medal is the top level those scorecards share, or gold where they differ: "238 of 240" |
| A factor 3 rules ahead and 1 behind read plainly "behind", and the factor tally of 2 ahead and 4 behind outweighed the rule tally of 15 ahead and 6 behind (A, B, C) | Box | Such a factor reads "behind, mixed"; the headline states the counting rule instead of hiding it in a collapsed section; track rows add the rule split ("by rule, 15 ahead and 6 behind of 30") |
| One level carried three names: "265 gold" in the catalog, "within-limit" on the scorecard and "Within-Limit" on pills (B, C) | Box | Kind pages use the scorecard's own level names; pills keep the level's case |
| The catalog grouped only the rows on screen: "57 go files with the same result" where the scorecard said 265 (B) | Box | Groups are counted over every row and then paged |
| "Measured · All 30 rules. A tool, connector or script read the fact" on a comparison with no values (A, B, C) | Box | The basis is named "Tool-checked": how a rule is decided, not whether it has run |
| The hero named the furthest track, but the attention list led with another track's scorecard (C) | Box | The status hero ends with a "Next:" line: the furthest track's lowest scorecard, its next level, how many entities to lift and the rule most of them need |
| The comparison did not say what it leaves out (A, C) | Portal | The OpenFGA track's description ends with what is not covered: other datastores, more than one node, sustained load and other releases |
| Lists of failing entities on campaign pages were unsorted, with no sizes (A, B, C) | Box | Campaign and scorecard lists lead with the furthest-off entities, using each rule's measure, and campaign rows show the evidence ("2629 lines, 1829 over the limit of 800") |
| The starter, one entity short, read "Needs attention" in red (B, C) | Box | With every campaign on target and two or fewer entities at none, the status reads "Almost there" |
| "gitHub releases" (A) | Box | Sentence-casing a description keeps a first word with a later capital, such as GitHub or OpenSSF |

Two findings were declined. Toolbox for Toolbox opens on its repository level where the other
portals open on a status (A): that portal has a repository standard and they do not, and its hero
carries the same status sentence. The unmeasured comparison still shows short ids ("openfga vs
spicedb") (A, B): a product's name lives in its record, and before the first run there is none.

## Round 3: what the panel found

The probes passed 212 of 212. Every targeted median held or rose: Consistency (U5) from 2 to 3,
the rest unchanged at 3, with Coherence (U6) still at 2. Five of the six tasks succeeded for all
three auditors; T6, reading the portals side by side, was partial for two. Four severity-3 findings
were open, so the round did not pass:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The OpenFGA comparison led with "behind on 4 of 6 factors" in red, where its own rules split 15 ahead and 6 behind and the experiment's pre-registered tally reads ahead on 4 (A, B) | Box | A comparison declares its verdict rule, `anyBehind` by default or `majority` ([D01M87WF2QZVPTD8CGDZKY133CD](../decisions/D01M87WF2QZVPTD8CGDZKY133CD.md)); derive applies it, and the report states it wherever it counts factors. The OpenFGA portal declares the majority it pre-registered. A factor with rules on both sides reads "ahead, mixed", "behind, mixed" or "even split", and leads with both counts |
| "Next: CI reliability to bronze" sat under a ring that counts only the top level, so the step did not say what it changes (A) | Box | The line names the top level the rings count: "to bronze, a step toward gold, the level the rings count", or "its top level" |
| A bronze-only scorecard was left out of Toolbox's CI average but counted in OpenFGA's (B, C) | Box | The exclusion has a reason that holds on both portals: on Toolbox's portal the scorecard caps the repository level, so its 100% is a ceiling, not progress. OpenFGA has no repository level, so nothing is capped, and the page says "No repository level: this portal names no repository standards" |
| Three hero shapes: a level board on Toolbox, a status on the others (B, C) | Box | One skeleton on every portal: verdict, the sentence behind it, then what needs attention, then the next step. The repository hero keeps its level board below that |

The first finding is the third time a panel scored the factor count severity 3. Each earlier fix
explained the rule better. This round's fix changed it: the rule is now part of a comparison's
definition, beside its subject and peers, so a pre-registered A/B leads with its own tally and
Toolbox for Toolbox keeps the stricter rule against Backstage. The independent adjudication of the
OpenFGA run ([ADJUDICATION.md](../../experiments/openfga-spicedb-compare/ADJUDICATION.md) §5, §7)
asked for the same change.

## Comparisons are generic

The OpenFGA portal compares OpenFGA v1.23.0 with SpiceDB v1.57.1 through the same comparative
scorecards ([D01M857ZB468ZM2YS6GNSEG6YB9](../decisions/D01M857ZB468ZM2YS6GNSEG6YB9.md)) that Toolbox
for Toolbox uses against Backstage. Six factors and thirty rules, with every bar set before the run,
went in as portal configuration:

| What it took | Where | Box or portal |
|---|---|---|
| Six factor scorecards, their rules, bars, "why" and "what would close it" lines | `examples/openfga-portal/standards/vs-spicedb.yaml` | Portal |
| A track that groups them, with the method and links | `examples/openfga-portal/toolbox.yaml` | Portal |
| Two product records, written by the harness | `examples/openfga-portal/comparisons/{openfga,spicedb}.yaml` | Portal (from `experiments/openfga-spicedb-compare`) |
| A state for a comparison whose records have not landed: "not measured yet", and every section agreeing with it | `report-ui/src/views/compare.js` | Box |
| Product names when a record is missing, instead of `product:openfga` | `report-ui/src/views/compare.js` | Box |

No Rust changed to add it: the comparison engine, the facts and the rendering are the ones Toolbox
for Toolbox already had. The one gap was a state Toolbox's own portal never reaches, a
comparison defined before it is measured. The second portal found it, the first panel scored it
severity 4, and the box now handles it for any portal. The same pages then rendered the measured
comparison without change: the factor cards, the rules OpenFGA is behind on with what would close
each, the largest leads, and how it was measured. The measured result and its adjudication are in
[RESULTS.md](../../experiments/openfga-spicedb-compare/RESULTS.md) and the independent adjudication
beside it in [experiments/openfga-spicedb-compare](../../experiments/openfga-spicedb-compare/).

The second gap showed only once the comparison was measured. The report's one verdict rule, behind
if any rule is behind, was stricter than the experiment's pre-registered tally, so the headline read
"ahead on 2 of 6 factors" where the tally gives four ahead, one at parity and one behind. All thirty
rule standings agreed with the harness; only the count differed. A comparison now declares its
rule (`compare.verdict`, D01M87WF2QZVPTD8CGDZKY133CD): one optional field in the model, the schema
and derive, which the OpenFGA portal sets and Toolbox for Toolbox does not. The adjudication upheld
the verdict and corrected two of SpiceDB's facts in SpiceDB's favour, neither of which changes a
factor.

## Sources

- Rubric and stop rule: [design-audit SKILL.md](../../plugins/toolbox-operator/skills/design-audit/SKILL.md), [design-audit.md](../design/design-audit.md).
- Stories and fixtures: [report-ui/README.md](../../report-ui/README.md).
- The loop this one follows: [toolbox-report-2026-11-24-2.md](./toolbox-report-2026-11-24-2.md) and its process audit, [design-loop-efficiency-2026-11-24.md](./design-loop-efficiency-2026-11-24.md).
