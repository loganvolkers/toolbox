# Gate Repair A/B: Fixing a Pull Request the Gate Failed

**Question:** when a coding agent's pull request fails the repository's quality gate, does Toolbox's
ratchet get it to a correct pass for less effort than reviewdog, which runs the same tools and
reports only the findings the branch added?

**Status: registered under protocol 2 (D01M83RDWQ5EVKY5SFHDWAD5HFH); round 1 run (§0.1), Toolbox
changed (§0.2), round 2 run (§0.3), round 3 registered (§5.1) and next.** The bar for rounds 1 and 2
is in §5 and was committed before the first scored launch; round 3's is in §5.1 and was committed
before its workspaces were built. Labels are the adjudicator's, in `ADJUDICATION.md` once every
round is in.

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
gate's five new findings, fixed them in one or two edits, and reran it. In arm A:

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

Uses outside this harness (P5): every repository's pull request gate (`actions/report` and CI logs),
`--against` in local loops, box-ab friction #4, and root-causes C1, all of them recorded before this
benchmark. On the planted branch, the new binary's whole first gate output is 2,892 characters.

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

## 6. Protocol

- **Rounds (P3).** Each round freezes one commit of this repository (`frozenAt`) for Toolbox, the
  harness and both arms. The Toolbox binaries are built from it. A Toolbox change made after round 1
  starts round 2, in which both arms rerun.
- **Runs (P2).** Three fresh runs per arm per round (five in round 3), each in its own workspace
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
- zizmor 1.32.0 `--format github` and `--format sarif`; actionlint 1.7.12 `-oneline` and
  `-format '{{json .}}'`: each tool's `--help`, run in this repository's CI tool pins.
- Protocol 2: [D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md).
