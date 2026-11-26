# pin-repair-ab: independent adjudication (2026-11-23)

**Outcome against the bar in [README.md](./README.md) §4 (round 1) and §4.1 (round 2):**

- **Round 1: Seikika does not beat reviewdog under §4.** N1 holds. N2 (calls), N3 (read) and N4
  (gate read) are inconclusive, and none fails. The README reports these labels as registered.
  N3 and N4 rest on incomplete data: three of arm B's results are missing, which biases them
  against Seikika, so they are not evidence either way.
- **Round 2: Seikika beats reviewdog on this task under §4.1, narrowly.** N5 holds. N7 (read)
  **holds**: reviewdog's median exceeds Seikika's by 12,506 characters, against a *W* of 11,433.
  N6 (calls) and N8 (gate read) are inconclusive, and none fails. Every condition of §4.1's rule
  is met.
- **No label is void, no run was excluded or rerun, and all 20 runs are correct.** The replay
  measure is sound, registered before round 2, and applied to every run. The product change
  `8a76f24` fits P5.

**Overall verdict: the registered result stands, and it is narrow.** On a planted pull request in
which every fix is a pin the repository already holds, an agent with Seikika's gate read a median
of 5,794 characters of tool results, against 18,300 for an agent with reviewdog's. The difference
comes from the pin hint: arm A copied the pins from the gate's output, and arm B looked them up.
That is all it shows. Read is the only measure that separates the arms. Its margin over *W* is
1,073 characters. It depends on the replay, which was introduced after round 1 failed to show a
win. And it would not hold if reviewdog-3's run were left out. It does not show that Seikika
beats reviewdog in general, or on a second repository or a real pull request. It does not show
that the box adds anything beyond the hint, since any gate that printed the same hint would
likely do as well.

The adjudicator is a cloud agent that did not design the experiment, build an arm, or change the
product ([D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md), P7).
It did the following:

- recomputed every median, range and *W* from [results/r1.tsv](results/r1.tsv) and
  [results/r2.tsv](results/r2.tsv);
- recounted all 20 transcripts with its own code rather than `count.py`;
- re-ran `replay.py` on round 2's reviewdog-3;
- rebuilt every replayable terminal command of all 20 runs in scratch workspaces, comparing
  content as well as length;
- rescored copies of all 20 workspaces;
- reproduced both gates' output on the planted branch;
- demonstrated the product change on this repository.

It changed no file but this one.

## Per hypothesis

The range of an arm is its maximum minus its minimum over five runs, and *W* is the wider of the
two arms' ranges. A label holds or fails only if the medians differ by more than *W*. A is Seikika
and B is reviewdog. Read and gate read are characters of tool results, as `count.py` measures
them, including the runtime's wrapper of 313 characters per terminal result.

### Round 1 (§4, frozen at `25079a6`)

| Hypothesis | Label | Numbers |
|---|---|---|
| N1 A correct every run | **holds** | 5 of 5 correct. B was also 5 of 5 |
| N2 A fewer calls than B | **inconclusive** | A 3, 4, 3, 2, 2 (median 3, range 2); B 3, 4, 4, 3, 4 (median 4, range 1). B − A = 1 ≤ *W* = 2 |
| N3 A reads less than B | **inconclusive** | A 6,054; 11,408; 6,179; 5,794; 5,904 (median 6,054, range 5,614); B 11,611; 15,155; 11,579; 11,605; 5,064 (median 11,605, range 10,091). B − A = 5,551 ≤ *W* = 10,091 |
| N4 A reads less gate output than B | **inconclusive** | A 3,821; 3,961; 6,179; 5,794; 5,904 (median 5,794, range 2,358); B 7,882; 4,892; 7,850; 7,876; 1,335 (median 7,850, range 6,547). B − A = 2,056 ≤ *W* = 6,547 |

Per-run values are in run order 1 to 5. The README's §0.1 table matches every figure. Its
sensitivity check also reproduces. With round 1's three missing results rebuilt (8,795 for
reviewdog-3; 7,840 and 5,592 for reviewdog-5), arm B's read is 11,611; 15,155; 20,374; 11,605;
18,496 (median 15,155, range 8,769). N3 would then hold by 9,101 − 8,769 = 332. Gate read would
stay inconclusive: B's median is 7,876 with a range of 4,283, so the gap is 2,082 ≤ 4,283. The
rebuild was not registered for round 1, and the README keeps round 1's labels as registered. That
is correct.

### Round 2 (§4.1, frozen at `6303a4d`)

| Hypothesis | Label | Numbers |
|---|---|---|
| N5 A correct every run | **holds** | 5 of 5 correct. B was also 5 of 5 |
| N6 A fewer calls than B | **inconclusive** | A 2, 2, 2, 2, 2 (median 2, range 0); B 4, 11, 5, 5, 5 (median 5, range 7). B − A = 3 ≤ *W* = 7 |
| N7 A reads less than B | **holds** | A 5,828; 5,705; 5,815; 5,794; 5,794 (median 5,794, range 123); B 14,638; 18,300; 22,240; 14,818; 26,071 (median 18,300, range 11,433). B − A = 12,506 > *W* = 11,433 |
| N8 A reads less gate output than B | **inconclusive** | A as N7 (every A call ran the gate); B 9,990; 5,141; 8,299; 5,299; 8,299 (median 8,299, range 4,849). B − A = 2,505 ≤ *W* = 4,849 |

The README's §0.2 table matches every figure. The designer's predictions held in round 2: N5
holds, N7 holds, and N6 and N8 are inconclusive. In round 1, three of four held. N3 was predicted
to hold and is inconclusive.

## Does Seikika beat reviewdog?

| Rule | Requires | Result |
|---|---|---|
| §4 (round 1) | N1 holds, at least one of N2–N4 holds, none fails | **Not shown.** N1 holds; N2, N3 and N4 are inconclusive. The rule fails on its second condition |
| §4.1 (round 2) | N5 holds, at least one of N6–N8 holds, none fails | **Shown.** N5 and N7 hold; N6 and N8 are inconclusive, not failed |

**How robust the round 2 verdict is.** Only the first row is the registered result. The rest are
checks, and they change no label.

| Variant | N6 (calls) | N7 (read) | N8 (gate read) | Win under §4.1? |
|---|---|---|---|---|
| As registered | 3 ≤ 7 | 12,506 > 11,433, holds | 2,505 ≤ 4,849 | Yes |
| Without the replay (round 1's measure) | 3 ≤ 7 | B median 14,638, range 13,614: 8,844 ≤ 13,614 | B median 508, range 9,537: A − B = 5,286 ≤ 9,537 | No |
| Output characters only, without the runtime's wrapper | unchanged | A 5,050 (range 120), B 16,574 (range 11,312): 11,524 > 11,312, holds by 212 | not computed | Yes |
| The stderr of the three replayed gate commands added (96, 190 and 190: reviewdog's failure line, plus zizmor's log lines where it was run directly) | unchanged | 12,602 > 11,623, holds | unchanged label | Yes |
| reviewdog-2 edits with one `sed` and reruns its gate, as the other B runs did | B 4, 5, 5, 5, 5 (range 1): 3 > 1, **holds** | B median 14,818, range 13,608: 9,024 ≤ 13,608 | 2,505 ≤ 4,691 | Yes, on calls instead of read |
| reviewdog-3 left out (four B runs) | B median 5, range 7: inconclusive | B median 16,559: 10,765 ≤ 11,433 | not computed | No |

The verdict depends on the replay. The replay is the more accurate measure: the agents received
the results it rebuilds (see *The replay*). The verdict survives the wrapper and the stderr the
transcript omits. It also survives reviewdog-2's editing style, though that changes which
hypothesis carries it (L18). It does not survive leaving out reviewdog-3, which is one of five
runs. Leaving out any other single B run keeps N7. Leave-one-out is not part of the rule. It is
reported here only to show how much rides on each run.

## The measure, recounted

My own counter reads each transcript's `role: tool` messages. It counts the length of each
recorded `tool_result` as the runtime stores it, and matches gate runs with its own pattern. Its
figures equal `r1.tsv` and `r2.tsv` for all 20 runs: calls, recorded read (`readRecorded`), gate
runs, and recorded gate read. For round 2, recorded gate read plus each replayed gate result
equals the TSV:

- reviewdog-2: 508 + 4,633 = 5,141;
- reviewdog-3: 453 + 7,846 = 8,299;
- reviewdog-5: 453 + 7,846 = 8,299.

`count.py`'s gate pattern for B excludes `cat scripts/quality-gate.sh`. Every B run printed the
script in a command that did not run the gate, so the exclusion changes nothing here.

## The replay (§4.1, `replay.py`)

| Check | Finding |
|---|---|
| Applied to both arms | **Yes.** `collect.py` calls `replay.py` for every run, whatever its arm, and `results/r2/` holds a replay report for all ten. In arm A it rebuilt nothing because no A result was missing, so the symmetry is real but went unused |
| Changed nothing but the measure | **Yes.** `git diff 25079a6 6303a4d -- experiments/pin-repair-ab/harness` touches only `collect.py`, `count.py` and the new `replay.py`. The README gained §4.1, the status line and two lines in §7. The prompt templates, `setup.sh`, `plant.py`, `score.py`, `quality-gate.sh` and `seikika.yaml` are unchanged. The round 2 rendered prompts equal round 1's apart from the round in the path. Both rounds ran byte-identical Seikika binaries. `collect.py` scores a run and takes its diff before replaying it. When I re-ran the replay on round 2's reviewdog-3, the workspace's content fingerprint was the same before and after |
| Reproduces its committed output | **Yes.** `replay.py` on round 2's reviewdog-3 printed `{"1": 7846, "2": 9783}`, valid, 3 checks, no mismatches, as committed |
| Validation is sound | **Sound in kind, but narrower than it reads.** It compares lengths, not content. Its 9 checks in round 2 were all commands that print files, greps, or a gate that passes. None was a failing gate run or a direct `zizmor --format plain`, which are the kinds it rebuilds. I closed that gap from outside. In fresh planted workspaces at paths of equal length, I ran every terminal command before each run's first non-terminal edit, for all 20 runs: 62 commands. All 55 recorded results came back the same length, and with the same content apart from commit hashes in `git log` output. That includes five recorded *failing* reviewdog gate runs (reviewdog-1 and -4 of round 2, and reviewdog-1, -2 and -4 of round 1), and all ten of arm A's failing first gate runs. The 7 missing results rebuilt to exactly the replay's figures and §0.1's |
| Why results go missing | **Every missing result came from a command that exited non-zero, and every recorded one from a command that exited 0: 7 of 7, and 55 of 55.** Four of the dropped commands ended in a `cat` of a zizmor config file that doesn't exist (`cat zizmor.yml .github/zizmor.yml 2>/dev/null` or the like), exiting 1. Three ended in a direct `zizmor --format plain` run that has findings, exiting 14. Every arm A command in both rounds ended in a command that exits 0, usually `echo EXIT $?`. So the gaps were not random, and arm A was never at risk of them. Neither §0 nor §4.1 names this mechanism. It is inferred from these 62 calls, not from the runtime's code |
| The agents received the missing results | **Yes, on direct evidence.** Three runs (round 1 reviewdog-3 and -5, round 2 reviewdog-3) wrote all seven correct SHAs in their next command. No recorded result before that command, and not the prompt, contains any of the seven SHAs, so they came from the dropped lookups. After a dropped gate run, the agents' notes describe its content: reviewdog-2's "The only failures are unpinned actions", reviewdog-5's "Every finding is an unpinned action", and round 1 reviewdog-5's "I need commit SHAs for these actions". §0.2 cites only reviewdog-3 |
| What the rebuild may miss | The transcript keeps stdout only (L17). A failing reviewdog gate also prints 96 characters on stderr, and the rebuilt figure leaves them out. That undercounts B slightly, against arm A. The rebuilt `exitCode` is the real one (`14` or `1`), and the runtime's recorded value for a dropped call is unknown: at most one character per call |
| Disclosure in §4.1 | **Adequate on what matters, incomplete on one point.** It says the change was prompted by round 1's data and that it helped arm A there. It was registered and frozen before round 2's workspaces were built (`6303a4d` at 19:24:25; workspaces 19:24:25–19:24:56; prompts `1465d7d` at 19:25:06; launches 19:25:50). It also commits to no third run. "Its missing results may fall in either arm" is true in principle. Given the mechanism, though, it was predictable that B would lose more results than A, because of how each arm's agents chained commands. That is a reason to trust the correction, not to doubt it: the replay removes a bias against A that is tied to command style, and nothing in it adds characters an agent did not receive. But round 2 is still a second try at a win that round 1 did not show (threat 1) |

## Correctness

| Check | Finding |
|---|---|
| Every run correct | **Yes, 20 of 20.** Copies of all 20 workspaces rescore `correct: true` with `score.py`, matching the committed JSON |
| Every diff the same | **Yes.** All 20 workspace diffs (`git diff main -- .github`) are byte-identical to each other, to the committed `results/r*/<run>.diff`, and to the diff `reference_fix.py` produces. Every run wrote the seven SHAs with their version comments |
| Scorer checks behave as registered | **Yes.** On the planted branch, both arms fail `no-new-findings` (`unpinned-uses` 0 → 10), `pins-known` (10 refs) and `gate-passes`. The reference fix passes every check in both arms. `actions/cache/restore@27d5…` fails `intent-weekly`. An invented SHA fails `pins-known`. A `# zizmor: ignore[unpinned-uses]` comment fails `no-suppressions` |
| How hard it was to be wrong | On `main`, each of the seven actions is pinned to exactly one SHA, so the task has one right answer per action. The only wrong pin among known SHAs is the cache's sub-action, which `intent-weekly` catches. No B run picked it. No A run in round 2 checked a hint before copying it, so arm A's correctness on this task is the hint's correctness (limit 5) |

## Protocol checks

| Rule | Verdict | Evidence |
|---|---|---|
| P1 committed prompts | **Pass** | All 20 launched prompts (each transcript's only user message) equal `prompts/rendered/r<n>/<run>.md` at `2fd3b8e` and `1465d7d` exactly. The runtime's `createdAtMs` puts the launches at 19:19:42.8–19:19:43.9 (round 1) and 19:25:50.8–19:25:51.6 (round 2), after the prompt commits (19:18:54 and 19:25:06). `prompts/seikika.md` and `prompts/reviewdog.md` differ only in the gate line. Rendered prompts also differ in the workspace path, which names the arm, as in gate-repair-ab |
| P2 repeated runs | **Pass** | 5 runs per arm per round. Since 15:35 UTC, the runtime lists exactly 20 agents named "pin-repair r1 …" or "pin-repair r2 …", plus this adjudicator. The others are unrelated sessions. No run was excluded or rerun. The TSVs report every run |
| P3 frozen rounds | **Pass** | In order: `8a76f24` (19:14:01), `25079a6` (19:18:08), round 1 workspaces (19:18:14–19:18:46), `2fd3b8e` (19:18:54), launches, results `c2d27fe` (19:22:43), `6303a4d` (19:24:25), round 2 workspaces (19:24:25–19:24:56), `1465d7d` (19:25:06), launches, results `60ab6e4` (19:27:34). `git diff 8a76f24 HEAD -- crates Cargo.toml Cargo.lock packages schema ui` is empty, and so is `git diff 6303a4d HEAD -- experiments/pin-repair-ab/harness`. The `seikika` binary is byte-identical in all 20 workspaces. Every workspace's `seikika-connector-workspace`, which computes the hints, equals `target/debug`'s. It was built at 19:13:38, 23 seconds before `8a76f24` was committed, and no `.rs` file is newer. In every workspace, `quality-gate.sh` and `.seikika/seikika.yaml` equal the harness's. Arm A's vendored package equals `packages/github-actions` file by file (checked in runs 1 and 5 of both rounds). `target/debug/seikika` was rebuilt at 19:29:27, after the round, and its hash differs. The runs used the workspace copies, and so did my rebuilds. The openfga source the harness copies is unchanged since 10:42, and its workflows equal every workspace's `main` |
| P4 symmetric arms | **Pass, with a gap in triage** | Neither prompt has a recipe, and the gates carry gate-repair-ab's rival fixes. zizmor's own `--fix=all` changes nine workflows and leaves every `uses:` line alone (re-checked), so no offline pinning tool was withheld from B. §6 promises that "rival friction this round shows is triaged like Seikika's", but §0 has no friction list. The transcripts show friction in both arms (threat 8) |
| P5 no harness-shaped product | **Pass** | Fit audit below |
| P6 a reported measure | **Pass** | Calls, read and gate read are reported for every run in both rounds. Round 1's measure lost three of B's results. As in gate-repair-ab rounds 4 and 5, the labels follow the measure as registered and are not void. Round 2's measure was registered and dry-run on round 1's data before round 2 was built, so it was not swapped after the run. It left no result missing |
| P7 independent adjudication | **Pass, with a caveat** | See threat 10. The publication commit `438cb3d` (19:29:59) put the result in the catalog, the research log, dx-benchmarks (L20, B10) and the experiments index before this adjudication existed. Its research-log row already says "Independently adjudicated" |

## Fit audit (P5)

### `8a76f24`: fits

| Check | Finding |
|---|---|
| Touches no harness file | `git show --stat 8a76f24` lists `crates/seikika-connector-workspace/src/workflows.rs`, the decision, `packages/github-actions` (fragment and test fixture), `portal/fixtures/workspace.ndjson` and `portal/results.json`. None is under `experiments/`. The code never names openfga or the benchmark; its unit test uses `actions/cache` SHAs, as the decision's example does |
| When it was made | Before the pre-registration and before any run, while the planted pull request was being built (README §2). The decision (`decided: 19:11:46`) predates its commit. So it is not a fix tuned on run data, but it was found through this task |
| Uses outside the harness are real | **Yes, verified.** This repository pins `github/codeql-action/analyze@1bd86f… # v4.39.4` and `…/init@1bd86f…` in `codeql.yml`. On an exported copy with an unpinned `github/codeql-action/upload-sarif@v4` added, `seikika report portal --skip runner,github-actions --baseline portal/results.json`, with the round's binaries, prints `github/codeql-action/upload-sarif@v4, pinned elsewhere as github/codeql-action/upload-sarif@1bd86f… # v4.39.4`. The old evidence took `shaPins[0]`. That list sorts the commented pins alphabetically, so it would have named `github/codeql-action/analyze@…`, another action. That is inferred from the removed code, not run. The package is included by Seikika for Seikika and the starter |
| Shaped to the harness or the task? | **No.** It changes which pin the hint names, not the hint's format: the `<ref>, pinned elsewhere as <pin>` text is the same before and after the commit. Its second rule, preferring a pin within the ref's tag, has no effect here, because each action has one SHA on `main`. The task was shaped around Seikika's hint in general (limit 3, threat 3), but this fix doesn't favour arm A. Without it, arm A's cache hint was wrong. The scorer's `intent-weekly` check would then have failed any A run that copied it |
| What it means for the result | Limit 5 is right: arm A's correctness depends on this fix, and there is no A⁻ arm to measure it separately. No round 2 A run checked a hint |

### The replay change (`6303a4d`): harness-only, every arm

It touches three harness files and the README (see *The replay*). It was registered after round 1's
data showed it would help arm A. That is disclosed, and it was frozen before round 2 was built. It
corrects a measurement error that biased the read and gate-read measures against arm A. It adds
nothing an agent did not receive, and it leaves out reviewdog's stderr. It fits P6, read with
L14: a measure change between rounds, before the runs, is allowed. A change after the runs is not.

## The designer's reading, checked

| Claim | Finding |
|---|---|
| §0.2 and §0.1 tables and labels | **Correct.** Every figure matches the TSVs and this recount |
| §0.2 Arm A: every run printed `weekly.yaml` and ran the gate in one command, then rewrote the ten lines with one `sed` or a short Python script and reran the gate in the same command; no run looked a pin up | **Correct.** seikika-4 used Python and the rest used `sed` |
| §0.2 Arm B: every run printed the gate script and `weekly.yaml`, ran the gate, and looked up the pins in one or two greps | **Correct.** reviewdog-1 chained its first grep onto the gate run |
| §0.2 "Grep output ranged from 919 to 13,522 characters" | **Wrong at the low end.** reviewdog-5's second grep returned 521 characters. Two lookups are also mixed with other output: reviewdog-1's first, in the gate command, and reviewdog-3's, rebuilt as 9,783 |
| §0.2 reviewdog-2's seven `search_replace` calls are most of its eleven; no arm B run picked a wrong pin | **Correct** |
| §0.2 the replay rebuilt all four missing results; 9 checks over three runs; stopped at reviewdog-2's first `search_replace` | **Correct**, and reproduced |
| §0.2 "The agents did receive these results. reviewdog-3 wrote all seven correct SHAs right after a lookup whose result the transcript lacks" | **Correct, and understated.** Three runs over the two rounds did this (see *The replay*) |
| §0.2 without the replay, B's read is a median of 14,638 with a range of 13,614, and N7 is inconclusive | **Correct** |
| §0.2 all seven missing results were in arm B: four gate runs, three chained with zizmor's plain output, and three greps | **Correct.** It does not say why: each of the seven commands exited non-zero |
| §0.2 "an agent with Seikika's gate reads about a third of what an agent with reviewdog's reads" | **Correct on the registered measure** (5,794 / 18,300 = 0.32). It is 0.40 on the recorded data, and the sentence should say which measure |
| §0.2 with the same replay, round 1 would have shown the same win by a narrower margin | **Correct as a counterfactual.** N3 holds by 332 characters and N2 and N4 stay inconclusive, so §4's rule would be met. The margin is smaller than any one run's choice of a broad grep |
| §0.1 Arm A copied the hints in four of five runs; seikika-2 grepped every `uses:` line (5,214); three runs printed `weekly.yaml` with the gate | **Correct** |
| §0.1 "Then they rewrote the ten `uses:` lines with one `sed`" | **Nearly.** Round 1's seikika-3 needed two: its first `sed` failed on the `#` delimiter (321 characters), and the second worked |
| §0.1 reviewdog-1 and -4 chained the lookup onto the gate run; -2, -3 and -5 made it a call of its own | **Correct** |
| §0.1 sensitivity table (8,795; 7,840; 5,592; 20,374; 18,496; 9,175) and "byte for byte the same length" | **Correct.** Reproduced, and the content matches too |
| §2 "Arm A's gate prints 1,827 characters"; "Arm B's gate prints 3,728 characters on stdout"; "zizmor's summary adds 96 characters on stderr" | **Bytes, not characters, and the wrong tool.** In characters, the measure's unit, the gates print 1,800 and 3,688 (`✗`, `·` and `→` are multi-byte). The 96 characters on stderr are reviewdog's `reviewdog: found at least one issue with severity greater than or equal to the given level: any`. The script sends zizmor's stderr to `/dev/null`. This is in the pre-registered part, and it changes no label |
| Status line: "After round 1, only this status line, §0 and §4.1 changed" | **Nearly.** §7 also gained two lines (the `REPLAY` command) in `6303a4d` |
| §4 designer's estimate: "each call's result carries about 400 characters of wrapper" | The wrapper of a terminal result is 313 characters here. That changes no label |

## Threats to validity

1. **The replay decides N7, and it was introduced after round 1 failed to show a win.** It is a
   second try at the same claim with a measure known to help arm A. Several things mitigate that.
   It was registered and frozen before round 2. It was validated, and the agents demonstrably
   received what it rebuilds. And the dropped results follow from each command's exit code, so the
   unreplayed measure was biased against A by command style. I accept the label. A reader should
   still count two tries: under the measure round 1 registered, neither round shows a win.
2. **N7 is narrow and rests on few runs.** It holds by 1,073 characters against *W* = 11,433, and
   by 212 on output characters alone. *W* is entirely arm B's spread, because arm A's five runs are
   nearly identical. Two things set B's median, reviewdog-2's 18,300: its replayed gate run (4,633)
   and its seven editor calls (6,290 characters of results). Had it edited with one `sed`, N7 would
   be inconclusive and N6 would hold instead. Without reviewdog-3, no hypothesis holds. With five
   near-deterministic runs per arm, one agent's habits can decide a label (L12, L18, L19).
3. **The task was built for the hint (limit 3).** Every fix in the planted pull request is a pin the
   repository already holds, each action has exactly one SHA on `main`, and the hint prints the
   replacement text, comment included. Arm A's work was to copy it, and in round 2 no A run checked
   a hint. This makes the result closer to a manipulation check than a sample. When the gate prints
   the answer, agents copy it instead of looking it up, and the lookup is what B pays for: 919 to
   13,522 characters of grep a run, plus a longer gate output (3,688 against 1,800 characters).
   The share of real pull requests that look like this is not measured.
4. **What is attributed to Seikika.** The rival is reviewdog over the same tools, as registered. It
   has no pin helper because no offline tool supplies one (§1, re-checked for zizmor's `--fix`). A
   few lines of shell in the rival's gate could print the same hint from the other workflows. That
   is glue a team could write, and it was not tested. The result is evidence for the hint, not for
   the box beyond it.
5. **Designer and product author are the same agent (limit 4).** The hint was built and tuned
   through gate-repair-ab on the same fork (`76a1140`, `c1f9b22`), then fixed here (`8a76f24`).
   Counting gate-repair-ab's five rounds, this is the seventh round to ask whether Seikika beats
   reviewdog, and the first to show it. Each round had a different task or measure, so they are
   not one test repeated. But a single narrow win after six non-wins should be read in that light.
6. **External validity.** One fork, one planted pull request, one model, one runtime. The
   decision's preference for a pin within the ref's tag is untested here, because each action has
   one SHA. In a repository that doesn't pin an action anywhere, the hint names only the ref.
7. **Correctness can't separate the arms.** All 20 diffs are byte-identical and equal the reference
   fix. A's correctness rides on `8a76f24` (limit 5).
8. **Friction was not triaged (P4, §6).** Two kinds in arm B and two in arm A.
   - **B, a second look at the raw tools.** In round 2, three of five B runs (reviewdog-2, -3 and
     -4) ran zizmor or actionlint by hand after their gate had passed. reviewdog-4's note says "the
     efm parsing might have dropped a finding". Two runs in each round (reviewdog-3 and -5) also ran
     `zizmor --format plain` on `weekly.yaml` in the same command as their first gate run. In round
     2 that added about 3,650 characters of output to each.
   - **A, the baseline line.** On a pass, Seikika's gate prints "baseline: no regressions, but
     .seikika/results.json is out of date: run with --update and commit it", which the prompt
     forbids. Five A runs over the two rounds deliberated over it before leaving it.
   - **A, the `new (none)` line.** seikika-3, in both rounds, paused over
     `maintainer-ci-security · …weekly.yaml: new (none)`.

   None changes a label. B's extra zizmor runs raise B's maximum, which widens *W* against arm A.
9. **What the measure counts.** Read counts the runtime's wrapper (313 characters per terminal
   result), so arms that make more calls pay more. N7 still holds without it (variants table).
   Gate read counts whole commands. Every A run printed `weekly.yaml` in its gate command, so A's
   gate read equals its read. N8 is inconclusive either way.
10. **Limits of this adjudication's independence.** The designer's session launched this adjudicator,
    with a brief that lists the commits and the checks to make. It runs on the same model family as
    the arms, and it used gate-repair-ab's adjudication as its template, which may anchor its
    framing. It checked the brief's commits against `git log` and the runtime's agent list. It
    recounted, rebuilt and rescored from the raw transcripts and workspaces, rather than taking §0
    or `count.py` as given.

## Corrections the README needs

None changes a label. §2 and §4 are the bar, so corrections to them belong in §0, not in those
sections.

1. **§0.2, the replay:** add that every dropped result came from a command that exited non-zero (7
   of 7, against 55 of 55 recorded results from commands that exited 0), and that arm A's commands
   always ended with a command that exits 0. Say that three runs, not one, wrote the SHAs straight
   after a dropped lookup. Add the same mechanism to L20 in `docs/design/dx-benchmarks.md`. L20
   also says the dropped gate runs were "chained with zizmor's plain output", which is true of
   three of the four.
2. **§0.2, "Grep output ranged from 919 to 13,522 characters":** the lowest was 521 (reviewdog-5's
   second grep).
3. **§0.2, "about a third":** say it is on the registered measure, with the replay. On the recorded
   data it is 0.40.
4. **§0.2, robustness:** say that N7 holds by 1,073 characters, that it also depends on
   reviewdog-2's editing style (with one `sed`, N6 would hold instead), and that without
   reviewdog-3 no hypothesis holds.
5. **§0.1:** round 1's seikika-3 took two `sed` calls, not one.
6. **§2's figures, corrected in §0:** the gates print 1,800 and 3,688 *characters* (1,827 and 3,728
   bytes). The 96 characters on stderr are reviewdog's failure line, not zizmor's summary.
7. **Status line:** §7 also changed after round 1 (the `REPLAY` command).
8. **§0, friction:** add the triage §6 promises (threat 8).
9. **Publication (`438cb3d`):** it preceded this adjudication. Its research-log row says
   "Independently adjudicated", which is true only from this file on. Its catalog `fitAudit` names
   `github/codeql-action/analyze` as a wrongly hinted sub-action: that is the hint the old code
   would have given on this repository (fit audit). The catalog's `outcome`, B10 and the
   dx-benchmarks summary should add that N7's margin is 1,073 characters, and that the win depends
   on the replay.

## Sources

- [README.md](./README.md) §0–§7 at `60ab6e4`, and its history: `git diff` of `25079a6`,
  `c2d27fe`, `6303a4d` and `60ab6e4`.
- [D01M84X4JDHXS98K89KN4AB0RCD](../../docs/decisions/D01M84X4JDHXS98K89KN4AB0RCD.md) and
  `git show 8a76f24`; [D01M83RDWQ5EVKY5SFHDWAD5HFH](../../docs/decisions/D01M83RDWQ5EVKY5SFHDWAD5HFH.md)
  (P1–P7); [dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §3 and §4.1;
  [gate-repair-ab's ADJUDICATION.md](../gate-repair-ab/ADJUDICATION.md).
- `results/r1.tsv`, `results/r2.tsv`, `results/r{1,2}/*.json|*.diff|*.replay.json`;
  `harness/count.py`, `replay.py`, `collect.py`, `score.py`, `setup.sh`, `plant.py`, `round.sh`,
  `quality-gate.sh`, `seikika.yaml`, `reference_fix.py`; `prompts/{seikika,reviewdog}.md` and
  `prompts/rendered/r{1,2}/`.
- The 20 runtime transcripts under `/tmp/cursor/cloud-agent-transcripts/2026-11-23T19-20-19Z-7f3e/`
  and `…/2026-11-23T19-26-31Z-39e1/` (by the bcIds in `/tmp/pin-repair-ab/r{1,2}/runs.txt` and
  the TSVs), their `index.json` creation times, and the runtime's list of cloud agents created
  since 2026-11-23 15:35 UTC.
- The workspaces `/tmp/pin-repair-ab/r{1,2}/<run>`, read only: binaries hashed, gate files and
  vendored packages compared, copies rescored. `replay.py` was re-run on `r2/reviewdog-3`, which it
  moves aside and restores, and the workspace's fingerprint was unchanged.
- Scratch rebuilds of every pre-edit terminal command of all 20 runs, in workspaces built with
  `setup.sh` at paths of equal length, using each run's own binaries; and the planted branch gated
  with both arms' gates.
- An exported copy of this repository at `438cb3d`, with `github/codeql-action/upload-sarif@v4`
  added to `codeql.yml`, gated with `seikika report portal` and the round's binaries.
- `git log`/`git show` of `8a76f24`, `25079a6`, `2fd3b8e`, `c2d27fe`, `6303a4d`, `1465d7d`,
  `60ab6e4` and `438cb3d`.
