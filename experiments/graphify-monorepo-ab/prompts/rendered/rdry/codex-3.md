You are answering questions about the Codex repository, checked out at /tmp/graphify-monorepo-ab/rdry/codex-3. The project root is /tmp/graphify-monorepo-ab/rdry/codex-3.

Rules:
- Work offline. Don't use the web or any network tool, except a server on 127.0.0.1 that a rule below names.
- Read only files under /tmp/graphify-monorepo-ab/rdry/codex-3. Don't read or search anything elsewhere on this machine.
- Don't change any file under /tmp/graphify-monorepo-ab/rdry/codex-3, except to write /tmp/graphify-monorepo-ab/rdry/codex-3/ANSWERS.md.
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

Answer the five questions below in /tmp/graphify-monorepo-ab/rdry/codex-3/ANSWERS.md, with one `## Qn` heading per question. In each answer, state the facts and cite where they come from: the file, and the function, type or crate. Answer from the code as it is in /tmp/graphify-monorepo-ab/rdry/codex-3. Where you are not sure, say so instead of guessing. Don't describe how you searched.

The Rust code is a Cargo workspace in `codex-rs/`. "Crate" means a package of that workspace, named by its Cargo package name (for example `codex-core`). A dependency means a normal dependency (`[dependencies]`, including target-specific ones), not a dev- or build-dependency.

- **Q1.** Which crates depend on `codex-tools`? List the direct dependents, and separately the indirect ones (crates that depend on it only through other crates).
- **Q2.** Which five crates have the most direct dependents among the workspace's crates, and how many direct dependents does each have?
- **Q3.** Which functions call `get_git_repo_root` (defined in `codex-git-utils`)? Name each calling function (with its type, for a method), its crate and its file. Calls to `get_git_repo_root_with_fs` don't count.
- **Q4.** Which crates refer to the type `ModelProviderInfo` in their Rust code? Count tests, not comments. List them all.
- **Q5.** On Linux, trace `codex sandbox -- <command>` from the `codex` executable's entry point to the point where `<command>` is executed in the sandbox. Name each function on the way, in order, with its crate, and say how control passes from one process to the next.

When ANSWERS.md is written, reply with one line: DONE.
