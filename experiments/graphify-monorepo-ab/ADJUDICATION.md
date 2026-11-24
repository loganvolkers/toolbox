# graphify-monorepo-ab round 1: adjudication

**Verdict: neither tool is shown to help or hurt, and neither beats the other.** Under the rules
in [README.md](README.md) §5, every hypothesis from M1 to M9 is inconclusive, and so are the guards
M12 and M13. M10 and M11 hold.

| Verdict (§5) | Result |
|---|---|
| Graphify helps / hurts / neither | **Neither shown.** No hypothesis from M1 to M3 holds or fails. |
| Toolbox helps / hurts / neither | **Neither shown.** No hypothesis from M4 to M6 holds or fails. |
| Graphify vs Toolbox | **Neither beats the other.** No hypothesis from M7 to M9 holds or fails. |
| M10 (every B run calls `graphify`) | **Holds**: 5, 4, 3, 3 and 4 calls. |
| M11 (every C run calls `toolbox`) | **Holds**: 2, 3, 3, 1 and 4 calls. |

The §1 table's "Otherwise" branch applies: D01M84ZW6K2V9CV2SD644DE5M0S stands for large monorepos
too, and the friction found is triaged into fixes.

This matches the designer's prediction on every outcome. The one miss is the "leaning to fail" on M2
and M3: arm B's medians were lower than arm A's on both read and calls, not higher, though within
*W* (§8 below).

## 1. Who adjudicated, and how

- **Who.** One fresh agent did all of it. It designed no part of the benchmark.
- **Phase 1, blind.** The agent read only README §4, the gold, and the 15 answers in
  `/tmp/graphify-monorepo-ab/blind-r1/`. The answers sat under random three-digit labels, with
  "graphify" and "toolbox" replaced by "[tool]" and the `crate:` prefix dropped. It graded every
  answer and wrote [results/r1/grades-r1.md](results/r1/grades-r1.md) before opening any of these: the
  label key, `prompts/rendered/r1/arms.tsv`, `results/r1.tsv`, the per-run JSON and the per-run
  answers. Where an answer disputed a gold item, it checked the claim in the code at
  `/tmp/graphify-monorepo-ab/src/codex`.
- **Phase 2, unblinded.** The agent then read the whole README, the key (copied to
  [results/r1/blind-key.tsv](results/r1/blind-key.tsv)), the arm map, `results/r1.tsv`, the per-run
  JSON and the Toolbox server logs. It applied §5 and §6. As an extra contamination check, it scanned
  the 15 runtime transcripts.
- **Gold hash check.** `sha256sum /tmp/graphify-monorepo-ab/gold/codex.md` gives
  `d597a2fb17109f6a92981fa936eaf9981b3c9694e7ba6f5a80d40e1c687823a2`. That matches the hash
  pre-registered in §4.

## 2. Contamination and harness faults

| Check | Result |
|---|---|
| `outsidePaths` (from `count.py`, `results/r1/<run>.json`) | Empty for all 15 runs; `outside` = 0 in `results/r1.tsv` |
| Extra check: relative paths and names in the transcripts | Clean (see below) |
| Void runs | **None** |
| Runs ending without `ANSWERS.md` | None; `answered` = true for all 15 |
| Toolbox servers serving all 120 crates at run start | Yes, for all 5 arm C servers (see below) |
| Harness faults, reruns | **None** |

**Extra contamination check.** `count.py` catches only absolute paths in tool arguments. A
relative escape, such as `../codex-3` or `cd ..` followed by a read, would get past it. So I scanned
every tool call's arguments in the 15 transcripts (the `batch-fetch-details` export behind
`r1-bcids.txt`). I looked for:

- three or more `../` levels, or `cd ..`;
- any path under `/tmp/graphify-monorepo-ab/` other than the run's own workspace and scratch directory;
- another run's workspace number;
- `gold`, `blind`, `arms.tsv`, `rdry`, the transcripts directory, `~/`, `$HOME`, `/agent/` and
  `experiments/`.

Nothing matched. The only `../` paths are two levels deep, such as `../../otel/src`, run from
`codex-rs/core/src`, so they stay inside the workspace. No run set a working directory outside its
workspace.

**Toolbox servers.** `serve.sh` polls each server until `toolbox query --kind Crate` returns 120
items, for at most 240 s. The logs (`/tmp/graphify-monorepo-ab/serve-r1-codex-{1,2,10,12,14}.log`)
show all five servers starting at 21:44:20 and the polling ending by 21:44:43, well short of the
timeout. That means each server reached 120 crates. Every run made its first tool call between
21:47:37 and 21:47:42. The first `toolbox get crate:codex-tools` in each C run returned the full
entity. `serve.sh`'s per-server "n crates" line isn't saved, so this ruling rests on the poll timing
rather than on a logged count.

**Missing tool results.** Eight runs have tool calls without a recorded result (column `missing` in
`results/r1.tsv`). A missing `read_file` result is imputed as the file (`readImputed`), as §5
defines. That is a measurement rule, not a harness fault. One more `run_terminal_cmd` result is
missing in each of codex-1, codex-4 and codex-9, and two in codex-8, and nothing is imputed for those.
The amounts are small next to the ranges, and they fall in all three arms.

## 3. Per run

The score is the mean of Q1 to Q4. The code subscore is the mean of Q2 to Q4, and the structure
subscore is Q1. Calls are `toolCalls`, and read is `resultCharsImputed`. The tool calls column shows
the shell calls that ran `graphify` or `toolbox`, with their subcommands (§9). The grade detail is in
[results/r1/grades-r1.md](results/r1/grades-r1.md).

| Run | Arm | Label | Score | Q1 | Q2 | Q3 | Q4 | Code | Calls | Read | Tool calls | Wall (s) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| codex-3 | A | 324 | 0.994 | 1.000 | 1.000 | 0.974 | 1.000 | 0.991 | 51 | 208,990 | — | 238 |
| codex-5 | A | 885 | 0.994 | 1.000 | 1.000 | 0.974 | 1.000 | 0.991 | 55 | 203,379 | — | 287 |
| codex-6 | A | 537 | 0.988 | 0.950 | 1.000 | 1.000 | 1.000 | 1.000 | 77 | 261,195 | — | 405 |
| codex-7 | A | 683 | 0.988 | 0.950 | 1.000 | 1.000 | 1.000 | 1.000 | 56 | 255,863 | — | 275 |
| codex-8 | A | 721 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 70 | 256,555 | — | 370 |
| codex-4 | B | 891 | 0.981 | 0.950 | 1.000 | 0.974 | 1.000 | 0.991 | 49 | 198,160 | `graphify` 5 (`query` 3, `explain` 3) | 290 |
| codex-9 | B | 421 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 54 | 225,758 | `graphify` 4 (`explain` 3, `query` 2) | 250 |
| codex-11 | B | 478 | 0.988 | 0.950 | 1.000 | 1.000 | 1.000 | 1.000 | 54 | 200,296 | `graphify` 3 (`query` 2, `explain` 2) | 315 |
| codex-13 | B | 210 | 0.994 | 1.000 | 1.000 | 0.974 | 1.000 | 0.991 | 58 | 204,870 | `graphify` 3 (`explain` 2, `query`, `path`) | 266 |
| codex-15 | B | 283 | 0.981 | 0.950 | 1.000 | 0.974 | 1.000 | 0.991 | 59 | 256,203 | `graphify` 4 (`query` 2, `explain` 2) | 287 |
| codex-1 | C | 855 | 0.981 | 0.950 | 1.000 | 0.974 | 1.000 | 0.991 | 73 | 275,061 | `toolbox` 2 (`get` 2, `search`) | 327 |
| codex-2 | C | 476 | 0.994 | 1.000 | 1.000 | 0.974 | 1.000 | 0.991 | 63 | 206,872 | `toolbox` 3 (`get`, `query`, `search`) | 277 |
| codex-10 | C | 565 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 52 | 230,325 | `toolbox` 3 (`get` 2, `search`) | 263 |
| codex-12 | C | 154 | 0.981 | 0.950 | 1.000 | 0.974 | 1.000 | 0.991 | 45 | 183,831 | `toolbox` 1 (`get`) | 250 |
| codex-14 | C | 833 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 1.000 | 54 | 199,292 | `toolbox` 4 (`get` 3, `query` 2, `describe`) | 273 |

Only two things separate the answers, and the rest are perfect:

- **Q1 = 0.950:** the answer lists 18 of the 22 indirect dependents. It leaves out `codex-chatgpt`
  and the three `*_test_support` crates, which are workspace packages but not in the explicit
  `members` list.
- **Q3 = 0.974:** the answer lists 19 of the 20 functions. It leaves out
  `ExternalAgentConfigMigrationScreen::display_description`.

No answer has a false positive anywhere.

## 4. Medians and ranges (five runs per arm)

| Measure | Arm | Median | Min | Max | Range |
|---|---|---|---|---|---|
| score | A | 0.9936 | 0.9875 | 1.0000 | 0.0125 |
| score | B | 0.9875 | 0.9811 | 1.0000 | 0.0189 |
| score | C | 0.9936 | 0.9811 | 1.0000 | 0.0189 |
| code subscore (Q2–Q4) | A | 1.0000 | 0.9915 | 1.0000 | 0.0085 |
| code subscore (Q2–Q4) | B | 0.9915 | 0.9915 | 1.0000 | 0.0085 |
| code subscore (Q2–Q4) | C | 0.9915 | 0.9915 | 1.0000 | 0.0085 |
| structure subscore (Q1) | A | 1.000 | 0.950 | 1.000 | 0.050 |
| structure subscore (Q1) | B | 0.950 | 0.950 | 1.000 | 0.050 |
| structure subscore (Q1) | C | 1.000 | 0.950 | 1.000 | 0.050 |
| calls | A | 56 | 51 | 77 | 26 |
| calls | B | 54 | 49 | 59 | 10 |
| calls | C | 54 | 45 | 73 | 28 |
| read (chars) | A | 255,863 | 203,379 | 261,195 | 57,816 |
| read (chars) | B | 204,870 | 198,160 | 256,203 | 58,043 |
| read (chars) | C | 206,872 | 183,831 | 275,061 | 91,230 |
| wall (s), not in the bar | A | 287 | 238 | 405 | 167 |
| wall (s), not in the bar | B | 287 | 250 | 315 | 65 |
| wall (s), not in the bar | C | 273 | 250 | 327 | 77 |

The scores take four exact values:

- (0.95 + 1 + 38/39 + 1)/4 = 0.98109;
- (0.95 + 3)/4 = 0.98750;
- (3 + 38/39)/4 = 0.99359;
- 1.

The code subscore is either 1 − 1/117 = 0.99145 or 1. Every comparison below uses the exact values.

## 5. M1 to M13

*W* is the wider of the two arms' ranges. A hypothesis holds or fails only if the medians differ by
more than *W* in the stated direction. Otherwise it is inconclusive. M1, M4, M12 and M13 are guards:
they can only fail or be inconclusive (§8 of the README).

| Id | Measure | Medians | *W* | Difference | Outcome |
|---|---|---|---|---|---|
| M1 | score, A vs B | A 0.9936, B 0.9875 | max(0.0125, 0.0189) = 0.0189 | med(A) − med(B) = 0.0061 ≤ 0.0189 | **Inconclusive** (did not fail) |
| M2 | read, A vs B | A 255,863, B 204,870 | max(57,816, 58,043) = 58,043 | med(A) − med(B) = 50,993 ≤ 58,043 | **Inconclusive** |
| M3 | calls, A vs B | A 56, B 54 | max(26, 10) = 26 | med(A) − med(B) = 2 ≤ 26 | **Inconclusive** |
| M4 | score, A vs C | A 0.9936, C 0.9936 | max(0.0125, 0.0189) = 0.0189 | 0 | **Inconclusive** (did not fail) |
| M5 | read, A vs C | A 255,863, C 206,872 | max(57,816, 91,230) = 91,230 | med(A) − med(C) = 48,991 ≤ 91,230 | **Inconclusive** |
| M6 | calls, A vs C | A 56, C 54 | max(26, 28) = 28 | med(A) − med(C) = 2 ≤ 28 | **Inconclusive** |
| M7 | score, B vs C | B 0.9875, C 0.9936 | max(0.0189, 0.0189) = 0.0189 | med(B) − med(C) = −0.0061; \|0.0061\| ≤ 0.0189 | **Inconclusive** |
| M8 | read, C vs B | C 206,872, B 204,870 | max(91,230, 58,043) = 91,230 | med(C) − med(B) = 2,002 ≤ 91,230 | **Inconclusive** |
| M9 | calls, C vs B | C 54, B 54 | max(28, 10) = 28 | 0 | **Inconclusive** |
| M10 | `graphifyCalls` in B runs | — | — | 5, 4, 3, 3, 4; minimum 3 ≥ 1 | **Holds** |
| M11 | `toolboxCalls` in C runs | — | — | 2, 3, 3, 1, 4; minimum 1 ≥ 1 | **Holds** |
| M12 | code subscore, A vs B | A 1.0000, B 0.9915 | max(1/117, 1/117) = 0.0085 | med(A) − med(B) = 1/117, **exactly *W*** | **Inconclusive** (did not fail) |
| M13 | structure subscore, A vs C | A 1.000, C 1.000 | max(0.05, 0.05) = 0.05 | 0 | **Inconclusive** (did not fail) |

**M12 sits on its threshold.** Arm A's median code subscore is 1 and arm B's is 1 − 1/117. That
difference equals *W* exactly, because both arms' ranges are the same single Q3 item. The rule needs
"more than *W*", so M12 does not fail. One more B run missing `display_description` would not change
this: B's median would stay at 0.9915, and the result rests on A's median. One A run fewer finding it
would give A a median of 0.9915 too.

**Directions within *W*.** None of these is a result:

- Arm B's median read is 20% below arm A's (M2), and arm C's is 19% below A's (M5).
- Both tool arms' median calls are 2 below arm A's (M3, M6).
- B and C have the same median calls (M9), and C reads 1% more than B (M8).

## 6. Verdicts under §5

| Verdict | Rule | Outcome |
|---|---|---|
| **Graphify helps** | None of M1–M3 fails, and M2 or M3 holds | No: none of M2 or M3 holds |
| **Graphify hurts** | M1 fails, or both M2 and M3 fail | No: M1 did not fail, and M2 and M3 did not fail |
| → Graphify | | **Neither shown** |
| **Toolbox helps** | None of M4–M6 fails, and M5 or M6 holds | No: none of M5 or M6 holds |
| **Toolbox hurts** | M4 fails, or both M5 and M6 fail | No |
| → Toolbox | | **Neither shown** |
| **Graphify beats Toolbox** | None of M7–M9 fails, and one holds | No: none holds |
| **Toolbox beats Graphify** | None of M7–M9 holds, and one fails | No: none fails |
| → Graphify vs Toolbox | | **Neither beats the other** |
| M10, M11 | If either fails, that arm's verdict is "not shown" whatever the others say | Both hold, so that override doesn't apply |

**What follows from §1:**

- "Graphify helps, with M1 holding" is not met.
- "Graphify helps on cost only" is not met.
- "Toolbox helps" is not met.
- "Graphify over Toolbox on score (M7)" is not met.
- **The "Otherwise" branch applies.** D01M84ZW6K2V9CV2SD644DE5M0S stands for large monorepos too,
  and the friction is triaged into fixes (§10).

## 7. Disputed gold items

**No answer names an item outside the gold**, on any of Q1, Q2 or Q3. So there is no outside-gold
claim to rule on.

Two gold items are disputed the other way: answers leave them out, and two answers argue against
one. I checked both in the code, and the gold stands for both. The detail is in the grades file.

| Gold item | Left out by | What the code shows | Ruling |
|---|---|---|---|
| Q3 `ExternalAgentConfigMigrationScreen::display_description` | 8 answers: A 2 of 5, B 3 of 5, C 3 of 5. 154 (codex-12, C) and 210 (codex-13, B) say it is two hops away | `codex-rs/tui/src/external_agent_config_migration.rs:233-237`. In its own body, outside the nested `reformat_description` (lines 194–208), its `"Migrate enabled plugins from "` branch calls `display_path_for` directly. | Gold correct |
| Q1 `codex-chatgpt`, `app_test_support`, `core_test_support`, `mcp_test_support` | 7 answers: A 2 of 5, B 3 of 5, C 2 of 5 | All four are path entries in `[workspace.dependencies]` (`codex-rs/Cargo.toml:131,148,243,244`) and among the 120 packages of `cargo metadata --no-deps`. Each has a normal dependency into the closure: `chatgpt` and `core/tests/common` on `codex-core`; `app-server/tests/common` on `codex-core` and `core_test_support`; `mcp-server/tests/common` on `codex-mcp-server` and `core_test_support`. | Gold correct |

**Would any change an outcome? No.** I recomputed M1, M4, M7, M12 and M13 three ways:

- with my one judgement call reversed, crediting 855 (codex-1, C) with `display_description`;
- with a Q3 gold of 19, without `display_description`;
- with a Q1 gold of 18 indirect dependents.

Every outcome stays inconclusive. The Q3 alternative reverses M12's sign (−1/117, still exactly *W*),
and the Q1 alternative reverses M1's and M7's signs (±0.0061 against a *W* of 0.0189). The cost
hypotheses don't depend on grading.

## 8. Against the designer's prediction (§5)

| Prediction | Outcome | Match |
|---|---|---|
| M1, M4, M7, M12 and M13 are inconclusive; the misses are single items, such as `display_description` | All inconclusive. The misses are `display_description` (8 runs) and the four implicit workspace members in Q1 (7 runs) | Yes. The Q1 miss was not foreseen: Q1 was 1.0 in every arm in both dry runs |
| M2 and M3 are inconclusive, leaning to fail (B reads and calls more than A) | Inconclusive, but the lean is the other way: B's median read is 50,993 below A's (*W* 58,043), and its median calls are 2 below | Outcome yes; lean no |
| M5 and M6 are inconclusive | Inconclusive (C reads 48,991 less than A, *W* 91,230; calls 2 fewer, *W* 28) | Yes |
| M8 and M9 are inconclusive | Inconclusive | Yes |
| M10 and M11 hold | Both hold | Yes |

## 9. What differs between the arms' answers

- **The answers converge.** The 15 answers share a structure: the same tables, the same exclusion
  lists, the same trace steps. Q2 and Q4 are perfect in every run. On Q2, every run told
  `dyn ThreadStore` dispatch apart from `LiveThread::read_thread`, `CodexThread::read_thread` and the
  free function `local::read_thread::read_thread`. On Q4, every run named the spawned task, the trait
  object, the generic dispatch, `transform` with the helper, and `codex-utils-pty`. The questions'
  hard parts didn't separate the arms. The separation is two boundary conventions, one per question,
  and the arms split on them about evenly.
- **Arm C used the portal for Q1's direct set only.** All five C runs began with
  `toolbox get crate:codex-tools`, and four of the five answers cite the portal's `dependents` field
  for the 7 direct dependents. The portal holds no transitive closure (declared in §2), so every C run
  computed the indirect set some other way. The three C runs that also ran `cargo metadata` (codex-2,
  codex-10, codex-14) listed all 22. The two that didn't (codex-1, codex-12) listed 18. In arms A and
  B the link is looser: A's codex-6 ran `cargo metadata` and still listed 18, while A's codex-8 and
  B's codex-13 didn't and listed 22. After Q1, C runs used the portal little: `search "read_thread"`,
  and `get` on `codex-thread-store`. Toolbox results were 1–10% of each C run's read.
- **Arm B oriented with the graph, then read the code.** Every B run asked `graphify query` which
  crates depend on `codex-tools`. It then used `explain` on `read_thread` / `ThreadStore` /
  `get_git_repo_root` and `query` (one run used `path`) for the `exec_command` trace. Then it did what A
  did: `rg`, `sed -n` and file reads at each call site. Graphify results were 4–15% of each B run's
  read. No B answer cites the graph as evidence for a listed item. The answers cite files and lines.
- **The tool arms read less, within noise.** B's and C's median reads are about 50k characters below
  A's, out of about 250k. B's range is also the tightest on calls (10, against 26 and 28). Under the
  pre-registered rule neither is a result. They are the closest any cost hypothesis came to holding
  (M2 at 88% of *W*).
- **Wall time** is reported but is not in the bar. The medians are 287 s (A), 287 s (B) and 273 s
  (C).
- **Counting tool calls.** `graphifyCalls` and `toolboxCalls` count shell calls, and M10 and M11 use
  those. `graphifyCommands` and `toolboxCommands` count subcommands, and one shell call can hold
  several. For example, codex-1 made 2 `toolbox` calls holding 3 subcommands, and codex-4 made 5
  `graphify` calls holding 6.

## 10. Friction found while adjudicating

1. **The blinding leaks arm C.** Four of five C answers (476, 565, 833, 855) say "the [tool]
   portal's `codex-tools` entity (`dependents`…)". `blind.py` redacts the tool's name but not
   "portal" or the entity fields, so a grader can tell those answers' arm. The grades are mechanical
   F1 and fact coverage, and I noted the leak before unblinding. But a benchmark with judged rubrics
   would need `blind.py` to redact those phrases too.
2. **`count.py`'s contamination check sees absolute paths only.** A relative escape would pass it. I
   ran a transcript scan to cover that gap (§2). The check could resolve relative paths against each
   call's working directory.
3. **The server readiness count isn't saved.** `serve.sh` echoes "n crates" per server to the
   terminal only. Writing it to the serve log would make the harness-fault rule checkable from
   files.
4. **Q1's gold depends on a convention the question leaves open.** Whether implicit workspace
   members count was split 8 to 7 across runs, independent of arm. The gold follows
   `cargo metadata`, which is defensible. But the question could name the source ("as
   `cargo metadata` lists them") so that the item measures reading, not a guess at the convention.
