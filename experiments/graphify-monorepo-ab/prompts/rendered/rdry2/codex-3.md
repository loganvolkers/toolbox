You are answering questions about the Codex repository, checked out at /tmp/graphify-monorepo-ab/rdry2/codex-3. The project root is /tmp/graphify-monorepo-ab/rdry2/codex-3.

Rules:
- Work offline. Don't use the web or any network tool, except a server on 127.0.0.1 that a rule below names.
- Read only files under /tmp/graphify-monorepo-ab/rdry2/codex-3. Don't read or search anything elsewhere on this machine, except your own scratch files.
- Don't change any file under /tmp/graphify-monorepo-ab/rdry2/codex-3, except to write /tmp/graphify-monorepo-ab/rdry2/codex-3/ANSWERS.md. Put any scratch files in /tmp/graphify-monorepo-ab/scratch/rdry2/codex-3, and nowhere else.
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

Answer the four questions below in /tmp/graphify-monorepo-ab/rdry2/codex-3/ANSWERS.md, with one `## Qn` heading per question. In each answer, state the facts and cite where they come from: the file, and the function, type or crate. Answer from the code as it is in /tmp/graphify-monorepo-ab/rdry2/codex-3. Where you are not sure, say so instead of guessing. Don't describe how you searched.

The Rust code is a Cargo workspace in `codex-rs/`. "Crate" means a package of that workspace, named by its Cargo package name (for example `codex-core`). A dependency means a normal dependency (`[dependencies]`, including target-specific ones), not a dev- or build-dependency. A test function is a function with a test attribute (such as `#[test]` or `#[tokio::test]`), or any function in a `#[cfg(test)]` module, a `tests/` directory or a `*_tests.rs` file. Name a function by its name, with its type for a method, and give its crate and file.

- **Q1.** Which crates depend on `codex-tools`? List the direct dependents, and separately the indirect ones (crates that depend on it only through other crates).
- **Q2.** `ThreadStore` (in `codex-thread-store`) is a trait with a method `read_thread`. Which functions call that trait method, whether through a `dyn ThreadStore`, a generic or an implementing type? Other methods and functions that are also named `read_thread` don't count. Don't count test functions.
- **Q3.** Which functions call `get_git_repo_root` (defined in `codex-git-utils`) through exactly one intermediate function? That is, each calls a function that calls `get_git_repo_root` directly, and doesn't call `get_git_repo_root` directly itself. Calls to `get_git_repo_root_with_fs` don't count. Don't count test functions, either as callers or as intermediates.
- **Q4.** On Linux, during a turn in a local session, the model returns a function call to the `exec_command` tool. Trace it from the function in `codex-core` that receives the completed response item from the model stream to the point where a process is spawned for the command. Name each function on the way, in order, with its crate. Say where control passes through a spawned task, a trait object or generic, and where the command is wrapped for the sandbox.

When ANSWERS.md is written, reply with one line: DONE.
