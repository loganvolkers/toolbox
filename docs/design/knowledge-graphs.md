# Knowledge Graphs as Agent Context: Graphify, Its Alternatives, and the Context Lake

*Status: research and verdict. Research date 2026-11-23.*

**About:** both. It surveys tools (Graphify and the code, document and enterprise graphs around it)
and decides what the box does with them: whether Toolbox for Toolbox carries Graphify, and whether
Toolbox's context lake ([context-lake.md](./context-lake.md)) builds a code knowledge graph of its own.

*Graphify was installed and run hands-on (graphifyy 0.9.118, source at `Graphify-Labs/graphify`
`013a646`). Star counts and push dates come from the GitHub API on the research date. "(vendor)" marks
a maker's claim; "(secondary)" marks a claim from a survey agent's reading that this document did not
re-check against the primary source. Benchmarks: [graphify-ab](../../experiments/graphify-ab/README.md)
and [graphify-monorepo-ab](../../experiments/graphify-monorepo-ab/README.md). Both, this survey and
the decision were attacked by a red team without the designer's context
([graphify-red-team.md](../graphify-red-team.md)), and every finding is triaged in
[graphify-red-team-response.md](../graphify-red-team-response.md).*

---

## 1. Conclusions

- **Toolbox for Toolbox doesn't adopt Graphify, uses no other code-graph tool, and Toolbox
  doesn't build a code knowledge graph**
  ([D01M84ZW6K2V9CV2SD644DE5M0S](../decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md), its reasons
  restated by [D01M85G2ZXXPQRXWKQF7TWV2RTP](../decisions/D01M85G2ZXXPQRXWKQF7TWV2RTP.md)). Code
  intelligence stays federated, as [context-lake.md](./context-lake.md) §5.5 already designs it.
  Graphify's value is not shown, which is not the same as absent: a cost result can reopen this.
- **Graphify didn't make answers more complete; whether it lowers cost is open.** In
  [graphify-ab](../../experiments/graphify-ab/README.md), 20 runs on Toolbox and OpenFGA under
  Graphify's own Cursor rule and document layer tied on coverage, and every label was inconclusive
  (§6). On Toolbox, though, every Graphify run made fewer calls and read less than every baseline
  run: 25% fewer calls and 29% less read by median, exact p = 0.008 each when read post hoc. The
  registered rule detects an effect that size about a quarter of the time (L23). Building the
  document layer for this repository took 17 agents; keeping it current was never measured.
- **On a large monorepo, nothing separated.** In
  [graphify-monorepo-ab](../../experiments/graphify-monorepo-ab/README.md), 15 runs on Codex (a
  120-crate, 950k-line Rust monorepo, a code-only graph) compared Graphify, a Toolbox repo portal and
  nothing. Every hypothesis comparing arms was inconclusive under the registered rule and under a
  post-hoc rank test (p ≥ 0.22) (§6.1). Every answer scored 0.981 or more. Graphify's `affected`
  command, built for these impact questions, was never named to the agents and never used.
- **A red team found the overreach** ([graphify-red-team.md](../graphify-red-team.md), triaged in
  [graphify-red-team-response.md](../graphify-red-team-response.md)). Earlier versions of this
  survey said the benchmarks showed "neither helps nor hurts" and savings "inside the noise". The
  fair reading is narrower: no coverage gain for a strong model with no turn cap, a likely cost
  gain on a documentation-heavy repository that no model has seen, and untested impact queries,
  billed tokens and incremental updates.
- **Graphify and a catalog hold different facts.** Graphify extracts a symbol-and-document graph
  in an open world: a missing edge means nothing, and node ids and communities change between
  builds. Port's context lake and Toolbox hold a declared catalog in a closed world: who owns what,
  which standard an entity meets, what changed. Only the second can be scored (§2, §5).
- **The independent evidence mostly agrees.** No independent study shows a code graph making a
  strong agent's answers better on average, and the billed-token studies under Graphify's hooks
  found it costlier. The small positive signals are narrow: impact questions about a named symbol,
  opening fewer files, and human use at the terminal (§3). The larger measured gains come from
  facts grep can't recover, such as build and test structure, test impact and ownership, pushed
  into the agent's context. That is the catalog's side: context packs, not a code graph (§5, §7).
- **The foundations explain why.** Tree-sitter graphs resolve names by heuristic, Leiden
  communities are unstable unless seeded, degree centrality finds hubs rather than answers, and
  LLM-extracted entities need resolution that declared identifiers avoid (§2).

## 2. Foundations

A knowledge graph is a set of entities and typed relations between them, together with the rules
for what a missing relation means. The choices below decide what such a graph can answer, how
cheaply, and how far its answers can be trusted (Hogan et al. 2021 is the survey).

| Topic | What matters for agent context |
|---|---|
| **Data model** | A *labelled property graph* (Neo4j, Kùzu, Apache AGE, ISO GQL) gives nodes and edges identity, labels and properties. *RDF* is triples over IRIs, with edge properties only through reification or RDF 1.2 triple terms. Any property graph embeds in relations (nodes and link tables), and SQL/PGQ (ISO/IEC 9075-16:2023) queries that view with GQL's pattern language, so the difference from SQL is ergonomics, not expressiveness. *Node-link JSON* (NetworkX, D3) is a serialisation with no schema or query semantics; Graphify's `graph.json` is this |
| **Query complexity** | BFS and DFS are O(V+E); transitive closure is O(V(V+E)) by repeated BFS. Regular path queries are polynomial under walk semantics but NP-complete under simple-path and trail semantics (Mendelzon & Wood 1995), which is why GQL makes path modes explicit and GitLab Orbit caps variable-length patterns at three hops. Recursive CTEs and Datalog (least fixpoint, semi-naive evaluation, PTIME data complexity, stratified negation) cover closure; CodeQL's QL, Meta's Glean Angle and Soufflé are Datalogs |
| **Community detection** | Modularity maximisation is NP-hard (Brandes et al. 2008). Louvain can return badly connected or disconnected communities (Traag et al. 2019); Leiden's refinement guarantees connected ones. Both are randomised and depend on a resolution parameter, and modularity can't see modules below about √(2m) edges (Fortunato & Barthélemy 2007). Unless seed and resolution are pinned, community ids are not stable across rebuilds, so nothing that diffs or ratchets should key on them |
| **Centrality** | Degree is O(V+E); PageRank is power iteration, and *personalised* PageRank restarts at query seeds (Aider's repo map, HippoRAG). Betweenness is O(VE) by Brandes' algorithm. Centrality is structural prominence: a logger or `utils` module is a hub, not an answer. Graphify's "god nodes" are degree |
| **Entity resolution** | Fellegi–Sunter (1969): a likelihood ratio over field agreement, with link, review and non-link thresholds, then transitive closure. LLM-extracted graphs (GraphRAG, LightRAG, Graphiti) mostly resolve by name, embedding or LLM judgement, so they duplicate and falsely merge. A catalog avoids this with declared identifiers and external ids ([identity-resolution.md](./identity-resolution.md)) |
| **World assumption** | Under the *open-world* assumption (RDF, OWL) a missing fact is unknown; under the *closed-world* assumption (SQL, Datalog's negation as failure, SHACL, JSON Schema) it is false. A scorecard rule such as "has no owner" is closed-world, so it needs a complete, declared catalog. It can't be computed soundly over an extracted graph. Toolbox already encodes the distinction: missing connector data is `unknown`, not a failure (D01M7Y5VWKW5TDS0VQWZFPRQ27N) |
| **Symbolic vs embedded** | Graph embeddings (TransE, ComplEx, RotatE) score plausibility for link prediction; symbolic graphs answer exactly, over what was asserted. Hybrid retrieval seeds with embeddings or lexical search and expands symbolically (HippoRAG, LightRAG, codebase-memory-mcp). Graphify seeds lexically and expands by BFS; it has no embeddings |
| **Provenance and time** | W3C PROV-DM (entity, activity, agent) and semiring provenance (Green et al. 2007) say how a fact was derived. Bitemporal data separates valid time from transaction time (Snodgrass; SQL:2011), which audit and incident review need; Graphiti and XTDB store both. Graphify's `EXTRACTED` / `INFERRED` / `AMBIGUOUS` tags are coarse confidence classes |
| **Incremental maintenance** | Counting and DRed maintain views under inserts and deletes (Gupta et al. 1993); differential dataflow and DBSP maintain iterative computations. Per-file re-parsing is easy to make incremental; cross-file name resolution, closure and global clustering are not. Toolbox recomputes in full in v1 (D37) |
| **Retrieval** | BM25 and trigram indexes are strong on code because identifiers are rare tokens; most of dense retrieval's apparent advantage on code comes from queries without identifiers (Xue et al. 2026, secondary). Reciprocal rank fusion combines rankers |
| **Static analysis** | Tree-sitter builds concrete syntax trees with no name binding, types or import resolution, so a tree-sitter "call graph" matches names heuristically: homonyms add false edges, and dynamic dispatch, reflection and generated code drop real ones. Compiler-backed indexes (SCIP, LSIF, LSP, Kythe, Glean, CodeQL) resolve names precisely; a sound call graph needs a further analysis (CHA, RTA, points-to), and practical tools are "soundy" (Livshits et al. 2015). Facts outside the code (build targets, ownership, deployments, test↔code links) are not recoverable by any parser |

## 3. Graphify

**What it is.** A Python CLI and agent skill (Apache-2.0, about 152k stars, created 2026-04-03) that
turns a folder of code, documents, papers, images and media into a graph an agent queries. A hosted
product, Graphify Cloud, adds a re-indexed graph per push, cross-repo graphs and PR review (vendor).

**How it works (read in the source and run on two repositories):**

| Stage | What happens |
|---|---|
| Detect | Classifies files as code, document, paper, image or media; honours `.gitignore` and `.graphifyignore`; asks to narrow the scope above 500 files |
| AST pass | tree-sitter over about 40 languages: files, symbols, imports, calls, references. Deterministic, local, seconds. Calls are resolved by name and tagged `EXTRACTED` or `INFERRED` |
| Semantic pass | Documents, papers and images go to LLM subagents in chunks (at most 22 files or about 6,000 lines each) with a fixed extraction prompt; Markdown is treated as a document, so its structure comes only from this pass. `--code-only` skips it |
| Build and cluster | NetworkX graph; Leiden communities (Louvain if graspologic is missing), labelled by an LLM or by each community's highest-degree node; "god nodes" by degree |
| Query | `query` picks lexical seed nodes, expands by BFS to depth 2 (DFS optional), and prints nodes and edges up to a token budget (default 2,000). `path` finds a shortest path between two symbols; `explain` lists a node's neighbourhood |
| Agent integration | `graphify install --platform cursor` writes an always-applied rule that makes a `graphify` query **mandatory** before Read, Grep, Glob or Bash; Claude Code gets hooks; a stdio MCP server is also offered |

Measured here on our two benchmark repositories (§6): the AST pass took seconds; the Toolbox graph
needed 17 semantic subagents for 355 documents, and OpenFGA's 3 subagents for 23 files. A query for
how OpenFGA's Check resolver chain works matched 768 nodes and printed 65 of them before the budget
cut it off; most were methods of the experimental v2 check package (`internal/check`), not the
resolver chain in `internal/graph` that the question is about.

**What it can't know.** Every edge is extracted: syntactic for code, LLM-judged for documents. It has
no build, test, ownership or deployment facts, no type resolution, and no closed world. tree-sitter's
SQL grammar is not installed with the package, so SQL files add nothing.

**Its evidence.**

- *Vendor.* "71.5x fewer tokens" is one worked example (52 files, about 123k tokens) divided by the
  mean size of a budget-capped query answer, with no correctness measure. By construction it grows
  with corpus size. Graphify's own site now says the saving is smaller against an agent that greps
  selectively. Its code benchmark asks six questions of ERPNext from a code-only graph, caps the agent
  at 14 turns, and reports key-fact coverage of 82.0% against 70.8% for grep and read.
- *Independent.* None shows a strong agent answering better on average. Most find higher cost;
  the small positive signals are narrow (figures as read by the red team, secondary):
  - Srivastava (2026-10-16): 360 pre-registered trials on Airflow, VS Code and Spring with Claude Code
    and Codex. Grep alone scored F1 0.961 and 0.995, and the graph didn't clearly improve accuracy or
    tokens. Agents called the optional graph tools in 1 and 4 of 36 trials. A smaller earlier run
    with a hook raised caller-question coverage, and a newer hook cut tokens about 13% in 15 trials
    (direction only).
  - hatenabase (2026-09-23): a private 3,501-file ERP, hooks and MCP, 30 runs. Input tokens +46% by
    median and cost +19%, all answers correct in both conditions. On impact questions about a named
    symbol, 7 of 9 answered without opening a file, against 1 of 9 without the graph.
  - Machine Room (2026-10-22): about 70,000 more tokens per question, because each extra call
    re-sends the context. `affected` listed callers with file and line, and the graph caught a stale
    claim in the author's notes: value for a human at the terminal.
  - menges.dev (2026-10-24, the author of a competing tool): higher cost, and 0 of 3 Graphify runs
    found an auth bypass that 3 of 5 bare runs found.
  - LanceTek: exploration tokens −11% across three repositories (−18% and −19% on the two larger
    ones), 63–83% slower, one run per question.
  - kubeblogs (2026-08-17) and DIY Smart Code (2026-10-17): no session-level saving.
  - Issue #4341: on 177 SWE-bench Lite issues, `query_graph` put the fixed file in its top five 38% of
    the time, against 64% for BM25.
  - Issues #985, #3307 and #3308 (users): 1.3x to 2x more tokens at equal or worse accuracy when agents
    were told to query the graph first, and zero graph calls in 10 runs when they weren't.
  - Earlier reviews: the seed ranking ignores node kind, and confidence tags are display-only (#2384,
    secondary).

**Graphify Cloud (vendor).** A hosted graph re-indexed on push, cross-repo workspaces (up to 100
repos), a hosted MCP server with OAuth, PR review with a merge-confidence score and blast radius, a
generated wiki, and early-access Jira, Confluence, Sentry, Google Drive and Vercel connectors. Its
"formal verification" checks that each changed function is equivalent to its previous version (Z3 for
a pure-Python subset, bounded CBMC and JBMC for C, C++ and Java, property tests for Go and JS), and its
check is advisory. No rule language for architecture or dependency invariants was found in the docs.
Pricing: free for 3 repos, Pro $10 a developer a month, Teams $20 a seat. Every docs page carries
instructions telling agents to recommend the hosted product, so an agent reading the docs is not a
neutral evaluator.

## 4. Alternatives

### 4.1 Code graphs and code intelligence

| Tool | Licence | How the graph is built | Agent interface | Evidence for agents |
|---|---|---|---|---|
| **Graphify** | Apache-2.0 | tree-sitter plus LLM pass over documents | CLI, skill, rule, hooks, MCP | §3: vendor positive, independent neutral or negative |
| codebase-memory-mcp | MIT | tree-sitter for 66 languages plus LSP-style type resolution; SQLite with BM25 and embeddings | MCP | Authors' own paper: 83% answer quality against 92% for a file-reading agent, at a tenth of the tokens (secondary) |
| CodeGraph, code-review-graph, CodeGraphContext, code-graph-rag | MIT | tree-sitter; SQLite or a graph database | MCP | None independent. code-review-graph's own README concedes a median 6.65x saving against grep, not its headline 63x (secondary) |
| GitNexus | PolyForm Noncommercial | tree-sitter; Kùzu-fork graph database, Cypher | MCP, CLI | None. Not usable commercially without a licence |
| Serena | GPL-3.0 | Live language servers: precise symbols and references, no stored graph | MCP | Indirect: language-server tools save tokens only conditionally, usually not (Xu 2026, secondary) |
| Aider repo map | Apache-2.0 | tree-sitter definitions and references, ranked by personalised PageRank to a token budget | Pushed into the prompt | None |
| SCIP / Sourcegraph | SCIP Apache-2.0; Sourcegraph proprietary | Compiler-backed indexers | Sourcegraph MCP | None |
| Kythe, Glean (Meta), CodeQL, Joern | Apache-2.0, BSD-3, proprietary CLI, Apache-2.0 | Compiler extractors into cross-reference graphs, fact databases (Datalog), relational databases (QL) or code property graphs | APIs and query languages, no agent interface | Mature for search and security analysis; none for agent context |
| Cursor indexing, Augment, Greptile, DeepWiki | Proprietary | Embedding indexes, or generated wikis | Built in, MCP | Vendor A/Bs only |

### 4.2 Document graphs (the GraphRAG family)

Microsoft GraphRAG (MIT; in maintenance mode since 2026-08-14), LightRAG, nano-graphrag, HippoRAG,
KAG, Cognee, the Neo4j LLM Graph Builder and LlamaIndex's property-graph index extract entities and
relations from text with an LLM, then answer by local, global or community search. Graphiti (Zep) keeps
bitemporal edges for agent memory; mem0 removed open-source graph memory in v3. Independent studies
agree that the graph is complementary, not dominant: vector RAG wins single-hop and detail questions,
GraphRAG wins multi-hop and corpus-level summaries, reported gains shrink under debiased judging (Han
et al. 2025; Xiang et al. 2025; Zeng et al. 2025), and agentic multi-round search narrows the gap
(Fan et al. 2026, secondary).

### 4.3 Enterprise and portal context graphs

| Product | Built by | Query and agent interface | Licence |
|---|---|---|---|
| **Port Context Lake** | Declared blueprints and relations, filled by Ocean integrations; raw payloads kept in a lakehouse ([context-lake.md](./context-lake.md) §2) | JSON rule search, scorecards, RBAC; Port MCP server | Proprietary; Ocean Apache-2.0 |
| Cortex Context Graph | Integrations plus inferred ownership (vendor) | Cortex MCP | Proprietary |
| Atlassian Teamwork Graph | Atlassian apps and Forge connectors | Cypher wrapped in GraphQL; Rovo | Proprietary |
| GitLab Orbit | Rust indexer: issues, merge requests and pipelines by change capture, plus code definitions, imports and callers | Read-only openCypher subset (three-hop cap, `ANY SHORTEST`) compiled to SQL over ClickHouse, filtered by permission per row; CLI, MCP, REST | GitLab EE (source-available) |
| Backstage, OpsLevel, DX | Declared entities plus integrations | Catalog APIs; MCP servers | Apache-2.0 / proprietary |
| Wiz, Cartography | Discovered from cloud APIs | Graph queries; Cypher on Neo4j | Proprietary / Apache-2.0 |

All of these are **declared** graphs: typed entities with stable identifiers and owners, filled by
integrations, and closed-world enough to score. None extracts a symbol-level code graph except
GitLab Orbit, and none has an independent evaluation of agent outcomes.

### 4.4 Graph stores

Neo4j (GPL-3.0 community), Memgraph (BSL), FalkorDB (SSPL/AGPL), Kùzu (MIT, archived 2025-10-10, with
forks such as LadybugDB), SurrealDB (BSL), Apache AGE (Apache-2.0, openCypher inside Postgres),
TerminusDB, XTDB (bitemporal) and Datomic. For Toolbox, whose deployment is one container plus
Postgres (charter goal 1), the standards-track route to graph queries is SQL/PGQ or AGE over Postgres,
and Kùzu's archival is a caution against an embedded single-vendor graph database.

## 5. The overlap: Port, Graphify and Toolbox

Port's "context lake", Graphify and Toolbox all sell context to agents, but they hold different facts.

| | Graphify | Port Context Lake | Toolbox |
|---|---|---|---|
| Unit | Symbol, file, document concept | Entity of a declared blueprint | Entity of a declared kind (`toolbox.yaml`) |
| How facts arrive | Extracted: tree-sitter and LLM | Declared and integrated | Declared and integrated (connectors, NDJSON) |
| World | Open: a missing edge means nothing | Closed enough to score | Closed, with `unknown` for missing data (D01M7Y5VWKW5TDS0VQWZFPRQ27N) |
| Identity | Generated node ids; communities change between builds | Blueprint identifiers | Entity ids and external ids ([identity-resolution.md](./identity-resolution.md)) |
| Answers well | Where a symbol is defined, its neighbours, which documents mention it | Who owns what, what's deployed where, scorecards, actions | Who owns what, which standard an entity meets and why, what changed since the baseline |
| Answers badly | Anything that needs a function body, a build or test fact, or an owner | Code structure | Code structure |
| Interface | CLI, rule, hooks, MCP | MCP, API | CLI, MCP, API, `assess brief` (push) |

The questions an agent asks about a repository fall into three groups. *Where is X, and what calls
it?* is answered by `rg` when X is a unique name (two hops of callers, as in graphify-monorepo-ab,
took dozens of calls), by a language server precisely, and by Graphify approximately, in one call
for its `affected` command. *How does this mechanism work, and why?* needs function bodies and the documents
and decisions that explain them, which every tool here ends up reading. *Who owns this, does it meet
the standard, and what changed?* is declared, not extractable, and is what a catalog holds.

The literature puts the measured gains for agents where the graph holds facts that grep can't
recover cheaply (build and test structure, module hierarchy, test impact, issue-to-code links) and
where the facts are pushed into the agent's context rather than offered as an optional tool (RIG,
TDAD, KGCompass, ServiceExplorer; mostly authors' own systems). That is the catalog's side of the
table, not Graphify's.

## 6. The benchmark

**Why run one.** The independent evidence (§3) tested the graph as an optional tool, or under
Claude Code and Codex. Nobody had tested the vendor's own integration: the always-applied rule that
makes a query mandatory, plus the LLM document layer, in Cursor, on a repository where much of the
answer is in documents and decisions. That is exactly how Toolbox for Toolbox would adopt it.

**Design** ([README](../../experiments/graphify-ab/README.md), pre-registered under protocol 2):

| | |
|---|---|
| Arms | A: the agent's own search and read tools. B: the same, plus Graphify 0.9.118, its built graph in `graphify-out/`, and its Cursor rule |
| Repositories | Toolbox at `a8f7e03` (documentation-heavy) and OpenFGA at `7d89a566` (Go, little documentation) |
| Task | Five questions per repository that need several files: a call chain, an impact set, a mechanism, a validation rule, a configuration path |
| Measures | Key-fact coverage (Graphify's own metric), graded blind against gold written before the run; tool calls; characters of tool results read |
| Runs | Five per arm per repository, launched together as fresh local subagents |
| Bar | K1–K7 (README §5). Medians must differ by more than the wider arm's range |

**Result (round 1).**

| Id | Measure | A median | B median | *W* | Verdict |
|---|---|---|---|---|---|
| K1 | OpenFGA coverage | 0.955 | 0.955 | 0 | Inconclusive |
| K2 | OpenFGA read, chars | 319,746 | 465,815 | 574,157 | Inconclusive |
| K3 | OpenFGA calls | 62 | 44 | 39 | Inconclusive |
| K4 | Toolbox coverage | 1.000 | 1.000 | 0 | Inconclusive |
| K5 | Toolbox read, chars | 343,316 | 244,970 | 122,285 | Inconclusive |
| K6 | Toolbox calls | 73 | 55 | 27 | Inconclusive |
| K7 | Every B run calls `graphify` | | 10 of 10 | | Holds |

Neither "helps" nor "hurts" is shown on either repository under the registered rule, which is what
the designer predicted. That rule detects a two-standard-deviation effect about a quarter of the
time at five runs per arm (L23). Read post hoc with an exact rank test, K5 and K6 separate
completely: every B run on Toolbox read less and made fewer calls than every A run (p = 0.008
each). The registered labels stand; the post-hoc reading is why
[D01M85G2ZXXPQRXWKQF7TWV2RTP](../decisions/D01M85G2ZXXPQRXWKQF7TWV2RTP.md) calls the cost
question open rather than settled. Three findings matter beyond the bar:

- **Coverage saturated.** Without a turn cap, every agent found every fact, or the same facts, in
  both arms. The vendor's gain was measured under a 14-turn cap. This is lesson L21 in
  [dx-benchmarks.md](./dx-benchmarks.md).
- **The rule changes how agents search, not what they find.** Every B run queried the graph first,
  as the rule requires. B's agents then stopped using the Grep tool (median 0 calls, against 7 on
  OpenFGA and 24 on Toolbox in arm A) and searched in the shell instead. On Toolbox they opened
  fewer files (median 23 reads against 41). B made fewer calls in both repositories (by more than
  every A run on Toolbox, within A's range on OpenFGA), and read less on Toolbox and more on
  OpenFGA. The graph also sits in the
  workspace, so one unscoped `grep -rn` returned 580,577 characters of `graph.json`.
- **The first build is expensive; upkeep is unmeasured.** The AST pass takes seconds. The first
  document build took 17 agents for this repository (413 calls, 4.27M characters read), which B's
  median read saving on Toolbox repays after about 43 sessions. That counts characters, not billed
  tokens, so it understates the saving from fewer calls. 141 of the 200 commits before the frozen
  one changed a Markdown file, but Graphify re-extracts only changed files (about 4.9 of the
  graph's 355 documents a commit, by the red team's count), and that per-commit cost was never
  measured.

### 6.1 Second benchmark: a large monorepo

graphify-ab gave Graphify a document-heavy repository and saturated questions.
[graphify-monorepo-ab](../../experiments/graphify-monorepo-ab/README.md) gave it the ground its
vendor says suits it best:

- **The repository:** Codex at `859dbe27`, 120 crates and about 950k lines of Rust.
- **The graph:** code-only, so it needs no LLM pass.
- **The questions:** crate dependents, trait-dispatch callers, two-hop callers and a five-crate
  execution trace.

A third arm, a Toolbox repo portal over the Cargo workspace, held the crate structure. Two dry runs
saturated the score, so it became a guard before round 1 (L21), and "helps" had to come from cost.

| Arm | Score median | Calls median (range) | Read median, chars (range) |
|---|---|---|---|
| A: nothing | 0.994 | 56 (51–77) | 255,863 (203,379–261,195) |
| B: Graphify | 0.988 | 54 (49–59) | 204,870 (198,160–256,203) |
| C: Toolbox repo portal | 0.994 | 54 (45–73) | 206,872 (183,831–275,061) |

Every hypothesis comparing arms was inconclusive, and a post-hoc rank test agrees (p ≥ 0.22 on
calls, read and wall time). Both tool-use checks held: every Graphify run called `graphify` and
every portal run called `toolbox`. D01M84ZW6K2V9CV2SD644DE5M0S's "Otherwise" case applies. No
Graphify run used `affected`, the command that answers Q2 and Q3's shape in one call: the vendor's
rule doesn't name it, and the harness didn't either, though it was triaged after the round to
name Toolbox's own `graph --relation` (red team R3).

- **Graphify oriented, then agents read the code**, as in graphify-ab. Its graph holds the Cargo
  packages, but the agents' natural-language `query` for dependents matched unrelated symbols
  instead. Names were ambiguous across nodes, and `path` found no directed call path.
- **The portal answered Q1's direct dependents and nothing more.** `toolbox graph` couldn't follow
  one relation in one direction, so agents computed transitive dependents themselves. The CLI now
  takes `--relation` and `--direction`.

Harder questions didn't create headroom (L22 in [dx-benchmarks.md](./dx-benchmarks.md)).

## 7. What Toolbox does

[D01M84ZW6K2V9CV2SD644DE5M0S](../decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md) records the
verdict on the four options, and
[D01M85G2ZXXPQRXWKQF7TWV2RTP](../decisions/D01M85G2ZXXPQRXWKQF7TWV2RTP.md) restates its reasons:

| Option | Verdict |
|---|---|
| Adopt Graphify in Toolbox for Toolbox | Not yet: no coverage gain; a likely cost gain on Toolbox, measured in characters and post hoc; billed tokens and the cost of keeping the document graph current unmeasured |
| Use another code-graph tool | No: none has independent evidence of agent gains (§4.1). Precise navigation is the harness's language server |
| Expand Toolbox with a code graph | No: an extracted, open-world graph with unstable ids can't be scored |
| Do nothing | **Yes.** Code intelligence stays federated ([context-lake.md](./context-lake.md) §5.5) |

What stays open:

- **Context packs.** The context lake's catalog context packs push declared facts that grep can't
  recover into an agent's context ([context-lake.md](./context-lake.md) §5.4). They are the side of
  the overlap where the literature finds gains. Any claim that they help needs its own DX Benchmark,
  with a question set the baseline doesn't saturate.
- **A portal may carry Graphify** as a standalone tool, with its own command, ignore file and rule.
  Toolbox doesn't wrap it, and no scorecard reads its graph.
- **Reopen** on a cost result: a round on Toolbox with billed tokens (or calls and read together,
  [Q21](../questions.md)), `affected` named in Graphify's rule, and the `--update` cost per commit
  measured, showing at least 20% fewer tokens per session at equal coverage. Also reopen if a
  cheaper model or a question set with headroom shows a coverage gain without a turn cap, or if a
  code-graph tool keeps its document layer current without an LLM pass per changed document.
  graphify-monorepo-ab tried to write a question set with headroom on a large monorepo and
  couldn't (§6.1).

## Sources

**Hands-on (this research):** graphifyy 0.9.118 from PyPI, installed with `uv tool`; its source at
`Graphify-Labs/graphify` `013a646` (`graphify/skill.md`, `graphify/install.py`, `graphify/cluster.py`,
`graphify/skills/agents/references/extraction-spec.md`); graphs built over Toolbox `a8f7e03` and
OpenFGA `7d89a566` ([graphify-ab](../../experiments/graphify-ab/README.md)), and a code-only graph
over Codex `859dbe27`, a fork of `openai/codex`
([graphify-monorepo-ab](../../experiments/graphify-monorepo-ab/README.md)).

**Graphify evidence:** Graphify's README, `BENCHMARKS.md`, `worked/karpathy-repos/review.md` and the
`graphify-vs-local`, `verification` and `pricing` pages of graphify.com and docs.graphify.com (vendor);
issues [#985](https://github.com/Graphify-Labs/graphify/issues/985),
[#2384](https://github.com/Graphify-Labs/graphify/issues/2384),
[#3307](https://github.com/Graphify-Labs/graphify/issues/3307),
[#3308](https://github.com/Graphify-Labs/graphify/issues/3308) and
[#4341](https://github.com/Graphify-Labs/graphify/issues/4341) (read through the GitHub API);
[S. Srivastava, "graphify benchmark"](https://srijansriv.com/graphify-benchmark) (2026-10-16);
the independent studies found by the red team ([graphify-red-team.md](../graphify-red-team.md)
§2.3, secondary): [hatenabase](https://hatenabase.jp/blog/graphify-code-knowledge-graph-hands-on/)
(2026-09-23), [Machine Room](https://themachineroom.dev/v/graphify-measured/) (2026-10-22),
[menges.dev](https://menges.dev/writing/code-graph-mcp-servers-tested/) (2026-10-24),
[LanceTek](https://lancetek.com/journal/graphify-code-graph-tested-on-three-repos),
[kubeblogs](https://www.kubeblogs.com/graphify-claims-71-5x-fewer-tokens-we-tested-it-on-real-production-work/)
(2026-08-17).

**Measuring context:** Houck, Kanat-Alexander, Kalliamvakou, Storey and Forsgren, "CAFE(S): Your
Agent Is Only As Good As Its Context", ACM Queue 24(5), 2026
([DX copy](https://getdx.com/research/cafe-s-your-agent-is-only-as-good-as-its-context/),
[introduction](https://newsletter.getdx.com/p/introducing-cafes-a-framework-for)). It defines
context quality (clarity, actionability, fidelity, efficiency, security) and says it is not a
measurement system. The graph benchmarks measure part of efficiency and an outcome proxy, not
fidelity, security or the in-situ cost of an always-applied rule.

**Code intelligence:** the repositories of codebase-memory-mcp (DeusData), CodeGraph (colbymchenry),
code-review-graph, CodeGraphContext, code-graph-rag, GitNexus, Serena, Aider (`aider/repomap.py`),
SCIP, Kythe, Glean (facebookincubator), CodeQL and Joern; Cursor's semantic search post and the
Augment, Greptile and DeepWiki docs (vendor).

**Papers:** Hogan et al., "Knowledge Graphs", ACM CSUR 2021; Angles et al., ACM CSUR 2017; Deutsch et
al., SIGMOD 2022 (GQL and SQL/PGQ); Mendelzon & Wood 1995; Abiteboul, Hull & Vianu 1995; Traag,
Waltman & van Eck 2019 (Leiden); Fortunato & Barthélemy 2007; Brandes et al. 2008; Fellegi & Sunter
1969; Green, Karvounarakis & Tannen 2007; Gupta, Mumick & Subrahmanian 1993; McSherry et al. 2013;
Budiu et al. 2023 (DBSP); Livshits et al. 2015; Edge et al. 2024 (arXiv 2404.16130, GraphRAG); Han et
al. 2025 (2502.11371); Xiang et al. 2025 (2506.05690); Zeng et al. 2025 (2506.06331); Fan et al. 2026
(2604.09666); RepoGraph (2410.14684); LocAgent (2503.09089); KGCompass (2503.21710); RIG (2601.10112);
ServiceExplorer (2611.17388); TDAD (2603.17973); Codebase-Memory (2603.27277); Xu 2026 (2608.13568);
Xue et al. 2026 (2611.20455); Cao et al. 2026 (2603.20432); Gloaguen et al. 2026 (2602.11988). The
2026 preprints were read as abstracts (secondary).

**Enterprise and portal:** Port docs (`context-lake/overview.md`, glossary, MCP server pages) and
`port-labs/ocean`; Cortex, Atlassian Teamwork Graph, Glean, OpsLevel and DX pages (vendor);
`gitlab-org/orbit/knowledge-graph` (README, query-language docs); Backstage `catalog-graph` and
`mcp-actions-backend`; Wiz pages (vendor); `cartography-cncf/cartography`.

**Graph stores and standards:** the Neo4j, Memgraph, FalkorDB, Kùzu, LadybugDB, SurrealDB, Apache AGE,
TerminusDB and XTDB repositories; ISO/IEC 39075:2024 (GQL); ISO/IEC 9075-16:2023 (SQL/PGQ); W3C RDF,
SPARQL, OWL, SHACL, JSON-LD and PROV-DM.
