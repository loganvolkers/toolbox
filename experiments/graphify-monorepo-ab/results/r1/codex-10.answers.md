# Answers

## Q1

Dependencies are normal dependencies only, taken from each crate's `Cargo.toml` in `codex-rs/`. The Seikika portal's `crate:codex-tools` entity lists the same seven direct dependents in its `dependents` field.

**Direct dependents (7).** Each lists `codex-tools` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-core` | `core/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents (22).** These reach `codex-tools` only through other crates. One example path is given for each:

| Crate | Manifest | Example path |
|---|---|---|
| `app_test_support` | `app-server/tests/common/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-app-server-client` | `app-server-client/Cargo.toml` | → `codex-app-server` → `codex-tools` |
| `codex-app-server-daemon` | `app-server-daemon/Cargo.toml` | → `codex-app-server-transport` → `codex-core` → `codex-tools` |
| `codex-app-server-test-client` | `app-server-test-client/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-app-server-transport` | `app-server-transport/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-chatgpt` | `chatgpt/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-cli` | `cli/Cargo.toml` | → `codex-app-server` → `codex-tools` |
| `codex-cloud-config` | `cloud-config/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-cloud-tasks` | `cloud-tasks/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-core-api` | `core-api/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-exec` | `exec/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-guardian` | `ext/guardian/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-lmstudio` | `lmstudio/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-mcp-server` | `mcp-server/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-memories-write` | `memories/write/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-ollama` | `ollama/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-thread-manager-sample` | `thread-manager-sample/Cargo.toml` | → `codex-core-api` → `codex-core` → `codex-tools` |
| `codex-tui` | `tui/Cargo.toml` | → `codex-app-server-client` → `codex-app-server` → `codex-tools` |
| `codex-utils-oss` | `utils/oss/Cargo.toml` | → `codex-core` → `codex-tools` |
| `codex-utils-sandbox-summary` | `utils/sandbox-summary/Cargo.toml` | → `codex-core` → `codex-tools` |
| `core_test_support` | `core/tests/common/Cargo.toml` | → `codex-core` → `codex-tools` |
| `mcp_test_support` | `mcp-server/tests/common/Cargo.toml` | → `codex-mcp-server` → `codex-core` → `codex-tools` |

`app_test_support`, `core_test_support` and `mcp_test_support` are workspace packages that exist to support tests, but they reach `codex-tools` through normal dependencies, so they are included.

## Q2

The trait method is `ThreadStore::read_thread(&self, params: ReadThreadParams)`, defined in `thread-store/src/store.rs`. Two types implement it: `LocalThreadStore` in `thread-store/src/local/mod.rs` and `InMemoryThreadStore` in `thread-store/src/in_memory.rs`.

No generic `T: ThreadStore` callers exist. Every non-test call goes through an `Arc<dyn ThreadStore>`.

These non-test functions call the trait method:

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `thread_title_from_thread_store` (free function) | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>`, on its `None` branch |
| `run_turn_stop_hooks` (free function) | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`Arc<dyn ThreadStore>`, from `SessionServices` in `core/src/state/service.rs`) |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` (two calls) |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` | `codex-app-server` | same file | same (two calls) |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same file | same |

These look similar but do not count:

- **Calls to other `read_thread` methods.** These take two booleans and call `LiveThread::read_thread` or `CodexThread::read_thread`, not the trait method:
  - `CodexThread::read_thread` in `core/src/codex_thread.rs`
  - `ThreadManager::spawn_subagent` in `core/src/thread_manager.rs`
  - the `Some` branch of `thread_title_from_thread_store`
  - `TurnRequestProcessor::start_detached_review` in `app-server/src/request_processors/turn_processor.rs`
  - `apply_bespoke_event_handling` in `app-server/src/bespoke_event_handling.rs`
- **The free function `read_thread::read_thread`** in `thread-store/src/local/read_thread.rs`. `LocalThreadStore`'s trait implementation and several functions in `local/update_thread_metadata.rs` and `local/live_writer.rs` call it, but it is not the trait method.
- **Test functions.** All the remaining `.read_thread(ReadThreadParams…)` calls are inside `#[cfg(test)] mod tests` blocks in `thread-store/src/local/{read_thread,mod,update_thread_metadata}.rs`.

## Q3

`get_git_repo_root` is defined in `git-utils/src/info.rs` (`codex-git-utils`). These non-test functions call it directly and serve as the intermediates:

| Intermediate | Crate | File |
|---|---|---|
| `git_diff_to_remote` | `codex-git-utils` | `git-utils/src/info.rs` |
| `git_check` | `codex-cli` | `cli/src/doctor/git.rs` |
| `terminal_title_project_root` | `codex-cli` | `cli/src/doctor/title.rs` |
| `write_session_meta` | `codex-rollout` | `rollout/src/recorder.rs` |
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics` | `analytics/src/reducer.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store` | `thread-store/src/thread_metadata_sync.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` |
| `run_exec_session` | `codex-exec` | `exec/src/lib.rs` |
| `run_turn` | `codex-core` | `core/src/session/turn.rs` |
| `Session::new` | `codex-core` | `core/src/session/session.rs` |
| `build_turn_metadata_header` | `codex-core` | `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core` | `core/src/turn_metadata.rs` |
| `display_path_for` | `codex-tui` | `tui/src/diff_render.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` |
| `environment_id_from_cwd` | `codex-secrets` | `secrets/src/lib.rs` |

These non-test functions reach `get_git_repo_root` through exactly one of those intermediates. None of them also calls `get_git_repo_root` directly:

| Function | Crate | File | Through |
|---|---|---|---|
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `git_diff_to_remote` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `RegularTask::run` (the `SessionTask` impl) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::display_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for`, directly in its body |
| `reformat_description` (a nested fn inside `display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |

`ExternalAgentConfigMigrationScreen::display_description` also reaches it at two hops, through `reformat_description`.

Excluded:

- `environment_id_from_cwd` (`codex-secrets`) is only called by the test `environment_id_fallback_has_cwd_prefix`.
- The remaining callers of `git_diff_to_remote`, `build_turn_metadata_header`, `TurnMetadataState::new`, `Session::new` and `display_path_for` are tests:
  - `core/src/git_info_tests.rs`
  - `core/src/turn_metadata_tests.rs`
  - `core/src/session/tests.rs`
  - the `diff_render.rs` tests module
- `run_turn` uses `get_git_repo_root_with_fs` on one branch, which doesn't count. It also calls `get_git_repo_root` on its `None` branch, which is why it is an intermediate.

## Q4

This traces a function call to `exec_command` from the model stream to process spawn on Linux, with a local (non-remote) environment. The default unified-exec shell mode is assumed, not ZshFork.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`). Its `match event` has a `ResponseEvent::OutputItemDone(item)` arm that receives the completed `ResponseItem::FunctionCall`. It builds a `HandleOutputCtx` and calls `handle_output_item_done`.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`).
   - It calls `ToolRouter::build_tool_call` (`core/src/tools/router.rs`), which turns the `FunctionCall` into a `ToolCall { tool_name: "exec_command", payload: ToolPayload::Function { arguments } }`.
   - It records the item, then calls `ToolCallRuntime::handle_tool_call` on `ctx.tool_runtime` and returns the boxed future as `tool_future`.
   - Back in `try_run_sampling_request`, that future is pushed onto the `in_flight` `FuturesOrdered`. `drain_in_flight` in `turn.rs` awaits it later.
3. **`ToolCallRuntime::handle_tool_call`**, then **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` calls `tokio::spawn(async move { … })`, wrapped in `AbortOnDropHandle`. The task takes the parallel-execution `RwLock`; it takes a read lock here, because `ExecCommandHandler::supports_parallel_tool_calls` is true.
   - The task then calls the router. The outer future selects between the task's join handle and cancellation.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`). These build a `ToolInvocation` and call the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`).
   - `ToolRegistry::tool` looks the tool up as an `Arc<dyn CoreToolRuntime>`.
   - It runs pre-tool-use hooks, then calls `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`). That method is generic over a closure `F: FnOnce() -> Fut`, and the closure calls `handle_any_tool`.
6. **`handle_any_tool`** (`codex-core`, `core/src/tools/registry.rs`). **Trait object:** it calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. `CoreToolRuntime` is a supertrait over `ToolExecutor<ToolInvocation>`, which is defined in `codex-tools` (`tools/src/tool_executor.rs`).
7. **`ExecCommandHandler::handle`** (the `ToolExecutor<ToolInvocation>` impl; `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - It parses the arguments and resolves the turn environment and cwd.
   - It builds the shell argv with `get_command`, applies the permission and approval checks, and tries `intercept_apply_patch`.
   - It then calls `UnifiedExecProcessManager::exec_command` on `session.services.unified_exec_manager`.
8. **`UnifiedExecProcessManager::exec_command`**, then **`UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`).
   - `open_session_with_sandbox` builds the env (`create_env`, `apply_unified_exec_env`) and the exec-policy approval requirement.
   - It also builds a `UnifiedExecToolRequest`, a `ToolOrchestrator` and a `UnifiedExecRuntime`, then calls `ToolOrchestrator::run`.
9. **`ToolOrchestrator::run`**, then **`ToolOrchestrator::run_attempt`** (`codex-core`, `core/src/tools/orchestrator.rs`).
   - **Generic:** both are generic over `T: ToolRuntime<Rq, Out>`, a trait in `core/src/tools/sandboxing.rs`. Here `T` is `UnifiedExecRuntime`.
   - `run` handles approval, then picks the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, when the policy requires a platform sandbox, this is `SandboxType::LinuxSeccomp`.
   - It builds a `SandboxAttempt`, and `run_attempt` calls `tool.run(req, attempt, ctx)`. On a sandbox denial, `run` may retry with a different `SandboxAttempt`.
10. **`UnifiedExecRuntime::run`** (the `ToolRuntime` impl; `codex-core`, `core/src/tools/runtimes/unified_exec.rs`). The command is wrapped here.
    - First it adjusts the shell command. For a local environment, `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`) can wrap a `shell -lc` command with the shell snapshot. The PowerShell adjustments don't apply on Linux.
    - Then `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`) makes a `SandboxCommand`.
    - **Sandbox wrapping:** `SandboxAttempt::env_for` (`core/src/tools/sandboxing.rs`) calls `SandboxManager::transform` (`codex-sandboxing`, `sandboxing/src/manager.rs`). For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`).
      - It then prepends the `codex-linux-sandbox` executable to the argv and sets `arg0` via `linux_sandbox_arg0_override`.
      - With `SandboxType::None`, the argv is unchanged.
    - `ExecRequest::from_sandbox_exec_request` turns the result into an `ExecRequest`. `run` then calls `UnifiedExecProcessManager::open_session_with_exec_env` with `NoopSpawnLifecycle`.
11. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`).
    - The Windows branch is compiled out on Linux.
    - Because `environment.is_remote()` is false, it doesn't use the exec-server backend. It splits `request.command` into program and args and spawns the process directly through `codex-utils-pty`, choosing the function by `tty`:
      - with `tty: false`, the `exec_command` default, it calls **`codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`**, then **`spawn_process_with_stdin_mode`** (`utils/pty/src/pipe.rs`);
      - with `tty: true`, it calls **`codex_utils_pty::pty::spawn_process_with_inherited_fds`**, then **`spawn_process_portable`** (`utils/pty/src/pty.rs`). It would go to `spawn_process_preserving_fds` instead only if inherited fds were present, which they are not with `NoopSpawnLifecycle`, as far as the code shows.
12. **The spawn point.**
    - On the pipe path, `spawn_process_with_stdin_mode` builds a `tokio::process::Command`, installs a `pre_exec` hook (`detach_from_tty`, `set_parent_death_signal` on Linux, `close_inherited_fds_except`) and calls `command.spawn()`.
    - On the PTY path, `spawn_process_portable` calls `pair.slave.spawn_command(command_builder)` (portable-pty).
    - The spawned program is the sandbox wrapper (`codex-linux-sandbox`, which applies bubblewrap or Landlock and seccomp) when the sandbox is on, or the command itself otherwise.
    - Control then returns through `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`).

I'm less sure about one detail: the internals of the `codex-linux-sandbox` helper after spawn (`codex-linux-sandbox` crate, `linux-sandbox/`) are outside this trace and were not followed.
