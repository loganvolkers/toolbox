# Answers

## Q1

`codex-tools` is the package at `codex-rs/tools` (`tools/Cargo.toml`, workspace entry `codex-tools = { path = "tools" }` in `codex-rs/Cargo.toml`). Only normal `[dependencies]` (including `[target.*.dependencies]`) of workspace members were counted; none of the edges below is marked `optional`.

**Direct dependents (7)**: each has `codex-tools = { workspace = true }` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-core` | `core/Cargo.toml` |
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents (18)**: these depend on `codex-tools` only through other crates. The column on the right lists their dependencies that themselves lead to `codex-tools`.

| Crate | Manifest | Via |
|---|---|---|
| `codex-app-server-client` | `app-server-client/Cargo.toml` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `app-server-daemon/Cargo.toml` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `app-server-test-client/Cargo.toml` | `codex-core` |
| `codex-app-server-transport` | `app-server-transport/Cargo.toml` | `codex-core` |
| `codex-cli` | `cli/Cargo.toml` | `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, `codex-cloud-tasks`, … |
| `codex-cloud-config` | `cloud-config/Cargo.toml` | `codex-core` |
| `codex-cloud-tasks` | `cloud-tasks/Cargo.toml` | `codex-core`, `codex-tui` |
| `codex-core-api` | `core-api/Cargo.toml` | `codex-core`, `codex-extension-api` |
| `codex-exec` | `exec/Cargo.toml` | `codex-core`, `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-guardian` | `ext/guardian/Cargo.toml` | `codex-core`, `codex-extension-api` |
| `codex-lmstudio` | `lmstudio/Cargo.toml` | `codex-core` |
| `codex-mcp-server` | `mcp-server/Cargo.toml` | `codex-core`, `codex-extension-api` |
| `codex-memories-write` | `memories/write/Cargo.toml` | `codex-core` |
| `codex-ollama` | `ollama/Cargo.toml` | `codex-core` |
| `codex-thread-manager-sample` | `thread-manager-sample/Cargo.toml` | `codex-core-api` |
| `codex-tui` | `tui/Cargo.toml` | `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-utils-oss` | `utils/oss/Cargo.toml` | `codex-core`, `codex-lmstudio`, `codex-ollama` |
| `codex-utils-sandbox-summary` | `utils/sandbox-summary/Cargo.toml` | `codex-core` |

## Q2

The trait method is `ThreadStore::read_thread` (`codex-rs/thread-store/src/store.rs`, crate `codex-thread-store`). Its implementations are `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`). Every non-test call goes through an `Arc<dyn ThreadStore>` (or `&Arc<dyn ThreadStore>`).

| Caller | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`Arc<dyn ThreadStore>`, `core/src/state/service.rs`) |
| `thread_title_from_thread_store` (only in the `None` branch; the `Some` branch calls `LiveThread::read_thread`) | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>` |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` (2 calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (2 calls) | `codex-app-server` | same | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same | same |

The following were excluded because they call a different function that is also named `read_thread`:
- `CodexThread::read_thread` (`core/src/codex_thread.rs`), and the functions that call it: the rollback handling in `app-server/src/bespoke_event_handling.rs`, `TurnRequestProcessor::start_detached_review` (`turn_processor.rs`) and `ThreadManager::spawn_subagent` (`thread_manager.rs`). `CodexThread::read_thread` calls `LiveThread::read_thread`, not the trait method.
- `LocalThreadStore::read_thread`, which calls the free function `local::read_thread::read_thread`. Other callers of that free function, such as those in `update_thread_metadata.rs`, were excluded for the same reason.

The remaining `.read_thread(...)` calls on a store are all in `#[cfg(test)]` modules: `thread-store/src/local/mod.rs`, `local/read_thread.rs` and `local/update_thread_metadata.rs`.

## Q3

`get_git_repo_root` is defined in `codex-rs/git-utils/src/info.rs` (crate `codex-git-utils`). Its direct non-test callers, which are the intermediates, are:

- `AnalyticsReducer::ingest_skill_invoked` (`codex-analytics`)
- `git_check` and `terminal_title_project_root` (`codex-cli`)
- `Session::new`, `run_turn`, `build_turn_metadata_header` and `TurnMetadataState::new` (`codex-core`)
- `discover_marketplace_paths_from_roots` (`codex-core-plugins`)
- `run_exec_session` (`codex-exec`)
- `git_diff_to_remote` (`codex-git-utils`)
- `write_session_meta` (`codex-rollout`)
- `environment_id_from_cwd` (`codex-secrets`)
- `ThreadMetadataSync::for_create` (`codex-thread-store`)
- `ChatWidget::status_line_project_root_for_cwd` and `display_path_for` (`codex-tui`)

The functions that call one of these but do not call `get_git_repo_root` directly are:

| Function | Crate | File | Intermediate |
|---|---|---|---|
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `AnalyticsReducer::ingest_skill_invoked` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` (`cli/src/doctor/git.rs`) |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` (`core/src/session/session.rs`) |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` (`core/src/session/turn.rs`) |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` (`core/src/turn_metadata.rs`) |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` (`thread_metadata_sync.rs`) |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` (`tui/src/diff_render.rs`) |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::reformat_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |

Notes:
- `environment_id_from_cwd` (`codex-secrets`) is called only from its `#[cfg(test)]` module, so it contributes no callers.
- `GitRequestProcessor::git_diff_to_remote` is a different function from `codex_git_utils::git_diff_to_remote`. It reaches `get_git_repo_root` only through two intermediates (`git_diff_to_origin`, then `codex_git_utils::git_diff_to_remote`), so it is not listed.
- `thread-manager-sample` defines its own local `run_turn`, which is a different function from `codex_core`'s `run_turn`.
- `run_turn` calls `get_git_repo_root` only in the branch where the turn has no primary environment. The other branch uses `get_git_repo_root_with_fs`, which doesn't count.

## Q4

This is the path for a local, non-remote environment on Linux, with the default (non-zsh-fork) unified-exec shell mode.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`). It reads the model stream from `ModelClientSession::stream`. On `ResponseEvent::OutputItemDone(item)` it builds a `HandleOutputCtx` and calls `handle_output_item_done`. It pushes the returned `tool_future` into its `FuturesOrdered` `in_flight` set.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`). It calls **`ToolRouter::build_tool_call`** (`core/src/tools/router.rs`), which turns a `ResponseItem::FunctionCall` into a `ToolCall` with `ToolPayload::Function { arguments }` and the tool name `exec_command`. It records the item, then calls `ToolCallRuntime::handle_tool_call`.
3. **`ToolCallRuntime::handle_tool_call`**, then **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` runs the dispatch in `tokio::spawn(async move { … })`, wrapped in an `AbortOnDropHandle`. Inside the task it takes the parallel-execution `RwLock` (read or write), then calls the router.
   - The returned future only `select!`s on that handle and on the cancellation token.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`). These build a `ToolInvocation` and call the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`).
   - It looks up the handler as an `Arc<dyn CoreToolRuntime>` and runs the pre-tool-use hooks.
   - It then calls `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`). That function is generic over a closure, and the closure calls `handle_any_tool`.
6. **`handle_any_tool`** (`codex-core`, `registry.rs`) calls `tool.handle(invocation)` on a **`&dyn CoreToolRuntime` trait object**.
   - `handle` belongs to the `ToolExecutor<ToolInvocation>` trait (`codex-tools`, `tools/src/tool_executor.rs`), and `CoreToolRuntime` is its subtrait.
   - If the tool's exposure was overridden, the call first passes through `ExposureOverride::handle` (`registry.rs`), which delegates to its inner `Arc<dyn CoreToolRuntime>`.
7. **`ExecCommandHandler::handle`** (impl `ToolExecutor<ToolInvocation>`; `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - It parses the arguments, resolves the turn environment and cwd, and allocates a process id.
   - It builds the shell argv with `get_command` (`core/src/tools/handlers/unified_exec.rs`) and checks `intercept_apply_patch`.
   - It then calls `UnifiedExecProcessManager::exec_command` with an `ExecCommandRequest`.
8. **`UnifiedExecProcessManager::exec_command`** (`codex-core`, `core/src/unified_exec/process_manager.rs`) calls `open_session_with_sandbox`.
9. **`UnifiedExecProcessManager::open_session_with_sandbox`** (same file).
   - It builds the environment (`create_env`, `apply_unified_exec_env`) and gets the exec approval requirement from `exec_policy.create_exec_approval_requirement_for_command`.
   - It creates a `UnifiedExecRuntime` and a `UnifiedExecToolRequest`, then calls `ToolOrchestrator::run`.
10. **`ToolOrchestrator::run`** (`codex-core`, `core/src/tools/orchestrator.rs`). It is **generic** over `T: ToolRuntime<Rq, Out>`, instantiated here with `UnifiedExecRuntime`.
    - It handles approval (`request_approval` / Guardian when needed).
    - It picks the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, when the policy requires a platform sandbox, this returns `SandboxType::LinuxSeccomp`.
    - It builds a `SandboxAttempt` and calls `ToolOrchestrator::run_attempt`. It retries without the sandbox (after approval) if the first attempt fails with a sandbox denial.
11. **`ToolOrchestrator::run_attempt`** (same file, also generic). It begins network approval, then calls `tool.run(req, attempt, ctx)` through the `ToolRuntime` trait (`core/src/tools/sandboxing.rs`).
12. **`UnifiedExecRuntime::run`** (impl `ToolRuntime`; `codex-core`, `core/src/tools/runtimes/unified_exec.rs`).
    - It wraps the command with `maybe_wrap_shell_lc_with_snapshot`, the shell-snapshot wrapper for local environments.
    - It calls `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`) to make a `SandboxCommand`.
    - It then calls `SandboxAttempt::env_for`.
13. **Sandbox wrapping:** **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`) calls **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`).
    - For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`).
    - It prepends the `codex-linux-sandbox` executable (`codex_linux_sandbox_exe`) to the argv and sets an `arg0` override (`linux_sandbox_arg0_override`).
    - For `SandboxType::None`, the argv passes through unchanged.
    - `ExecRequest::from_sandbox_exec_request` turns the result into core's `ExecRequest`.
14. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `process_manager.rs`), called with `NoopSpawnLifecycle`, so there are no inherited fds.
    - `environment.is_remote()` is false for a local session, so the remote exec-server branch (`get_exec_backend().start`) is skipped.
    - It splits the argv into program and args and spawns the process directly through `codex-utils-pty`:
      - **TTY requested:** `codex_utils_pty::pty::spawn_process_with_inherited_fds`, then `spawn_process_portable` (`utils/pty/src/pty.rs`). The process is spawned at `pair.slave.spawn_command(command_builder)` (portable-pty).
      - **No TTY:** `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`, then `spawn_process_with_stdin_mode` (`utils/pty/src/pipe.rs`). The process is spawned at `command.spawn()` on a `tokio::process::Command`.
    - The result is wrapped by `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`).

Other notes:
- When the shell mode is `UnifiedExecShellMode::ZshFork`, `UnifiedExecRuntime::run` first tries `zsh_fork_backend::maybe_prepare_unified_exec`. It still goes through `build_sandbox_command` and `env_for` (the same sandbox wrapping) before `open_session_with_exec_env`.
- After the spawn, `exec_command` starts output streaming (`start_streaming_output`) and stores the process (`store_process`, which starts `spawn_exit_watcher`). Those are background tasks that run after the process exists.
