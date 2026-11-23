# gate-repair-ab: independent adjudication (2026-11-23)

**Outcome against the bar in [README.md](./README.md) §5, §5.1, §5.2 and §5.3:**

- **Round 1: Toolbox lost to reviewdog.** G1 holds, but G2 (calls) and G3 (read) **fail**.
- **Round 2: the improvement is not shown under §5.** G1′ and G6 hold, and every other hypothesis
  is inconclusive. G5 is inconclusive twice over: its gap is inside the noise, and arm B's read
  moved more than its range allows. The designer's reading in §0.3 is correct.
- **Round 3: the improvement is shown under §5.1.** G7 and G8 hold, and G9 and G10 are
  inconclusive, not failed. G11, G2″ and G3″ are inconclusive.
- **Round 4: Toolbox does not beat reviewdog under §5.2.** G12 and G16 hold. G13 (calls)
  **fails**. G14, G15 and G17 are inconclusive. The designer's labels in §0.5 are correct, but
  four of its explanations are wrong or unsupported (see *The designer's reading of round 4,
  checked*).
- **Round 5: Toolbox does not beat reviewdog under §5.3.** G18 holds. G19 (calls), G20 (read) and
  G21 (gate read) are inconclusive, and so are G22 and G23 (attribution). None fails. The
  designer's labels in §0.6 are correct, but two of its factual claims are wrong and one is
  unsupported (see *The designer's reading of round 5, checked*).

What round 3 shows is narrow. The change cut what an agent reads from the gate, from a median of
18,751 characters to 3,876. That gap is more than four times the wider range. It does not show
that a repair costs fewer calls or less reading in all, or that Toolbox's gate is cheaper than
reviewdog's: every such hypothesis in rounds 2 and 3 is inconclusive, and round 1's are failures.
The round 3 measure was chosen after rounds 1 and 2 were seen, and it measures the product change
almost directly. Read G8 as confirming that the change does what it says, not as an independent
win.

Round 4 is a loss on calls. With the second change, a repair took a median of 7 calls against
reviewdog's 3, and both arms' ranges were 1 call. That gap is not just editing style. Half of it
came from arm A's extra exploration, and either half alone fails G13 (threat 12). The read hypotheses
can't separate the arms. Three of arm A's five runs read `pull_request.yaml` with no recorded
result, so arm A's recorded read is low by about 13.5k characters in each (threat 13). G16
confirms that the second change shortened the gate's output, as G8 did for the first.

Round 5 neither wins nor loses: no comparison separates the arms. Arm A's medians are below reviewdog's on all three measures: 3 calls
against 4, 10,859 characters read against 13,960, and 3,102 of gate output against 5,918. Every gap
is inside the wider range, and no correction of the missing results changes that (threat 24). That
Toolbox no longer loses is not the change's doing. Round 4's Toolbox, run alongside as A⁻, also ties reviewdog on every
measure. And the same Toolbox that took a median of 7 calls in round 4 took 4 in round 5
(threat 22). Calls still turn on how the agent edits (threat 23). The gate-read gap is mostly what
B's agents chained onto their gate runs: on the gates' own output, the arms are 52 characters apart
(threat 25).

**The Toolbox changes between rounds (`dc42499` before round 2, `76a1140` before round 4,
`c1f9b22` before round 5) fit (P5). The rival changes (`3f1b225` before round 4, `eb45c5d` before
round 5) and the measure change `295b0d3` are harness-only (P4).** One caution on `c1f9b22`: its
five-line gap fill is what quotes the planted typo (fit audit). No run was excluded or rerun, every
launched prompt matches its committed file byte for byte, and no label is void.

The adjudicator is a cloud agent that did not design the experiment, build an arm, or change the
product ([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P7). It
recomputed every median, range and *W* from [results/r1.tsv](results/r1.tsv),
[results/r2.tsv](results/r2.tsv), [results/r3.tsv](results/r3.tsv),
[results/r4.tsv](results/r4.tsv) and [results/r5.tsv](results/r5.tsv). It also read all 57
runtime transcripts: 27 for rounds 1–3, 15 for round 4 and 15 for round 5. For rounds 4 and 5 it
rescored every workspace (from copies, in round 5), recounted every transcript with `count.py`,
and reproduced each arm's first gate output from a fresh copy of the planted branch. Threats 10,
20 and 30 state the limits of its independence.

## Per hypothesis

Medians are over an arm's runs in a round. The range is maximum minus minimum, and *W* is the wider
of the two ranges compared. A is the changed Toolbox, A⁻ (rounds 3 to 5) the previous one, and B
is reviewdog. Subscripts in G4–G5 are round numbers. Round 5's *read* is `read` in `r5.tsv`, with a
missing `read_file` result imputed (§5.3).

| Hypothesis | Label | Numbers |
|---|---|---|
| G1 (r1) A correct every run | **holds** | 3 of 3 correct |
| G2 (r1) A fewer calls than B | **fails** | A 6, 7, 7 (med 7, range 1); B 4, 4, 4 (med 4, range 0). med(A) − med(B) = 3 > *W* = 1 |
| G3 (r1) A reads less than B | **fails** | A 28,023; 29,767; 36,248 (med 29,767, range 8,225); B 8,755; 11,160; 11,178 (med 11,160, range 2,423). med(A) − med(B) = 18,607 > *W* = 8,225 |
| G1′ (r2) | **holds** | 3 of 3 correct |
| G2′ (r2) | **inconclusive** | A 4, 6, 7 (med 6, range 3); B 3, 4, 6 (med 4, range 3). A − B = 2 ≤ *W* = 3 |
| G3′ (r2) | **inconclusive** | A 11,021; 13,249; 31,901 (med 13,249, range 20,880); B 20,354; 25,416; 26,915 (med 25,416, range 6,561). B − A = 12,167 ≤ *W* = 20,880 |
| G4 (r2) the change cut A's calls | **inconclusive** | med(A₁) − med(A₂) = 7 − 6 = 1 ≤ *W*(A₁, A₂) = 3. The noise control passes: B's median moved 4 → 4, which is 0 ≤ *W*(B₁, B₂) = 3 |
| G5 (r2) the change cut A's read | **inconclusive** | med(A₁) − med(A₂) = 29,767 − 13,249 = 16,518 ≤ *W*(A₁, A₂) = 20,880. The noise control also fails: B's median moved 11,160 → 25,416, which is 14,256 > *W*(B₁, B₂) = 6,561 |
| G6 (r2) A still correct | **holds** | 3 of 3 correct |
| G7 (r3) A correct every run | **holds** | 5 of 5 correct. A⁻ and B were also 5 of 5 |
| G8 (r3) the change cut gate read | **holds** | A 3,801; 3,876; 3,876; 3,952; 4,242 (med 3,876, range 441); A⁻ 16,695; 16,714; 18,751; 19,137; 19,884 (med 18,751, range 3,189). A⁻ − A = 14,875 > *W* = 3,189 |
| G9 (r3) the change cut calls | **inconclusive** | A 3, 4, 5, 6, 6 (med 5, range 3); A⁻ 6, 6, 7, 8, 8 (med 7, range 2). A⁻ − A = 2 ≤ *W* = 3 |
| G10 (r3) the change cut read | **inconclusive** | A 11,021; 14,769; 24,517; 26,432; 33,837 (med 24,517, range 22,816); A⁻ 25,429; 27,839; 28,614; 29,700; 30,386 (med 28,614, range 4,957). A⁻ − A = 4,097 ≤ *W* = 22,816 |
| G11 (r3) A's gate read below B's | **inconclusive** | A med 3,876 (range 441); B 3,112; 3,263; 3,263; 3,557; 4,632 (med 3,263, range 1,520). The median favours B: A − B = 613 ≤ *W* = 1,520. Threat 3 says why this label flatters A |
| G2″ (r3) A fewer calls than B | **inconclusive** | A med 5 (range 3); B 3, 3, 4, 5, 6 (med 4, range 3). A − B = 1 ≤ *W* = 3 |
| G3″ (r3) A reads less than B | **inconclusive** | A med 24,517 (range 22,816); B 10,266; 10,333; 17,883; 24,755; 25,169 (med 17,883, range 14,903). The median favours B: A − B = 6,634 ≤ *W* = 22,816 |
| G12 (r4) A correct every run | **holds** | 5 of 5 correct. A⁻ and B were also 5 of 5. All 15 rescore to the committed JSON |
| G13 (r4) A fewer calls than B | **fails** | A 6, 6, 7, 7, 7 (med 7, range 1); B 3, 3, 3, 4, 4 (med 3, range 1). A − B = 4 > *W* = 1 |
| G14 (r4) A reads less than B | **inconclusive** | A 9,542; 9,761; 10,297; 10,692; 20,464 (med 10,297, range 10,922); B 10,340; 10,340; 10,374; 11,171; 13,330 (med 10,374, range 2,990). B − A = 77 ≤ *W* = 10,922. Threat 13: A's read is undercounted, and corrected, the median favours B |
| G15 (r4) A reads less gate output than B | **inconclusive** | A 2,527; 2,527; 2,527; 2,527; 2,669 (med 2,527, range 142); B 3,186; 3,337; 3,337; 3,371; 5,099 (med 3,337, range 1,913). B − A = 810 ≤ *W* = 1,913. Threat 14 says why this label flatters A |
| G16 (r4) the change cut gate read | **holds** | A⁻ 3,876; 3,943; 3,952; 4,018; 4,018 (med 3,952, range 142). A⁻ − A = 1,425 > *W* = 142 |
| G17 (r4) the change cut read | **inconclusive** | A⁻ 12,576; 22,339; 24,198; 26,651; 27,232 (med 24,198, range 14,656). A⁻ − A = 13,901 ≤ *W* = 14,656. Threat 13: the label rests on incomplete data in both arms |
| G18 (r5) A correct every run | **holds** | 5 of 5 correct. A⁻ and B were also 5 of 5. All 15 rescore to the committed scores |
| G19 (r5) A fewer calls than B | **inconclusive** | A 3, 3, 3, 6, 6 (med 3, range 3); B 3, 4, 4, 4, 4 (med 4, range 1). B − A = 1 ≤ *W* = 3 |
| G20 (r5) A reads less than B | **inconclusive** | A 9,121; 9,372; 10,859; 18,697; 20,189 (med 10,859, range 11,068); B 7,016; 12,869; 13,960; 14,172; 16,213 (med 13,960, range 9,197). B − A = 3,101 ≤ *W* = 11,068. Neither the recorded read nor a corrected `reviewdog-3` changes the label (threat 24) |
| G21 (r5) A reads less gate output than B | **inconclusive** | A 3,102; 3,102; 3,102; 3,178; 3,320 (med 3,102, range 218); B 338; 4,398; 5,918; 5,918; 8,740 (med 5,918, range 8,402). B − A = 2,816 ≤ *W* = 8,402. With `reviewdog-3` corrected, *W* = 4,342, still above the gap. Threat 25 says why this label flatters A |
| G22 (r5) the change cut calls | **inconclusive** | A⁻ 3, 3, 4, 6, 7 (med 4, range 4). A⁻ − A = 1 ≤ *W* = 4 |
| G23 (r5) the change cut read | **inconclusive** | A⁻ 8,338; 9,027; 9,238; 10,741; 22,224 (med 9,238, range 13,886). The median favours A⁻: A − A⁻ = 1,621 ≤ *W* = 13,886 |

Every run in every arm and round was scored correct, and all 57 diffs are byte-identical.
Correctness therefore can't separate the arms (threat 6). The designer's predictions were right in
rounds 1 and 3. In round 1 the prediction was G1 holds, G2 inconclusive or fails, and G3 fails. In
round 3 it was G7 and G8 hold, and G9, G10 and G11 are inconclusive. In round 4 one of six was
wrong: G13 was predicted inconclusive and fails. G12 and G16 hold, G14 and G17 are inconclusive,
and G15 is inconclusive, one of the two outcomes predicted for it. In round 5 three of six were
wrong: G20, G22 and G23 were predicted to hold and are inconclusive. G18 holds, and G19 and G21
are inconclusive, as predicted.

Not registered, and reported only:

- In round 4, A⁻ against B is inconclusive on every measure. On calls the gap is 6 − 3 = 3 ≤
  *W* = 3, on read 13,824 ≤ 14,656 and on gate read 615 ≤ 1,913.
- In round 5, A⁻ against B is also inconclusive on every measure. On calls the gap is 4 − 4 = 0 ≤
  *W* = 4, on read 4,722 ≤ 13,886 and on gate read 3,315 ≤ 8,402 (≤ 4,342 with `reviewdog-3`
  corrected).
- In round 5, arm A's gate read is above A⁻'s, as §5.3 expected: 3,102 against 2,603, a gap of
  499 ≤ *W* = 1,486.

## Is the improvement shown? Does Toolbox beat reviewdog?

| Rule | Requires | Result |
|---|---|---|
| §5 (round 2) | G6 holds, at least one of G4 and G5 holds, neither fails | **Not shown.** G6 holds. G4 and G5 are both inconclusive |
| §5.1 (round 3) | G7 holds, G8 holds, neither G9 nor G10 fails | **Shown.** G7 and G8 hold. G9 and G10 are inconclusive |
| §5.2 (round 4): Toolbox beats reviewdog | G12 holds, at least one of G13, G14 and G15 holds, none of them fails | **Not shown.** G12 holds. G13 fails, and G14 and G15 are inconclusive. The rule fails on both of its last two conditions |
| §5.3 (round 5): Toolbox beats reviewdog | G18 holds, at least one of G19, G20 and G21 holds, none of them fails | **Not shown.** G18 holds. G19, G20 and G21 are inconclusive. The rule fails on its second condition only |

The §5.1 verdict follows the rule as written, and the rule was committed before round 3's
workspaces were built. Its scope is that the gate is cheaper to read. The total cost of a repair is
unproven, and Toolbox is not shown to beat reviewdog on any measure.

The §5.2 verdict is robust. Correcting the undercounted read can't make G14 or G15 hold (threats 13
and 14). G13's failure survives removing either cause of the gap (threat 12). §5.2 says the round
is not rerun.

The §5.3 verdict is robust to every correction this adjudication could make:

| Variant | G20 (read) | G21 (gate read) | G23 (read, A vs A⁻) |
|---|---|---|---|
| As registered (`read`) | 3,101 ≤ 11,068 | 2,816 ≤ 8,402 | A − A⁻ = 1,621 ≤ 13,886 |
| `readRecorded` instead of `read` | A med 9,372, range 12,023: 4,588 ≤ 12,023 | unchanged | A − A⁻ = 9,372 − 9,027 = 345 ≤ 12,023 |
| `reviewdog-3` corrected (+4,272) | B med 13,960, range 4,925: 3,101 ≤ 11,068 | B med 5,918, range 4,342: 2,816 ≤ 4,342 | unchanged |
| Both | 4,588 ≤ 12,023 | 2,816 ≤ 4,342 | 345 ≤ 12,023 |

No correction of `reviewdog-3` could make G21 hold: B's other four runs alone span 4,342. G20's
*W* is arm A's own spread. Even without `toolbox-1`, the one A run with an imputed result, arm A's
read ranges 9,576. §5.3 says the round is not rerun.

## Fit audit (P5)

### Before round 2: `dc42499` fits

| Check | Finding |
|---|---|
| Touches no harness file | `git show --stat dc42499` lists `crates/toolbox-cli/src/{causes,report}.rs`, `crates/toolbox-cli/tests/ratchet.rs`, the new decision, `docs/design/{ratchets,results-spec,root-causes}.md`, the `report` skill, `portal/results.json` and one site page. None is under `experiments/gate-repair-ab/` (0 of 10 files). No harness schema or convention is referenced in the code |
| box-ab friction #4 existed before 60723d6 | Yes. `experiments/box-ab/README.md` at `703b53a`, line 87, reads "The gate's regression lists every finding in the file and doesn't mark the new ones; the baseline keeps only counts (B6, A6) · **Open:** needs a fingerprint per finding". It was added in `144caf2` (2026-11-22 17:48), before 60723d6 (2026-11-23 10:59) |
| root-causes C1 existed before 60723d6 | Yes. At `703b53a`, `docs/design/root-causes.md` line 75 reads "C1 · Evidence identity: say which evidence items are new … by recording items (or their fingerprints) in the baseline". The file's history goes back to `0b9b225` and `567ed12` (2026-11-21). The results spec's "fingerprints per finding" row also predates it |
| Uses outside the harness are real | Mostly. (a) The bare `--baseline` gate that the stdout change targets is the command `AGENTS.md` tells maintainers to run on this repo. (b) Evidence keys and `+` marking reach the regression detail and therefore `--verdict` and the action's annotations. (c) `--against` gets the same keys (`results_at` now returns them). One overstatement: README §0.2 lists "`actions/report` and CI logs" as uses, but `actions/report/run.sh` passes `-o md --out`, so the verdict-on-stdout half of the change does not reach the action or CI. Only the evidence-marking half does |
| Shaped to the measure? | Partly motivated by it, but it doesn't favour Toolbox. The decision says the verdict moved to stdout because "some agent harnesses record only stdout", and README §0.2 says "so the measure now counts it". That brings A's verdict *into* the count, which raises A's measured read. The part that cut A's read is dropping the full report from a gate's default output. A user reading a terminal benefits from that just as much |
| Other changes between rounds | Between rounds 1 and 2: `3762524` (README, round 2 prompts). Between rounds 2 and 3: `d30dcb2` (harness: `count.py` adds `gateChars`, `collect.py` reads `arms.tsv`, `round.sh` adds the A⁻ arm, rounds 1–2 re-collected) and `d53569d` (README). No product change. `git diff --stat dc42499 d53569d -- crates ui schema packages` is empty. The re-collection added `gateRead` and left every round 1–2 call and read figure unchanged. Every harness change applies to all arms |

### Before round 4: `76a1140` fits, and `3f1b225` is a harness-only rival fix (P4)

| Check | Finding |
|---|---|
| `76a1140` touches no harness file | `git show --stat 76a1140` lists 12 files: `crates/toolbox-cli/src/{causes,report}.rs`, `tests/ratchet.rs`, the new decision, two design docs, `packages/github-actions` (fragment and tests), the `report` skill, `portal/STANDARD.md`, `portal/results.json` and `portal/tests`. None is under `experiments/gate-repair-ab/`. The code and package diffs never mention openfga, `pr-summary`, reviewdog or the benchmark. The decision cites the benchmark as its motivation, which P5 allows |
| `76a1140` names uses outside the harness, and they are real | Yes. (a) The bare `--baseline` gate, the command `AGENTS.md` gives maintainers, and `--against`. (b) `actions/report`: `run.sh` passes `--out`, so the summary goes to stderr (`report.rs`). `run.sh` captures that stderr in `ratchet.txt`, then prints it to the CI log and the job summary. Unlike `dc42499`'s stdout half, this change does reach the action. The annotations come from `--verdict`, which is unchanged, as the decision says. (c) `packages/github-actions` is included by Toolbox for Toolbox and the starter, and the same commit updates `portal/results.json` and `portal/tests`. The decision's `short-functions` example was not re-run here |
| Shaped to the measure? | Tuned on this task, but not shaped to the harness. Each part came from round 3's transcripts of this one task (decision §1), and each has a general reading. Two cautions. First, the SHA hint helps only when the repository already pins the action elsewhere, which the planted task guarantees (README §3). Second, `zizmor-clean` no longer counts `unpinned-uses` in any portal, which changes a gold rule's meaning; `pinned-to-sha` still fails on it. Neither is a harness file, schema or convention |
| `3f1b225` is harness-only | Yes: two files, `harness/quality-gate.sh` and `harness/round.sh` |
| `3f1b225` fixes rival friction recorded in round 3 | Yes. README §0.4 records "A pass says nothing" (two of five round 3 B agents ran zizmor by hand after a pass). The fix prints `quality gate: passed` (21 characters with the newline) only when both tools pass, and it worked: every B pass printed it, and no round 4 B run ran a tool by hand. The `round.sh` change vendors A⁻'s package from `PREV_CHECKOUT`, because the package changed, and stops discarding `setup.sh`'s stderr. It affects only how A⁻ is set up |
| Other changes between rounds 3 and 4 | Nine commits after round 3's results (`94258f7`), up to `d163449`: this adjudication, publication, and review work on other parts of the repository. Then `76a1140`, `3f1b225`, `02f9de5` (the pre-registration), `ac040c8` (prompts), `bb5aa20` (results) and `866caec` (the designer's reading). `git diff 76a1140 HEAD -- crates ui schema packages` and `git diff 02f9de5 HEAD -- experiments/gate-repair-ab/harness` are both empty. See threat 17 for what changed in `dc42499..d163449` |

### Before round 5: `c1f9b22` fits, with one caution; `eb45c5d` and `295b0d3` are harness-only

**`c1f9b22`, the Toolbox change (P5):**

| Check | Finding |
|---|---|
| Touches no harness file | `git show --stat c1f9b22` lists 12 files: `crates/toolbox-cli/src/causes.rs`, `crates/toolbox-connector-workspace/src/workflows.rs`, the decision, two design docs, `packages/github-actions` (fragment, fixture and tests), the `report` skill, and `portal/{fixtures,results.json,tests}`. None is under `experiments/gate-repair-ab/`. The code never names openfga, `pr-summary`, reviewdog or the benchmark. Its doc comment and unit test use `# v6.0.2`, the version the planted fork pins, as the example comment. That is text, not behaviour |
| Names uses outside the harness, and they are real | Yes, verified. (a) This repository's own gate. On an exported copy with one `ci.yml` checkout unpinned, `toolbox report portal --skip runner,github-actions --baseline portal/results.json` quotes `17 \|       - uses: actions/checkout@v6`. Its hint reads `pinned elsewhere as actions/checkout@11d5960a… # v4`, as the decision says. (b) `actions/report/run.sh` passes `--out` and captures stderr in `ratchet.txt`, which it prints to the log and the job summary. So the quote reaches the action. (c) Toolbox for Toolbox includes the package: the same commit updates `portal/results.json`, `portal/fixtures` and `portal/tests` |
| General, not tailored to the planted branch | **Mostly.** The quote applies to any entity with a `path` field and any finding text with `at line N`, read from the working tree, at most 16 lines of 120 characters. `unpinnedAt` and `shaPins` are computed for every workflow. One caution: gaps of up to five lines are filled. On the planted branch the findings name lines 31, 36, 37, 39 and 40. The four-line gap from 31 to 36 holds `needs: lnt`, which no finding names, because actionlint points at the job key on line 31. So the quote shows the typo only because of the gap fill. Five lines is a plausible default, but a designer who had seen this task chose it. The pin hint still helps only where the repository already pins the action (threat 19) |
| Other product changes | None. No product file differs between `76a1140` and `39d1114`, and `git diff c1f9b22 HEAD -- crates packages schema ui Cargo.toml Cargo.lock` is empty. So arm A is exactly A⁻ plus `c1f9b22` |

**`eb45c5d`, the rival change (P4):**

| Check | Finding |
|---|---|
| Harness-only | Yes: one file, `harness/quality-gate.sh` |
| Fixes rival friction recorded in round 4 | **Not as framed.** §0.5's friction list records no reviewdog friction ("reviewdog's pass line worked"). The friction §5.3 cites is true of round 4's transcripts: all five B runs printed `git diff main` and `sed -n` output, four after the gate and `reviewdog-5` just before it in the same command. But arm A did the same (D01M84EPMMR9740K4N25G7A6FAY §1), and the friction was first written down in §5.3 itself. The change is better read as P4's first rule, the same level of help in every arm: Toolbox's gate now quotes code, so the rival's does too. That is a legitimate reason |
| Filtering unchanged | Yes. On the planted branch it reports the same six findings (four from zizmor, two from actionlint), all five B runs passed with it, and all five score correct. actionlint prints no snippet for the expression finding inside a `run:` block, so one finding of six stays unquoted. That is actionlint's own output, passed through |
| Makes the rival stronger | **Not shown.** It quotes the source, as §5.3 says. But in round 5 every B run still printed the files (5 of 5). The failing output grew by 1,003 characters, against arm A's 562: the quoted source, plus zizmor's notes and documentation links. On the registered measures, that counts against B (threat 26). It changes no label, since no comparison with B holds |

**`295b0d3`, the measure change:**

| Check | Finding |
|---|---|
| Harness-only, every arm | Yes: `harness/count.py` and `harness/collect.py`. `collect.py` passes every run's checkout to `count.py`, whatever its arm. Registered in §5.3 before the run, so P6's "never swapped after the run" holds |
| Correct | Yes. On round 4's `toolbox-5`, the imputed `release.yaml` result equals the recorded one character for character (9,562). Round 4's three missing `pull_request.yaml` reads impute to 13,515 each |
| For or against arm A | **Against it on G20, both ways on G23.** Only `read_file` is imputed, and only A and A⁻ used it (`toolbox-1`, `toolbox-10`). It raised A's median read from 9,372 to 10,859 and A⁻'s from 9,027 to 9,238. It also widened A⁻'s range from 2,403 to 13,886, which is G23's *W*. The call it doesn't impute is in arm B (`reviewdog-3`). Leaving that at 0 widens B's ranges, which also works against A on G21. No label depends on it (threat 24) |

**Other changes between rounds 4 and 5.** After round 4's results (`bb5aa20`) come `866caec`
(reading), `1794e9c` (adjudication), `2b1d3a3` (publication) and `39d1114` (a merge of `main`).
Then `c1f9b22`, `295b0d3`, `eb45c5d`, `0249ecb` (pre-registration), `9ca1605` (prompts), `eb2c1af`
(results) and `c8efdea` (reading). `git diff 0249ecb HEAD -- experiments/gate-repair-ab/harness` is
empty. The prompt templates, `setup.sh`, `round.sh` and `harness/toolbox.yaml` are unchanged since
round 4's prompts (`ac040c8`).

## Protocol checks

**Rounds 1–3:**

| Rule | Verdict | Evidence |
|---|---|---|
| P1 committed prompts | **Pass** | All 27 launched prompts (each transcript's first user message) equal `prompts/rendered/r<n>/<run>.md` at that round's prompt commit, byte for byte. Runtime creation times are after the prompt commits: round 1 launched 11:00:06 (prompts in `60723d6`, 10:59:31), round 2 at 11:15:49 (`3762524`, 11:14:48), round 3 at 11:21:28 (`21a2a44`, 11:20:17). Results commits follow (`19d6965`, `967a508`, `94258f7`). Round 3's bar (`d53569d`, 11:19:27) predates its workspaces (built 11:19:35–11:20:09). `prompts/toolbox.md` and `prompts/reviewdog.md` differ only in the gate line. A and A⁻ share one template. Caveat: commit times are set by the author and the branch is unpushed; the runtime's `createdAtMs` corroborates them |
| P2 repeated runs | **Pass** | Rounds 1–2 had 3 runs per arm and round 3 had 5. Listing every cloud agent created since 10:30 gives 31: 27 scored runs, 2 dry runs, 1 telemetry probe and this adjudicator. Nothing was excluded or rerun. The TSVs report every run |
| P3 frozen rounds | **Pass, with notes** | No Rust changed in `703b53a..60723d6` (or `7c8dc45..703b53a`), or in `dc42499..d53569d`. `packages/github-actions` is the same at `703b53a` and `d53569d`. Round 1's workspaces were built at 10:58:00, before `60723d6` was committed (10:59:31), from a tree whose Toolbox content was identical. The round 1 results commit names `7c8dc45`, the README names `60723d6`/`703b53a`, and these agree in substance. A⁻'s binary (built in `/tmp/tb-old` at `703b53a`) hashes differently from round 1's, but its first gate output equals round 1's arm A byte for byte apart from commit hashes. Round 3's arm A output likewise equals round 2's |
| P4 symmetric arms | **Pass** | Neither prompt has a recipe, and the workspaces differ only in the gate (`setup.sh`). No rival friction was fixed or deferred, and the transcripts show none that blocks B. In round 3, two B agents (`reviewdog-3`, `reviewdog-4`) ran zizmor directly after their gate passed, to confirm. That may hint that a silent pass is under-informative; it wasn't triaged, and there is no further round |
| P5 no harness-shaped product | **Pass** | Fit audit above |
| P6 a reported measure | **Pass, with gaps** | *Calls* and *read* were shown for both arms in the dry run. *Gate read* is derived from the same transcript field by `count.py` and was computed for both arms of rounds 1–2 before round 3 was registered. Gaps: stdout only (threat 1), two `read_file` results missing (threat 2), and how gate commands are matched (threat 3). None removes a measure for an arm, so no label is void |
| P7 independent adjudication | **Pass, with a caveat** | See threat 10 |

**Round 4:**

| Rule | Verdict | Evidence |
|---|---|---|
| P1 committed prompts | **Pass** | All 15 launched prompts (each transcript's only user message) equal `prompts/rendered/r4/<run>.md` at `ac040c8` exactly, not just modulo trailing whitespace. Runs were launched between 14:27:00.249 and 14:27:01.104 (runtime `createdAtMs`), after `ac040c8` (14:25:48). A and A⁻ share one template. A and B differ only in the workspace path and the gate line |
| P2 repeated runs | **Pass** | 5 runs per arm. The runtime lists exactly 15 agents named "Repair run r4 …" since 11:30, plus this adjudicator. None was excluded or rerun. `count.py` on each transcript reproduces every calls, read, gate runs and gate read figure in `r4.tsv` |
| P3 frozen round | **Pass, with notes** | Order: `76a1140` (14:21:58), `3f1b225` (14:24:10), pre-registration `02f9de5` (14:24:57), workspaces (14:25:07–14:25:42), prompts `ac040c8` (14:25:48), launches (14:27:00), results `bb5aa20` (14:29:46), reading `866caec` (14:30:29). §5.2 is unchanged since `02f9de5`. Every workspace's `quality-gate.sh` and `.toolbox/toolbox.yaml` equal `02f9de5`'s. A's vendored package equals `02f9de5`'s, and A⁻'s equals `d163449`'s. A's binary is byte-identical to `target/debug/toolbox`. It was built at 14:20:39, before `76a1140` was committed, but no source file is newer than it and the tree is clean. A⁻'s was built at 14:24:59 in a clean worktree of `d163449`. From a fresh copy of the planted branch, A's binary prints 1,541 bytes (1,529 characters) on stdout and nothing on stderr. That equals all five A transcripts' first gate output and the 1,541 in §5.2. A⁻ prints 2,892 bytes, byte-identical to round 3's arm A transcripts. Commit times are set by the author. The runtime's times and file mtimes agree with them |
| P4 symmetric arms | **Pass** | Fit audit above. No rival friction is left deferred |
| P5 no harness-shaped product | **Pass** | Fit audit above |
| P6 a reported measure | **Pass, with a larger gap** | All three measures are reported for every run. Three of the five A runs' `read_file` results are missing, which cuts their read by about 13.5k characters each. One A⁻ result appears cut short (threat 13). As in round 3, a measure is reported for every arm, so no label is void. But G14's and G17's labels rest on incomplete data |
| P7 independent adjudication | **Pass, with a caveat** | See threat 20 |

The catalog entry `portal/benchmarks/gate-repair-ab.yaml` says `status: complete` but still gives
`runsPerArm: 3`, and it has nothing on round 4. It needs updating to match this record. This
adjudication does not edit it.

**Round 5:**

| Rule | Verdict | Evidence |
|---|---|---|
| P1 committed prompts | **Pass** | All 15 launched prompts (each transcript's only user message) equal `prompts/rendered/r5/<run>.md` at `9ca1605` exactly, not just modulo trailing whitespace, and the files are unchanged at HEAD. Runs were launched between 15:10:01.202 and 15:10:01.694 (runtime `createdAtMs`), after `9ca1605` (15:08:29). A and A⁻ share one template. A and B differ only in the workspace path (two lines) and the gate line. The templates are unchanged since round 4, and each rendered prompt equals round 4's apart from the round number |
| P2 repeated runs | **Pass** | 5 runs per arm. Since 14:30 UTC the runtime lists exactly 15 agents named "Repair run r5 …", plus round 4's adjudicator and this one. The three transcript fetches between 15:01 and 15:08 re-read round 4's runs only. None was excluded or rerun. `count.py` on each transcript reproduces every calls, read, `readRecorded`, `missing`, gate runs and gate read figure in `r5.tsv` |
| P3 frozen round | **Pass** | Order: `c1f9b22` (15:04:07), `295b0d3` and `eb45c5d` (15:05:43), pre-registration `0249ecb` (15:06:32), A⁻'s binary (15:07:25, in a clean worktree of `76a1140`), A's binary (15:07:34), workspaces (15:07:39–15:08:14), prompts `9ca1605` (15:08:29), launches (15:10:01), results `eb2c1af` (15:11:54), reading `c8efdea` (15:13:19). §5.3 is unchanged since `0249ecb`. Every B workspace's `quality-gate.sh` and every Toolbox workspace's `.toolbox/toolbox.yaml` equal `0249ecb`'s harness. A's vendored package equals `packages/github-actions` at `0249ecb` file by file, and A⁻'s equals `76a1140`'s. A's binary is byte-identical to `target/debug/toolbox`. Unlike rounds 1 and 4, it was built after its commit, and no source file is newer. From fresh copies of the planted branch: A prints 2,091 characters (2,103 bytes) on stdout, nothing on stderr, quoting 11 lines. A⁻ prints 1,529, and B prints 2,225 on stdout and 192 on stderr. All three match §5.3. Every recorded first gate result contains that output exactly (14 of 15; `reviewdog-3`'s is missing). A⁻'s output also appears verbatim in all five of round 4's arm A transcripts. Commit times are set by the author, but the runtime's times and file mtimes agree with them |
| P4 symmetric arms | **Pass, with a note** | `eb45c5d` is harness-only and gives the rival the same kind of help. It is not a fix of friction §0.5 recorded, and it is not shown to make the rival stronger (fit audit). No rival friction from round 4 is left deferred |
| P5 no harness-shaped product | **Pass, with a caution** | Fit audit above: the gap fill |
| P6 a reported measure | **Pass, with gaps** | All three measures are reported for every run. Three calls have no recorded result: two `read_file` calls, imputed (`toolbox-1` in A, `toolbox-10` in A⁻), and one terminal command, not imputed (`reviewdog-3` in B). The measure change was registered before the run. No label rests on the missing data (threat 24) |
| P7 independent adjudication | **Pass, with a caveat** | See threat 30 |

The catalog entry `portal/benchmarks/gate-repair-ab.yaml` has nothing on round 5: `frozenAt`
lists rounds 1–4, and the measure text names "rounds 3 and 4". This adjudication does not edit it.

## The designer's reading of round 4 (README §0.5), checked

The table's figures and the labels are correct. Four explanations are wrong or unsupported, and
there are smaller errors elsewhere in the record. None changes a label.

| Claim | Finding |
|---|---|
| The per-arm table and the six readings | **Correct.** Every figure matches `r4.tsv` and this recomputation |
| "The gap comes from how the agents edited, not from what the gate told them … it decides G13" | **Wrong in part.** Edits and gate runs account for 2 of the 4-call gap, and exploration for the other 2 (threat 12). Either half alone fails G13. In `toolbox-3` and `toolbox-4`, the separate lookup was for the version comment, which the gate's SHA hint leaves out. Their notes say "I'll search for that comment to match it" and "Next I'm looking for the version comment". Those calls follow from what the gate's output left out |
| "arm A's read fell to reviewdog's level" | **Not supported.** It holds for the recorded figures only. Three A runs' reads of `pull_request.yaml` have no recorded result. Corrected, A's median is about 23.1k, over twice B's (threat 13) |
| Arm A: every run made two search-and-replace edits; "Four of five first read a file, which that tool requires" | **The count is correct, the reason is wrong.** The tool doesn't require a read. `toolbox-2` read no file, and `toolbox-5` read only `release.yaml` before editing `pull_request.yaml` too. A⁻'s `toolbox-6` and `toolbox-10` also edited without a read. All of those edits succeeded |
| Arm B: both edits in one `python3` heredoc, with the gate rerun in the same command | **Correct** for all five runs |
| Arm A⁻: two runs (`toolbox-7`, `toolbox-8`) used a heredoc, three used search-and-replace, "with the same gate output as round 3's arm A" | **Correct.** The output is byte-identical |
| "So this is agent style, not a property of either gate" | **Not shown.** Heredoc editing was 0 of 5 in A, 2 of 5 in A⁻ and 5 of 5 in B. The arms differ in more than the gate's output; for example, every B agent printed the gate's shell script before its first gate run. The split doesn't say what caused it |
| Every A run looked up `upload-artifact` after the gate named its SHA; three in a separate call, a fourth with the grep tool | **Correct.** `toolbox-2`, `-3` and `-4` used a separate call. The result of `toolbox-1`'s grep matched the `actions/upload-artifact` lines in `release.yaml` and `scorecard.yml` |
| Every diff copies the version comments (`# v6.0.2`, `# v7.0.1`), and the hint doesn't print them | **Correct.** All 15 diffs are identical and carry both comments. The first gate output contains no `# v` |
| reviewdog-5's 5,099 characters came from `git diff main` in its first gate command and an edit in its second | **Correct.** 3,236 + 1,863 |
| No B run checked a pass by running a tool by hand | **Correct.** No round 4 command ran zizmor or actionlint outside a gate |

Smaller errors in the record:

- §5.2 and D01M84CCDH1ZWF9726SVDDM1JDX give the first gate output as 2,892 and 1,541 "characters".
  Those are UTF-8 bytes. The measure counts characters, and the outputs are 2,860 and 1,529.
- D01M84CCDH1ZWF9726SVDDM1JDX's `decided: 2026-11-23T15:20:00Z` is later than its commit (14:21:58)
  and than the round.
- §5.2 calls `d163449` "round 3's arm A plus unrelated commits". Those commits change 41 product
  files, including `packages/github-actions`. On this task the gate's output is unchanged (threat
  17).
- §0.1 says "Arm B's gate prints everything on stdout". On a failure, reviewdog also prints 96
  characters per tool to stderr, 192 on the planted branch, and no round counts them.
- The README's status list says G13 fails "because of how the agents edited". That is half the
  cause, as the second row of the table above says.

## The designer's reading of round 5 (README §0.6), checked

The table's figures and the six labels are correct. Two factual claims are wrong, and one is
supported for only one run of three. None changes a label.

| Claim | Finding |
|---|---|
| The per-arm table and the six readings | **Correct.** Every figure matches `r5.tsv`, `count.py`'s recount of all 15 transcripts, and this recomputation |
| "The designer predicted that G20, G22 and G23 would hold, and none did" | **Correct.** The other three predictions were right |
| "Two A runs (`toolbox-1`, `toolbox-10`) read `pull_request.yaml` with no recorded result" | **Wrong.** `toolbox-10` is arm A⁻ (`arms.tsv`). One A run and one A⁻ run have the missing read. The 13,515 imputed for each is correct |
| `reviewdog-3` chained `git diff main` onto its first gate run, and its result is missing; about 4,300 characters; corrected ranges about 4,900 and 4,300; no label changes | **Correct.** Rebuilt as a recorded result is (`git diff main` is 1,645 characters, then the gate's 2,225), it is 4,272. The corrected ranges are 4,925 and 4,342 |
| "Every run printed the branch's files or diff (15 of 15)" before editing, "from about 3,500 to more than 15,000 characters a run" | **Correct.** Every run printed them with `git show`, `git diff`, `sed -n`, `cat` or `read_file`, including all five A runs after a gate that had quoted the lines. The range is right, but those commands also carry chained lookups, so not all of it is reading the branch |
| The gate leaves out the fix for `needs: lnt`; actionlint doesn't name the jobs that exist | **Correct.** Neither gate names `lint`. Arm A's quote shows lines 31–40, and `needs: lnt` only through the gap fill. Four runs grepped for the job names (`toolbox-6`, `toolbox-10`, `reviewdog-2`, `reviewdog-3`). The rest read them in the files they printed |
| Three A runs made both edits in one `python3` heredoc, with the gate rerun in the same command, in 3 calls each; the two that used search-and-replace took 6 | **Correct.** `toolbox-2`, `-3` and `-5`; `toolbox-1` and `-4` |
| "Editing style moved" (round 4: no A heredoc runs) | **Correct as a count, but not caused by the change.** The same Toolbox went from 0 heredoc runs of 5 in round 4 (as arm A) to 3 of 5 in round 5 (as A⁻). The move happened in both Toolbox arms (threat 22) |
| All five B runs used a heredoc | **Correct** |
| "four made a separate SHA lookup (4 calls)" | **Wrong in one run.** `reviewdog-2`, `-4` and `-5` looked up the `upload-artifact` pin in a later call of its own. `reviewdog-3` looked up both pins in the same call as its file print, as `reviewdog-1` did. Its extra call is its first: it printed the gate script without running the gate |
| "No A heredoc run made a separate lookup: they copied the pin from the gate's hint" | **The first half is correct; the second is supported for one run of three.** No A run made a separate SHA lookup. But `toolbox-3` and `toolbox-5` grepped for `actions/checkout` and `upload-artifact` in the same call as their file print. Only `toolbox-2` ran no lookup, so only there is the hint clearly the source of the pin. A⁻'s heredoc runs, whose hint lacks the version comment, took 3, 3 and 4 calls. `toolbox-7` and `-8` chained the lookup into their print, as A's did |
| The status list: Toolbox "no longer loses" to reviewdog | **Correct as a summary of the labels, but not due to the change.** Round 4's Toolbox (A⁻) also ties reviewdog on every measure in round 5 |
| §5.3: "This makes the rival stronger" | **Not shown** (fit audit) |
| §5.3: the first gate output rose from 1,529 characters to 2,091 and quotes 11 lines; reviewdog's from 1,222 to 2,225 on stdout, with 192 on stderr | **Correct, and reproduced.** These are characters now (2,091 is 2,103 bytes) |
| §5.3: the measure change reproduces round 4's `toolbox-5` (9,562) and the 13,515 | **Correct** |

Smaller errors in the record:

- D01M84EPMMR9740K4N25G7A6FAY §1 says "four of five Toolbox runs read `pull_request.yaml` before
  editing it" in round 4. Three did (`toolbox-1`, `-3` and `-4`). `toolbox-5` read only
  `release.yaml`.
- That decision's `decided: 2026-11-23T15:10:00Z` is later than its commit (15:04:07). It is the
  minute the round was launched.

## Threats to validity

**Rounds 1–3.**

1. **Stdout only, verified.** Each transcript keeps `output`, the terminal's stdout. Round 1's arm A
   first gate output (15,677 characters) and A⁻'s contain neither "worse" nor "grew", so the
   verdict on stderr was not counted. The designer gives 3,711 characters for that stderr; it was
   not re-measured here, since doing so would mean rerunning a gate. All five A⁻ runs and all three
   round 1 A runs also piped their second gate run through `grep`/`rg`. Adding 3,711 to each
   undercounted run changes no label:
   - G3's fail gets larger;
   - G8 gets larger;
   - G10 becomes about 7.8k ≤ 22.8k;
   - G5 becomes 20,229 ≤ 20,880. That is close, but G5 is inconclusive by its noise control anyway.
2. **Two `read_file` results are missing from the transcript (not disclosed in the README).** In
   round 2 `toolbox-3` and round 3 `toolbox-5`, both arm A, the call to read `pull_request.yaml` has
   no `tool_result`, so *read* counts 0 for it. The whole file is 12,932 characters. Adding about
   13.5k to both runs changes no label:
   - G3′ becomes B − A = 895;
   - G5 becomes 5,246 ≤ 18,652;
   - G10 becomes 4,093 ≤ 19,068;
   - G3″ becomes A − B = 6,638 ≤ 19,068.

   The bias favours arm A.
3. **Gate read counts whole commands.** `count.py` treats any terminal command that contains the
   gate as a gate run, and counts all its output: `git status`/`git log`, `git diff main`, `cat
   scripts/quality-gate.sh`, `ls ../bin`, and Python edits chained before the gate. What the gates
   alone print when failing on the planted branch:
   - reviewdog: 1,222 characters;
   - the changed Toolbox: 2,867;
   - the previous Toolbox: about 15.7k on stdout plus stderr.

   G8 is robust to this: the first gate run alone differs by about 12.8k, against *W* = 3,189. G11
   is not. With the gate's own output only, reviewdog's would very likely be the smaller, so G11's
   inconclusive label is generous to Toolbox. The label stands because it follows the measure as
   registered.
4. **Read includes the runtime's wrapper.** `count.py` measures `str(tool_result)`, which adds
   about 520 characters of JSON per terminal call (16,200 counted against 15,677 of output). The
   overhead scales with calls, and it is the same per call in every arm.
5. **The round 3 measure was chosen after rounds 1 and 2 were seen (disclosed in §5.1).** G8 is
   close to a manipulation check: the change shrank the gate's output, and G8 measures the gate's
   output. It was registered before round 3, and the comparison between both Toolboxes in one round
   removes drift between rounds. Both are real strengths. But round 3's guards (G9, G10) and the
   comparisons with the rival (G11, G2″, G3″) all stayed inconclusive.
6. **The agents are close to deterministic, and correctness is at its ceiling.** Round 2
   `toolbox-3` and round 3 `toolbox-5` are separate sessions in separate workspaces, yet they made
   the same six tool calls with the same result sizes (read 11,021, gate read 3,876). All 27 runs
   produced the same diff. The spread in *read* comes mostly from whether an agent ran one broad
   `cat`/`grep` over the workflows (14–22k characters), which is a choice unrelated to the gate. So
   with 3–5 runs, *read* has a few modes rather than a distribution. The correctness hypotheses
   can't separate arms on this task.
7. **Asymmetry between arms is inherent, not added.** A's gate scores the whole portal, so it
   reports eight regressions, including Toolbox's own pin rules, where B reports six findings. B's
   gate is a short script that every B agent read with `cat`, and that counts toward B's read. The
   prompts differ only in the gate command. The planted job sets `timeout-minutes`, so A's extra
   timeout rule adds no work.
8. **External validity.** One fork, one planted pull request, one model. The harness records arm
   A's baseline with the binary under test, so the changed arm always has evidence keys. A real
   repository's existing baseline has no keys and shows the whole list until someone runs
   `--update`.
9. **Small errors in the record, none of which changes a label.**
   - README §0.1 says each B run read "five new findings". The reviewdog gate printed six (four
     from zizmor, two from actionlint) in every round 1 run.
   - D01M841J2SN4V3AVZZ5D91YAR51 says "Two piped later runs through `grep`". In round 1 all three
     A runs did, as README §0.1 says.
   - README §0.3 gives the broad reads as 11,714–20,746. Counted with the wrapper, they are
     11,723–20,784.
10. **Limits of this adjudication's independence.** The adjudicator was launched from the
    designer's session with a brief the designer wrote, including the list of key commits, and it
    runs on the same model family as the arms. It checked the brief's commits against `git log` and
    the runtime's agent list rather than taking them as given.

**Round 4.** None of these changes a label.

11. **The designer saw rounds 1–3 and tuned the change on this task (disclosed in §5.2).** Each
    part of `76a1140` answers something round 3's agents did on this planted pull request. So G16
    is a manipulation check, as G8 was: the change shortened the gate's output, and G16 measures
    that output. The rival comparisons G13–G15 are the round's independent test, and none of them
    holds.
12. **Calls depend on how agents edit and explore, and G13 fails on either alone.** Each call is
    counted in one of two groups:

    | Arm | Calls on gate runs and edits (runs) | Median | Exploration calls (runs) | Median |
    |---|---|---|---|---|
    | A | 4, 4, 4, 4, 4 | 4 | 2, 2, 3, 3, 3 | 3 |
    | A⁻ | 2, 2, 4, 4, 4 | 4 | 1, 2, 2, 2, 2 | 2 |
    | B | 2, 2, 2, 2, 2 | 2 | 1, 1, 1, 2, 2 | 1 |

    A edits with two search-and-replace calls and reruns its gate separately. B edits and reruns its
    gate in one command. A's extra exploration is a `read_file` before editing (four runs) and a
    separate version-comment lookup (three runs). Two counterfactuals:
    - If arm A edited as B did, its calls would be 4, 4, 5, 5, 5 (median 5, range 1), against B's
      median of 3. That still fails G13: 2 > 1.
    - If arm A explored as little as B, its median would be 5, which also fails.

    So the label doesn't hinge on editing style.
13. **Read is undercounted for arm A, more than in round 3.** In `toolbox-1`, `-3` and `-4`, the
    `read_file` of `pull_request.yaml` has no `tool_result`, so `count.py` counts 0 for it. Using
    `toolbox-5`'s recorded `read_file` as a template, a full read of the file on `pr` would be
    13,515 characters. In A⁻'s `toolbox-9`, the same read records only the first 49 of 347 lines,
    1,758 characters, even though its arguments ask for the whole file. Whether that agent saw more
    is unknown. Sensitivity:
    - Correcting A, A becomes 10,692; 20,464; 23,057; 23,276; 23,812 (median 23,057, range
      13,120). G14: A − B = 12,683 ≤ *W* = 13,120, still inconclusive, with the median now favouring
      B. The correction can't make G14 fail: it raises A's median and range together.
    - Correcting both A and A⁻: G17 is 1,276 ≤ 13,120, inconclusive.
    - Correcting A⁻ alone: G17 would hold (14,036 > 10,922).

    The labels follow the recorded measure, under the same rule round 3's threat 2 applied. Neither
    G14 nor G17 is evidence either way. The bias favours arm A on G14 and against it on G17.
14. **Gate read counts whole commands, and on the gates' own output reviewdog's is shorter.** Four
    of five B runs printed `scripts/quality-gate.sh` (930 characters) in the same command as their
    first gate run. reviewdog-5 chained `git diff main` and an edit. What the gates alone print in
    every run:
    - arm A: 1,529 failing plus 103 passing, 1,632 in all;
    - arm B: 1,222 plus 21, 1,243 in all, plus 192 characters on stderr that no transcript keeps.

    Measured on the gates' output alone, G15 would fail. It is inconclusive because the registered
    measure counts what B's agents chained. The label stands as registered, and it is generous to
    Toolbox, as G11's was in round 3. The pre-registration disclosed this (§5.2).
15. **The P4 fix made the rival stronger.** It added 21 characters to each B run's gate read, and no
    B agent ran a tool by hand to confirm a pass. Round 3's B median was 4 calls, and round 4's is 3.
    That may be the fix or chance. Either way, it widened G13's gap. That is what P4 asks for.
16. **Repeated rounds on one task, with near-deterministic agents.** This is the fourth round on one
    fork, one planted pull request and one model. All 42 diffs are byte-identical. Arm A's gate read
    is identical in four of five runs, and two B runs have the same read, 10,340. Ranges as small
    as 1 call make *W* tiny, so habits that cluster by arm decide labels. Threat 12 shows that G13
    survives the habit that clusters most.
17. **A⁻ is not literally round 3's arm A.** Between `dc42499` and `d163449`, 41 product files
    changed, including 197 lines of `packages/github-actions/fragments/github-actions.yaml`. On the
    planted branch, A⁻'s first gate output is byte-identical to round 3's arm A, and its gate read
    repeats round 3's 3,876 and 3,952. So on this task it is the same arm.
18. **Arm A's binary was built before its commit.** It is dated 14:20:39, and `76a1140` was
    committed at 14:21:58. No source file is newer than the binary, the tree is clean, and its
    output matches the decision's description. Round 1 had the same pattern (P3 above).
19. **External validity.** The SHA hint names a SHA only when the repository already pins the
    action elsewhere. In this task that is guaranteed by design, and in a real pull request it may
    not be. Threat 8 also still applies: a real repository's existing baseline has no evidence keys.
20. **Limits of this adjudication's independence.** The designer's session launched this adjudicator,
    with a brief that lists the commits and the claims to check, and it runs on the same model family
    as the arms. It extended the round 1–3 adjudication, which may anchor its style and framing. It
    checked the brief against `git log` and the runtime's agent list, rescored and recounted every
    run, and reproduced the gate outputs rather than taking §0.5's figures as given.

**Round 5.** None of these changes a label.

21. **The fifth round on one task, and the second try at the same win (disclosed in §5.3).** The
    designer saw every round 4 transcript before changing the product and the rival's gate, and
    before writing §5.3. Two tries at "beats reviewdog" raise the chance of a win by chance. No
    comparison with the rival held, so nothing here needs discounting. A later win on this task
    should be read against all five rounds.
22. **The same Toolbox moved by 3 calls between rounds.** Round 4's arm A and round 5's A⁻ run the
    same product. No product file differs between `76a1140` and `39d1114`, the first gate output is
    byte-identical, and the prompts and set-up are equal apart from the round number. Yet its calls
    went from 6, 6, 7, 7, 7 (median 7, range 1) to 3, 3, 4, 6, 7 (median 4, range 4). Its heredoc
    editing went from 0 runs of 5 to 3. That drift is three times round 4's *W*. It doesn't relabel
    round 4, because the rule compares arms within a round. But five near-deterministic runs
    understate how far a round can move. And round 5's tie is not the change's doing: A⁻ also ties
    reviewdog on every measure. B's gate changed between rounds, so B's own move (median 3 → 4) can't
    be read the same way.
23. **Calls still turn on how the agent edits.** Each call is counted in one of two groups:

    | Arm | Calls on gate runs and edits (runs) | Median | Exploration calls (runs) | Median |
    |---|---|---|---|---|
    | A | 4, 2, 2, 4, 2 | 2 | 2, 1, 1, 2, 1 | 1 |
    | A⁻ | 4, 2, 2, 2, 4 | 2 | 2, 1, 1, 2, 3 | 2 |
    | B | 2, 2, 2, 2, 2 | 2 | 1, 2, 2, 2, 2 | 2 |

    Heredoc runs took 3, 3 and 3 calls in A, 3, 3 and 4 in A⁻, and 3, 4, 4, 4 and 4 in B.
    Search-and-replace runs took 6 and 6 in A, and 6 and 7 in A⁻. Arm A's lower median comes from
    exploring one call less than B. If arm A had edited as B did, its calls would be 4, 3, 3, 4, 3
    (median 3, range 1). B − A = 1 ≤ *W* = 1, so G19 would still be inconclusive. On this task, G19
    can hold only if arm A also explores less than B in nearly every run.
24. **Missing results, and an imputation that covers only `read_file`.** Three calls have no
    recorded result:
    - the `read_file` of `pull_request.yaml` in `toolbox-1` (A) and `toolbox-10` (A⁻), each imputed
      as 13,515;
    - `reviewdog-3`'s first gate command, counted as 0. Rebuilt as a recorded result is, it would be
      4,272 characters.

    The robustness table under *Does Toolbox beat reviewdog?* shows that no label moves with
    `readRecorded`, with `reviewdog-3` corrected, or with both. The imputation also assumes the agent
    received the whole file. Round 4's A⁻ `toolbox-9` shows that a read can come back cut short
    (threat 13).
25. **Gate read counts whole commands, and on the gates' own output the arms are 52 characters
    apart.**
    - Arm A's gate prints 2,091 characters failing and 103 passing, 2,194 in all.
    - reviewdog's prints 2,225 and 21, 2,246 in all, plus 192 characters on stderr that no
      transcript keeps.

    Round 4's order (1,632 against 1,243) reversed because B's failing output grew by 1,003
    characters with `eb45c5d`, and A's by 562 with `c1f9b22`. G21's gap of 2,816 is mostly what B's
    agents chained onto their gate runs:
    - three printed `scripts/quality-gate.sh` in their first gate command;
    - two printed `git diff` with their rerun;
    - `reviewdog-4` chained a grep and a 30-line `sed` (8,740 in all).

    The label is inconclusive either way. It would flatter A if it held.
26. **The rival change is not neutral on the measure.** `eb45c5d` gave B the kind of help
    `c1f9b22` gave A, which P4 asks for. But in round 5 no B run used the quote in place of printing
    the files (5 of 5 printed them). So on this task the change raised B's read and gate read without
    removing any work that can be seen. That favours arm A on G20 and G21, neither of which holds. A
    more compact rival configuration was not tried.
27. **The attribution is not shown.** G22 and G23 are inconclusive, and arm A's median read is
    higher than A⁻'s. This round does not show that `c1f9b22` cut any cost. It shows only that the
    gate's output now carries the named lines and the pins as written, at 562 characters more.
28. **Near-determinism and a ceiling on correctness.** All 30 diffs of rounds 4 and 5 are
    byte-identical, and equal to those of rounds 1–3. Three A runs have a gate read of exactly 3,102,
    and two B runs 5,918. With so little spread inside an arm, one run's habit sets *W*, as in round
    4 (threat 16). G21's *W* comes from `reviewdog-3`'s missing result and `reviewdog-4`'s chained
    print. G20's comes from the two A runs that read the whole of `pull_request.yaml`: `toolbox-1`
    (imputed) and `toolbox-2` (with `cat`).
29. **Tuned on this task; external validity.** Each part of `c1f9b22` answers something round 4's
    agents did on this one planted pull request. Its gap fill is just wide enough to quote the
    planted typo (fit audit). The decision's example on this repository reproduces, so the change
    has a use outside the harness, but there it quotes a single line. One fork, one pull request
    and one model; threats 8 and 19 still apply.
30. **Limits of this adjudication's independence.** The designer's session launched this
    adjudicator, with a brief that lists the commits and paraphrases the claims to check. It runs on
    the same model family as the arms, and it extended the rounds 1–4 adjudication, which may anchor
    its framing. It checked the brief against `git log` and the runtime's agent list. It rescored
    copies of all 15 workspaces, recounted all 15 transcripts, and reproduced all three gates'
    output and the decision's example. It read every round 5 transcript's tool calls and notes rather
    than taking §0.6 as given.

## Sources

- [README.md](./README.md) §0, §5, §5.1, §5.2, §5.3;
  [D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md) (P1–P7);
  [D01M841J2SN4V3AVZZ5D91YAR51](../../docs/decisions/D01M841J2SN4V3AVZZ5D91YAR51.md);
  [D01M84CCDH1ZWF9726SVDDM1JDX](../../docs/decisions/D01M84CCDH1ZWF9726SVDDM1JDX.md);
  [D01M84EPMMR9740K4N25G7A6FAY](../../docs/decisions/D01M84EPMMR9740K4N25G7A6FAY.md).
- `results/r{1,2,3,4,5}.tsv` and `results/r{1,2,3,4,5}/*.json|*.diff`; `harness/count.py`,
  `score.py`, `collect.py`, `setup.sh`, `round.sh`, `quality-gate.sh`;
  `prompts/rendered/r{3,4,5}/arms.tsv`.
- Runtime transcripts of all 57 scored runs (`batch-fetch-details`, by the bcIds in the TSVs), and
  the runtime's lists of cloud agents created since 2026-11-23 10:30 UTC (rounds 1–3), 11:30 UTC
  (round 4) and 14:30 UTC (round 5).
- Round 4's workspaces under `/tmp/gate-repair-ab/r4/`: each rescored with `score.py`, their diffs,
  binaries and vendored packages compared, and the planted branch re-gated in a copy with A's, A⁻'s
  and B's gates.
- Round 5's workspaces under `/tmp/gate-repair-ab/r5/`, read only: copies of all 15 rescored with
  `score.py`, their diffs, binaries, gate scripts, portals and vendored packages compared with the
  frozen commits, and the planted branch re-gated in copies with A's, A⁻'s and B's gates. Round 4's
  transcripts, re-read for §5.3's account of round 4 and for A⁻'s gate output.
- An exported copy of this repository at `c8efdea`, with one `ci.yml` checkout unpinned, gated with
  `toolbox report portal` (the decision's example).
- `git log`/`git show` of `60723d6`, `19d6965`, `dc42499`, `3762524`, `967a508`, `d30dcb2`,
  `d53569d`, `21a2a44`, `94258f7`, `144caf2` (box-ab friction table), `703b53a` (root-causes C1),
  `d163449`, `76a1140`, `3f1b225`, `02f9de5`, `ac040c8`, `bb5aa20`, `866caec`, `39d1114`,
  `c1f9b22`, `295b0d3`, `eb45c5d`, `0249ecb`, `9ca1605`, `eb2c1af` and `c8efdea`;
  `actions/report/run.sh` and `crates/toolbox-cli/src/report.rs` (where the summary goes).
