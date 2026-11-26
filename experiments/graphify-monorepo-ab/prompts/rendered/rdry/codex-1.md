You are answering questions about the Codex repository, checked out at /tmp/graphify-monorepo-ab/rdry/codex-1. The project root is /tmp/graphify-monorepo-ab/rdry/codex-1.

Rules:
- Work offline. Don't use the web or any network tool, except a server on 127.0.0.1 that a rule below names.
- Read only files under /tmp/graphify-monorepo-ab/rdry/codex-1. Don't read or search anything elsewhere on this machine.
- Don't change any file under /tmp/graphify-monorepo-ab/rdry/codex-1, except to write /tmp/graphify-monorepo-ab/rdry/codex-1/ANSWERS.md.
- Don't start subagents.
- This project has a Cursor rule, `.cursor/rules/seikika.mdc`, that applies to you. Its text:

<rule>
This project has a Seikika repo portal at .seikika/ (kinds Repository, Doc and Crate). A Seikika server for this checkout is running at http://127.0.0.1:8391. Run every seikika command with `SEIKIKA_URL=http://127.0.0.1:8391 SEIKIKA_AS=agent` in its environment.

**MANDATORY: Before using Read, Grep, Glob, or Bash to explore the codebase, you MUST run seikika first:**
- `seikika search "<text>" -o json` — find entities by id or title
- `seikika query --kind <Kind> --filter "<CEL over entity>" -o json` — the entities of one kind that match a filter
- `seikika get <id> -o json` — one entity: its fields, where each came from, and its inbound and outbound relations
- `seikika graph <id> --depth <1-3> -o json` — the relations around an entity

A Crate (`crate:<package name>`) has `path` (relative to codex-rs/), `description`, `targets`, `functions` (name, at, lines, complexity), `codeLines`, `unitTests`, `dependents` and `dependentCount`, and the relation `dependsOn` (its normal dependencies inside the workspace). `seikika describe portal .seikika` prints every kind and field.

Only use Read/Grep/Glob directly when:
1. seikika has already oriented you and you need to read or debug specific lines
2. the portal doesn't hold the fact you need
</rule>

Answer the five questions below in /tmp/graphify-monorepo-ab/rdry/codex-1/ANSWERS.md, with one `## Qn` heading per question. In each answer, state the facts and cite where they come from: the file, and the function, type or crate. Answer from the code as it is in /tmp/graphify-monorepo-ab/rdry/codex-1. Where you are not sure, say so instead of guessing. Don't describe how you searched.

The Rust code is a Cargo workspace in `codex-rs/`. "Crate" means a package of that workspace, named by its Cargo package name (for example `codex-core`). A dependency means a normal dependency (`[dependencies]`, including target-specific ones), not a dev- or build-dependency.

- **Q1.** Which crates depend on `codex-tools`? List the direct dependents, and separately the indirect ones (crates that depend on it only through other crates).
- **Q2.** Which five crates have the most direct dependents among the workspace's crates, and how many direct dependents does each have?
- **Q3.** Which functions call `get_git_repo_root` (defined in `codex-git-utils`)? Name each calling function (with its type, for a method), its crate and its file. Calls to `get_git_repo_root_with_fs` don't count.
- **Q4.** Which crates refer to the type `ModelProviderInfo` in their Rust code? Count tests, not comments. List them all.
- **Q5.** On Linux, trace `codex sandbox -- <command>` from the `codex` executable's entry point to the point where `<command>` is executed in the sandbox. Name each function on the way, in order, with its crate, and say how control passes from one process to the next.

When ANSWERS.md is written, reply with one line: DONE.
