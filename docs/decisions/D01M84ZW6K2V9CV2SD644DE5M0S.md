---
id: D01M84ZW6K2V9CV2SD644DE5M0S
decided: 2026-11-23T20:10:00Z
---

# Toolbox for Toolbox doesn't adopt Graphify, and the context lake doesn't build a code knowledge graph

**Toolbox for Toolbox doesn't carry Graphify or any other code-graph tool, and Toolbox doesn't build
or ingest a code knowledge graph. Code intelligence stays federated, as the context lake already
designs it ([context-lake.md](../design/context-lake.md) §5.5). The expansion the evidence points to
is pushing declared facts that grep can't recover into an agent's context. That is the context-pack
work already in the context lake's phasing, and it waits for its own benchmark.** The survey and the
reasoning are in [knowledge-graphs.md](../design/knowledge-graphs.md).

1. **The four options.**

   | Option | Verdict | Why |
   |---|---|---|
   | Adopt Graphify in Toolbox for Toolbox | No | [graphify-ab](../../experiments/graphify-ab/README.md) round 1, under Graphify's own Cursor rule and document layer, showed neither "helps" nor "hurts" on Toolbox or OpenFGA. Building the document layer took 17 agents and 4.27M characters read for this repository, and documents change on most commits |
   | Use another tool | No | No code-graph tool in the survey has an independent result showing agent gains ([knowledge-graphs.md](../design/knowledge-graphs.md) §4.1). Compiler-accurate navigation is already the harness's job: a language server, not a graph in the repository |
   | Expand Toolbox with a code graph | No | An extracted graph is open-world and its ids change between builds, so a scorecard can't be computed over it soundly. Its questions (where is X, what calls it) are answered by `rg` in a call or two |
   | Do nothing | **Yes** | The context lake keeps code intelligence federated. Nothing in this decision changes the product |

2. **The benchmark was worth running once.** Independent studies had tested Graphify's graph as an
   optional tool, or under other agents' harnesses. None had tested the vendor's mandatory rule and
   document layer in Cursor on a documentation-heavy repository like this one, and the adoption
   question needed that test. The answer is that it isn't shown. No second round is planned: coverage
   hit the ceiling (L21 in [dx-benchmarks.md](../design/dx-benchmarks.md)). Arm B's lower call counts
   (a median of 18 fewer on both repositories) didn't clear the noise. Even at face value, its read
   saving on Toolbox (98k characters a session) repays one document build only after about 43
   sessions.
3. **Reopen when** either of these happens:
   - Graphify, or a tool like it, keeps the document layer current without a full LLM pass over
     each changed document;
   - a question set that leaves the baseline arm headroom shows a coverage gain under no turn cap.

   Either reopening reruns graphify-ab with its harness as committed.
4. **A portal may still carry Graphify** as a standalone tool, with its own command, ignore file
   and rule. Toolbox neither wraps it nor ships a package for it, and nothing in a portal's
   scorecards reads its graph.
5. **Next for the context lake.** The literature finds gains where a graph holds facts grep can't
   recover (build and test structure, test impact, issue-to-code links, ownership) and pushes them
   into the agent's context. That describes catalog context packs
   ([context-lake.md](../design/context-lake.md) §5.4), not a code graph. A claim that context packs
   help agents needs a DX Benchmark before it is made.
