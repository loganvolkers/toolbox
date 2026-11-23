The Rust code is a Cargo workspace in `codex-rs/`. "Crate" means a package of that workspace, named by its Cargo package name (for example `codex-core`). A dependency means a normal dependency (`[dependencies]`, including target-specific ones), not a dev- or build-dependency.

- **Q1.** Which crates depend on `codex-tools`? List the direct dependents, and separately the indirect ones (crates that depend on it only through other crates).
- **Q2.** Which five crates have the most direct dependents among the workspace's crates, and how many direct dependents does each have?
- **Q3.** Which functions call `get_git_repo_root` (defined in `codex-git-utils`)? Name each calling function (with its type, for a method), its crate and its file. Calls to `get_git_repo_root_with_fs` don't count.
- **Q4.** Which crates refer to the type `ModelProviderInfo` in their Rust code? Count tests, not comments. List them all.
- **Q5.** On Linux, trace `codex sandbox -- <command>` from the `codex` executable's entry point to the point where `<command>` is executed in the sandbox. Name each function on the way, in order, with its crate, and say how control passes from one process to the next.
