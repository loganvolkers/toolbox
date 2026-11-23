# graphify-monorepo-ab: Graphify vs a Toolbox repo portal vs nothing, on a large Rust monorepo

**About:** tools ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)). Graphify is a
standalone tool a portal may carry, and arm C is the box's own read path (a repo portal and its CLI
reads). This benchmark asks what each gives a coding agent on Graphify's home ground.

**Status:** pre-registered. §0 is filled in after the run; nothing else changes after the first
launch of round 1, except the dry run's outcome in §8.

## 0. Results

To be filled in after round 1.

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
  [dx-benchmarks.md](../../docs/design/dx-benchmarks.md)). Here four of the five questions are sets
  scored by F1, so a missed or extra item moves the score, and the dry run checks that arm A doesn't
  saturate before round 1 is frozen.

graphify-ab dropped a Toolbox arm because its questions were about code, which a catalog doesn't
hold. Here two of the five questions (Q1, Q2) are about crate structure, which a repo portal over a
Cargo workspace does hold, and three (Q3 to Q5) are about code, which it doesn't. So the third arm is
the box as a repository would adopt it, asked the questions Graphify is built for and some it shares.

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

Five questions ([prompts/questions-codex.md](prompts/questions-codex.md)), each scored 0 to 1; a
run's **score** is their mean:

| Q | Asks | Scored by | Gold size |
|---|---|---|---|
| Q1 | Direct and indirect dependents of `codex-tools` | Mean of the two sets' F1 | 7 direct, 22 indirect |
| Q2 | The five crates with the most direct dependents, with counts | ½ names, ½ exact counts | 5 |
| Q3 | Functions that call `get_git_repo_root` | F1 | 15 functions in 10 crates |
| Q4 | Crates whose code refers to `ModelProviderInfo` | F1 | 11 crates |
| Q5 | `codex sandbox` on Linux, entry point to exec | Key-fact coverage (covered 1, partial ½) | 6 facts |

Q1 and Q2 are about crate structure (the **structure subscore** is their mean); Q3 to Q5 are about
code (the **code subscore**). The gold and its rubric are kept outside the repository until the runs
finish and then committed as `gold/codex.md`. Its hash, recorded now:

| File | sha256 |
|---|---|
| `gold/codex.md` | `1eec308cfd9c55a4afb88ea91b0ff78c2aa14f1b4e076d6a908c1cea598ae878` |

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
| M1 | With Graphify, answers score higher than with nothing | med(B) − med(A) > *W*, on score | med(A) − med(B) > *W* |
| M2 | With Graphify, the agent reads less | med(A) − med(B) > *W*, on read | med(B) − med(A) > *W* |
| M3 | With Graphify, the agent makes fewer calls | As M2, on calls | As M2, on calls |
| M4 | With Toolbox, answers score higher than with nothing | As M1, C for B | As M1, C for B |
| M5 | With Toolbox, the agent reads less | As M2, C for B | As M2, C for B |
| M6 | With Toolbox, the agent makes fewer calls | As M3, C for B | As M3, C for B |
| M7 | Graphify scores higher than Toolbox | med(B) − med(C) > *W*, on score | med(C) − med(B) > *W* |
| M8 | Graphify reads less than Toolbox | med(C) − med(B) > *W*, on read | med(B) − med(C) > *W* |
| M9 | Graphify makes fewer calls than Toolbox | As M8, on calls | As M8, on calls |
| M10 | Arm B's agents use the graph | Every B run calls `graphify` | Any B run doesn't |
| M11 | Arm C's agents use the portal | Every C run calls `toolbox` | Any C run doesn't |
| M12 | Graphify helps where it is supposed to: code | As M1, on the code subscore | As M1, on the code subscore |
| M13 | Toolbox helps where it is supposed to: crate structure | As M4, on the structure subscore | As M4, on the structure subscore |

**Graphify helps** if none of M1 to M3 fails, and one of them holds. **Graphify hurts** if M1 fails,
or both M2 and M3 fail. **Toolbox helps** and **hurts** are the same over M4 to M6.
**Graphify beats Toolbox** if none of M7 to M9 fails and one holds; **Toolbox beats Graphify** if
none holds and one fails. If M10 (M11) fails, the runs without a call are reported, and the verdict
for Graphify (Toolbox) is "not shown", whatever the others say.

**The designer's prediction, before the run:**

- **M1 and M4 are inconclusive.** Without a turn cap, arm A finds callers and references with `rg`
  and dependents with `cargo metadata` or the Cargo.toml files. Its errors are in mapping
  directories to package names and in separating direct from indirect dependents, which are a few
  items out of many.
- **M13 holds:** the portal's `dependentCount` and `dependents` make Q2 exact and Q1's direct set a
  single read. **M12 is inconclusive:** Graphify's `calls` edges list Q3's callers, but A's `rg`
  does too.
- **M2 and M5 are inconclusive.** Graphify's answers are node and edge lists, and Toolbox's search and
  get return whole entities with every function, so both still read a lot.
- **M3 and M6 are inconclusive; M7 to M9 are inconclusive; M10 and M11 hold.**

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
