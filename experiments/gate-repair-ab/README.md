# Gate Repair A/B: Fixing a Pull Request the Gate Failed

**Question:** when a coding agent's pull request fails the repository's quality gate, does Toolbox's
ratchet get it to a correct pass for less effort than reviewdog, which runs the same tools and
reports only the findings the branch added?

**Status: registered under protocol 2 (D01M83RDWQ5EVKY5SFHDWAD5HFH); round 1 not yet run.** The bar
is in §5 and was committed before the first scored launch. Results go in §0.

This is about the box ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)): both arms
carry the same tools, zizmor and actionlint, at the same pins, with the same flags and the same
`.github/actionlint.yaml`. What differs is how the gate tells the agent what is wrong. The measure is
of R4 in [repo-value.md](../../docs/design/repo-value.md) ("without becoming a chore"). The friction
it targets was left open by the box A/B: *the gate's regression lists every finding in the file and
doesn't mark the new ones; the baseline keeps only counts* ([box-ab](../box-ab/README.md) §0.3 #4,
C1 in [root-causes.md](../../docs/design/root-causes.md)).

## 0. Results

Not run yet.

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

## 6. Protocol

- **Rounds (P3).** Each round freezes one commit of this repository (`frozenAt`) for Toolbox, the
  harness and both arms. The Toolbox binaries are built from it. A Toolbox change made after round 1
  starts round 2, in which both arms rerun.
- **Runs (P2).** Three fresh runs per arm per round, each in its own workspace
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
# launch the six rendered prompts as fresh subagents; then, per run:
experiments/gate-repair-ab/harness/score.py <arm> /tmp/gate-repair-ab/r1/<arm>-<n>
experiments/gate-repair-ab/harness/count.py <arm> <transcript.json>
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
