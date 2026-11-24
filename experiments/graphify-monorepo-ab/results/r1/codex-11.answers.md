# Answers

## Q1

The dependency edges come from the `[dependencies]` and `[target.*.dependencies]` tables of each workspace member's `Cargo.toml`. Dev- and build-dependencies are ignored. `codex-tools` is the package at `codex-rs/tools` (`codex-rs/tools/Cargo.toml`), and the workspace maps it as `codex-tools = { path = "tools" }` in `codex-rs/Cargo.toml`.

**Direct dependents (7).** Each lists `codex-tools = { workspace = true }` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-core` | `codex-rs/core/Cargo.toml` |
| `codex-app-server` | `codex-rs/app-server/Cargo.toml` |
| `codex-extension-api` | `codex-rs/ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `codex-rs/ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `codex-rs/ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `codex-rs/ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `codex-rs/ext/web-search/Cargo.toml` |

**Indirect dependents (18).** These depend on `codex-tools` only through other crates. The table names the dependents of `codex-tools` that each one depends on directly:

| Crate | Reaches `codex-tools` via |
|---|---|
| `codex-app-server-client` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `codex-core` |
| `codex-app-server-transport` | `codex-core` |
| `codex-cli` | `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, `codex-cloud-tasks`, `codex-memories-write`, `codex-app-server-daemon`, `codex-app-server-test-client` |
| `codex-cloud-config` | `codex-core` |
| `codex-cloud-tasks` | `codex-core`, `codex-tui` |
| `codex-core-api` | `codex-core`, `codex-extension-api` |
| `codex-exec` | `codex-core`, `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-guardian` | `codex-core`, `codex-extension-api` |
| `codex-lmstudio` | `codex-core` |
| `codex-mcp-server` | `codex-core`, `codex-extension-api` |
| `codex-memories-write` | `codex-core` |
| `codex-ollama` | `codex-core` |
| `codex-thread-manager-sample` | `codex-core-api` |
| `codex-tui` | `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-utils-oss` | `codex-core`, `codex-lmstudio`, `codex-ollama` |
| `codex-utils-sandbox-summary` | `codex-core` |

## Q2

The trait method is `ThreadStore::read_thread` in `codex-rs/thread-store/src/store.rs` (crate `codex-thread-store`). Its implementations are `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`). Every non-test caller below calls it through an `Arc<dyn ThreadStore>`. No non-test caller uses a generic or a concrete implementing type.

**`codex-thread-store`**
- `LiveThread::read_thread` in `thread-store/src/live_thread.rs`, through `self.thread_store`, an `Arc<dyn ThreadStore>`.
  - `LocalThreadStore::read_thread` and `load_history` call the free function `local::read_thread::read_thread`, not the trait method, so they don't count. The same goes for `update_thread_metadata.rs` and `live_writer.rs`.
  - All the `.read_thread(...)` calls in `local/mod.rs`, `local/read_thread.rs` and `local/update_thread_metadata.rs` are inside `#[cfg(test)] mod tests`.

**`codex-core`**
- `run_turn_stop_hooks` in `core/src/hook_runtime.rs`, through `sess.services.thread_store`. That field is `SessionServices::thread_store: Arc<dyn ThreadStore>` in `core/src/state/service.rs`.
- `thread_title_from_thread_store` in `core/src/session/mod.rs`, through the `thread_store: &Arc<dyn ThreadStore>` parameter (in its `None` branch).
- `ThreadManagerState::read_stored_thread` in `core/src/thread_manager.rs`, through `self.thread_store: Arc<dyn ThreadStore>`.

**`codex-app-server`**: methods of `ThreadRequestProcessor` in `app-server/src/request_processors/thread_processor.rs`. All of them call through `self.thread_store: Arc<dyn ThreadStore>`.
- `ThreadRequestProcessor::thread_archive_response` (two calls)
- `ThreadRequestProcessor::load_persisted_thread_for_read`
- `ThreadRequestProcessor::load_thread_turns_list_history`
- `ThreadRequestProcessor::read_stored_thread_for_resume`
- `ThreadRequestProcessor::read_stored_thread_for_new_fork`
- `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls)
- `ThreadRequestProcessor::attach_thread_name`
- `ThreadRequestProcessor::get_thread_summary_response_inner`

**Excluded:** these call other methods that are also named `read_thread`, so they don't count:
- `CodexThread::read_thread` (`core/src/codex_thread.rs`) calls `LiveThread::read_thread`.
- `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`), `apply_bespoke_event_handling` (`app-server/src/bespoke_event_handling.rs`, `ThreadRolledBack` arm) and `TurnRequestProcessor::start_detached_review` (`app-server/src/request_processors/turn_processor.rs`) call `CodexThread::read_thread`.
- The `LiveThread::read_thread` branch of `thread_title_from_thread_store` doesn't count either.

## Q3

`get_git_repo_root` is in `codex-rs/git-utils/src/info.rs` (crate `codex-git-utils`). Its direct non-test callers, which are the intermediates, are:
- `git_diff_to_remote` (`codex-git-utils`, `git-utils/src/info.rs`)
- `AnalyticsReducer::ingest_skill_invoked` (`codex-analytics`, `analytics/src/reducer.rs`)
- `Session::new` (`codex-core`, `core/src/session/session.rs`)
- `run_turn` (`codex-core`, `core/src/session/turn.rs`)
- `build_turn_metadata_header` (`codex-core`, `core/src/turn_metadata.rs`)
- `TurnMetadataState::new` (`codex-core`, `core/src/turn_metadata.rs`)
- `environment_id_from_cwd` (`codex-secrets`, `secrets/src/lib.rs`)
- `ChatWidget::status_line_project_root_for_cwd` (`codex-tui`, `tui/src/chatwidget/status_surfaces.rs`)
- `display_path_for` (`codex-tui`, `tui/src/diff_render.rs`)
- `discover_marketplace_paths_from_roots` (`codex-core-plugins`, `core-plugins/src/marketplace.rs`)
- `ThreadMetadataSync::for_create` (`codex-thread-store`, `thread-store/src/thread_metadata_sync.rs`)
- `write_session_meta` (`codex-rollout`, `rollout/src/recorder.rs`)
- `git_check` (`codex-cli`, `cli/src/doctor/git.rs`)
- `terminal_title_project_root` (`codex-cli`, `cli/src/doctor/title.rs`)
- `run_exec_session` (`codex-exec`, `exec/src/lib.rs`)

These are the functions exactly one hop further out. None of them calls `get_git_repo_root` directly, and none is a test function.

| Function | Crate and file | Calls (intermediate) |
|---|---|---|
| `AnalyticsReducer::ingest` | `codex-analytics`, `analytics/src/reducer.rs` | `AnalyticsReducer::ingest_skill_invoked` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server`, `app-server/src/request_processors/git_processor.rs` | `git_diff_to_remote` |
| `Codex::spawn_internal` | `codex-core`, `core/src/session/mod.rs` | `Session::new` |
| `RegularTask::run` (impl of `SessionTask`) | `codex-core`, `core/src/tasks/regular.rs` | `run_turn` |
| `spawn_review_thread` | `codex-core`, `core/src/session/review.rs` | `TurnMetadataState::new` |
| `Session::make_turn_context` | `codex-core`, `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write`, `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui`, `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui`, `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui`, `tui/src/history_cell/patches.rs` | `display_path_for` |
| `from` in `impl From<DiffSummary> for Box<dyn Renderable>` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::display_description` | `codex-tui`, `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `reformat_description`, a nested fn inside `display_description` | `codex-tui`, `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `list_marketplaces_with_home` | `codex-core-plugins`, `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `LiveThread::create` | `codex-thread-store`, `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout`, `rollout/src/recorder.rs` | `write_session_meta` |
| `build_report` | `codex-cli`, `cli/src/doctor.rs` | `git_check` |
| `terminal_title_check` | `codex-cli`, `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `run_main` | `codex-exec`, `exec/src/lib.rs` | `run_exec_session` |

Notes:
- **`environment_id_from_cwd` adds no caller.** Its only caller is the test `environment_id_fallback_has_cwd_prefix` in `secrets/src/lib.rs`.
- **Excluded test callers:**
  - `git_diff_to_remote` is also called from `core/src/git_info_tests.rs`.
  - `build_turn_metadata_header` and `TurnMetadataState::new` are also called from `core/src/turn_metadata_tests.rs`.
  - `Session::new` is also called from `core/src/session/tests.rs`.
  - `display_path_for` is also called from `display_path_prefers_cwd_without_git_repo` in `diff_render.rs`.
- **Possible ambiguities.** I counted a call made inside a closure or an `async` block as a call by the enclosing function. This applies to `Session::new`, where the call is inside a telemetry counter call, and to `build_report`, where `git_check(...)` is an argument to `run_async_check` inside `tokio::join!`. `reformat_description` is a nested item, so it is listed separately from `display_description`, and both call `display_path_for` directly.

## Q4

This is the path for a local (non-remote) environment on Linux with the default `UnifiedExecShellMode::Direct`. All functions are in `codex-core` unless another crate is named.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`): matches `ResponseEvent::OutputItemDone(item)` from the model stream and calls `handle_output_item_done`. It pushes the returned `tool_future` onto the `in_flight` `FuturesOrdered`, which it drains later with `drain_in_flight` in the same file.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`): calls `ToolRouter::build_tool_call(item)` (`core/src/tools/router.rs`), which turns the `FunctionCall` item into a `ToolCall` whose `tool_name` is `exec_command`. It then records the item and calls `ToolCallRuntime::handle_tool_call`.
3. **`ToolCallRuntime::handle_tool_call`, then `ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` is a plain `fn`, so it runs immediately. It calls `tokio::spawn(async move { ... })`, wrapped in an `AbortOnDropHandle`.
   - Inside the task, it takes the parallel-execution `RwLock` and calls the router.
   - The outer future selects between the task's result and cancellation.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`, then `ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`): builds the `ToolInvocation`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`): looks the handler up as an `Arc<dyn CoreToolRuntime>` and runs the pre-tool-use hooks. It then calls `handle_any_tool` inside the closure it passes to `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`).
6. **`handle_any_tool`** (`codex-core`, `core/src/tools/registry.rs`): calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`.
   - **Trait object:** `handle` is a method of the `ToolExecutor<ToolInvocation>` trait, defined in `codex-tools` (`tools/src/tool_executor.rs`), and is dispatched dynamically.
   - If the tool's exposure was overridden, `ExposureOverride::handle` in `registry.rs` forwards to the inner handler through a second `dyn` call. Whether `exec_command` is wrapped that way depends on configuration.
7. **`ExecCommandHandler::handle`** (`codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`): parses the arguments and resolves the turn environment and cwd. It then:
   - builds the shell argv with `get_command`, for example `bash -lc <cmd>`;
   - checks permissions and `intercept_apply_patch`;
   - calls `UnifiedExecProcessManager::exec_command`.
8. **`UnifiedExecProcessManager::exec_command`, then `UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`). `open_session_with_sandbox` builds the env and the exec-policy approval requirement, creates a `UnifiedExecRuntime`, and calls `ToolOrchestrator::run`.
9. **`ToolOrchestrator::run`** (`codex-core`, `core/src/tools/orchestrator.rs`).
   - **Generic:** it is generic over `T: ToolRuntime<Rq, Out>`, here `UnifiedExecRuntime`.
   - It handles approval, then picks the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, when sandboxing is required, `get_platform_sandbox` returns `SandboxType::LinuxSeccomp`.
   - It builds a `SandboxAttempt` and calls `ToolOrchestrator::run_attempt`. On a sandbox denial with escalation, it may call `run_attempt` a second time.
10. **`ToolOrchestrator::run_attempt`** (`codex-core`, `core/src/tools/orchestrator.rs`): calls `tool.run(req, &attempt, ctx)`, a static (generic) dispatch to `ToolRuntime::run`.
11. **`UnifiedExecRuntime::run`** (`impl ToolRuntime<UnifiedExecRequest, UnifiedExecProcess>`, `codex-core`, `core/src/tools/runtimes/unified_exec.rs`):
    - applies `maybe_wrap_shell_lc_with_snapshot` (a shell-snapshot wrap, not the sandbox);
    - calls `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`) to make a `SandboxCommand`;
    - calls `SandboxAttempt::env_for`;
    - calls `self.manager.open_session_with_exec_env`.
12. **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`): calls **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`), then `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing`).
    - **Sandbox wrap:** for `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`codex-sandboxing`, `sandboxing/src/landlock.rs`).
    - It then prepends the `codex-linux-sandbox` executable (`codex_linux_sandbox_exe`) to the argv and sets `arg0` via `linux_sandbox_arg0_override`.
    - With `SandboxType::None`, the argv passes through unchanged.
13. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`).
    - The Windows branch is compiled out on Linux.
    - Because `environment.is_remote()` is false, it skips the exec-server backend.
    - It splits `request.command` into the program and args, and calls the `codex-utils-pty` crate:
      - with `tty`: `codex_utils_pty::pty::spawn_process_with_inherited_fds` (`utils/pty/src/pty.rs`);
      - otherwise: `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds` (`utils/pty/src/pipe.rs`).
14. **Process spawn** (`codex-utils-pty`):
    - **PTY:** `spawn_process_with_inherited_fds` calls `spawn_process_portable` when there are no inherited fds. That function calls `openpty`, then `pair.slave.spawn_command(...)` from portable-pty. With inherited fds, it calls `spawn_process_preserving_fds`, which uses `StdCommand::spawn()`.
    - **Pipe:** `spawn_process_no_stdin_with_inherited_fds` calls `spawn_process_with_stdin_mode`, which calls `Command::new(program)` and then `command.spawn()`.
    - The result is wrapped by `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`).

**Other branches:**
- **Zsh-fork mode.** With `UnifiedExecShellMode::ZshFork`, `UnifiedExecRuntime::run` first tries `zsh_fork_backend::maybe_prepare_unified_exec`, and on success calls `open_session_with_exec_env` with the prepared request. I did not trace that branch further.
- **Remote environments.** These go through `environment.get_exec_backend().start(...)` (`codex-exec-server`) instead of `codex-utils-pty`.
- **Upstream of the start point.** `try_run_sampling_request` is reached from `run_turn` (`core/src/session/turn.rs`), which is called by `RegularTask::run` (`core/src/tasks/regular.rs`). That is before the requested start point, so it is not traced here.
