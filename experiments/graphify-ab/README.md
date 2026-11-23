# graphify-ab: does a Graphify knowledge graph help an agent answer questions about a repository?

**About:** a tool ([toolbox-and-tools.md](../../docs/design/toolbox-and-tools.md)). Graphify is a
candidate for the box; this benchmark decides whether Toolbox for Toolbox carries it.

**Status:** pre-registered. §0 is filled in after the run; nothing above §0's results changes after
the first launch.

## 0. Results

Not run yet.

## 1. Why

[Graphify](https://github.com/Graphify-Labs/graphify) (graphifyy 0.9.118, Apache-2.0) turns a folder
of code and documents into a knowledge graph that an agent queries instead of reading files. Its
install for Cursor writes an always-applied rule that makes `graphify query`, `path` or `explain`
mandatory before Read, Grep, Glob or Bash. It is the most-starred agent-context tool, and its
evidence for code is thin. Its own code benchmark (vendor, unreplicated) asks six questions of one
repository (ERPNext), builds the graph from code only, caps the agent at 14 turns, and reports
key-fact coverage of 82.0% with the graph against 70.8% with grep and read.

The question is whether Toolbox should adopt Graphify for this repository (Toolbox for Toolbox), and
whether Toolbox's context lake ([context-lake.md](../../docs/design/context-lake.md)) needs a code
knowledge graph of its own.

| If | Then |
|---|---|
| Graphify helps on Toolbox and doesn't hurt on OpenFGA (§5) | T4T carries Graphify as a tool in the box: pinned, its ignore file and rule committed, the graph rebuilt by a script; a package for portals follows |
| Otherwise | T4T doesn't adopt it; the context lake stays catalog-level and federates code graphs rather than building one, and the verdict is recorded with this result |

## 2. Arms

| Arm | What the agent has |
|---|---|
| **A** (baseline) | The repository and the agent's own tools: read, search (`rg`), shell. This is what agents working on Toolbox have today |
| **B** (Graphify) | The same, plus a prebuilt Graphify graph in `graphify-out/` and Graphify's Cursor rule, quoted verbatim in the prompt and present at `.cursor/rules/graphify.mdc`. `graphify` 0.9.118 is on PATH in both arms |

Arm B is Graphify as its README tells a Cursor user to set it up (`/graphify .`, then
`graphify install --platform cursor`), which is more than its own benchmark gave it: the graph
holds the documents' semantic layer as well as the code's AST (§3). The prompts are identical
except for the rule block ([prompts/task.md](prompts/task.md)); a local subagent doesn't load a
workspace's rules, so the rule is quoted.

**A third arm was considered and dropped.** Toolbox's own catalog (the T4T portal over MCP) holds
entity-level facts: scorecards, owners, decisions, connectors. These questions are about code and
the documents that explain it, which the catalog doesn't hold, and questions it does answer would
be unfair to Graphify. The overlap is compared by capability in
[knowledge-graphs.md](../../docs/design/knowledge-graphs.md) instead.

## 3. Repositories and the graph build

| Repository | Commit | Language | Files Graphify detects |
|---|---|---|---|
| OpenFGA, the fork `loganvolkers/openfga` | `7d89a566` | Go | 531: 508 code, 21 documents, 2 images |
| Toolbox (this repository) | `a8f7e03` | Rust, Markdown, YAML | 795: 440 code, 355 documents |

Both are `git archive` exports, so no arm reads history. The graph is built by
[harness/build-graph.sh](harness/build-graph.sh), which follows Graphify's skill
(`graphify/skill.md`):

1. Detect, with a `.graphifyignore` per repository
   ([toolbox](harness/graphifyignore.toolbox), [openfga](harness/graphifyignore.openfga)) that drops
   generated experiment outputs, rendered prompts, licence texts and illustrations.
2. Run the AST pass (tree-sitter).
3. Chunk the documents, and give each chunk to a `general-purpose` subagent with the extraction
   spec's prompt verbatim: 17 chunks for Toolbox, 3 for OpenFGA.
4. Finish with [harness/finish-graph.py](harness/finish-graph.py): coverage check, merge, cache,
   build, Leiden clustering, report and health check.

**Departures from the skill, declared before the run:**

- Toolbox has about 800 files, over the skill's 500-file prompt to narrow the scope. The answer
  given is "the whole repository, with the ignore file".
- Communities are named by Graphify's LLM-free labeller (`label_communities_by_hub`), not by the
  host agent. Names affect the report, not `query`, `path` or `explain`.
- Steps 6 to 9 (HTML, manifest, cost tracker) are skipped. They don't change what the queries
  read. Arm B's workspace gets what step 9's cleanup leaves.
- tree-sitter's SQL grammar is not installed with graphifyy, so `.sql` files (15 in OpenFGA, 8 in
  Toolbox) add nothing. That is Graphify as installed.

The build's cost (calls and characters of the 20 semantic subagents) is reported with the result;
it is not in the bar.

## 4. Scoring

Five questions per repository ([openfga](prompts/questions-openfga.md),
[toolbox](prompts/questions-toolbox.md)) that need several files: a call chain, an impact set, a
mechanism, a validation rule, a configuration path. Several also need a document or a decision.
Each has four or five **key facts**, written and checked against the frozen commit before the run:
22 for OpenFGA, 21 for Toolbox. The gold is kept outside the repository until the runs finish and
then committed as `gold/`. Its hashes, recorded now:

| File | sha256 |
|---|---|
| `gold/openfga.md` | `baeea1dc3516a70cb7d32227fb539d9a1d6203fcc84ac2550d2a97aa5a189ea1` |
| `gold/toolbox.md` | `fd1580021b41a02fb3a7e7362d705a15193592d1c0045d9de4baa260c7d2a3e8` |

**Coverage** is Graphify's own metric: (covered + ½ partial) / key facts, per run, over the
repository's five questions. The adjudicator grades each fact covered, partial or missed, with the
quote that decides it, following the rubric at the top of each gold file. Answers are graded blind:
[harness/blind.py](harness/blind.py) shuffles them under random labels and redacts the word
"graphify". The adjudicator sees the arm map only after the grade table is written.

## 5. Measures and the bar (pre-registered)

**Measures (P6).** Coverage (§4), plus two from each run's runtime transcript
([harness/count.py](harness/count.py)):

- **calls**: tool calls the agent made;
- **read**: characters of tool results the agent received. Local subagents report no token counts;
  a `read_file` without a recorded result counts as the file.

Wall time and Graphify calls are reported but are not in the bar. The dry run (§8) checks that both
transcript measures are reported for both arms.

**Rules.** Medians and ranges are over the five runs of an arm on a repository. *W* is the wider of
the two arms' ranges. A hypothesis holds or fails only if the medians differ by more than *W*;
otherwise it is inconclusive (P2).

| Id | Repository | Hypothesis | Holds | Fails |
|---|---|---|---|---|
| K1 | OpenFGA | With Graphify, answers cover more key facts | med(B) − med(A) > *W*, on coverage | med(A) − med(B) > *W*, on coverage |
| K2 | OpenFGA | With Graphify, the agent reads less | med(A) − med(B) > *W*, on read | med(B) − med(A) > *W*, on read |
| K3 | OpenFGA | With Graphify, the agent makes fewer calls | As K2, on calls | As K2, on calls |
| K4 | Toolbox | As K1 | As K1 | As K1 |
| K5 | Toolbox | As K2 | As K2 | As K2 |
| K6 | Toolbox | As K3 | As K3 | As K3 |
| K7 | Both | Arm B's agents use the graph | Every B run calls `graphify` at least once | Any B run doesn't |

**Graphify helps** on a repository if none of its three hypotheses fails, and either its coverage
hypothesis holds or one of its cost hypotheses holds. **Graphify hurts** on a repository if its
coverage hypothesis fails, or both its cost hypotheses fail. If K7 fails, the runs without a
`graphify` call are reported, and the verdict on adoption is "not shown", whatever K1 to K6 say:
the rule, which is the product's way to get used, didn't work.

**The designer's prediction, before the run:**

- **Coverage (K1, K4) is inconclusive.** Without a turn cap, a capable agent with `rg` finds these
  facts. The vendor's gain was 11 points under a 14-turn cap.
- **Read (K2, K5) is inconclusive or fails.** Graphify's answers are node and edge lists of up to
  2,000 tokens. The questions need function bodies, so B's agents still read the files, and the
  rule makes them query first.
- **Calls (K3, K6) are inconclusive.**
- **K7 holds.**

## 6. Protocol

- **Runs (P2).** Five fresh runs per arm per repository: 20 runs, launched together, as local
  subagents of the designer's session, on the session's model. Workspace numbers are shuffled per
  repository ([harness/round.py](harness/round.py)), so a path doesn't say its arm;
  `prompts/rendered/r1/arms.tsv` maps runs to arms.
- **Prompts (P1).** Rendered from [prompts/task.md](prompts/task.md) and committed under
  `prompts/rendered/` before the first launch.
- **Frozen (P3).** This commit fixes the harness, prompts, questions and graph build. Graphify is
  graphifyy 0.9.118.
- **Symmetric (P4).** Same prompt, tools and repository in both arms. Arm B adds what Graphify's
  setup adds, and nothing that helps with these questions in particular.
- **Contamination.** A run is void if it reads answer-bearing content outside its workspace:
  another checkout of the same repository (such as `/agent/repos/toolbox`), the build directory,
  another run's workspace, or the gold. `count.py` lists every absolute path outside the workspace
  (`outsidePaths`), and the adjudicator rules on each. A voided run is reported, not rerun.
- **Harness faults.** A run that ends without `ANSWERS.md` because of a runtime error, not the
  agent's choice, is rerun once, named in §0 with its error. No run is dropped for its result.
- **Adjudication (P7).** A fresh agent that designed no part of this grades the answers blind, then
  applies §5 and writes `ADJUDICATION.md`.

## 7. Running

```bash
export PATH=$HOME/.local/bin:$PATH   # graphify 0.9.118 (uv tool install graphifyy==0.9.118)
TOOLBOX_CHECKOUT=. OPENFGA_CHECKOUT=/path/to/openfga experiments/graphify-ab/harness/build-graph.sh prepare
# one general-purpose subagent per /tmp/graphify-ab/semantic-prompts/*.md, then:
experiments/graphify-ab/harness/build-graph.sh finish
python3 experiments/graphify-ab/harness/round.py 1 5 --workspaces
# launch one subagent per prompts/rendered/r1/<run>.md; when all are done, fetch their
# transcripts with batch-fetch-details (includeTranscripts), then:
python3 experiments/graphify-ab/harness/collect.py 1 <transcripts-dir> <run>=<bcId> ...
python3 experiments/graphify-ab/harness/blind.py 1
```

## 8. Dry run

One run per arm on OpenFGA (`round.py dry 1 --workspaces`), not scored, to check that the
transcript reports calls and read for both arms and that `graphify` answers in arm B's workspace.
Its outcome is recorded here before round 1 launches.

## Sources

- Graphify repository, `Graphify-Labs/graphify` at `013a646` (graphifyy 0.9.118): `graphify/skill.md`,
  `graphify/install.py` (`_CURSOR_RULE`), `graphify/skills/agents/references/extraction-spec.md`,
  `graphify/cluster.py`. Read hands-on.
- Graphify's code benchmark and its numbers: the vendor's `graphify-vs-local` page and benchmark
  write-up (vendor claims, not replicated here).
- Protocol 2 and the house harness: [dx-benchmarks.md](../../docs/design/dx-benchmarks.md) §4.1,
  [gate-repair-ab](../gate-repair-ab/README.md).
