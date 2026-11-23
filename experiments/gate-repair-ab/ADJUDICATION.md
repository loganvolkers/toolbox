# gate-repair-ab: independent adjudication (2026-11-23)

**Outcome against the bar in [README.md](./README.md) §5, §5.1 and §5.2:**

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

**The Toolbox changes between rounds (`dc42499` before round 2, `76a1140` before round 4) fit
(P5). The rival fix before round 4 (`3f1b225`) is harness-only (P4).** No run was excluded or
rerun, every launched prompt matches its committed file byte for byte, and no label is void.

The adjudicator is a cloud agent that did not design the experiment, build an arm, or change the
product ([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P7). It
recomputed every median, range and *W* from [results/r1.tsv](results/r1.tsv),
[results/r2.tsv](results/r2.tsv), [results/r3.tsv](results/r3.tsv) and
[results/r4.tsv](results/r4.tsv). It also read all 42 runtime transcripts: 27 for rounds 1–3 and
15 for round 4. For round 4 it also rescored all 15 workspaces, recounted all 15 transcripts with
`count.py`, and reproduced both Toolboxes' first gate output from a fresh copy of the planted
branch. Threats 10 and 20 state the limits of its independence.

## Per hypothesis

Medians are over an arm's runs in a round. The range is maximum minus minimum, and *W* is the wider
of the two ranges compared. A is the changed Toolbox, A⁻ (rounds 3 and 4) the previous one, and B
is reviewdog. Subscripts in G4–G5 are round numbers.

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

Every run in every arm and round was scored correct, and all 42 diffs are byte-identical.
Correctness therefore can't separate the arms (threat 6). The designer's predictions were right in
rounds 1 and 3. In round 1 the prediction was G1 holds, G2 inconclusive or fails, and G3 fails. In
round 3 it was G7 and G8 hold, and G9, G10 and G11 are inconclusive. In round 4 one of six was
wrong: G13 was predicted inconclusive and fails. G12 and G16 hold, G14 and G17 are inconclusive,
and G15 is inconclusive, one of the two outcomes predicted for it.

Not registered, and reported only: in round 4 A⁻ against B is inconclusive on every measure. On
calls the gap is 6 − 3 = 3 ≤ *W* = 3, on read 13,824 ≤ 14,656 and on gate read 615 ≤ 1,913.

## Is the improvement shown? Does Toolbox beat reviewdog?

| Rule | Requires | Result |
|---|---|---|
| §5 (round 2) | G6 holds, at least one of G4 and G5 holds, neither fails | **Not shown.** G6 holds. G4 and G5 are both inconclusive |
| §5.1 (round 3) | G7 holds, G8 holds, neither G9 nor G10 fails | **Shown.** G7 and G8 hold. G9 and G10 are inconclusive |
| §5.2 (round 4): Toolbox beats reviewdog | G12 holds, at least one of G13, G14 and G15 holds, none of them fails | **Not shown.** G12 holds. G13 fails, and G14 and G15 are inconclusive. The rule fails on both of its last two conditions |

The §5.1 verdict follows the rule as written, and the rule was committed before round 3's
workspaces were built. Its scope is that the gate is cheaper to read. The total cost of a repair is
unproven, and Toolbox is not shown to beat reviewdog on any measure.

The §5.2 verdict is robust. Correcting the undercounted read can't make G14 or G15 hold (threats 13
and 14). G13's failure survives removing either cause of the gap (threat 12). §5.2 says the round
is not rerun.

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

## Sources

- [README.md](./README.md) §0, §5, §5.1, §5.2;
  [D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md) (P1–P7);
  [D01M841J2SN4V3AVZZ5D91YAR51](../../docs/decisions/D01M841J2SN4V3AVZZ5D91YAR51.md);
  [D01M84CCDH1ZWF9726SVDDM1JDX](../../docs/decisions/D01M84CCDH1ZWF9726SVDDM1JDX.md).
- `results/r{1,2,3,4}.tsv` and `results/r{1,2,3,4}/*.json|*.diff`; `harness/count.py`, `score.py`,
  `collect.py`, `setup.sh`, `round.sh`, `quality-gate.sh`; `prompts/rendered/r{3,4}/arms.tsv`.
- Runtime transcripts of all 42 scored runs (`batch-fetch-details`, by the bcIds in the TSVs), and
  the runtime's lists of cloud agents created since 2026-11-23 10:30 UTC (rounds 1–3) and 11:30 UTC
  (round 4).
- Round 4's workspaces under `/tmp/gate-repair-ab/r4/`: each rescored with `score.py`, their diffs,
  binaries and vendored packages compared, and the planted branch re-gated in a copy with A's, A⁻'s
  and B's gates.
- `git log`/`git show` of `60723d6`, `19d6965`, `dc42499`, `3762524`, `967a508`, `d30dcb2`,
  `d53569d`, `21a2a44`, `94258f7`, `144caf2` (box-ab friction table), `703b53a` (root-causes C1),
  `d163449`, `76a1140`, `3f1b225`, `02f9de5`, `ac040c8`, `bb5aa20` and `866caec`;
  `actions/report/run.sh` and `crates/toolbox-cli/src/report.rs` (where the summary goes).
