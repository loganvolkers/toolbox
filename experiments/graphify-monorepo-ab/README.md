# graphify-monorepo-ab: Graphify vs a Toolbox repo portal vs nothing, on a large Rust monorepo

**About:** tools ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)). Graphify is a
standalone tool a portal may carry, and arm C is the box's own read path (a repo portal and its CLI
reads). This benchmark asks what each gives a coding agent on Graphify's home ground.

**Status:** complete (round 1). §0 was filled in after the run. Nothing else changed after the first
launch, except a `yamark format` of `harness/portal/toolbox.yaml` that parses to the same data.

## 0. Results

**Neither Graphify nor a Toolbox repo portal helped or hurt on Codex, and neither beat the other.**
In round 1 (15 runs, 2026-11-23), M1 to M9, M12 and M13 are inconclusive, and M10 and M11 hold. That
matches the designer's prediction on every outcome, though not on M2 and M3's lean: arm B read and
called less than arm A, not more, within *W*. §1's "Otherwise" row applies:
[D01M84ZW6K2V9CV2SD644DE5M0S](../../docs/decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md) stands for large
monorepos too, and the friction is triaged below. A fresh agent graded the answers blind and applied
§5 ([ADJUDICATION.md](ADJUDICATION.md)). No run was void and no run was rerun.

| Id | Measure | A median (range) | B median (range) | C median (range) | *W* | Verdict |
|---|---|---|---|---|---|---|
| M1 | Score, A vs B (guard) | 0.994 (0.988–1) | 0.988 (0.981–1) | | 0.019 | Inconclusive |
| M2 | Read, chars, A vs B | 255,863 (203,379–261,195) | 204,870 (198,160–256,203) | | 58,043 | Inconclusive |
| M3 | Calls, A vs B | 56 (51–77) | 54 (49–59) | | 26 | Inconclusive |
| M4 | Score, A vs C (guard) | 0.994 | | 0.994 (0.981–1) | 0.019 | Inconclusive |
| M5 | Read, chars, A vs C | 255,863 | | 206,872 (183,831–275,061) | 91,230 | Inconclusive |
| M6 | Calls, A vs C | 56 | | 54 (45–73) | 28 | Inconclusive |
| M7 | Score, B vs C | | 0.988 | 0.994 | 0.019 | Inconclusive |
| M8 | Read, C vs B | | 204,870 | 206,872 | 91,230 | Inconclusive |
| M9 | Calls, C vs B | | 54 | 54 | 28 | Inconclusive |
| M10 | B runs calling `graphify` | | 5 of 5 (3 to 5 calls) | | | Holds |
| M11 | C runs calling `toolbox` | | | 5 of 5 (1 to 4 calls) | | Holds |
| M12 | Code subscore, A vs B (guard) | 1 (0.991–1) | 0.991 (0.991–1) | | 0.0085 | Inconclusive, exactly at *W* |
| M13 | Structure subscore, A vs C (guard) | 1 (0.95–1) | | 1 (0.95–1) | 0.05 | Inconclusive |

What the numbers say, beyond the bar:

- **The questions saturated again.** Two hardenings (§8) took Q2 to trait dispatch, Q3 to two hops
  with test functions excluded at both, and Q4 across five crates, a spawned task, a trait object and
  a generic trait. Every answer still scored 0.981 or more, Q2 and Q4 were perfect in every run, and
  no answer listed a false positive. Only two boundary conventions separated answers: four workspace
  packages outside the explicit `members` list (Q1, missed by 7 runs) and a call made beside a nested
  function (Q3, missed by 8). Both splits fell across all three arms. On a 950k-line Rust monorepo,
  without a turn cap, agents with only `rg` and file reads answered as completely as with either tool.
- **Both tool arms read about a fifth less than the baseline, inside *W*.** That is the nearest any
  cost hypothesis came to holding: M2's difference is 88% of *W*. Calls barely moved (56, 54, 54). One
  run in each tool arm read as much as arm A's typical run.
- **Graphify oriented but didn't answer.** Every B run queried the graph and then read the code as
  arm A did. Graphify results were 4–15% of each B run's read, and no B answer cites the graph as
  evidence. The graph has no crate or Cargo nodes, so `graphify query "which crates depend on
  codex-tools"` returned nodes unrelated to dependents; `explain read_thread` was ambiguous across
  6 nodes; and `path` found no directed path from `handle_output_item_done` to `spawn_pty_process`.
- **The portal answered Q1's direct set and nothing more.** Every C run began with `toolbox get
  crate:codex-tools`, and four of five answers cite its `dependents` field for the 7 direct
  dependents. That field is direct only, and the CLI's `graph` couldn't follow one relation in one
  direction, so each run computed the 22 indirect ones another way. The three C runs that ran `cargo
  metadata` listed all 22; the two that didn't listed 18. Toolbox results were 1–10% of each C run's
  read.

**Friction, triaged.**

| Friction | Where | Triage |
|---|---|---|
| `toolbox graph` had only `--depth`, though the REST and MCP reads already filter by relation and direction | Toolbox CLI | **Fixed** (`--relation`, `--direction`, and the `graph` skill). On a frozen arm C workspace, `toolbox graph crate:codex-tools --relation dependsOn --direction in --depth 3` returns all 29 dependents in one call |
| The graph read clamps depth to 3 | Toolbox API | **Open** ([Q20](../../docs/questions.md)). Codex's deepest dependent is 3 hops away; a deeper workspace would need a transitive read |
| `search` matches ids and titles only, so `search read_thread` found nothing, though the portal lists each crate's functions | Toolbox API | **Open** (Q20). Recommendation: don't widen it; a portal that indexes code becomes the code index `rg` already is |
| `get` returned about 16k characters per crate, and agents cut it with `head -c` | Harness rule | The `get` and `query` skills already name `--fields`; this benchmark's `toolbox.mdc` didn't. A next round's rule names it |
| No crate or Cargo nodes; ambiguous names; no directed call paths | Graphify | A tool's own gaps; published here, not worked around |
| `blind.py` redacts tool names but not "portal" or entity field names, so four C answers were recognisable | Harness | Didn't affect these mechanical grades. A benchmark with judged rubrics redacts each arm's vocabulary |
| `count.py`'s contamination check sees absolute paths only | Harness | The adjudicator scanned every transcript for relative escapes and found none. A next harness resolves paths against each call's working directory |
| `serve.sh` prints each server's crate count to the terminal only | Harness | Readiness was ruled from poll timing. A next harness logs the count |
| Q1 leaves open whether implicit workspace members count | Question | The gold follows `cargo metadata`, and the adjudicator upheld it. A next question names its source |

Files: [results/r1.tsv](results/r1.tsv) (measures), `results/r1/` (each run's measures and answers,
and the [blind grades](results/r1/grades-r1.md)), [results/r1/blind-key.tsv](results/r1/blind-key.tsv)
(blind labels), [gold/](gold/) (hash as in §4).

## 1. Why

[graphify-ab](../graphify-ab/README.md) found neither "helps" nor "hurts" for Graphify on Toolbox
and OpenFGA, and its decision ([D01M84ZW6K2V9CV2SD644DE5M0S](../../docs/decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md))
kept code intelligence out of the context lake. Two limits of that result motivate this one:

- **Wrong ground for Graphify.** Both repositories had about 500 to 800 files and Toolbox is
  document-heavy, so the graph's document layer cost 17 agents to build. Graphify's own code
  benchmark (vendor, unreplicated) builds a code-only graph of one large repository, and its README
  says the token saving grows with corpus size (vendor claim). This benchmark gives it that: a large,
  code-dominant monorepo, a code-only graph that needs no LLM, and questions about calls, references,
  crate dependencies and an execution path, which its `query`, `path`, `explain` and `affected`
  commands and its Cargo pass are built for.
- **No headroom.** Every answer scored the same key-fact coverage (L21 in
  [dx-benchmarks.md](../../docs/design/dx-benchmarks.md)). Here three of the four questions are sets
  scored by F1, so a missed or extra item moves the score, and the dry run checks that arm A doesn't
  saturate before round 1 is frozen. The first dry run did saturate, and the questions were hardened
  (§8): trait dispatch, a second ring of callers and a cross-crate trace replace lookups that `rg`
  and `cargo metadata` answer exactly. The second dry run saturated too (arm A 0.994), so, per L21,
  the score is not frozen as a measure that can show a gain: it is a guard that can only show harm,
  and a gain has to come from cost (§5). That is also the claim Graphify makes for large corpora:
  fewer tokens for the same answers (vendor claim).

graphify-ab dropped a Toolbox arm because its questions were about code, which a catalog doesn't
hold. Here one of the four questions (Q1) is about crate structure, which a repo portal over a Cargo
workspace does hold, and three (Q2 to Q4) are about code, which it doesn't. So the third arm is the
box as a repository would adopt it, asked the questions Graphify is built for and one it shares.

| If | Then |
|---|---|
| Graphify helps (§5), with M1 holding | D01M84ZW6K2V9CV2SD644DE5M0S's second reopen trigger is met (a score gain with headroom and no turn cap). A new decision says where Graphify helps, and the repo-portal docs recommend carrying it as a standalone tool in large code repositories |
| Graphify helps on cost only (M2 or M3) | A new decision records the cost gain and the repository size it holds at; Toolbox for Toolbox still doesn't adopt it (this repository is neither large nor code-dominant) |
| Toolbox helps (§5) | A repo portal's crate facts help agents; the starter gains a Crate kind for Rust workspaces, and the claim is narrowed to crate-structure questions if only M13 holds |
| Graphify over Toolbox on score (M7) | The decision that the context lake builds no code graph is revisited: a new decision on whether repo portals federate a code graph's call and reference edges |
| Otherwise | D01M84ZW6K2V9CV2SD644DE5M0S stands for large monorepos too; the friction found is triaged into fixes |

## 2. Arms

| Arm | What the agent has |
|---|---|
| **A** (nothing) | The repository and the agent's own tools: read, search (`rg`), shell. `cargo` works offline in `codex-rs/` |
| **B** (Graphify) | A, plus a prebuilt Graphify graph in `graphify-out/` and Graphify's Cursor rule ([prompts/graphify.mdc](prompts/graphify.mdc), what `graphify cursor install` writes), quoted verbatim in the prompt and present at `.cursor/rules/graphify.mdc` |
| **C** (Toolbox) | A, plus a repo portal at `.toolbox/toolbox.yaml` ([harness/portal/toolbox.yaml](harness/portal/toolbox.yaml)), a `toolbox serve` of it on 127.0.0.1 started before the run, and a rule ([prompts/toolbox.mdc](prompts/toolbox.mdc)) quoted in the prompt and present at `.cursor/rules/toolbox.mdc` |

`graphify` 0.9.118 and `toolbox` are on PATH in every arm. The prompts are identical except for the
rule block ([prompts/task.md](prompts/task.md)). A local subagent doesn't load a workspace's rules,
so the rule is quoted.

**Arm C's departures, declared before the run:**

- **Toolbox ships no agent rule** for reading a repo portal (`toolbox install` installs packages and
  connectors, not rules). Arm C's rule is written for this benchmark in the form of Graphify's: the
  same "MANDATORY ... before Read, Grep, Glob, or Bash" opening and exceptions, with the read
  commands from the operator skills (`search`, `query`, `get`, `graph`) and one line on the Crate
  kind. It names the server's port and the two variables the CLI needs.
- **The portal is the starter plus crates**, as a Rust repository would adopt it: the starter's
  Repository and Doc kinds without its scorecards, and the Crate facts from
  `portal/standards/crates.yaml` (fields, `dependsOn`, and `dependents` and `dependentCount`
  computed from inbound `dependsOn`). Nothing is added for these questions in particular: no
  transitive closure, no call or reference edges. A second workspace connector is rooted at
  `codex-rs/`, because the connector runs `cargo metadata` in its root and Codex's Cargo workspace
  isn't at the repository root.
- `toolbox owner` is left out of the rule: the portal computes no ownership field, so it fails on
  every crate.
- Toolbox is the debug build of this repository at `9bac4f3` (main). The server keeps an in-memory
  store and runs without OpenFGA (dev mode); each arm C run has its own server on its own copy.

## 3. Repository and builds

| Repository | Commit | Files | Rust | Cargo workspace |
|---|---|---|---|---|
| Codex, the fork `loganvolkers/codex` of `openai/codex` | `859dbe27` (2026-06-02) | 4,770 | 2,080 files, 948,570 lines | 120 packages in `codex-rs/`, 599 normal dependency edges between them |

The repository is a `git archive` export, so no arm reads history. [harness/build.sh](harness/build.sh)
builds both arms' additions from it:

- **Graphify:** `graphify extract . --code-only --cargo`, then `graphify cluster-only . --no-viz`,
  with `GRAPHIFY_BACKEND=none`. This is Graphify's headless build for CI with its code-only flag, as
  its own code benchmark used, plus its Cargo pass. No LLM runs, and the default ignores apply (the
  repository's `.gitignore`). It takes 59 s and gives 63,888 nodes, 181,569 edges (50,837 `calls`,
  36,283 `references`, 597 `depends_on`) and 1,243 communities, named by the LLM-free hub labeller.
  tree-sitter's SQL grammar isn't installed with graphifyy, so 39 `.sql` files add nothing.
- **Toolbox:** the portal is copied in and validated (`toolbox validate .toolbox`); each server reads
  the workspace when it starts (about 25 s for 120 crates).

## 4. Scoring

Four questions ([prompts/questions-codex.md](prompts/questions-codex.md)), each scored 0 to 1; a
run's **score** is their mean:

| Q | Asks | Scored by | Gold size |
|---|---|---|---|
| Q1 | Direct and indirect dependents of `codex-tools` | Mean of the two sets' F1 | 7 direct, 22 indirect |
| Q2 | Non-test functions that call the trait method `ThreadStore::read_thread`, not same-named inherent methods | F1 | 12 functions in 3 crates |
| Q3 | Non-test functions that call `get_git_repo_root` through exactly one intermediate function | F1 | 20 functions in 10 crates |
| Q4 | An `exec_command` tool call on Linux, from the completed response item to the process spawn | Key-fact coverage (covered 1, partial ½) | 8 facts |

Q1 is about crate structure (the **structure subscore**); Q2 to Q4 are about code (the **code
subscore** is their mean). Q2 needs dispatch through `dyn ThreadStore` told apart from two inherent
methods of the same name; Q3 needs the callers of 15 callers, with test functions excluded at both
hops; Q4 crosses five crates, a spawned task, a trait object and a generic trait. The gold and its
rubric are kept outside the repository until the runs finish and then committed as `gold/codex.md`.
Its hash, recorded now:

| File | sha256 |
|---|---|
| `gold/codex.md` | `d597a2fb17109f6a92981fa936eaf9981b3c9694e7ba6f5a80d40e1c687823a2` |

Answers are graded blind: [harness/blind.py](harness/blind.py) shuffles them under random labels and
replaces "graphify" and "toolbox" with "[tool]" and drops the portal's `crate:` id prefix. The
adjudicator sees the arm map only after the grade table is written.

## 5. Measures and the bar (pre-registered)

**Measures (P6).** The score (§4), plus two from each run's runtime transcript
([harness/count.py](harness/count.py)): **calls**, the tool calls the agent made, and **read**, the
characters of tool results it received (a `read_file` without a recorded result counts as the file).
Wall time and per-tool calls are reported but not in the bar.

**Rules.** Medians and ranges are over the five runs of an arm. *W* is the wider of the two arms'
ranges. A hypothesis holds or fails only if the medians differ by more than *W*; otherwise it is
inconclusive (P2).

| Id | Hypothesis | Holds | Fails |
|---|---|---|---|
| M1 | With Graphify, answers score no lower than with nothing (a guard) | Not tested: the score has no headroom (§8) | med(A) − med(B) > *W*, on score |
| M2 | With Graphify, the agent reads less | med(A) − med(B) > *W*, on read | med(B) − med(A) > *W* |
| M3 | With Graphify, the agent makes fewer calls | As M2, on calls | As M2, on calls |
| M4 | With Toolbox, answers score no lower than with nothing (a guard) | Not tested (§8) | As M1, C for B |
| M5 | With Toolbox, the agent reads less | As M2, C for B | As M2, C for B |
| M6 | With Toolbox, the agent makes fewer calls | As M3, C for B | As M3, C for B |
| M7 | Graphify scores higher than Toolbox | med(B) − med(C) > *W*, on score | med(C) − med(B) > *W* |
| M8 | Graphify reads less than Toolbox | med(C) − med(B) > *W*, on read | med(B) − med(C) > *W* |
| M9 | Graphify makes fewer calls than Toolbox | As M8, on calls | As M8, on calls |
| M10 | Arm B's agents use the graph | Every B run calls `graphify` | Any B run doesn't |
| M11 | Arm C's agents use the portal | Every C run calls `toolbox` | Any C run doesn't |
| M12 | Graphify doesn't hurt where it is supposed to be strong: code | Not tested (§8) | As M1, on the code subscore |
| M13 | Toolbox doesn't hurt where it is supposed to be strong: crate structure | Not tested (§8) | As M4, on the structure subscore |

M1, M4, M12 and M13 were written as gains and became guards before round 1 was frozen, because two
dry runs saturated the score (§8, L21). They can fail or be inconclusive. M7 compares two tool arms
and stays two-sided, but at saturation it can only show that one tool arm lost points.

**Graphify helps** if none of M1 to M3 fails, and one of them holds (so M2 or M3). **Graphify hurts** if M1 fails,
or both M2 and M3 fail. **Toolbox helps** and **hurts** are the same over M4 to M6.
**Graphify beats Toolbox** if none of M7 to M9 fails and one holds; **Toolbox beats Graphify** if
none holds and one fails. If M10 (M11) fails, the runs without a call are reported, and the verdict
for Graphify (Toolbox) is "not shown", whatever the others say.

**The designer's prediction, before the run** (revised after both dry runs, §8):

- **M1, M4, M7, M12 and M13 are inconclusive.** Every arm scored 0.99 or more in the second dry run;
  the misses are single items (Q3's `display_description`, which sits next to a nested function that
  also calls the intermediate).
- **M2 and M3 are inconclusive, leaning to fail.** In the second dry run arm B made 4 `graphify`
  calls and then read the code as A did; it made more calls and read more than A (62 and 290k
  characters, against 56 and 240k). The graph's answers orient the agent but don't replace reading
  each call site, which these questions need.
- **M5 and M6 are inconclusive.** Arm C read the least in the second dry run (184k) but also ran
  `cargo metadata` for Q1, and the code questions, three of four, cost every arm about the same.
- **M8 and M9 are inconclusive; M10 and M11 hold.**

## 6. Protocol

- **Runs (P2).** Five fresh runs per arm, 15 in all, launched together as local subagents of the
  designer's session, on the session's model. Workspace numbers are shuffled
  ([harness/round.py](harness/round.py)), so a path doesn't say its arm;
  `prompts/rendered/r1/arms.tsv` maps runs to arms and arm C's ports.
- **Prompts (P1).** Rendered from [prompts/task.md](prompts/task.md) and committed under
  `prompts/rendered/` before the first launch.
- **Frozen (P3).** The commit that adds `prompts/rendered/r1/` fixes the harness, prompts, questions,
  graph build and portal. Graphify is graphifyy 0.9.118.
- **Symmetric (P4).** Same prompt, tools and repository in every arm. Arms B and C add what their
  setup adds, and nothing that helps with these questions in particular.
- **One repository.** The request was one large monorepo where Graphify should be strong. A result
  here doesn't generalise to other languages or sizes; §0 says so.
- **Contamination.** A run is void if it reads answer-bearing content outside its workspace: another
  copy of Codex (the clone, the build directories, another run's workspace) or the gold. `count.py`
  lists every absolute path outside the workspace (`outsidePaths`), and the adjudicator rules on
  each. A voided run is reported, not rerun.
- **Harness faults.** A run that ends without `ANSWERS.md` because of a runtime error, not the agent's
  choice, is rerun once, named in §0 with its error. A Toolbox server that isn't serving all 120
  crates when its run starts is a harness fault. No run is dropped for its result.
- **Adjudication (P7).** A fresh agent that designed no part of this grades the answers blind, then
  applies §5 and writes `ADJUDICATION.md`.

## 7. Running

```bash
# graphify 0.9.118 (uv tool install graphifyy==0.9.118) and toolbox linked into /usr/local/bin
CODEX_CHECKOUT=/path/to/codex experiments/graphify-monorepo-ab/harness/build.sh
python3 experiments/graphify-monorepo-ab/harness/round.py 1 5 --workspaces
experiments/graphify-monorepo-ab/harness/serve.sh 1
# launch one subagent per prompts/rendered/r1/<run>.md; when all are done, fetch their
# transcripts with batch-fetch-details (includeTranscripts), then:
python3 experiments/graphify-monorepo-ab/harness/collect.py 1 <transcripts-dir> <run>=<bcId> ...
python3 experiments/graphify-monorepo-ab/harness/blind.py 1
experiments/graphify-monorepo-ab/harness/serve.sh 1 stop
```

## 8. Dry run

One run per arm (`round.py dry 1 --workspaces`), graded by the designer against the gold, to check
that the transcript reports calls and read for every arm, that `graphify` and `toolbox` answer in
their workspaces, and that arm A leaves headroom (L21): if arm A scores 0.9 or more, the questions are
hardened, the change and the new gold hash are recorded here, and the dry run is repeated. Its
outcome is recorded here before round 1 launches.

**First dry run** (`prompts/rendered/rdry/`, five questions, gold hash `1eec308c…ae878`): every arm
scored 1.0 on every question, so there was no headroom.

| Arm | Run | Score | Calls | Read (chars) | Tool calls | Wall (s) |
|---|---|---|---|---|---|---|
| A | codex-2 | 1.0 | 36 | 140,066 | — | 199 |
| B | codex-3 | 1.0 | 32 | 139,699 | `graphify` 4 (`query`, `path`, `explain`) | 172 |
| C | codex-1 | 1.0 | 28 | 123,081 | `toolbox` 4 (`search`, `query`, `get`) | 168 |

Every arm, including B and C, computed the crate questions with `cargo metadata` and a script, and
answered the caller and reference questions with `rg`. The trace (old Q5) followed one binary's
`main` in two crates. The harness worked: calls and read came out for every arm, and `graphify` and
`toolbox` answered in their workspaces. Two harness changes followed. Every arm wrote scratch
files to the shared `/tmp`, where concurrent runs could collide, so each run now gets its own
scratch directory, named in the prompt and allowed by `count.py`. And `count.py` counted a Python
line that contained "toolbox d" as a Toolbox call, so it now matches only Toolbox's own commands (C's
count above is with the fix; it was 5).

**Hardening.** Q1 is kept. Old Q2 (top five crates by dependents), Q3 (direct callers of
`get_git_repo_root`) and Q4 (crates referring to `ModelProviderInfo`) are dropped: each is one
`cargo metadata` script or one `rg`. The new Q2 to Q4 (§4) need what `rg` doesn't give: which
`read_thread` a call resolves to, callers of callers with test functions excluded at both hops, and a
path through a task spawn, a trait object and a generic trait into two other crates. The gold was
checked at every call site by hand, and Q2's and Q3's sets were checked against the references in a
SCIP index from rust-analyzer 1.95.0 (`rust-analyzer scip`, build scripts and proc macros off),
which gives the same 12 and 19 functions (gold hash `66b9afb0…462c2`). The second dry run is
`prompts/rendered/rdry2/`.

**Second dry run** (four questions):

| Arm | Run | Score | Q3 F1 | Calls | Read (chars) | Tool calls | Wall (s) |
|---|---|---|---|---|---|---|---|
| A | codex-1 | 0.994 | 0.974 | 56 | 240,433 | — | 250 |
| B | codex-3 | 0.994 | 0.974 | 62 | 290,304 | `graphify` 4 (`query` 3, `explain`, `path`) | 339 |
| C | codex-2 | 1.000 | 1.000 | 57 | 183,587 | `toolbox` 3 (`search`, `query`, `get`) | 312 |

Q1, Q2 and Q4 scored 1.0 in every arm. Every arm told the trait method from the inherent
`read_thread`s and named all eight trace facts. **The gold had an error, and arm C found it:**
`ExternalAgentConfigMigrationScreen::display_description` calls `display_path_for` itself (in its
"Migrate enabled plugins from" branch), as well as containing the nested `reformat_description` that
also calls it. The call-site helper had attributed that call to the nested function, and the SCIP
cross-check reused the helper to find enclosing functions. Recomputing Q3 from the SCIP index's own
function extents gives 20 functions, so the gold now has 20, and the scores above are against it
(A and B missed `display_description`). No run read outside its workspace and scratch directory.

**Decision before round 1.** Arm A scored 0.994, so the score still has no headroom. A third
hardening was tried and dropped: a blast-radius question (every non-test function that reaches a
function at any depth) has a 40 to 90 item gold for many targets, but its gold depends on rules that
agents would contest (a call through `dyn Trait` reaching an impl, nested functions, test helpers,
binaries that share a `main` name). A turn cap would make headroom, but then the cap is part of the
claim (L21), and D01M84ZW6K2V9CV2SD644DE5M0S's reopen trigger asks for a gain with no turn cap. So
the question set is frozen as it is, and the score hypotheses become guards (§5): round 1 asks
whether either tool cuts the cost of answers this hard on Graphify's home ground without lowering
them, which is the claim Graphify makes for large corpora. The new gold hash is in §4.

## Sources

- Graphify repository, `Graphify-Labs/graphify` at `013a646` (graphifyy 0.9.118): `graphify/cli.py`
  and `graphify/__main__.py` (`extract --code-only --cargo`, `cluster-only`),
  `graphify/cargo_introspect.py`, `graphify/install.py` (`_CURSOR_RULE`). Read and run hands-on.
- Graphify's README and code benchmark (`graphify-vs-local`): the token-saving and ERPNext coverage
  figures are vendor claims, not replicated here.
- Codex: `openai/codex` via the fork `loganvolkers/codex` at `859dbe27`; gold computed from
  `cargo metadata` and the code (gold/codex.md, committed after the run).
- Protocol 2 and the house harness: [dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §4.1,
  [graphify-ab](../graphify-ab/README.md).
