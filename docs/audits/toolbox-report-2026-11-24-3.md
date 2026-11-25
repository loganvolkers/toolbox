# Design audit: one report, three portals, 2026-11-24 (loop 3)

*Status: record of the first design loop run under the revised design-review stage
([D01M87PSM9YQGGW3YQ5X4XCJP63](../decisions/D01M87PSM9YQGGW3YQ5X4XCJP63.md)): story probes first,
captures from Storybook, a blind three-auditor panel scored by median, and two reference portals
plus the starter. About the box: how the report shows what the tools found, whichever portal it
serves.*

Eleven rounds are recorded, and the loop has not stopped. Usability medians rose from 24 to 31 of
40 by round 8, held at round 9, and fell to 30 at round 10 when honest states fell from 4 to 3.
Round 11 held at 30. Judgement rose from 10 to 11 of 16 at round 2 and has not moved since. Every
reader task succeeded for all three auditors from round 5, except rounds 6, 9 and 11 (17 of 18
each). Round 11 did not pass: every median held, and three severity-3 findings were open. Those are
fixed in the build the next round will score. Rounds 1 to 9 used one model; rounds 10 and 11 used
another, which round 10's section records.

## Scope

| Item | Value |
|---|---|
| Surface | `<toolbox-report>` from `report-ui/src`, rendered by Storybook stories over the fixtures in `report-ui/fixtures/` |
| Portals | OpenFGA (`examples/openfga-portal` over OpenFGA v1.23.0, compared with SpiceDB v1.57.1), Toolbox for Toolbox (`portal/`, compared with Backstage), the repo portal starter, and the starter before its first run |
| Stories | 22 page stories: 12 OpenFGA, 6 Toolbox, 1 starter, 3 no-data |
| Viewports | 1280×800 and 390×844, light and dark, plus a full-page desktop light shot |
| Gate before auditors | `npm --prefix report-ui run probe`: axe (WCAG 2.0–2.2 A/AA), horizontal overflow at 390 px, text squeezed into a column a few letters wide, and script errors, on 53 stories × 4 views |
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

A dimension's score is the median of the three auditors. Tasks succeeded counts each auditor
separately, so the maximum is 18.

| Dimension | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 | R11 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| U1 Answer first | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U2 Reader's language | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U3 Recognition over recall | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U4 Hierarchy and scent | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U5 Data integrity | 2 | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U6 Consistency | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 3 | 3 | 3 | 3 |
| U7 Wayfinding and control | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 4 | 3 | 3 | 3 |
| U8 Honest states | 1 | 2 | 3 | 3 | 3 | 3 | 4 | 3 | 4 | 3 | 3 |
| U9 Accessibility | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U10 Density and responsiveness | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| J1 Visceral | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| J2 Behavioral | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 | 2 |
| J3 Reflective | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| J4 Voice | 2 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 | 3 |
| U total | 24 | 27 | 29 | 29 | 29 | 29 | 30 | 31 | 31 | 30 | 30 |
| J total | 10 | 11 | 11 | 11 | 11 | 11 | 11 | 11 | 11 | 11 | 11 |
| Tasks succeeded (of 18) | 13 | 16 | 16 | 17 | 18 | 17 | 18 | 18 | 17 | 18 | 17 |

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

## Round 4: what the panel found

The probes passed 212 of 212. Every median held: the same ten 3s and two 2s as round 3, with
Coherence (U6) and Behavioural joy (J2) at 2. All six tasks succeeded for two auditors and T6 was
partial for the third. Four severity-3 findings were open, so the round did not pass:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The two portals now count factors by different rules under the same ring and the same words, and only body text says so (A, B, C) | Box | The rule is named where the tally is shown: a "counted by the majority of rules" or "behind if any rule is behind" tag beside the factor counts on the compare hero and on every comparison track card, and the headline ends with the rule |
| "Not covered" sat at the bottom of a 2,900 px page, not beside the method (A) | Box | A track description's "Not covered:" sentence moves up into the compare hero, under Method |
| Custom top levels ("within-limit", "ten") were painted as gold medals, and the entity page title-cased them as "Within-Limit" (A) | Box | A one-level scorecard's medal carries a check, not the gold star, and custom level names keep their own case and quotes |
| The track ring averaged shares at the top level beside "Medals: 3 none", with nothing to reconcile them (A) | Box | The card adds "51 entities still at none" under the ring, and the medal row says "with no medal yet" |
| The behind side had no ratios: "Widest lead: 3.0× smaller" but the 16× gap only on the factor page (B) | Box | "Widest gap" sits beside "Widest lead", and "Where OpenFGA is behind" opens with the same ratio tiles as the leads |
| The starter's "Repository standards" title met "this portal names no repository standards" (C) | Box and portal | The note now says the portal does not combine its scorecards into one level. The starter's scorecards (and OpenFGA's copies) no longer describe "a pilot repository (examples/repo-portal-starter)" |

The panel's severity-2 findings that two or more auditors raised were fixed in the same pass:

- campaigns show progress since the saved start, or say there is none;
- campaign goals name rule titles, not ids ("rules ten");
- the not-measured state says "not measured", not "? openfga unknown", and no longer doubles its badge;
- the empty portal hides level legends, filters and an empty Campaigns tab;
- grouped rows name their furthest-off member ("tuples.go, 2629 lines, 1829 over the limit of 800");
- same-named members show their paths;
- `+uncommitted` is explained on the factor page;
- the header reads "facts snapshot #7", or "no facts collected yet".

A scorecard whose ladder stops below gold still counts in OpenFGA's ring, and is still left out of
Toolbox's where it caps the repository level. The card now names it in both portals.

## Round 5: what the panel found

The probes passed 212 of 212 and every median held, with Coherence (U6) and Behavioural joy (J2)
still at 2. All six tasks succeeded for all three auditors, the first round with no partial. Three
severity-3 findings were open, so the round did not pass:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The ratio tiles added in round 4 for where a product is behind used the lead's green, so "16× lower" and "707× fewer" read as wins (B; A at severity 2) | Box | Gaps are drawn in the behind red with ▼, on the tiles and on the "Widest gap" line |
| Naming the counting rule beside the tally was not enough: the same 4 / 1 / 1 still reads as one measure across portals (A, B, C) | Box | Every comparison now also states its tally under the other rule ("Counted the other way, where a factor is behind if any one of its rules is behind, it would be ahead on 2 and behind on 4"), or that it is the same. The hero counts mixed factors under each figure and shows the rule tally (15 / 9 / 6) at the same size as the factor tally |
| One pass/fail split was drawn three ways: level colours on the scorecard, green and red on the rule page, grey on the campaign (A) | Box | The rule page draws a pass in the colour of the rule's level, as the scorecard does, and says so. The campaign bar keeps its grey-until-target encoding, which its caption explains |

Severity-2 findings fixed in the same pass:

- the "Next" line leads with the work ("10 entities at none need Every job has timeout to reach bronze on CI reliability") and says when the track's percentage moves;
- fix commands on an entity page name that entity (`toolbox assess brief crate:toolbox-cli`);
- the first run says connectors are the ones listed under `connectors:` in the portal's `toolbox.yaml`;
- a campaign rendered without its baseline says so, instead of contradicting its own description.

Declined, with reasons:

- **Product names in the not-measured state.** They come from the comparison records, so before the
  harness runs there is no name to show. The ids stay.
- **Toolbox design links on OpenFGA's Go scorecards.** They belong to the `file-size` package the
  portal includes, and the package is Toolbox's.
- **A report-wide entity search.** It is a feature, not a fix, and is left for its own change.

## Round 6: what the panel found

The probes passed 212 of 212 and every median held: U6 and J2 at 2, the rest at 3. Every task
succeeded, except one partial on T6 (C). Five severity-3 findings were open, and they repeat two
themes from earlier rounds, so the round did not pass:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The OpenFGA hero names the track furthest from its goal, then gives a step that does not move its percentage (A, B, C) | Box | "Next" keeps the quickest medal and says why it comes first ("CI and supply chain is the track furthest from its goal"). When that medal is not a top level, a second line names the step that moves the track's percentage most: "lift 9 entities from bronze to silver on CI speed, about 16 points" |
| The factor tally's counting rule differs by portal, but the overview and track tallies show only the numbers (A, C; B at severity 2) | Box | Every tally names its rule. Where the other rule gives a different count, the tally also gives that count: "counted by the majority of rules (behind if any rule is behind: ahead on 2, behind on 4)". The rule stays per comparison (D01M87WF2QZVPTD8CGDZKY133CD) |
| Third-party actions pinned counts in OpenFGA's CI ring but is left out of Toolbox's (A; B, C at severity 2) | Box | One rule in every portal: each scored scorecard counts at its own top level. A scorecard that caps the repository level gets a lock note and is no longer left out |

Severity-2 findings fixed in the same pass:

- the starter's all-clear line names the scorecard that still has entities below its top level;
- the top-level tile draws the highest top level present and says when top levels differ;
- an entity's headline level says "✓ top" when it is at the top of every scorecard;
- the scorecard page draws a not-scored scorecard with the same "?" medal as the list, and its
  sidebar says "Nothing is scored yet" instead of a sort order;
- the sidebar's comparison-factor heading links to Compare;
- level pills in entity rows wrap instead of running into the name;
- the first-run "Next" leads with `toolbox doctor`.

Declined, with reasons:

- **The OpenFGA example in a crate rule's "Why".** Toolbox's own tests do use OpenFGA, so on
  Toolbox's portal the example is about the portal's subject.
- **Future tense in the not-measured method text.** The method is the comparison's own description
  from the manifest, not report copy.
- **Product names in the not-measured state** and **entity search**, as in round 5.

## Round 7: what the panel found

Round 7 was the first round built from TypeScript sources bundled by Vite
([D01M880QMWD7Z7FTGKJT99AHWBX](../decisions/D01M880QMWD7Z7FTGKJT99AHWBX.md)). Before capture, the
migration was checked as a no-op on the pages: matching DOM snapshots, 212 of 212 probes and the same
interactions. Its type check also exposed a dead filter: the Tool-checked filter on a scorecard
showed no rules, and now shows 34. The probes passed 212 of 212. Honesty (U8) rose from 3 to 4 and
no median fell, leaving U6 and J2 at 2. Every task succeeded for all three auditors. Three
severity-3 findings were open, so the round did not pass:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The two portals still lead with factor counts under different rules, and the other count sits mid-sentence (A, B; C at severity 2) | Box | Every tally shows both counts in the same order on every portal: strict ("behind if any rule is behind") first, then majority. The comparison's declared rule is marked "this comparison's verdict" on that row. The headline ends with the rule it used. The rule stays per comparison (D01M87WF2QZVPTD8CGDZKY133CD); the two portals now differ only in which row is marked |
| Custom top levels ("within-limit", "clean", "ten") drawn in gold under a legend that lists only metals (B; A at severity 2) | Box | The legend names each scorecard's own levels under the metal they rank as: "gold, or a scorecard's own level ranked as gold: “clean”, “ten”, “within-limit”" |
| The second next step ("lift 9 entities from bronze to silver on CI speed") named no rule (C; A, B at severity 2) | Box | It names the rule and how many entities fail it. If no rule fails and only an unrun check holds them, it says so: "fail nothing for silver, but its check Fast feedback was not run in this build" |

Severity-2 findings fixed in the same pass:

- catalog groups list their furthest-off member first, with evidence ("tuples.go (2629 lines, 1829 over the limit of 800)"), and failing groups come first;
- the rule page's outcome bar runs fail, unknown, pass, worst on the left like every level bar;
- the scorecard sidebar's comparison-factor link is an inline link, not a dangling heading;
- "even split" became "parity, mixed", the word Compare uses;
- the catalog filter says "Filter by name…", not "Filter GoFile by name…".

Declined, with reasons:

- **Product names in the not-measured state**, as in rounds 5 and 6.
- **Toolbox design links on OpenFGA's Go scorecard** (B, C), as in round 5: they come from the `file-size` package.
- **"Passes by default" on an unassessed rule** (B, C): that is how an assessed rule is scored (no open finding, no failure), and the page already marks it "not assessed yet". Changing it changes scoring, not the page.
- **Campaigns built without a baseline** (A, B, C): the stories render the fixture without a baseline file, and the page says so. Showing momentum needs the   baseline in the fixture, which is a fixture change, not a page fix.

## Round 8: what the panel found

The declined baseline item from round 7 was fixed before capture after all. The published report
and the OpenFGA example now build with `--baseline` (the pages workflow,
`scripts/regen-site-report.sh` and `report-ui/scripts/fixtures.sh`), and OpenFGA gains a committed
`results.json`. Campaigns therefore show "since the start" rather than a missing-baseline note. The
probes passed 212 of 212 and every task succeeded for all three auditors. Consistency (U6) rose from
2 to 3 and wayfinding (U7) from 3 to 4. Honesty (U8) fell from 4 to 3, so the round did not pass.
Three severity-3 findings were open:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The strict count calls a factor 3 rules ahead and 1 behind "behind" (A; B at severity 2) | Box | The strict count says what it measures: "ahead with no rule behind on 2, with a rule behind on 4". The majority count keeps "ahead" and "behind". The order stays strict first, and the declared rule's row is still the one marked |
| The two portals give their headline under different rules, and the factor page defines a factor win differently on each (B; C at severity 2) | Box | One headline shape on every portal: "by either count" when the counts agree, otherwise both counts, strict first, with the verdict marked. The factor page defines both counts on every portal and then names the comparison's rule |
| The crate page says "lowest of 2 scorecards" without naming the scorecard; the one holding it comes second, about 1,700 px down (C) | Box | The header names it and its blocking rule: "Held at bronze by Crates, reviewed by agents: “Assessed” fails", a link that jumps to the section. Sections are ordered lowest level first |

Severity-2 findings fixed in the same pass:

- a next step that waits on an unrun check is headed "Unverified, run a check first" and says "may already meet silver … so nothing is known yet", instead of "fail nothing for silver";
- a header medal reached only through default passes on an unassessed scorecard is drawn faded inside a dashed ring;
- entities whose basenames collide are named by the shortest path suffix that tells them apart ("internal/check/check.go"), wherever the report shortens an id;
- in the not-measured state, products take the spelling the portal's own track prose uses ("OpenFGA vs SpiceDB", not "openfga vs spicedb"), and the empty section reads "Where each product leads (not measured yet)";
- every compare hero has a "Not covered:" line, and says "not stated" when the track gives none; Toolbox's comparison gained one taken from its benchmark's threats to validity;
- the OpenFGA track says "Toolbox's maintainers, not OpenFGA's or SpiceDB's, chose the factors";
- counts carry units ("Scorecards by medal", "by lowest level, 5 at none"), and the column is "Lowest level", not "Weakest entity";
- the empty Scorecards tab gives the `toolbox doctor` next step, and an empty scorecard says "No entities collected yet";
- no text below 12 px (bare `<small>` and inline `code` have a 12 px floor), and the scorecard search placeholder fits a phone.

Declined, with reasons:

- **Toolbox design links on OpenFGA's Go scorecard** (A), as in rounds 5 and 7.
- **One hero model for every portal** (B): a portal that combines its scorecards into a repository level shows it; one that does not says "No overall level". The difference is the portal's configuration, and the hero names it.
- **Shorter compare pages and track cards** (B, C at severity 1–2): deferred to a density pass. Folding them now would hide the method a reader needs before trusting the verdict.

## Round 9: what the panel found

The probes passed 212 of 212. Every task succeeded except T6 for one auditor (A, partial). Honesty
(U8) rose back to 4. Wayfinding (U7) fell from 4 to 3, the U and J totals held at 31 and 11, and no
targeted median fell. The round did not pass, because three severity-3 findings were open, one of
them a regression from round 8:

| Finding (auditors) | Box or portal | Fix |
|---|---|---|
| The no-data Scorecards callout added in round 8 lays out each inline piece as a flex column on a phone, so "run" reads "r u n" and `toolbox doctor` reads "tool box doct or" (A, B) | Box | The callout is a paragraph, styled like the overview's first-run step. The probes gained a check, run on every story and view, for text squeezed into a column a few letters wide. Run against the round 9 build, it failed exactly the two mobile views of that story |
| The overview's "Next" fix moves the track's percentage by 0 points; the only step with a stated gain is an unverified check (C, A; B at severity 2) | Box | The next step is the verified step that moves the track's percentage most, with its gain ("the track gains about 4 points"). The next medal follows as a second line, saying it "adds no points to the track's percentage until they reach" its top level. An unverified step is shown only when it would gain more |
| The comparison leads with the strict count while the majority count is its verdict, stated three times in the first viewport (C, B; A at severity 3 as a consistency finding) | Box | The headline is the declared verdict: "Ahead on 4 of 6 factors, behind on 1, at parity on 1, by the majority count, this comparison's verdict". A secondary line carries a "Verdict: majority count" pill and the other count once. The same order is used for the track strip and the ring's label. D01M87WF2QZVPTD8CGDZKY133CD names no order, only that the rule is stated |

Severity-1 and severity-2 findings fixed in the same pass:

- the header reads "entities tracked … data from snapshot #7", not "entities in the box … facts snapshot #7", on every portal;
- a links line is headed "Further reading, from loganvolkers/toolbox:", so package docs that every portal includes say whose docs they are;
- a repository capped by a standard that is fully met says so first: "Capped at bronze: Third-party actions pinned is fully met, but its top level is bronze…";
- the catalog lists groups and rows furthest behind first, as a scorecard page does;
- a baseline saved in the same build reads "no change yet … Progress appears here once a later build moves these numbers", not "0% of the way";
- the not-measured compare page's jump link is "Who leads", matching its section;
- a campaign page draws its percentage bar once, in the header, unless milestones need a second bar;
- the entity-count tile names its largest kind when that kind is a third or more of the total ("the largest group is 493 Go files (staticcheck)"), and kind names keep proper nouns capitalised;
- the no-data overview's last column is "Status", not "At none"; links have a 3 px underline offset so underscores show; the scorecard sidebar reads "6 comparison factors, on the Compare tab".

Declined, with reasons:

- **Tabs on two rows on a phone** (B, C at severity 1): below 600 px the tabs are a grid on purpose, so every view stays visible without sideways scrolling.
- **One hero model for every portal** (A, C), as in round 8.
- **A per-scorecard summary table on the entity page** (C at severity 2): the header already names the scorecard holding the entity and its rule. The table is a candidate for the density pass.

## Round 10: what the panel found

Rounds 1 to 9 were scored by Claude Opus. That model was out of weekly usage, so this round's three
auditors were Grok 4.7, still blind to the source, the diff and each other, on the same rubric and
the same screenshots. The probes passed 212 of 212, and every task succeeded for all three
auditors. No median rose. Honest states (U8) fell from 4 to 3, so the usability total fell from 31
to 30, and the round did not pass. The model change is a reason to read the drop cautiously; the
severity-3 findings are not, because each one is a sentence still on the page:

| Finding (auditors) | Fix |
|---|---|
| The overview's next box still offers three actions, and the unverified one claims the larger gain (A, B, C) | One "Next" line. The next medal and the unverified check sit under "Other steps". The unverified line says the gain is unknown until the check runs |
| An unassessed rule reads "passes, not assessed yet", and a level whose checks did not run still wears its medal (A, B; C at severity 2) | The outcome is "not assessed yet" and is left out of "N of M checks pass". When a check the shown level needs did not run, the header says "Not established" and draws the unknown medal |
| The comparison states both counts in the first sentence (B, C) | The headline is one count. The other count is the first line of "How a factor and a rule are counted" |
| With no data, the scorecard intro still defines "at none", and the empty section is titled "Most common failures" (C) | The intro says there is nothing to show. The section is "Nothing has failed or passed" |

The OpenFGA overview's one undersized control, the "Ten" link at 22 px (A and C, severity 2), now
reads "a score of 10" and next-step links have a 24 px minimum width. "Other steps" is at least 24
px tall.

## Round 11: what the panel found

The same model as round 10, Grok 4.7, scored this round, still blind to the source, the diff and
each other. The probes passed 212 of 212. Every median held, at 30 of 40 and 11 of 16. One auditor
marked T4 partial, so tasks succeeded 17 of 18. The round did not pass, because three severity-3
findings were open:

| Finding (auditors) | Fix |
|---|---|
| The overview's next step is OpenSSF Scorecard, about 4 points, while the track card's first line is CI reliability, a medal that adds no points (A, B) | The track card's "Next" line is that same step, and it names the two checks: "License and Pinned-Dependencies on OpenSSF Scorecard need a score of 10. The track gains about 4 points." CI reliability is a separate "Lowest level" line, which says lifting it adds no points until its top level |
| `toolbox-cli` is headlined "Not established" while Crates, reviewed by agents is bronze and Rust crates is drawn as provisional silver (A, C) | The header is the worst level a scorecard has established, here bronze, and names the rule that holds it (Assessed). A scorecard whose checks below the reported level did not run says "not established" and names those checks. It does not wear the higher level |
| A Tool-checked badge sits between the rule and the verb in the next-step sentence (C) | The badge stays on the rule. The next sentence is the action, then the point gain, with nothing between the rule and the verb |

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
