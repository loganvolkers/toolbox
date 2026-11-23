# Gate Repair A/B: Fixing a Pull Request the Gate Failed

**Question:** when a coding agent's pull request fails the repository's quality gate, does Toolbox's
ratchet get it to a correct pass for less effort than reviewdog, which runs the same tools and
reports only the findings the branch added?

**Status: complete under protocol 2 (D01M83RDWQ5EVKY5SFHDWAD5HFH). The labels are the independent
adjudicator's, in [ADJUDICATION.md](ADJUDICATION.md):**

- **Round 1:** Toolbox lost to reviewdog (G2 and G3 fail).
- **Round 2:** after the Toolbox change (§0.2), the round showed nothing (G4 and G5 inconclusive).
- **Round 3:** the round showed the improvement under its pre-registered rule (§5.1).
  - G8 holds: with the change, the median gate read fell from 18,751 characters to 3,876.
  - G9 and G10, on calls and total read, are inconclusive, not failed.
- **Round 4:** Toolbox does not beat reviewdog under its pre-registered rule (§5.2, §0.5).
  - G13 fails: the repair took more calls with Toolbox's gate. Half the gap came from how the
    agents edited, half from arm A's extra exploration.
  - G14 and G15, on read and gate read, are inconclusive.
  - G16 holds: the second change cut gate read again.
- **Round 5:** Toolbox does not beat reviewdog under its pre-registered rule (§5.3, §0.6). It no
  longer loses either, but round 4's unchanged Toolbox ties reviewdog in round 5 too.
  - G18 holds: Toolbox was correct in every run.
  - G19, G20 and G21, on calls, read and gate read, are inconclusive. Arm A's medians are lower on
    all three, inside the noise.

Toolbox is not shown to beat reviewdog on any measure. The bar for rounds 1 and 2 is in §5, round
3's is in §5.1, round 4's in §5.2 and round 5's in §5.3, each committed before its workspaces were
built.

This is about the box ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)): both arms
carry the same tools, zizmor and actionlint, at the same pins, with the same flags and the same
`.github/actionlint.yaml`. What differs is how the gate tells the agent what is wrong. The measure is
of R4 in [repo-value.md](../../docs/design/repo-value.md) ("without becoming a chore"). The friction
it targets was left open by the box A/B: *the gate's regression lists every finding in the file and
doesn't mark the new ones; the baseline keeps only counts* ([box-ab](../box-ab/README.md) §0.3 #4,
C1 in [root-causes.md](../../docs/design/root-causes.md)).

## 0. Results

### 0.1 Round 1 (frozen at `60723d6`; Toolbox's Rust as at `703b53a`)

Every run in both arms was correct. Per run: [results/r1.tsv](results/r1.tsv), with each run's
score and diff in [results/r1/](results/r1/).

| Arm | Calls (runs) | Median, range | Read, chars (runs) | Median, range |
|---|---|---|---|---|
| A: Toolbox | 7, 7, 6 | 7, 1 | 29,767; 28,023; 36,248 | 29,767; 8,225 |
| B: reviewdog | 4, 4, 4 | 4, 0 | 11,160; 11,178; 8,755 | 11,160; 2,423 |

**Friction, from the transcripts.** The rival arm showed none worth fixing (P4). Each B run read its
gate's six new findings (four from zizmor, two from actionlint), fixed them in one or two edits,
and reran it. In arm A:

1. **The first gate run was mostly report.** About 15.5k characters of every scorecard on stdout
   came first, old debt included. Then the verdict followed, on stderr.
2. **Old and new evidence were mixed.** `zizmor-clean evidence grew 9 → 12` listed ten of the
   twelve items. Every A agent worked out by hand which three were new. Two of them wrote about the
   old cache-poisoning finding, whose line had moved, before leaving it alone.
3. **Later runs were filtered.** Every A agent piped its later gate runs through `grep` or `rg` to
   find the verdict in the report.

**A gap in the measure.** The runtime transcript keeps a terminal command's stdout only. The agents
saw stderr: one cites "the evidence count went from 9 to 12", which appears only in the verdict, and
the transcript never contains "grew". So round 1's *read* for arm A leaves out the verdict. On the
planted branch, round 1's binary prints 15,521 characters on stdout and 3,711 on stderr. Arm A's
real read in round 1 is therefore higher than recorded. That makes G3's gap an underestimate, and
makes G4 and G5 harder to pass. Arm B's gate prints everything on stdout.

### 0.2 The Toolbox change before round 2

[D01M841J2SN4V3AVZZ5D91YAR51](../../docs/decisions/D01M841J2SN4V3AVZZ5D91YAR51.md), in
`crates/toolbox-cli` only; no harness file changed:

- **A gate prints its verdict.** `toolbox report --baseline` without `-o` or `--out` prints the verdict
  alone, on stdout, so the measure now counts it.
- **New evidence is marked.** The baseline records a key per evidence item, and a grown list shows
  only the new items, marked `+`, then "and 9 already in the baseline".

Uses outside this harness (P5): the bare `--baseline` check that `AGENTS.md` tells maintainers to
run, `--against` in local loops, the `+` marking in `actions/report`'s verdict and annotations,
box-ab friction #4, and root-causes C1. Box-ab friction #4 and root-causes C1 were recorded before
this benchmark. (This paragraph first listed `actions/report` for the stdout verdict too. The
action passes `--out`, so only the marking reaches it; ADJUDICATION.md fit audit. The decision's
"five new findings" should read six.) On the planted branch, the new binary's whole first gate output is 2,892 characters.

### 0.3 Round 2 (frozen at `3762524`; Toolbox's Rust as at `dc42499`)

Every run in both arms was correct. Per run: [results/r2.tsv](results/r2.tsv) and
[results/r2/](results/r2/).

| Arm | Calls (runs) | Median, range | Read, chars (runs) | Median, range |
|---|---|---|---|---|
| A: Toolbox | 7, 4, 6 | 6, 3 | 13,249; 31,901; 11,021 | 13,249; 20,880 |
| B: reviewdog | 6, 3, 4 | 4, 3 | 20,354; 25,416; 26,915 | 25,416; 6,561 |

By the designer's reading of §5 (the adjudicator's labels decide), round 2 does not show the
improvement. Arm A's median read fell by 16,518 characters, which is less than *W* (20,880), and arm
B's median read moved by 14,256 with nothing changed, more than its range allows. So G4 and G5 are
inconclusive.

**Why, from the transcripts.** The two parts of *read* moved differently. Characters of gate output
(`gateRead`, added to the TSVs after round 2 and descriptive only for rounds 1 and 2) per run:

| Arm | Round 1 | Round 2 |
|---|---|---|
| A: Toolbox | 18,596; 18,264; 27,518 | 3,801; 3,952; 3,876 |
| B: reviewdog | 5,073; 5,077; 3,708 | 5,077; 4,632; 4,632 |

The rest of *read* is the agent's own exploration of the workflows. That doesn't depend on the gate,
and it swung in both arms. In round 2, four of six agents ran one broad `cat`/`grep` over
`.github/workflows/` that returned between 11,714 and 20,746 characters. In round 1 no agent did.
With three runs, that noise is wider than the effect *read* was meant to detect. A between-round
control can't separate it either, because it moves arm B too.

### 0.4 Round 3 (frozen at `d53569d`; arm A's Rust as at `dc42499`, arm A⁻'s as at `703b53a`)

All fifteen runs were correct. Per run: [results/r3.tsv](results/r3.tsv) and
[results/r3/](results/r3/).

| Arm | Calls: median, range | Read, chars: median, range | Gate read, chars (runs) | Median, range |
|---|---|---|---|---|
| A: changed Toolbox | 5, 3 | 24,517; 22,816 | 4,242; 3,801; 3,952; 3,876; 3,876 | 3,876; 441 |
| A⁻: previous Toolbox | 7, 2 | 28,614; 4,957 | 19,884; 19,137; 18,751; 16,714; 16,695 | 18,751; 3,189 |
| B: reviewdog | 4, 3 | 17,883; 14,903 | 3,263; 4,632; 3,112; 3,557; 3,263 | 3,263; 1,520 |

The adjudicator's labels:

- G7 and G8 hold.
- G9, G10, G11, G2″ and G3″ are inconclusive.

So **the improvement is shown** under §5.1, and only in the sense §5.1 defines. The change cut what
an agent reads from the gate by a factor of about 4.8, with no measured cost moved elsewhere. No
round shows that a repair takes fewer calls, or less reading overall. ADJUDICATION.md lists ten
threats, none of which changes a label. Two are worth reading first. G8 is close to a manipulation
check (threat 5). And on the gates' own output, reviewdog's 1,222 characters are still fewer than
Toolbox's 2,867 (threat 3).

**Friction still open, for the next round or benchmark:**

- **Toolbox's failing gate still says more than reviewdog's.** It reports eight regressions where
  reviewdog reports six findings. One defect appears under up to three rules. The unpinned
  checkout, for example, shows up under `pinned-everywhere`, `pinned-to-sha` and zizmor's
  `unpinned-uses`, and the title injection under actionlint, `no-high-findings` and `zizmor-clean`.
  Every regression also ends with a `from: connector …` line.
- **The agent still explores on its own.** In every arm, the spread in *read* comes from whether
  the agent ran one broad `cat`/`grep` over the workflows. Both gates name the file and, for most
  findings, the line. A check that also names the pinned SHA already in the repository might remove
  the reason to look.
- **A pass says nothing.** In round 3, two reviewdog agents ran zizmor directly after their gate
  passed, to confirm.

### 0.5 Round 4 (frozen at `02f9de5`; arm A as at `76a1140`, arm A⁻ as at `d163449`)

The question was whether Toolbox now beats reviewdog. It doesn't. All fifteen runs were correct.
Per run: [results/r4.tsv](results/r4.tsv) and [results/r4/](results/r4/).

| Arm | Calls (runs) | Median, range | Read, chars: median, range | Gate read, chars (runs) | Median, range |
|---|---|---|---|---|---|
| A: changed Toolbox | 7, 6, 7, 7, 6 | 7, 1 | 10,297; 10,922 | 2,527; 2,527; 2,527; 2,527; 2,669 | 2,527; 142 |
| A⁻: previous Toolbox | 6, 4, 3, 6, 6 | 6, 3 | 24,198; 14,656 | 4,018; 4,018; 3,952; 3,876; 3,943 | 3,952; 142 |
| B: reviewdog | 4, 3, 3, 3, 4 | 3, 1 | 10,374; 2,990 | 3,186; 3,371; 3,337; 3,337; 5,099 | 3,337; 1,913 |

By the designer's reading of §5.2 (the adjudicator's labels decide):

| Id | Reading |
|---|---|
| G12 | Holds: 5 of 5 correct |
| G13 | **Fails**: med(A) − med(B) = 4 calls, more than *W* = 1 |
| G14 | Inconclusive: medians 77 characters apart, *W* = 10,922 |
| G15 | Inconclusive: arm A read 810 characters less gate output, but *W* = 1,913 |
| G16 | Holds: med(A⁻) − med(A) = 1,425 > *W* = 142 |
| G17 | Inconclusive: 13,901 < *W* = 14,656 |

So **Toolbox does not beat reviewdog**. The change did what it was for: G16 holds, so the agent read
less from the gate. But the repair took more calls, and the read measures can't separate the arms.
ADJUDICATION.md confirms these labels. It also corrected four of the designer's first explanations,
and the paragraphs below follow its corrections.

**Why the calls differ (ADJUDICATION.md threat 12).** Half of the 4-call gap came from editing and
half from exploring. Either half alone fails G13.

| Arm | Calls on gate runs and edits: median | Exploration calls: median |
|---|---|---|
| A: changed Toolbox | 4 | 3 |
| A⁻: previous Toolbox | 4 | 2 |
| B: reviewdog | 2 | 1 |

- **Editing.** Every A run made its two edits with the editor's search-and-replace tool and reran
  the gate in its own call. Every B run made both edits in one `python3` heredoc, with the gate
  rerun in the same command. Heredoc editing was 0 of 5 runs in A, 2 of 5 in A⁻ and 5 of 5 in B. The
  round doesn't show what caused that split; the arms differ in more than the gate's output.
- **Exploring.** Four A runs read a file before editing, which the tool does not require. Three
  made a separate lookup for the version comment that the existing pins carry (`# v6.0.2`,
  `# v7.0.1`). Their notes say so ("I'll search for that comment to match it"). Every run's diff, in
  every arm, copies those comments, and the gate's SHA hint doesn't print them.

**Read is undercounted for arm A (threat 13).** In three A runs, the `read_file` of
`pull_request.yaml` has no recorded result, so `count.py` counts it as 0. Corrected, arm A's median
read is about 23,100 characters, more than twice reviewdog's. G14 stays inconclusive either way. So
this round does not show that arm A reads less overall.

**Gate read counts whole commands (threat 14).** Arm A's lower gate read comes from what B's agents
chained onto their gate runs: four printed the gate script with the first one. On the gates' own
output, reviewdog's is shorter, 1,243 characters in all against 1,632.

**Corrections to the record.**
- The 2,892 and 1,541 that §5.2 and §0.2 give as characters are bytes. In characters, which the
  measure counts, they are 2,860 and 1,529.
- `d163449` is not just round 3's arm A plus unrelated commits: those commits change 41 product
  files. On this task its gate output is byte-identical to round 3's arm A.
- reviewdog prints 192 characters to stderr on the planted branch, and no round counts them.

**Friction still open:**

- **The SHA hint leaves out the version comment.** A hint that carried the existing pin as written
  (`actions/upload-artifact@043fb46d… # v7.0.1`) would remove the lookup that three A runs made.
- **Calls depend on how the agent edits.** Through its output, Toolbox's gate could at most offer a
  fix the agent applies in one command. Even then, by threat 12's counterfactuals, arm A's median
  would be 5 calls, and G13 would still fail against reviewdog's 3.
- **The read measure misses some `read_file` results** (threat 13). A future round needs
  `count.py` to flag a call whose result is missing, so the measure doesn't count it as zero
  without saying so.
- **reviewdog's pass line worked.** No B run checked a pass by hand (round 3: two of five).

The designer predicted G12 and G16 would hold and G13, G14 and G17 would be inconclusive, with G15
about even. G13 failing was not predicted. As registered, this round is not rerun.

### 0.6 Round 5 (frozen at `0249ecb`; arm A as at `c1f9b22`, arm A⁻ as at `76a1140`)

The question was round 4's again: does Toolbox now beat reviewdog? It doesn't, and it no longer
loses, but not because of the change. All fifteen runs were correct, and every comparison is
inconclusive. Arm A's medians are lower than reviewdog's on all three measures, and each gap is
inside the noise. Round 4's Toolbox, run alongside as A⁻, also ties reviewdog on every measure,
though in round 4 it lost on calls (ADJUDICATION.md threat 22). The prompts were
committed in `9ca1605` before launch, and the results in `eb2c1af`. Per run:
[results/r5.tsv](results/r5.tsv) and [results/r5/](results/r5/).

| Arm | Calls (runs) | Median, range | Read, chars: median, range | Gate read, chars (runs) | Median, range |
|---|---|---|---|---|---|
| A: changed Toolbox | 6, 3, 3, 6, 3 | 3, 3 | 10,859; 11,068 | 3,102; 3,102; 3,178; 3,102; 3,320 | 3,102; 218 |
| A⁻: previous Toolbox | 6, 3, 3, 4, 7 | 4, 4 | 9,238; 13,886 | 2,486; 2,603; 3,972; 2,603; 2,527 | 2,603; 1,486 |
| B: reviewdog | 3, 4, 4, 4, 4 | 4, 1 | 13,960; 9,197 | 5,918; 5,918; 338; 8,740; 4,398 | 5,918; 8,402 |

By the designer's reading of §5.3 (the adjudicator's labels decide):

| Id | Reading |
|---|---|
| G18 | Holds: 5 of 5 correct |
| G19 | Inconclusive: med(B) − med(A) = 1 call, *W* = 3 |
| G20 | Inconclusive: arm A read 3,101 characters less, *W* = 11,068 |
| G21 | Inconclusive: arm A read 2,816 characters less gate output, *W* = 8,402 |
| G22 | Inconclusive: med(A⁻) − med(A) = 1 call, *W* = 4 |
| G23 | Inconclusive: arm A read 1,621 characters *more* than A⁻, *W* = 13,886 |

So **Toolbox does not beat reviewdog** under §5.3: G18 holds, but none of G19, G20 and G21 holds.
None fails either; in round 4, G13 failed. The designer predicted that G20, G22 and G23 would
hold, and none did. ADJUDICATION.md confirms these labels. It also corrected three of the
designer's first claims, and the paragraphs below follow its corrections.

**Missing results.** One A run (`toolbox-1`) and one A⁻ run (`toolbox-10`) read
`pull_request.yaml` with no recorded result, and the measure fix counts each as the file, 13,515
characters. One B run
(`reviewdog-3`) chained `git diff main` onto its first gate run, and that command has no recorded
result. `count.py` imputes only `read_file`, so that run's read and gate read are low by about
4,300 characters (the diff and the gate's output on `pr`). Corrected, B's read range is about
4,900 and its gate-read range about 4,300. Arm A's read range of 11,068 is still the wider one,
and gate read's gap of 2,816 is still under *W*. No label changes.

**Every run printed the branch's files or diff (15 of 15).** In every arm, every run printed the
workflows before editing, with `git show HEAD`, `git diff main`, `sed -n`, `cat` or `read_file`,
even when the gate had quoted the lines. That reading is the floor of the read measure: from about
3,500 to more than 15,000 characters a run, against a first gate output of 2,091. The quote did not
replace it. One thing the gate leaves out is the fix for `needs: lnt`. actionlint's message says
that the job "does not exist in this workflow" and doesn't name the jobs that do, so the agent has
to look for `lint`. Four runs grepped for the job names; the rest read them in the files they
printed.

**Editing style moved in both Toolbox arms.** Three A runs (`toolbox-2`, `-3`, `-5`) made both
edits in one `python3` heredoc with the gate rerun in the same command, and each took 3 calls, as
few as any B run. The two that used search-and-replace took 6. The move is not the change's
doing: the same Toolbox went from no heredoc runs as round 4's arm A to three as round 5's A⁻. All
five B runs used a heredoc. Three of them (`reviewdog-2`, `-4`, `-5`) looked up a pin in a call of
its own, and the other two did it in the call that printed the files. No A run made a separate
lookup, but `toolbox-3` and `toolbox-5` grepped for the pins while printing the files. Only
`toolbox-2` clearly took the pin from the gate's hint alone.

**Corrections to the record.**
- D01M84EPMMR9740K4N25G7A6FAY says four of round 4's A runs read `pull_request.yaml` before
  editing it. Three did; `toolbox-5` read only `release.yaml`.
- Its `decided` time (15:10) is later than its commit (15:04); it is the minute round 5 launched.
- §5.3 says the rival fix "makes the rival stronger". That is not shown: every B run still printed
  the files, and the fix lengthened B's gate output more than the change lengthened A's.

**Friction still open:**

- **The `needs` finding doesn't name the jobs that exist.** A hint listing them (`lint`, `test`, …)
  would cover the one fix the quote can't show.
- **Agents print the branch before they fix it.** Every run in every arm did. A gate output can't
  remove that, so the read measure can't separate the arms by much more than the gate outputs
  differ.
- **`count.py` imputes only `read_file`.** A terminal command without a recorded result still
  counts as 0 (`reviewdog-3`).
- **Calls still turn on how the agent edits.** The heredoc runs tie reviewdog's best; the
  search-and-replace runs take two calls more.

As registered, this round is not rerun.

**Can a later round on this task win?** Not with a fair rival, by the designer's arithmetic from
rounds 4 and 5. The adjudicator has not checked this.

- **Calls (G19).** Every run in every arm printed the branch before editing, so a repair is at
  least a gate run, a look and an edit with a rerun: 3 calls. reviewdog's runs already take 3 or
  4, so arm A can't get far enough below them to beat *W*.
- **Read (G20).** The same printing is 3,500 to more than 15,000 characters a run in every arm.
  The gates' outputs differ by far less than that spread.
- **Gate read (G21).** A one-line-per-finding output, with the quote dropped and the pin rules
  folded, would print about 1,050 characters, half the current 2,091. Against round 5's reviewdog,
  the gap would be about 3,900, under its range of 4,342. It would also come close only because
  reviewdog's gate still quotes the source. P4 gave reviewdog that quoting because Toolbox's gate
  quotes. If Toolbox stops quoting, reviewdog's gate goes back to one line per finding, and the two
  outputs differ by a few hundred characters. What is left is what agents chain onto their gate
  commands, which no product change controls.

Toolbox's one advantage on this task is the pin hint, which saves reviewdog's lookup. That
lookup costs about one call or a few thousand characters, and the agents' own variation is larger
than either. A fair win needs a different question, as
[dx-benchmarks.md](../../docs/design/dx-benchmarks.md) L19 and B10 say. One candidate is a task
where most fixes need knowledge from other files.

## 1. Why this benchmark

Every repository that adopts Toolbox meets the gate on its pull requests, much more often than
anything else in Toolbox. Most of the time the reader is a coding agent whose change made something
worse. Round 1 measures what that costs, against the tool a team would otherwise reach for: reviewdog
with `-filter-mode=added`, the usual way to gate only on findings a pull request introduces.

## 2. Arms

| | Arm A: Toolbox | Arm B: reviewdog |
|---|---|---|
| Gate | `toolbox report .toolbox --skip github-actions --baseline .toolbox/results.json` | `scripts/quality-gate.sh` |
| What it compares | Every rule outcome against the committed baseline (`.toolbox/results.json`) | Every finding against the lines `git diff main` added |
| Portal or script | [harness/toolbox.yaml](harness/toolbox.yaml): the repo-portal starter plus `packages/github-actions`, vendored | [harness/quality-gate.sh](harness/quality-gate.sh): zizmor and actionlint piped into reviewdog 0.21.0 |
| Tools | zizmor 1.32.0 (`--offline`, workflows only, ignores honoured), actionlint 1.7.12 (`-shellcheck= -pyflakes=`) | The same |

**Strongest rival.** reviewdog is the widely used diff filter for linters in CI. Its `added` mode
solves exactly the problem the ratchet solves with a baseline: fail on what the change introduced,
not on old debt. Both arms honour `# zizmor: ignore` comments as their tools do, and the scorer
rejects any that are added.

## 3. Fork and planted pull request

The fork is [loganvolkers/openfga](https://github.com/loganvolkers/openfga) at `7d89a566`, which
ratchet-replay also used. It has 12 workflows with 38 existing zizmor findings, so the gate has old
debt to tell apart from new findings. [harness/setup.sh](harness/setup.sh) commits the fork plus
the arm's gate as `main`, records arm A's baseline, then [harness/plant.py](harness/plant.py) commits
the pull request on branch `pr`:

- `pull_request.yaml` gets a `pr-summary` job: `needs: lnt` (a typo; actionlint), `actions/checkout@v6`
  (unpinned; zizmor, and Toolbox's own pin rules), no `persist-credentials: false` (zizmor
  `artipacked`), and `${{ github.event.pull_request.title }}` inline in `run:` (zizmor
  `template-injection`; actionlint `expression`). It sets `timeout-minutes` so that Toolbox's
  `every-job-has-timeout` rule, which reviewdog's gate has no counterpart for, adds no work.
- `release.yaml` gets an upload of `digests.txt` with `actions/upload-artifact@v7` (unpinned).

Every fix is possible offline: both pinned SHAs are already in the repository.
[harness/reference_fix.py](harness/reference_fix.py) is one correct repair. On it both gates pass
and the scorer says correct; on the planted branch both fail.

## 4. Scoring

[harness/score.py](harness/score.py) is hidden from the agents and reads the tools directly, not
either gate. A run is **correct** only if all of these hold:

| Check | What it requires |
|---|---|
| `no-new-findings` | For every (tool, file, rule), no more findings in the working tree than on `main` |
| `intent-pr-summary` | `pr-summary` exists, needs only jobs that exist, and still writes the PR title to `$GITHUB_STEP_SUMMARY` |
| `intent-release-digests` | The goreleaser job still uploads `digests.txt` with `actions/upload-artifact` |
| `gate-untouched` | The gate's files match `main` (A: `.toolbox/`; B: `scripts/quality-gate.sh`; both: `.github/actionlint.yaml`), and no zizmor config was added |
| `no-suppressions` | No `zizmor: ignore` comment was added |
| `gate-passes` | The arm's own gate exits 0 |

## 5. Measures and the bar (pre-registered)

**Measures (P6).** [harness/count.py](harness/count.py) reads each run's runtime transcript (what
`batch-fetch-details` writes for the subagent's id):

- **calls**: tool calls the agent made;
- **read**: characters of tool results the agent received, the volume it had to read (local
  subagents report no token counts).

Wall time and gate runs are reported but are not in the bar. The dry run (§8) showed the runtime
reports both measures for both arms.

**Rules.** Medians and ranges are over the three runs of an arm in a round. *W* is the wider of the
two ranges being compared (maximum minus minimum). A hypothesis about a measure holds or fails only
if the medians differ by more than *W*; otherwise it is inconclusive (P2).

| Id | Round | Hypothesis | Holds | Fails |
|---|---|---|---|---|
| G1 | 1 | Arm A is correct in every run | 3 of 3 correct | Any run incorrect |
| G2 | 1 | With Toolbox's gate, a repair takes fewer calls than with reviewdog's | med(B) − med(A) > *W* | med(A) − med(B) > *W* |
| G3 | 1 | With Toolbox's gate, the agent reads less than with reviewdog's | med(B) − med(A) > *W* | med(A) − med(B) > *W* |
| G4 | 2 | The Toolbox change between rounds cut arm A's calls | med(A₁) − med(A₂) > *W*(A₁, A₂), and arm B's medians moved by no more than *W*(B₁, B₂) | med(A₂) − med(A₁) > *W*(A₁, A₂) |
| G5 | 2 | The Toolbox change between rounds cut what arm A reads | As G4, on read | As G4, on read |
| G6 | 2 | Arm A is still correct in every run | 3 of 3 correct | Any run incorrect |

In round 2, G1–G3 are applied again, as G1′–G3′. If arm B's medians moved by more than its range
(G4 and G5's noise control), G4 and G5 are inconclusive, whatever arm A did. **The improvement is
shown** only if G6 holds, at least one of G4 and G5 holds, and neither fails. If arm B is incorrect
in a run, that is reported; it does not change arm A's labels.

**The designer's prediction, before round 1:** G1 holds, G2 is inconclusive or fails, and G3 fails.
In the dry run, Toolbox's first gate run printed the whole repository's report before the
regressions, about 16k characters, and listed old and new findings together, so the agent reasoned
about which were new.

### 5.1 Round 3 (pre-registered after round 2, before round 3's workspaces were built)

Round 3 asks round 2's question again, with an answer that round 2's noise can't hide. The designer
had seen rounds 1 and 2 when writing this, and chose round 3's primary measure from them (§0.3).
Round 3 is a replication of what they suggest, and the adjudicator should read it that way.

**What changes from rounds 1 and 2:**

- **Both Toolboxes in one round.** A third arm, A⁻ (`toolbox-prev`), runs the Toolbox from before the
  change (the Rust at `703b53a`, built in a worktree of that commit). It runs alongside arm A, which
  runs the changed Toolbox at the frozen commit, and arm B. They run on the same harness, at the same
  time, with the same prompt as arm A. That removes drift between rounds, which in round 2 was larger
  than the effect. `packages/github-actions` is the same at both commits, so A⁻'s portal is A's.
  Workspaces are named `toolbox-1…5` (A) and `toolbox-6…10` (A⁻), so a path doesn't say which
  Toolbox it holds; `prompts/rendered/r3/arms.tsv` maps runs to arms.
- **Five runs per arm** (P2 asks for at least three); fifteen runs in all, launched together.
- **A third measure, gate read**: characters of tool results from the agent's gate runs, as
  `count.py` matches them (`gateChars`). It is what the change altered. *Calls* and *read* stay,
  as guards.

The transcript's stdout-only gap (§0.1) still applies. A⁻ prints its verdict on stderr, so A⁻'s gate
read and read are undercounted. That makes G8 and G10 harder to pass, not easier.

| Id | Hypothesis | Holds | Fails |
|---|---|---|---|
| G7 | Arm A is correct in every run | 5 of 5 correct | Any run incorrect |
| G8 | The change cut what the agent reads from the gate | med(A⁻) − med(A) > *W*, on gate read | med(A) − med(A⁻) > *W*, on gate read |
| G9 | The change cut calls | med(A⁻) − med(A) > *W*, on calls | med(A) − med(A⁻) > *W*, on calls |
| G10 | The change cut what the agent reads in all | med(A⁻) − med(A) > *W*, on read | med(A) − med(A⁻) > *W*, on read |
| G11 | With the change, Toolbox's gate output costs less to read than reviewdog's | med(B) − med(A) > *W*, on gate read | med(A) − med(B) > *W*, on gate read |

*W* is as in §5, over the two arms compared, now of five runs each. G2 and G3 are applied again to
arms A and B, as G2″ and G3″. **The improvement is shown** if G7 holds, G8 holds, and neither G9 nor
G10 fails. Arm A⁻ or B being incorrect in a run is reported; it changes no label.

**The designer's prediction:** G7 and G8 hold; G9 and G10 are inconclusive; G11 is inconclusive.

### 5.2 Round 4 (pre-registered after round 3, before round 4's workspaces were built)

Round 4 asks whether Toolbox now beats reviewdog. It uses the same task, fork, planted pull request,
prompt templates and decision rule (P2) as rounds 1–3. The designer had seen round 3, and made two
changes from its open friction (§0.4):

- **The Toolbox change**
  ([D01M84CCDH1ZWF9726SVDDM1JDX](../../docs/decisions/D01M84CCDH1ZWF9726SVDDM1JDX.md), commit
  `76a1140`; no harness file).
  - A failing gate lists each new finding once, under its entity, with the rules that report it.
  - The `github-actions` package gives a finding the same text across rules and stops counting
    zizmor's `unpinned-uses` under `zizmor-clean`.
  - `pinned-everywhere` names the SHA the repository already pins the action to.
  - On the planted branch, the first gate output fell from 2,892 characters to 1,541. A pass prints
    103.
- **The rival fix (P4).** reviewdog's gate printed nothing on a pass, and in round 3 two agents ran
  zizmor by hand to confirm. `quality-gate.sh` now prints `quality gate: passed` (20 characters),
  in a harness-only commit.

**Arms.** Five runs each, fifteen in all, launched together:

- A, the changed Toolbox;
- B, reviewdog with the P4 fix;
- A⁻, the Toolbox before the change: the Rust and `packages/github-actions` as at `d163449`, which
  was round 3's arm A plus unrelated commits. `round.sh` now vendors A⁻'s package from
  `PREV_CHECKOUT`, because the package changed. Workspaces and `arms.tsv` are as in round 3.

**Measures:** calls, read and gate read, as in §5.1, with `count.py` unchanged.

| Id | Hypothesis | Holds | Fails |
|---|---|---|---|
| G12 | Arm A is correct in every run | 5 of 5 correct | Any run incorrect |
| G13 | With Toolbox's gate, a repair takes fewer calls than with reviewdog's | med(B) − med(A) > *W*, on calls | med(A) − med(B) > *W*, on calls |
| G14 | With Toolbox's gate, the agent reads less than with reviewdog's | As G13, on read | As G13, on read |
| G15 | With Toolbox's gate, the agent reads less gate output than with reviewdog's | As G13, on gate read | As G13, on gate read |
| G16 | The change cut what the agent reads from the gate (attribution) | med(A⁻) − med(A) > *W*, on gate read | med(A) − med(A⁻) > *W*, on gate read |
| G17 | The change cut what the agent reads in all (attribution) | As G16, on read | As G16, on read |

*W* is as in §5, over the two arms compared. **Toolbox beats reviewdog** only if all of these hold:

- G12 holds;
- at least one of G13, G14 and G15 holds;
- none of G13, G14 and G15 fails.

G16 and G17 say whether a win is due to this change; they don't decide it. A run of arm B or A⁻ that
is incorrect is reported and changes no label. This is the only round under this registration: if
it doesn't show a win, the result is published as it is, not rerun.

**Before the run, the designer reads the odds as follows.**

- **Calls (G13) can't realistically hold.** Arm B's calls ranged 0, 3 and 3 in rounds 1–3, around
  a median of 4. With *W* = 3, arm A would need a median under 1.
- **Read (G14) holds only if arm B's runs bunch high.** In round 3, B's read was bimodal (10k to
  25k), depending on whether the agent ran one broad `cat` or `grep`. The SHA hint should remove arm
  A's reason to search, but not B's.
- **Gate read (G15) is about even odds.** Gate read counts the whole result of any command that runs
  the gate:
  - In round 3, four of five B runs put `cat scripts/quality-gate.sh` (about 1,000 characters) in
    the same command as their first gate run. That first command's result was 2,791 characters.
  - Both arms' agents chained `git status`, `git log`, edits or `git diff` onto their gate runs.
  - From round 3's commands, the new first output gives arm A about 2,050 characters on its first
    gate run, and about 2,550 in all. Arm B's median was 3,263, with second gate runs from 321 to
    1,841 characters.
  - So the gap is about 700. G15 holds only if neither arm's range is wider than that. B's range
    was 1,520 in round 3, because one run put an edit and its diff into the second gate run.
  - Two things favour arm A. B's pass line adds about 20 characters to its gate read. And reading B's
    script in the same command counts as gate read: the script is the gate's whole configuration,
    and arm A has nothing like it to read.
- **The designer's prediction:** G12 holds; G13 and G14 are inconclusive; G15 holds or is
  inconclusive, about equally likely; G16 holds; G17 is inconclusive.

### 5.3 Round 5 (pre-registered after round 4, before round 5's workspaces were built)

Round 5 asks round 4's question again: does Toolbox now beat reviewdog? It uses the same task, fork,
planted pull request, prompt templates and decision rule (P2). §5.2 said round 4 is not rerun, and
round 5 doesn't rerun it. It follows product changes made from round 4's open friction (§0.5), as
[dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §11 asks ("rerun after the fixes").

**Disclosure.** This is the fifth round on one task, and the second that asks whether Toolbox
beats reviewdog. The designer had seen every transcript of round 4 when making the changes and
writing this section. A second attempt at the same question raises the chance of a win by chance,
and the adjudicator should weigh that. The rule itself is unchanged.

**What changed since round 4, in three commits:**

- **The Toolbox change**
  ([D01M84EPMMR9740K4N25G7A6FAY](../../docs/decisions/D01M84EPMMR9740K4N25G7A6FAY.md), commit
  `c1f9b22`; no harness file).
  - A failing gate quotes the lines of each entity's file that its new findings name.
  - `pinned-to-sha` names each unpinned `uses:` with its line.
  - `pinned-everywhere`'s hint carries the existing pin as written, version comment included.
  - On the planted branch, the first gate output rose from 1,529 characters to 2,091. It quotes
    11 lines, and a pass prints 103 characters, as before.
- **The rival fix (P4, commit `eb45c5d`, harness only).** In round 4, every reviewdog run printed
  the changed files (`git diff`, `sed -n`) after its gate. Toolbox's gate now quotes the code, and
  reviewdog's tools can too.
  - `quality-gate.sh` reads zizmor's `plain` format and actionlint's default format, which quote
    the source, through multi-line errorformats.
  - The failing output rose from 1,222 characters to 2,225 on stdout, and stderr is unchanged at
    192.
  - Pass, failure and filtering are as before. Both gates pass on `reference_fix.py`, and a fix of
    the pins alone still fails.
  - This makes the rival stronger.
- **The measure fix (commit `295b0d3`, harness only, every arm).** `count.py` names each call
  without a recorded result (`missing` in the TSV). *Read* now counts a `read_file` without a
  result as the runtime's result for that file on `pr`, built as a recorded one is.
  - On round 4's runs this reproduces `toolbox-5`'s recorded read of `release.yaml` exactly
    (9,562 characters), and the adjudicator's 13,515 for `pull_request.yaml`.
  - The recorded read stays in the TSV as `readRecorded`. Threat 13's undercount applied only
    to arm A, so this works against arm A.

**Arms.** Five runs each, fifteen in all, launched together:

- A, the changed Toolbox, at the frozen commit;
- B, reviewdog with the P4 fix;
- A⁻, round 4's arm A, with the Rust and `packages/github-actions` as at `76a1140`.

Workspaces and `arms.tsv` are as in rounds 3 and 4.

**Measures:**
- *calls* and *gate read*, as in §5.2;
- *read*, with the measure fix.

| Id | Hypothesis | Holds | Fails |
|---|---|---|---|
| G18 | Arm A is correct in every run | 5 of 5 correct | Any run incorrect |
| G19 | With Toolbox's gate, a repair takes fewer calls than with reviewdog's | med(B) − med(A) > *W*, on calls | med(A) − med(B) > *W*, on calls |
| G20 | With Toolbox's gate, the agent reads less than with reviewdog's | As G19, on read | As G19, on read |
| G21 | With Toolbox's gate, the agent reads less gate output than with reviewdog's | As G19, on gate read | As G19, on gate read |
| G22 | The change cut calls (attribution) | med(A⁻) − med(A) > *W*, on calls | med(A) − med(A⁻) > *W*, on calls |
| G23 | The change cut what the agent reads in all (attribution) | As G22, on read | As G22, on read |

*W* is as in §5, over the two arms compared. **Toolbox beats reviewdog** only if all of these hold:

- G18 holds;
- at least one of G19, G20 and G21 holds;
- none of G19, G20 and G21 fails.

G22 and G23 say whether a win is due to this change; they don't decide it. A run of arm B or A⁻
that is incorrect is reported and changes no label. Arm A's gate read is expected to rise against
A⁻'s, because the quote is longer. It is reported, not hypothesised. If this round doesn't show a
win, the result is published as it is, and this round is not rerun.

**Before the run, the designer reads the odds as follows.**

- **Calls (G19) can't realistically hold, and may fail.** Without exploring, arm A's repair is a
  gate run, one or two edits and a rerun: 4 calls with two search-and-replace edits, or 3 with a
  heredoc. Arm B still has to look up both SHAs, so it likely stays at 3. With *W* = 1, arm A fails
  G19 if its median is 5, so one exploration call in most A runs fails it.
- **Read (G20) is the likeliest to hold.** Arm A's planned path has no `read_file` and no lookup.
  From round 4's commands, it reads about 2,600 characters on the first gate run, about 2,000 on
  two edits and about 500 on the rerun: about 5,000 in all. Arm B no longer needs to print the
  files, but it still greps for the SHAs. Its read is about 3,900 on the first gate command (the
  script and the gate), 700 to 4,500 on the lookup, and about 500 on the edit and rerun: about
  5,000 to 9,000. G20 holds only if B keeps a broad lookup and neither arm's range is wide.
- **Gate read (G21) is likely inconclusive.** Arm A's gate output grew by 562 characters and B's
  by 1,003. B's agents also tend to print the script with their first gate run. That gives arm A
  about 3,200 and arm B about 4,200, a gap of about 1,000. A single chained `git diff` widens
  either range past that.
- **The designer's prediction:** G18 holds; G19 is inconclusive; G20 holds; G21 is
  inconclusive; G22 and G23 hold.

## 6. Protocol

- **Rounds (P3).** Each round freezes one commit of this repository (`frozenAt`) for Toolbox, the
  harness and both arms. The Toolbox binaries are built from it. A Toolbox change made after round 1
  starts round 2, in which both arms rerun.
- **Runs (P2).** Three fresh runs per arm per round (five in rounds 3 to 5), each in its own workspace
  (`/tmp/gate-repair-ab/r<round>/<arm>-<n>`) with its own copies of the tools. Each run is a fresh
  local subagent with the inherited model and only the rendered prompt. No run is dropped for its
  result.
- **Prompts (P1).** [prompts/toolbox.md](prompts/toolbox.md) and
  [prompts/reviewdog.md](prompts/reviewdog.md) differ only in the gate command. Each round's
  rendered prompts are committed under `prompts/rendered/r<round>/` before its first launch.
- **Symmetry (P4).** Rival friction that a round shows is fixed before the next round, as carefully
  as Toolbox's.
- **Fit (P5).** A Toolbox change between rounds must name a use outside this harness and touch no
  harness file. The adjudicator audits every change (`fitAudit`).
- **Adjudication (P7).** The designer is the agent that wrote this README and runs the rounds. A
  fresh agent that designed no part of it and changed nothing applies the bar, and writes
  `ADJUDICATION.md` with only holds, fails, inconclusive or void per hypothesis.
- **Run conditions.** Every run sees the same workspace rules the runtime injects. Runs ask for no
  network; the planted fix needs none.

## 7. Running it

```bash
export PATH=<dir with zizmor 1.32.0, actionlint 1.7.12 and reviewdog 0.21.0>:$PATH
cargo build                                       # at the round's frozen commit
experiments/gate-repair-ab/harness/round.sh 1 <openfga export at 7d89a566> "$PWD" "$PWD/target/debug"
# round 3: RUNS=5 PREV_TARGET=<worktree of 703b53a>/target/debug round.sh 3 …
# round 4: RUNS=5 PREV_TARGET=<worktree of d163449>/target/debug PREV_CHECKOUT=<that worktree> round.sh 4 …
# round 5: RUNS=5 PREV_TARGET=<worktree of 76a1140>/target/debug PREV_CHECKOUT=<that worktree> round.sh 5 …
# launch the rendered prompts as fresh subagents; fetch their transcripts; then:
experiments/gate-repair-ab/harness/collect.py <round> <transcripts dir> <run>=<bcId> …
```

## 8. Dry run (unscored)

One run per arm on Toolbox `7c8dc45` (the same Rust as `703b53a`), to check the measures and that the task is possible offline.
Both runs were correct.

| Arm | Calls | Read (chars) | Gate runs | Wall (s) |
|---|---|---|---|---|
| A: Toolbox | 8 | 34,709 | 2 | 17 |
| B: reviewdog | 4 | 13,462 | 2 | 13 |

After the dry run, `count.py` stopped counting `cat scripts/quality-gate.sh` as a gate run.

## Sources

- reviewdog 0.21.0, `-filter-mode=added` and `-efm`: `reviewdog -h` and
  [github.com/reviewdog/reviewdog](https://github.com/reviewdog/reviewdog) (README, filter modes).
- zizmor 1.32.0 `--format github`, `--format plain` and `--format sarif`; actionlint 1.7.12's
  default format, `-oneline` and
  `-format '{{json .}}'`: each tool's `--help`, run in this repository's CI tool pins.
- Protocol 2: [D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md).
