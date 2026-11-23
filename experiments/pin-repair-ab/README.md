# pin-repair-ab: repairing pins the repository already knows, Toolbox vs reviewdog

**Status: complete. Toolbox beats reviewdog on this task in round 2 (frozen at `6303a4d`), on
read.** Toolbox was correct in every run and read a median of 5,794 characters against
reviewdog's 18,300 (N7). Calls and gate read were inconclusive. Round 1 (frozen at `25079a6`)
showed no win under its rule, with a measure that dropped some of reviewdog's results. Round 2
was registered with a measure that rebuilds them (§4.1). After round 1, only this status line,
§0, §4.1 and two lines of §7 (the `REPLAY` command) changed. The win is narrow: N7 holds by
1,073 characters over *W*, and it depends on the replay. The independent adjudication is in
[ADJUDICATION.md](ADJUDICATION.md).

This benchmark is about the box: a ratchet gate that reports what the repository already knows.
Its tools are zizmor and actionlint, the same in both arms.

**Question.** A pull request adds a workflow that writes its actions as tags (`actions/checkout@v6`)
where the rest of the repository pins them to commit SHAs. Does Toolbox's gate get a coding agent
to a correct pass for less effort than reviewdog over the same tools? Toolbox's gate names the pin
the repository already uses for each action. reviewdog's tools only say the action is unpinned.

**Why this question.** [gate-repair-ab](../gate-repair-ab/README.md) asked whether Toolbox's gate
beats reviewdog's on a repair of mostly local mistakes. Over five rounds it never did. Every run
in every arm printed the branch's large changed workflows before editing, which set a floor and a
noise that the gate's output could not move (L19 in
[dx-benchmarks.md](../../docs/design/dx-benchmarks.md)). Its one cross-file fix, pin hints for two
actions, saved reviewdog's agents a lookup worth less than that noise. Here most of the fixes need
knowledge from other files, and the planted file is new and small, so printing it costs little.

**What we'll do with the answer.** If a hypothesis holds, the pin hint is evidence for the
claim that the ratchet catches regressions without becoming a chore (R4 in
[repo-value.md](../../docs/design/repo-value.md)): a gate that names the fix costs the agent less. If none holds, the hint is a convenience, not a
measured saving, and B10's next step moves to another question.

## 0. Results

### 0.2 Round 2: Toolbox beats reviewdog, on read

Ten fresh runs, five per arm, launched together at `6303a4d` with the prompts committed in
`1465d7d`, and collected with `REPLAY` set, as registered in §4.1. Every run said DONE, and the
scorer marked all ten correct. Per-run numbers are in [results/r2.tsv](results/r2.tsv), and each
run's diff, score and replay report are in `results/r2/`.

| Measure | Arm A (Toolbox): median, range | Arm B (reviewdog): median, range | *W* | Label |
|---|---|---|---|---|
| Correct | 5 of 5 | 5 of 5 | | **N5 holds** |
| Calls | 2, 0 | 5, 7 | 7 | N6 inconclusive |
| Read | 5,794, 123 | 18,300, 11,433 | 11,433 | **N7 holds** (gap 12,506) |
| Gate read | 5,794, 123 | 8,299, 4,849 | 4,849 | N8 inconclusive |

**Toolbox beats reviewdog on this task in round 2, under the registered rule**: N5 and N7 hold,
and none of N6 to N8 fails. The designer predicted that outcome. In this task, on the registered
measure, an agent with Toolbox's gate reads about a third of what an agent with reviewdog's reads
(0.32). On the recorded data alone, without the replay, it is 0.40. Every arm A run took
two calls and read 5,705 to 5,828 characters.

**What the runs did.**

- **Arm A.** Every run did the same two things. It printed `weekly.yaml` and ran the gate in one
  command. Then it rewrote the ten `uses:` lines from the hints, with one `sed` or a short Python
  script, and reran the gate in the same command. No run looked a pin up.
- **Arm B.** Every run printed the gate script and `weekly.yaml`, ran the gate, and then looked up
  the pins in one or two greps. Grep output ranged from 521 to 13,522 characters. Then it made
  the edit and checked the result. reviewdog-2 made the edit as seven `search_replace` calls,
  which is most of its eleven (L18). No arm B run picked a wrong pin.

**The replay (§4.1) decides N7.** The runtime again recorded no result for four terminal commands,
all in arm B: reviewdog-2's gate run, reviewdog-3's gate run and pin lookup, and reviewdog-5's
gate run.

- **What the replay did.** It rebuilt all four. Every recorded command result it checked matched
  its rebuilt length: 9 checks over three runs. It stopped at reviewdog-2's first
  `search_replace`, after its missing result.
- **The agents did receive these results.** Three runs over the two rounds (round 1's
  reviewdog-3 and -5, round 2's reviewdog-3) wrote all seven correct SHAs right after a lookup
  whose result the transcript lacks. No recorded result before it held any of them.
- **Why they went missing.** Every dropped result came from a command that exited non-zero: 7 of
  7, against 55 of 55 recorded results from commands that exited 0 (ADJUDICATION.md). Four ended
  in a `cat` of a zizmor config that doesn't exist, and three in a direct `zizmor --format plain`
  with findings. Every arm A command ended in one that exits 0, so arm A was never at risk. The
  mechanism is inferred from these 62 calls, not from the runtime's code.
- **Without the replay,** arm B's read would be a median of 14,638 with a range of 13,614. N7
  would then be inconclusive.
- **In both rounds,** all seven missing results were in arm B: four gate runs, three of them
  chained with zizmor's plain output, and three greps over the workflows' `uses:` lines.

**How narrow it is.** N7 holds by 1,073 characters against *W* = 11,433, and *W* is all arm B's
spread. reviewdog-2's editing style sets arm B's median: had it edited with one `sed` instead of
seven `search_replace` calls, N7 would be inconclusive and N6 would hold instead. Without
reviewdog-3, no hypothesis holds. Read with the adjudication's threats 1 to 5: this is the
seventh round, over two benchmarks, to ask whether Toolbox beats reviewdog, and the first to show
it, on a task built where the hint should help.

**Friction, triaged (§6).** None changes a label.

| Arm | Friction | Triage |
|---|---|---|
| B | Three round 2 runs (reviewdog-2, -3, -4) ran zizmor or actionlint by hand after their gate passed. reviewdog-4 noted that "the efm parsing might have dropped a finding". Two runs per round also ran `zizmor --format plain` beside their first gate run, about 3,650 characters each | Rival friction, unfixed: reviewdog's pass line doesn't say how many raw findings it parsed. A next round's gate should print that count (P4) |
| A | On a pass, the gate prints "baseline: no regressions, but .toolbox/results.json is out of date: run with --update and commit it", which the prompt forbids. Five runs over the two rounds deliberated over it | Product friction, unfixed: a passing ratchet should not tell a pull request's author to rewrite the baseline. Candidate for a decision |
| A | toolbox-3, in both rounds, paused over `maintainer-ci-security · …weekly.yaml: new (none)` | Product friction, unfixed: a `new (none)` line on a pass reads like a failure |

**§2's figures, corrected.** §2 says the gates print 1,827 and 3,728 characters. Those are bytes.
In characters, the measure's unit, they are 1,800 and 3,688. The 96 characters on stderr are
reviewdog's failure line, not zizmor's summary. §4 estimates a wrapper of about 400 characters a
call: it is 313. No label changes.

**Measured against round 1.** Round 1's labels stand, and the rounds are not pooled. Round 2's arm
A was tighter than round 1's: no run grepped to check the hints, as toolbox-2 did in round 1.
With the same replay, round 1 would have shown the same win on read by a narrower margin (§0.1).

### 0.1 Round 1: no win under the registered rule

Ten fresh runs, five per arm, launched together at `25079a6` with the prompts committed in
`2fd3b8e`. Every run said DONE, and the scorer marked all ten correct. Per-run numbers are in
[results/r1.tsv](results/r1.tsv), and each run's diff and score are in `results/r1/`.

| Measure | Arm A (Toolbox): median, range | Arm B (reviewdog): median, range | *W* | Label |
|---|---|---|---|---|
| Correct | 5 of 5 | 5 of 5 | | **N1 holds** |
| Calls | 3, 2 | 4, 1 | 2 | N2 inconclusive |
| Read | 6,054, 5,614 | 11,605, 10,091 | 10,091 | N3 inconclusive |
| Gate read | 5,794, 2,358 | 7,850, 6,547 | 6,547 | N4 inconclusive |

**Toolbox does not beat reviewdog on this task in round 1.** N1 holds and none of N2 to N4 fails,
but none holds either. The designer predicted that N3 would hold.

**What the runs did.** Both arms took the same path. In one or two calls they printed
`weekly.yaml` and ran the gate. Arm B also grepped the other workflows for SHA pins. Then they
rewrote the ten `uses:` lines with one `sed` (two for toolbox-3, whose first `sed` failed on its
`#` delimiter), and the same command reran the gate.

- Every arm B run found the pins with that one grep and picked no wrong pin. reviewdog-1 and -4
  chained it onto the gate run; reviewdog-2, -3 and -5 made it its own call.
- Four of five arm A runs copied the hints without looking anything up. toolbox-2 grepped every
  `uses:` line anyway (5,214 characters), which makes arm A's range.
- Arm A's gate read is higher than the designer estimated, because three runs printed
  `weekly.yaml` in the same command as the gate (L17: a chained command's whole result counts).

**The measure undercounts arm B.** The runtime recorded no result for three terminal commands, all
in arm B: reviewdog-5's gate run (chained with zizmor) and its pin lookup, and reviewdog-3's pin
lookup. The registered measure imputes only `read_file` results, so these count as nothing.
reviewdog-5's read, 5,064, is the lowest of all ten runs, and it alone makes arm B's range.

**A sensitivity check, not the label.** Every missing command ran before any edit, so its output
can be rebuilt by rerunning it in a fresh planted workspace at the same path. Done that way, the
five recorded results of the same calls in reviewdog-1, -2, -3 and -5 come out byte for byte the
same length as recorded.

| Run | Missing result, rebuilt | Read: recorded, rebuilt | Gate read: recorded, rebuilt |
|---|---|---|---|
| reviewdog-3 | pin lookup, 8,795 | 11,579, 20,374 | 7,850, 7,850 |
| reviewdog-5 | gate run 7,840; pin lookup 5,592 | 5,064, 18,496 | 1,335, 9,175 |

With the rebuilt results, arm B's read is a median of 15,155 with a range of 8,769. N3 would then
hold by 332 characters (9,101 against *W* 8,769), and gate read would stay inconclusive (7,876,
range 4,283, against 5,794, range 2,358). The rebuild was not registered, so it doesn't change
round 1's labels. It does show that the result turns on the measure dropping results, not only
on the agents.

**What follows (L3, close known friction before running).** The measure replays a missing
command's result for every arm, and keeps the replay only when it reproduces each recorded result
of the same run. Round 2 is registered with that measure, at a new frozen commit, with fresh runs
and nothing else changed.

## 1. Arms

| | Arm A: Toolbox | Arm B: reviewdog |
|---|---|---|
| Gate | `toolbox report .toolbox --skip github-actions --baseline .toolbox/results.json` | `scripts/quality-gate.sh` |
| What it compares | Every rule outcome against the committed baseline (`.toolbox/results.json`) | Every finding against the lines `git diff main` added |
| Portal or script | [harness/toolbox.yaml](harness/toolbox.yaml): the repo-portal starter plus `packages/github-actions`, vendored | [harness/quality-gate.sh](harness/quality-gate.sh): zizmor and actionlint piped into reviewdog 0.21.0 |
| Tools | zizmor 1.32.0 (`--offline`), actionlint 1.7.12 (`-shellcheck= -pyflakes=`) | The same |

Both gates are gate-repair-ab's as they stood after its round 5, including the rival's two fixes
(P4): reviewdog's gate prints `quality gate: passed`, and it reads zizmor's and actionlint's
formats that quote the source.

**Strongest rival.** reviewdog with `-filter-mode=added` is the usual way to gate a pull request
only on what it adds. No tool in the rival's box pins offline: zizmor 1.32.0's `--fix=all` changes
nine of the fork's workflows for other audits and leaves every unpinned `uses:` alone (checked on
the planted branch). pinact and ratchet resolve tags through the GitHub API, which the runs don't
have (§5, limit 1).

## 2. Fork and planted pull request

The fork is [loganvolkers/openfga](https://github.com/loganvolkers/openfga) at `7d89a566`, as in
gate-repair-ab: 11 workflows pinning 30 actions by SHA, each with its version in a comment.
[harness/setup.sh](harness/setup.sh) commits the fork plus the arm's gate as `main`, records arm
A's baseline, then [harness/plant.py](harness/plant.py) commits the pull request on branch `pr`.

The pull request adds `.github/workflows/weekly.yaml` (75 lines): three jobs (golangci-lint,
govulncheck, tests with coverage) that use seven actions as tags, at ten sites. Every one of them
is pinned elsewhere in the repository:

| Ref in the PR | Sites | Pinned elsewhere as |
|---|---|---|
| `actions/checkout@v6` | 3 | `de0fac2e…` `# v6.0.2` (22 sites) |
| `actions/setup-go@v6` | 2 | `4a360112…` `# v6.4.0` (13) |
| `golangci/golangci-lint-action@v9` | 1 | `1e7e51e7…` `# v9.2.0` (1) |
| `golang/govulncheck-action@v1` | 1 | `b625fbe0…` `# v1.0.4` (1) |
| `actions/cache@v5` | 1 | `27d5ce7f…` `# v5.0.5` (6, and `actions/cache/restore` and `/save` at the same SHA) |
| `codecov/codecov-action@v6` | 1 | `57e3a136…` `# v6.0.0` (6) |
| `actions/upload-artifact@v7` | 1 | `043fb46d…` `# v7.0.1` (1) |

The workflow is otherwise clean: it sets `permissions`, `timeout-minutes` and
`persist-credentials: false`. zizmor reports ten `unpinned-uses` findings on it and nothing else;
actionlint reports nothing.

On the planted branch:

- **Arm A's gate** prints 1,827 characters. Each action gets one line, for example
  `pinned-everywhere: + actions/cache@v5, pinned elsewhere as actions/cache@27d5ce7f… # v5.0.5`.
- **Arm B's gate** prints 3,728 characters on stdout: ten `unpinned-uses` findings, each quoting its
  line. zizmor's summary adds 96 characters on stderr.

[harness/reference_fix.py](harness/reference_fix.py) is one correct repair. On it both gates pass
and the scorer says correct; on the planted branch both fail.

**A product fix found while building this, before any run.** The first prototype's hint for
`actions/cache@v5` named `actions/cache/restore@27d5…`, another action in the same repository.
Copied as written, it would replace a cache that restores and saves with one that only restores.
[D01M84X4JDHXS98K89KN4AB0RCD](../../docs/decisions/D01M84X4JDHXS98K89KN4AB0RCD.md) (commit
`8a76f24`, no harness file) writes the hint on the ref's own path, at the version its tag names.
The arms run with it. No run had been launched, so no run's data shaped it.

## 3. Scoring

[harness/score.py](harness/score.py) is hidden from the agents and reads the tools directly, not
either gate. A run is **correct** only if all of these hold:

| Check | What it requires |
|---|---|
| `no-new-findings` | For every (tool, file, rule), no more findings in the working tree than on `main` |
| `intent-weekly` | `weekly.yaml` keeps the planted name, triggers and permissions, and every planted job and step by name. Each step is unchanged except for its `uses:` ref, which still names the same action path (`actions/cache`, not `actions/cache/restore`) |
| `pins-known` | Each of those `uses:` is pinned to a full SHA that `main` already pins that action's repository to: offline, the only SHAs known to be real commits |
| `gate-untouched` | The gate's files match `main` (A: `.toolbox/`; B: `scripts/quality-gate.sh`; both: `.github/actionlint.yaml`), and no zizmor config was added |
| `no-suppressions` | No `zizmor: ignore` comment was added |
| `gate-passes` | The arm's own gate exits 0 |

Checked before registration: the planted branch fails `no-new-findings`, `pins-known` and
`gate-passes` in both arms. The reference fix passes every check in both. A repair that pins the
cache to `actions/cache/restore@27d5…` fails `intent-weekly`, and an invented SHA fails
`pins-known`.

## 4. Measures and the bar (pre-registered)

**Measures (P6)**, from each run's runtime transcript by [harness/count.py](harness/count.py),
unchanged from gate-repair-ab's round 5, where the runtime reported all three for both arms:

- **calls**: tool calls the agent made;
- **read**: characters of tool results the agent received; a `read_file` with no recorded result
  counts as that file on `pr`;
- **gate read**: characters of the results of the agent's gate runs, as `count.py` matches them.

Wall time, gate runs and calls by tool are reported, not in the bar.

**Runs (P2).** Five fresh runs per arm, ten in all, launched together. Workspaces are
`/tmp/pin-repair-ab/r1/toolbox-1…5` and `reviewdog-1…5`. Medians and ranges are over an arm's five
runs. *W* is the wider of the two arms' ranges. A cost hypothesis holds or fails only if the
medians differ by more than *W*; otherwise it is inconclusive.

| Id | Hypothesis | Holds | Fails |
|---|---|---|---|
| N1 | Arm A is correct in every run | 5 of 5 correct | Any run incorrect |
| N2 | With Toolbox's gate, a repair takes fewer calls than with reviewdog's | med(B) − med(A) > *W*, on calls | med(A) − med(B) > *W*, on calls |
| N3 | With Toolbox's gate, the agent reads less than with reviewdog's | As N2, on read | As N2, on read |
| N4 | With Toolbox's gate, the agent reads less gate output than with reviewdog's | As N2, on gate read | As N2, on gate read |

**Toolbox beats reviewdog on this task** only if all of these hold:

- N1 holds;
- at least one of N2, N3 and N4 holds;
- none of N2, N3 and N4 fails.

A run of arm B that is incorrect is reported and changes no label. This round is run once. If it
doesn't show a win, the result is published as it is.

**Before the run, the designer reads the odds as follows.** The estimates use gate-repair-ab's
transcripts: each call's result carries about 400 characters of wrapper.

- **Calls (N2) is likely inconclusive, and could fail.** Arm A's path is a gate run, a look at the
  file, an edit and a rerun: 4 calls, or 3 if the edit and rerun share a command. Arm B adds a
  lookup of seven SHAs, one or two calls. But how agents edit varies more than that (L18): seven
  search-and-replace edits instead of one `sed` or script adds six calls in either arm.
- **Read (N3) is the likeliest to hold.**
  - Arm A: about 2,300 characters on the first gate run, 2,600 to print `weekly.yaml`, about
    1,000 on the edit and 500 on the rerun. That is about 6,500 in all, plus whatever it greps to
    check the hints.
  - Arm B: about 4,100 on the first gate run, or about 5,500 with `cat scripts/quality-gate.sh`
    in the same command, as most of gate-repair-ab's B runs did. Then the same print, edit and
    rerun, and a lookup of seven SHAs. One `grep -rh uses: .github/workflows` is about 9,000
    characters, and per-action greps are 2,000 to 5,000. That is about 12,000 to 20,000 in all.
  - The gap is about 6,000 to 12,000. It exceeds *W* unless an arm's runs spread as widely as
    gate-repair-ab's did, where one run printed two whole workflows.
- **Gate read (N4) is about even.** Arm A's gate read is about 2,800, and its runs differ only in
  what they chain onto a gate run. Arm B's is about 4,600 to 6,000, depending on whether the
  script is printed with the first run. The gap is about 2,000 to 3,000. In gate-repair-ab, arm
  B's range on this measure was 1,913 in round 4 and 4,342 in round 5.
- **The designer's prediction:** N1 holds; N2 is inconclusive; N3 holds; N4 is inconclusive.

### 4.1 Round 2 (registered after round 1, before its runs)

Round 2 asks round 1's question again, with one change: the measure. Everything else is as
registered for round 1: the arms, the fork, the planted pull request, the prompts template, the
scorer, the Toolbox binaries (no Rust changed after `8a76f24`), five fresh runs per arm, P2 and
*W*.

**The measure change (P6).** `collect.py` runs with `REPLAY` set. A `run_terminal_cmd` whose result
the transcript lacks counts as its result rebuilt by [harness/replay.py](harness/replay.py). The
replay runs the run's commands in order in a fresh workspace at the same path. It stops at the
first call it can't redo, such as a `search_replace`, whose arguments the transcript doesn't keep.
It is kept only if every recorded command result it reaches has the same length when rebuilt.
A rebuilt gate run counts in gate read too. On round 1 it rebuilt reviewdog-3's and reviewdog-5's
missing results, matching all five recorded results it checked. It applies to every arm; in round
1 all three missing results were in arm B.

| Id | Hypothesis | Holds | Fails |
|---|---|---|---|
| N5 | Arm A is correct in every run | 5 of 5 correct | Any run incorrect |
| N6 | As N2 (calls) | med(B) − med(A) > *W* | med(A) − med(B) > *W* |
| N7 | As N3 (read) | As N6, on read | As N6, on read |
| N8 | As N4 (gate read) | As N6, on gate read | As N6, on gate read |

**Toolbox beats reviewdog on this task** if, in round 2, N5 holds, at least one of N6 to N8 holds,
and none of them fails. Round 1's labels stand as they are, and round 2 is not pooled with it. If
round 2 shows no win, the result is published as it is, and the task is not run a third time.

**The designer's odds, after round 1.**

- N5 is likely to hold: all ten round 1 runs were correct.
- N6 is likely inconclusive: both arms' runs took two to four calls.
- N7 is about even. With the replay, round 1's read gap cleared *W* by 332 characters out of
  9,101, and one arm A run that greps anyway sets arm A's range.
- N8 is likely inconclusive: round 1's gap was 2,082 against a *W* of 4,283 even with the replay.
- The designer's prediction: N5 holds, N7 holds, N6 and N8 are inconclusive. That is a win, at
  odds the designer puts a little under even.

**Disclosed.** The measure change was prompted by round 1's data. Round 1 showed it helps arm A
there, because the missing results were arm B's. Round 2 is registered and frozen before its runs
are launched, and its missing results may fall in either arm.

## 5. Known limits, before the run

1. **No network.** With network, reviewdog's agents could resolve tags with `git ls-remote`, or a
   team could run pinact or ratchet in CI. Those pin to whatever the tag points at now, which
   need not be the version the repository pins elsewhere. This benchmark measures the offline
   repair only.
2. **One repository and one planted pull request.** The result says nothing about repositories
   that pin little, or about pull requests whose fixes are mostly local (gate-repair-ab's).
3. **The designer chose the task knowing Toolbox's strength.** That is the point of the question
   (L19), but the task is not a sample of real pull requests. The share of unpinned-action fixes
   in real pull requests is not measured here.
4. **Designer and product author are the same agent**, and the hint fix above was made while
   building this benchmark. The adjudicator audits it (P5).
5. **No A⁻ arm.** The hint fix is not attributed separately. Without it, arm A's cache hint
   would have been wrong, so a correct arm A partly depends on that fix.

## 6. Protocol

- **Frozen commit (P3).** The commit that adds this README freezes Toolbox, the harness, the
  prompts and both arms. The Toolbox binaries are built from it.
- **Runs (P2).** Five fresh runs per arm, each in its own workspace with its own copies of the
  tools. Each run is a fresh local subagent with the inherited model and only the rendered prompt.
  No run is dropped for its result.
- **Prompts (P1).** [prompts/toolbox.md](prompts/toolbox.md) and
  [prompts/reviewdog.md](prompts/reviewdog.md) differ only in the gate command. The rendered prompts
  are committed under `prompts/rendered/r1/` before the first launch.
- **Symmetry (P4).** Both gates carry gate-repair-ab's fixes. Rival friction this round shows is
  triaged like Toolbox's.
- **Fit (P5).** The adjudicator audits the hint fix (`fitAudit`).
- **Adjudication (P7).** A fresh agent that designed no part of this and changed nothing applies
  the bar and writes `ADJUDICATION.md`, with holds, fails, inconclusive or void per hypothesis.
- **Run conditions.** Every run sees the same workspace rules the runtime injects. The prompts ask
  for no network; the fix needs none.

## 7. Running it

```bash
export PATH=<dir with zizmor 1.32.0, actionlint 1.7.12 and reviewdog 0.21.0>:$PATH
cargo build                                       # at the frozen commit
RUNS=5 experiments/pin-repair-ab/harness/round.sh 1 <openfga export at 7d89a566> "$PWD" "$PWD/target/debug"
# commit prompts/rendered/r1/; launch each rendered prompt as a fresh subagent; fetch transcripts; then:
experiments/pin-repair-ab/harness/collect.py 1 <transcripts dir> <run>=<bcId> …
# from round 2, rebuild dropped command results (§4.1):
REPLAY=<openfga export>:"$PWD":"$PWD/target/debug" experiments/pin-repair-ab/harness/collect.py 2 <transcripts dir> <run>=<bcId> …
```

## Sources

- reviewdog 0.21.0, `-filter-mode=added` and `-efm`: `reviewdog -h` and
  [github.com/reviewdog/reviewdog](https://github.com/reviewdog/reviewdog).
- zizmor 1.32.0 `--format plain`, `--fix` and the `unpinned-uses` audit:
  `zizmor --help` and [docs.zizmor.sh/audits](https://docs.zizmor.sh/audits/#unpinned-uses).
- actionlint 1.7.12: `actionlint -h`.
- pinact and ratchet resolve tags through the GitHub API:
  [github.com/suzuki-shunsuke/pinact](https://github.com/suzuki-shunsuke/pinact) and
  [github.com/sethvargo/ratchet](https://github.com/sethvargo/ratchet) (READMEs; secondary for
  this benchmark, not run).
- Measures and harness: [gate-repair-ab](../gate-repair-ab/README.md) §5 and its
  [ADJUDICATION.md](../gate-repair-ab/ADJUDICATION.md).
