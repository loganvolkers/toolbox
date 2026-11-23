# gate-repair-ab: independent adjudication (2026-11-23)

**Outcome against the bar in [README.md](./README.md) §5 and §5.1:**

- **Round 1: Toolbox lost to reviewdog.** G1 holds, but G2 (calls) and G3 (read) **fail**.
- **Round 2: the improvement is not shown under §5.** G1′ and G6 hold, and every other hypothesis
  is inconclusive. G5 is inconclusive twice over: its gap is inside the noise, and arm B's read
  moved more than its range allows. The designer's reading in §0.3 is correct.
- **Round 3: the improvement is shown under §5.1.** G7 and G8 hold, and G9 and G10 are
  inconclusive, not failed. G11, G2″ and G3″ are inconclusive.

What round 3 shows is narrow. The change cut what an agent reads from the gate, from a median of
18,751 characters to 3,876. That gap is more than four times the wider range. It does not show
that a repair costs fewer calls or less reading in all, or that Toolbox's gate is cheaper than
reviewdog's: every such hypothesis in rounds 2 and 3 is inconclusive, and round 1's are failures.
The round 3 measure was chosen after rounds 1 and 2 were seen, and it measures the product change
almost directly. Read G8 as confirming that the change does what it says, not as an independent
win.

**The Toolbox change between rounds (`dc42499`) fits (P5).** No run was excluded or rerun, every
launched prompt matches its committed file byte for byte, and no label is void.

The adjudicator is a cloud agent that did not design the experiment, build an arm, or change the
product ([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P7). It
recomputed every median, range and *W* from [results/r1.tsv](results/r1.tsv),
[results/r2.tsv](results/r2.tsv) and [results/r3.tsv](results/r3.tsv). It also read all 27 runtime
transcripts. Threat 10 states the limits of its independence.

## Per hypothesis

Medians are over an arm's runs in a round. The range is maximum minus minimum, and *W* is the wider
of the two ranges compared. A is the changed Toolbox, A⁻ (round 3) the previous one, and B is
reviewdog. Subscripts in G4–G5 are round numbers.

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

Every run in every arm and round was scored correct, and all 27 diffs are byte-identical.
Correctness therefore can't separate the arms (threat 6). The designer's predictions were right in
both rounds where they were made. In round 1 the prediction was G1 holds, G2 inconclusive or fails,
and G3 fails. In round 3 it was G7 and G8 hold, and G9, G10 and G11 are inconclusive.

## Is the improvement shown?

| Rule | Requires | Result |
|---|---|---|
| §5 (round 2) | G6 holds, at least one of G4 and G5 holds, neither fails | **Not shown.** G6 holds. G4 and G5 are both inconclusive |
| §5.1 (round 3) | G7 holds, G8 holds, neither G9 nor G10 fails | **Shown.** G7 and G8 hold. G9 and G10 are inconclusive |

The §5.1 verdict follows the rule as written, and the rule was committed before round 3's
workspaces were built. Its scope is that the gate is cheaper to read. The total cost of a repair is
unproven, and Toolbox is not shown to beat reviewdog on any measure.

## Fit audit (P5)

**`dc42499` fits.**

| Check | Finding |
|---|---|
| Touches no harness file | `git show --stat dc42499` lists `crates/toolbox-cli/src/{causes,report}.rs`, `crates/toolbox-cli/tests/ratchet.rs`, the new decision, `docs/design/{ratchets,results-spec,root-causes}.md`, the `report` skill, `portal/results.json` and one site page. None is under `experiments/gate-repair-ab/` (0 of 10 files). No harness schema or convention is referenced in the code |
| box-ab friction #4 existed before 60723d6 | Yes. `experiments/box-ab/README.md` at `703b53a`, line 87, reads "The gate's regression lists every finding in the file and doesn't mark the new ones; the baseline keeps only counts (B6, A6) · **Open:** needs a fingerprint per finding". It was added in `144caf2` (2026-11-22 17:48), before 60723d6 (2026-11-23 10:59) |
| root-causes C1 existed before 60723d6 | Yes. At `703b53a`, `docs/design/root-causes.md` line 75 reads "C1 · Evidence identity: say which evidence items are new … by recording items (or their fingerprints) in the baseline". The file's history goes back to `0b9b225` and `567ed12` (2026-11-21). The results spec's "fingerprints per finding" row also predates it |
| Uses outside the harness are real | Mostly. (a) The bare `--baseline` gate that the stdout change targets is the command `AGENTS.md` tells maintainers to run on this repo. (b) Evidence keys and `+` marking reach the regression detail and therefore `--verdict` and the action's annotations. (c) `--against` gets the same keys (`results_at` now returns them). One overstatement: README §0.2 lists "`actions/report` and CI logs" as uses, but `actions/report/run.sh` passes `-o md --out`, so the verdict-on-stdout half of the change does not reach the action or CI. Only the evidence-marking half does |
| Shaped to the measure? | Partly motivated by it, but it doesn't favour Toolbox. The decision says the verdict moved to stdout because "some agent harnesses record only stdout", and README §0.2 says "so the measure now counts it". That brings A's verdict *into* the count, which raises A's measured read. The part that cut A's read is dropping the full report from a gate's default output. A user reading a terminal benefits from that just as much |
| Other changes between rounds | Between rounds 1 and 2: `3762524` (README, round 2 prompts). Between rounds 2 and 3: `d30dcb2` (harness: `count.py` adds `gateChars`, `collect.py` reads `arms.tsv`, `round.sh` adds the A⁻ arm, rounds 1–2 re-collected) and `d53569d` (README). No product change. `git diff --stat dc42499 d53569d -- crates ui schema packages` is empty. The re-collection added `gateRead` and left every round 1–2 call and read figure unchanged. Every harness change applies to all arms |

## Protocol checks

| Rule | Verdict | Evidence |
|---|---|---|
| P1 committed prompts | **Pass** | All 27 launched prompts (each transcript's first user message) equal `prompts/rendered/r<n>/<run>.md` at that round's prompt commit, byte for byte. Runtime creation times are after the prompt commits: round 1 launched 11:00:06 (prompts in `60723d6`, 10:59:31), round 2 at 11:15:49 (`3762524`, 11:14:48), round 3 at 11:21:28 (`21a2a44`, 11:20:17). Results commits follow (`19d6965`, `967a508`, `94258f7`). Round 3's bar (`d53569d`, 11:19:27) predates its workspaces (built 11:19:35–11:20:09). `prompts/toolbox.md` and `prompts/reviewdog.md` differ only in the gate line. A and A⁻ share one template. Caveat: commit times are set by the author and the branch is unpushed; the runtime's `createdAtMs` corroborates them |
| P2 repeated runs | **Pass** | Rounds 1–2 had 3 runs per arm and round 3 had 5. Listing every cloud agent created since 10:30 gives 31: 27 scored runs, 2 dry runs, 1 telemetry probe and this adjudicator. Nothing was excluded or rerun. The TSVs report every run |
| P3 frozen rounds | **Pass, with notes** | No Rust changed in `703b53a..60723d6` (or `7c8dc45..703b53a`), or in `dc42499..d53569d`. `packages/github-actions` is the same at `703b53a` and `d53569d`. Round 1's workspaces were built at 10:58:00, before `60723d6` was committed (10:59:31), from a tree whose Toolbox content was identical. The round 1 results commit names `7c8dc45`, the README names `60723d6`/`703b53a`, and these agree in substance. A⁻'s binary (built in `/tmp/tb-old` at `703b53a`) hashes differently from round 1's, but its first gate output equals round 1's arm A byte for byte apart from commit hashes. Round 3's arm A output likewise equals round 2's |
| P4 symmetric arms | **Pass** | Neither prompt has a recipe, and the workspaces differ only in the gate (`setup.sh`). No rival friction was fixed or deferred, and the transcripts show none that blocks B. In round 3, two B agents (`reviewdog-3`, `reviewdog-4`) ran zizmor directly after their gate passed, to confirm. That may hint that a silent pass is under-informative; it wasn't triaged, and there is no further round |
| P5 no harness-shaped product | **Pass** | Fit audit above |
| P6 a reported measure | **Pass, with gaps** | *Calls* and *read* were shown for both arms in the dry run. *Gate read* is derived from the same transcript field by `count.py` and was computed for both arms of rounds 1–2 before round 3 was registered. Gaps: stdout only (threat 1), two `read_file` results missing (threat 2), and how gate commands are matched (threat 3). None removes a measure for an arm, so no label is void |
| P7 independent adjudication | **Pass, with a caveat** | See threat 10 |

The catalog entry `portal/benchmarks/gate-repair-ab.yaml` still says `runsPerArm: 3`, hypotheses
G1–G6 and `status: designed`. Its `measure` leaves out gate read, and it has no `fitAudit`,
`spread` or outcome. It needs updating to match this record. This adjudication does not edit it.

## Threats to validity

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

## Sources

- [README.md](./README.md) §0, §5, §5.1;
  [D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md) (P1–P7);
  [D01M841J2SN4V3AVZZ5D91YAR51](../../docs/decisions/D01M841J2SN4V3AVZZ5D91YAR51.md).
- `results/r{1,2,3}.tsv` and `results/r{1,2,3}/*.json|*.diff`; `harness/count.py`, `score.py`,
  `collect.py`, `setup.sh`, `round.sh`; `prompts/rendered/r3/arms.tsv`.
- Runtime transcripts of all 27 scored runs (`batch-fetch-details`, by the bcIds in the TSVs) and
  the runtime's list of cloud agents created since 2026-11-23 10:30 UTC.
- `git log`/`git show` of `60723d6`, `19d6965`, `dc42499`, `3762524`, `967a508`, `d30dcb2`,
  `d53569d`, `21a2a44`, `94258f7`, `144caf2` (box-ab friction table) and `703b53a` (root-causes C1).
