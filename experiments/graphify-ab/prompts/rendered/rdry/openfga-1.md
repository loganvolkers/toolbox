You are answering questions about the OpenFGA repository, checked out at /tmp/graphify-ab/rdry/openfga-1. The project root is /tmp/graphify-ab/rdry/openfga-1.

Rules:
- Work offline. Don't use the web or any network tool.
- Read only files under /tmp/graphify-ab/rdry/openfga-1. Don't read or search anything elsewhere on this machine.
- Don't change any file under /tmp/graphify-ab/rdry/openfga-1, except to write /tmp/graphify-ab/rdry/openfga-1/ANSWERS.md.
- Don't start subagents.
- This project has a Cursor rule, `.cursor/rules/graphify.mdc`, that applies to you. Its text:

<rule>
This project has a graphify knowledge graph at graphify-out/.

**MANDATORY: Before using Read, Grep, Glob, or Bash to explore the codebase, you MUST run graphify first:**
- `graphify query "<question>"` — scoped subgraph for any codebase or architecture question
- `graphify path "<A>" "<B>"` — dependency path between two symbols
- `graphify explain "<concept>"` — all nodes related to a concept

This applies to YOU and to every subagent you spawn. Include this rule explicitly in every subagent prompt that involves code exploration. Do not skip graphify because files are "already known" or because you are executing a plan — the graph surfaces cross-file dependencies and INFERRED edges that grep and Read cannot find.

Only use Read/Grep/Glob directly when:
1. graphify has already oriented you and you need to modify or debug specific lines
2. `graphify-out/graph.json` does not exist yet

- If `graphify-out/wiki/index.md` exists, navigate it instead of reading raw files
- Read `graphify-out/GRAPH_REPORT.md` only for broad architecture review when query/path/explain do not surface enough context
- After modifying code files, run `graphify update .` to keep the graph current (AST-only, no API cost)
</rule>

Answer the five questions below in /tmp/graphify-ab/rdry/openfga-1/ANSWERS.md, with one `## Qn` heading per question. In each answer, state the facts and cite where they come from: the file, and the function, type or section. Answer from the code and docs as they are in /tmp/graphify-ab/rdry/openfga-1. Where you are not sure, say so instead of guessing. Don't describe how you searched.

- **Q1.** When a client calls the Check API, which resolvers does the request pass through before relationship tuples are read, in what order, and what decides whether each one is present? Name the types and the code that builds the chain.
- **Q2.** Which types implement the storage method `ReadUserTuple`, and which production code (not tests or mocks) calls it? If its signature changed, what else would have to change?
- **Q3.** After tuples are written, how does OpenFGA decide that a cached Check result is stale? Describe the mechanism, where it lives, and its limits.
- **Q4.** What constraints does OpenFGA place on the tupleset relation of a tuple-to-userset rewrite (`X from Y`), and where in the code are they enforced when a model is written?
- **Q5.** How does OpenFGA limit concurrent datastore reads during a single Check, and how does an operator configure that limit?

When ANSWERS.md is written, reply with one line: DONE.
