# Answers

## Q1

Dependents of `codex-tools` (package `codex-tools`, `codex-rs/tools/Cargo.toml`), counting only normal `[dependencies]` between workspace packages, from each crate's `Cargo.toml` (none of the edges below are dev- or build-only; all `codex-tools` edges are plain, non-optional, non-target-specific).

**Direct dependents (7)**, each with `codex-tools` in `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-core` | `codex-rs/core/Cargo.toml` |
| `codex-app-server` | `codex-rs/app-server/Cargo.toml` |
| `codex-extension-api` | `codex-rs/ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `codex-rs/ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `codex-rs/ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `codex-rs/ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `codex-rs/ext/web-search/Cargo.toml` |

**Indirect dependents (22)**, which reach `codex-tools` only through other crates. The second column shows the dependents of `codex-tools` that each one depends on directly.

| Crate | Reaches `codex-tools` via |
|---|---|
| `codex-app-server-client` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `codex-core` |
| `codex-app-server-transport` | `codex-core` |
| `codex-chatgpt` | `codex-core` |
| `codex-cli` | `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, … |
| `codex-cloud-config` | `codex-core` |
| `codex-cloud-tasks` | `codex-core`, `codex-tui` |
| `codex-core-api` | `codex-core`, `codex-extension-api` |
| `codex-exec` | `codex-core`, `codex-app-server-client`, … |
| `codex-guardian` | `codex-core`, `codex-extension-api` |
| `codex-lmstudio` | `codex-core` |
| `codex-mcp-server` | `codex-core`, `codex-extension-api` |
| `codex-memories-write` | `codex-core` |
| `codex-ollama` | `codex-core` |
| `codex-thread-manager-sample` | `codex-core-api` |
| `codex-tui` | `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-utils-oss` | `codex-core`, `codex-lmstudio`, `codex-ollama` |
| `codex-utils-sandbox-summary` | `codex-core` |
| `core_test_support` | `codex-core`, `codex-extension-api` |
| `app_test_support` | `codex-core`, `core_test_support` |
| `mcp_test_support` | `codex-mcp-server`, `core_test_support` |

The last three (`core_test_support`, `app_test_support`, `mcp_test_support`) are workspace packages used as test-support libraries. Their own `[dependencies]` reach `codex-tools`, so they are listed. They would drop out only if you exclude test-support packages as a whole.

## Q2

The trait method is `ThreadStore::read_thread` (`codex-rs/thread-store/src/store.rs`, crate `codex-thread-store`). It has two implementations: `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`).

Every non-test caller below calls it through an `Arc<dyn ThreadStore>` field or parameter:

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`SessionServices`, `core/src/state/service.rs`, `Arc<dyn ThreadStore>`) |
| `thread_title_from_thread_store` | `codex-core` | `core/src/session/mod.rs` | parameter `thread_store: &Arc<dyn ThreadStore>`, in the `None` branch |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` (two call sites) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two call sites) | `codex-app-server` | same | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same | same |

These calls don't count, because they go to other functions named `read_thread`:

- `CodexThread::read_thread` (`core/src/codex_thread.rs`) calls `LiveThread::read_thread`, not the trait method.
- Calls on a `CodexThread` also don't count:
  - `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`, through `fork_source`);
  - `apply_bespoke_event_handling` (`app-server/src/bespoke_event_handling.rs`, through `conversation: Arc<CodexThread>`);
  - `TurnRequestProcessor::start_detached_review` (`app-server/src/request_processors/turn_processor.rs`, through `review_thread`).
- `thread_title_from_thread_store` also calls `LiveThread::read_thread` in its `Some` branch. It is listed above only for its `None` branch.
- `<LocalThreadStore as ThreadStore>::read_thread` delegates to the free function `local::read_thread::read_thread`, not the trait method.

All other `.read_thread(` calls through a store are in test code and are excluded: the `#[cfg(test)] mod tests` blocks in `thread-store/src/local/mod.rs`, `local/read_thread.rs` and `local/update_thread_metadata.rs`.

## Q3

`get_git_repo_root` is defined in `codex-rs/git-utils/src/info.rs` (crate `codex-git-utils`). Its non-test direct callers, which are the intermediates, are:

- `AnalyticsReducer::ingest_skill_invoked` (`codex-analytics`, `analytics/src/reducer.rs`)
- `git_diff_to_remote` (`codex-git-utils`, `git-utils/src/info.rs`)
- `Session::new` (`codex-core`, `core/src/session/session.rs`)
- `run_turn` (`codex-core`, `core/src/session/turn.rs`)
- `build_turn_metadata_header` and `TurnMetadataState::new` (`codex-core`, `core/src/turn_metadata.rs`)
- `environment_id_from_cwd` (`codex-secrets`, `secrets/src/lib.rs`)
- `ChatWidget::status_line_project_root_for_cwd` (`codex-tui`, `tui/src/chatwidget/status_surfaces.rs`)
- `display_path_for` (`codex-tui`, `tui/src/diff_render.rs`)
- `discover_marketplace_paths_from_roots` (`codex-core-plugins`, `core-plugins/src/marketplace.rs`)
- `ThreadMetadataSync::for_create` (`codex-thread-store`, `thread-store/src/thread_metadata_sync.rs`)
- `write_session_meta` (`codex-rollout`, `rollout/src/recorder.rs`)
- `git_check` (`codex-cli`, `cli/src/doctor/git.rs`)
- `terminal_title_project_root` (`codex-cli`, `cli/src/doctor/title.rs`)
- `run_exec_session` (`codex-exec`, `exec/src/lib.rs`)

The functions that call one of these intermediates and do not call `get_git_repo_root` directly themselves are:

| Function | Crate | File | Intermediate it calls |
|---|---|---|---|
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::reformat_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |

Exclusions:

- `environment_id_from_cwd` (`codex-secrets`) is called only from its `#[cfg(test)]` tests, so it contributes no caller.
- The other callers of `git_diff_to_remote` are tests in `core/src/git_info_tests.rs`.
- The other callers of `build_turn_metadata_header` and `TurnMetadataState::new` are in `core/src/turn_metadata_tests.rs`.
- The other callers of `Session::new` are in `core/src/session/tests.rs`.
- The other caller of `display_path_for` is the test module of `tui/src/diff_render.rs`.
- `MessageProcessor::handle_initialized_client_request` (`codex-app-server`) calls `GitRequestProcessor::git_diff_to_remote`, a different function that shares the name. It reaches `get_git_repo_root` only at three hops, so it is excluded.
- `codex_thread_manager_sample`'s own `run_turn` (`thread-manager-sample/src/main.rs`) is a different function, so its callers are not callers of `codex-core`'s `run_turn`.

## Q4

The path runs from the model stream to the process spawn for a local, non-remote environment on Linux, with the default (non-zsh-fork) unified-exec shell mode.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`)
   - It reads the model stream (`stream.next()`) and, on `ResponseEvent::OutputItemDone(item)`, builds a `HandleOutputCtx` and calls `handle_output_item_done`.
   - The tool future it gets back is pushed onto a `FuturesOrdered` named `in_flight`, which `drain_in_flight` awaits later.
   - It is reached from `run_turn` through `run_sampling_request`.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`)
   - It calls `ToolRouter::build_tool_call` (`core/src/tools/router.rs`), which turns `ResponseItem::FunctionCall` into a `ToolCall` with tool name `exec_command` and `ToolPayload::Function`.
   - It records the item, then boxes `ToolCallRuntime::handle_tool_call(call, token)` as the tool future.
3. **`ToolCallRuntime::handle_tool_call` → `ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`)
   - **Spawned task:** `handle_tool_call_with_source` calls `tokio::spawn` (wrapped in `AbortOnDropHandle`) as soon as it is called.
   - Inside the task it takes the parallel-execution `RwLock` (read or write, depending on `supports_parallel`) and calls the router. The returned future only `select!`s on the task handle and on cancellation.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome` → `ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`)
   - It builds a `ToolInvocation` and calls the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`)
   - **Trait object:** `ToolRegistry::tool` returns an `Arc<dyn CoreToolRuntime>`.
   - It runs the pre-tool-use hooks, then calls `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`). That function is **generic** over a closure (`F: FnOnce() -> Fut`), which calls `handle_any_tool`.
6. **`handle_any_tool`** (`codex-core`, `core/src/tools/registry.rs`)
   - It calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. This is **dynamic dispatch** to `ToolExecutor::handle`, an `#[async_trait]` trait in `codex-tools` (`tools/src/tool_executor.rs`).
   - If the registry wrapped the handler in `ExposureOverride`, its `handle` forwards to the inner handler first.
7. **`ExecCommandHandler::handle`** (impl `ToolExecutor<ToolInvocation>`, `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`)
   - It parses the arguments, resolves the turn environment and cwd, and builds the argv with `get_command` (`core/src/tools/handlers/unified_exec.rs`), using the user shell and shell mode.
   - It applies the permission checks and `intercept_apply_patch`.
   - It then calls `UnifiedExecProcessManager::exec_command` with an `ExecCommandRequest`.
8. **`UnifiedExecProcessManager::exec_command` → `UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`)
   - It builds the environment (`create_env`, `apply_unified_exec_env`).
   - It gets the approval requirement from `exec_policy.create_exec_approval_requirement_for_command`.
   - It builds a `UnifiedExecToolRequest` and calls `ToolOrchestrator::run` with a `UnifiedExecRuntime`.
9. **`ToolOrchestrator::run`** (`codex-core`, `core/src/tools/orchestrator.rs`)
   - It is **generic** over `T: ToolRuntime<Rq, Out>`, here `UnifiedExecRuntime`.
   - It handles approval, then chooses the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`).
   - On Linux, `get_platform_sandbox` returns `SandboxType::LinuxSeccomp` when the policy requires a sandbox, and `SandboxType::None` otherwise (for example under full access or an approved bypass).
   - It builds a `SandboxAttempt` and calls `ToolOrchestrator::run_attempt`. That function calls `tool.run(...)`, the generic `ToolRuntime::run`. On a sandbox denial it may retry `run_attempt` without the sandbox, after approval.
10. **`UnifiedExecRuntime::run`** (impl `ToolRuntime`, `codex-core`, `core/src/tools/runtimes/unified_exec.rs`)
    - It wraps the command for the shell snapshot with `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`), because the environment is not remote.
    - It builds a `SandboxCommand` with `build_sandbox_command`, then calls `SandboxAttempt::env_for`.
11. **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`) → **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). **This is where the command is wrapped for the sandbox.**
    - For `SandboxType::LinuxSeccomp`, it checks bubblewrap support with `ensure_linux_bubblewrap_is_supported` and builds the helper's arguments with `create_linux_sandbox_command_args_for_permission_profile` (`codex-sandboxing`, `sandboxing/src/landlock.rs`).
    - It prepends the `codex-linux-sandbox` executable (`codex_linux_sandbox_exe`) to the argv and sets an `arg0` override with `linux_sandbox_arg0_override`.
    - For `SandboxType::None`, the argv is passed through unchanged.
    - The result is converted with `ExecRequest::from_sandbox_exec_request` (`codex-core`, `core/src/sandboxing/mod.rs`).
12. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`)
    - The Windows branch is `cfg(target_os = "windows")` and the `environment.is_remote()` branch goes to the exec-server backend, so neither applies here.
    - It splits `request.command` into program and args and calls `codex-utils-pty`:
      - with `tty = false` (the default, `default_tty` in `core/src/tools/handlers/unified_exec.rs`), it calls `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`;
      - with `tty = true`, it calls `codex_utils_pty::pty::spawn_process_with_inherited_fds`.
13. **The spawn** (`codex-utils-pty`)
    - **Pipe path** (`utils/pty/src/pipe.rs`): `spawn_process_no_stdin_with_inherited_fds` → `spawn_process_with_stdin_mode`, which builds a `tokio::process::Command` and calls `command.spawn()`.
    - **PTY path** (`utils/pty/src/pty.rs`): `spawn_process_with_inherited_fds` → `spawn_process_portable`, which calls `openpty` and `pair.slave.spawn_command(...)` through `portable_pty`. When inherited file descriptors are present, it calls `spawn_process_preserving_fds` instead, which uses `std::process::Command::spawn`.
    - On the sandboxed path, the process spawned is the `codex-linux-sandbox` helper. The helper then applies bubblewrap, Landlock and seccomp and runs the real command; that happens in the `codex-linux-sandbox` binary, beyond this trace.
    - After the spawn, `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`) wraps the result.

Variant: with `UnifiedExecShellMode::ZshFork`, `UnifiedExecRuntime::run` first tries `zsh_fork_backend::maybe_prepare_unified_exec`, which uses the same `env_for` wrapping. If that succeeds, it calls `open_session_with_exec_env` with the prepared request and spawn lifecycle. Otherwise it falls back to the path above.
