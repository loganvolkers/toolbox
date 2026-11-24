# Answers

## Q1

`codex-tools` is the package in `codex-rs/tools` (`codex-rs/tools/Cargo.toml`). It is declared in the workspace as `codex-tools = { path = "tools" }` in `codex-rs/Cargo.toml`. The lists below count only `[dependencies]` and `[target.*.dependencies]`, across the 114 workspace members.

**Direct dependents (7).** Each of these has `codex-tools = { workspace = true }` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `codex-rs/app-server/Cargo.toml` |
| `codex-core` | `codex-rs/core/Cargo.toml` |
| `codex-extension-api` | `codex-rs/ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `codex-rs/ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `codex-rs/ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `codex-rs/ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `codex-rs/ext/web-search/Cargo.toml` |

**Indirect dependents (18).** These do not list `codex-tools` themselves, but reach it through normal dependencies. The table shows the dependents of `codex-tools` that each one depends on directly.

| Crate | Manifest | Reaches `codex-tools` through |
|---|---|---|
| `codex-app-server-client` | `app-server-client/Cargo.toml` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `app-server-daemon/Cargo.toml` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `app-server-test-client/Cargo.toml` | `codex-core` |
| `codex-app-server-transport` | `app-server-transport/Cargo.toml` | `codex-core` |
| `codex-cli` | `cli/Cargo.toml` | `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, `codex-cloud-tasks`, `codex-memories-write`, `codex-app-server-daemon`, `codex-app-server-test-client` |
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

All paths in the second table are relative to `codex-rs/`.

## Q2

The trait method is `ThreadStore::read_thread(&self, params: ReadThreadParams)`, declared in `codex-rs/thread-store/src/store.rs` (crate `codex-thread-store`). It is implemented by `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`).

Every non-test call to it goes through an `Arc<dyn ThreadStore>`. There are 12 calling functions:

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store: Arc<dyn ThreadStore>` (field of `SessionServices`, `core/src/state/service.rs`); reads the parent thread for a `SubagentStop` hook |
| `thread_title_from_thread_store` | `codex-core` | `core/src/session/mod.rs` | parameter `thread_store: &Arc<dyn ThreadStore>`, used only on the branch with no `LiveThread` |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` (two call sites) |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same file | same field |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same file | same field |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same file | same field |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same file | same field |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` | `codex-app-server` | same file | same field (two call sites) |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same file | same field |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same file | same field |

These calls to methods named `read_thread` do not count, because they resolve to other functions:

- `LiveThread::read_thread(include_archived, include_history)`, which `thread_title_from_thread_store` (when it has a `LiveThread`) and `CodexThread::read_thread` (`core/src/codex_thread.rs`) call.
- `CodexThread::read_thread`, which these functions call:
  - `ThreadManager::spawn_subagent` in `core/src/thread_manager.rs`;
  - `apply_bespoke_event_handling` in `app-server/src/bespoke_event_handling.rs`;
  - `start_detached_review` in `app-server/src/request_processors/turn_processor.rs`.
- The free function `read_thread::read_thread` in `thread-store/src/local/read_thread.rs`. The `LocalThreadStore` trait impl, other `LocalThreadStore` methods, `update_thread_metadata.rs` and `live_writer.rs` call it.

All remaining `.read_thread(` calls on a store are in `#[cfg(test)]` modules or `tests/` directories, so they are excluded.

## Q3

The non-test functions that call `get_git_repo_root` (`codex-rs/git-utils/src/info.rs`, crate `codex-git-utils`) directly are the intermediates. The table lists, for each intermediate, the non-test functions that call it. None of those callers calls `get_git_repo_root` directly itself.

| Caller (answer) | Crate, file | Intermediate (calls `get_git_repo_root` directly) |
|---|---|---|
| `AnalyticsReducer::ingest` | `codex-analytics`, `analytics/src/reducer.rs` | `AnalyticsReducer::ingest_skill_invoked` (same file) |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server`, `app-server/src/request_processors/git_processor.rs` | `git_diff_to_remote` (`codex-git-utils`, `git-utils/src/info.rs`) |
| `Codex::spawn_internal` | `codex-core`, `core/src/session/mod.rs` | `Session::new` (`core/src/session/session.rs`) |
| `RegularTask::run` (impl of `SessionTask`) | `codex-core`, `core/src/tasks/regular.rs` | `run_turn` (`core/src/session/turn.rs`) |
| `spawn_review_thread` | `codex-core`, `core/src/session/review.rs` | `TurnMetadataState::new` (`core/src/turn_metadata.rs`) |
| `Session::make_turn_context` | `codex-core`, `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write`, `memories/write/src/runtime.rs` | `build_turn_metadata_header` (`codex-core`, `core/src/turn_metadata.rs`, re-exported from `codex_core`) |
| `list_marketplaces_with_home` | `codex-core-plugins`, `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` (same file) |
| `LiveThread::create` | `codex-thread-store`, `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` (`thread-store/src/thread_metadata_sync.rs`) |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout`, `rollout/src/recorder.rs` | free fn `write_session_meta` (same file) |
| `build_report` | `codex-cli`, `cli/src/doctor.rs` | `git_check` (`cli/src/doctor/git.rs`) |
| `terminal_title_check` | `codex-cli`, `cli/src/doctor/title.rs` | `terminal_title_project_root` (same file) |
| `run_main` | `codex-exec`, `exec/src/lib.rs` | `run_exec_session` (same file) |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui`, `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` (same file) |
| `Notification::display` | `codex-tui`, `tui/src/chatwidget/notifications.rs` | `display_path_for` (`tui/src/diff_render.rs`) |
| `new_view_image_tool_call` | `codex-tui`, `tui/src/history_cell/patches.rs` | `display_path_for` |
| `from` in `impl From<DiffSummary> for Box<dyn Renderable>` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::display_description` | `codex-tui`, `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `reformat_description` (a nested fn inside `display_description`) | `codex-tui`, `tui/src/external_agent_config_migration.rs` | `display_path_for` |

Three of the direct callers add no answer:

- `environment_id_from_cwd` (`codex-secrets`, `secrets/src/lib.rs`) is called only from the test `environment_id_fallback_has_cwd_prefix`.
- The other caller of `display_path_for` in `diff_render.rs` is the test `display_path_prefers_cwd_without_git_repo`.
- The `git_diff_to_remote` free function is otherwise called only from tests in `core/src/git_info_tests.rs`.

`MessageProcessor::handle_initialized_client_request` (`app-server/src/message_processor.rs`) calls `GitRequestProcessor::git_diff_to_remote`. That is a different method, and it reaches `get_git_repo_root` through two intermediates, so it does not count.

## Q4

The trace below assumes a local (not remote) environment, the default pipe-based shell mode (not `ZshFork`) and a sandboxed first attempt. Paths are relative to `codex-rs/`.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`)
   - It reads the model stream from `ModelClientSession::stream`.
   - On `ResponseEvent::OutputItemDone(item)` it builds a `HandleOutputCtx` and calls `handle_output_item_done`.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`)
   - It calls **`ToolRouter::build_tool_call`** (`codex-core`, `core/src/tools/router.rs`). That turns `ResponseItem::FunctionCall` into a `ToolCall` with `ToolName` `exec_command` and `ToolPayload::Function { arguments }`.
   - It then calls `record_completed_response_item` and **`ToolCallRuntime::handle_tool_call`**.
   - The result is boxed into a future. `try_run_sampling_request` pushes it into its `in_flight` `FuturesOrdered`, and `drain_in_flight` (same file) awaits it later.
3. **`ToolCallRuntime::handle_tool_call`** → **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`)
   - **Spawned task:** `handle_tool_call_with_source` is a plain `fn`. It immediately calls `tokio::spawn`, wrapped in `AbortOnDropHandle`, to start an async block that takes the parallel-execution `RwLock` and calls the router. The returned future only awaits that task's handle or cancellation.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`** → **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`). The second builds a `ToolInvocation`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`)
   - It looks the tool up as an `Arc<dyn CoreToolRuntime>`.
   - It runs pre-tool-use hooks (`run_pre_tool_use_hooks`).
   - It passes a closure to **`SessionTelemetry::log_tool_result_with_tags`** (`codex-otel`, `otel/src/events/session_telemetry.rs`), which awaits it. The closure calls **`handle_any_tool`** (same registry file).
6. **`handle_any_tool`** calls `tool.handle(invocation)`.
   - **Trait object:** `tool` is `&dyn CoreToolRuntime`, and `handle` is a method of the `ToolExecutor<ToolInvocation>` trait (`codex-tools`, `tools/src/tool_executor.rs`; an `async_trait`).
   - The handler is registered in `core/src/tools/spec_plan.rs` with `PlannedTools::add(ExecCommandHandler::new(..))`. It is not wrapped in `ExposureOverride`, so the call resolves to:
7. **`ExecCommandHandler::handle`** (`impl ToolExecutor<ToolInvocation>`, `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`)
   - It parses the arguments and calls `resolve_tool_environment`.
   - It resolves the command line with `get_command`.
   - It applies permission checks and calls `intercept_apply_patch`.
   - It then calls `UnifiedExecProcessManager::exec_command`.
8. **`UnifiedExecProcessManager::exec_command`** → **`UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`)
   - The second builds the env (`create_env`), the exec-policy approval requirement and a `UnifiedExecRequest`.
   - It creates `ToolOrchestrator::new()` and `UnifiedExecRuntime::new(..)`, then calls `orchestrator.run(&mut runtime, ..)`.
9. **`ToolOrchestrator::run`** (`codex-core`, `core/src/tools/orchestrator.rs`)
   - **Generic:** it is generic over `T: ToolRuntime<Rq, Out>`, here `UnifiedExecRuntime`, so dispatch is static.
   - It handles approval (`request_approval` if needed).
   - It picks the sandbox with **`SandboxManager::select_initial`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux this returns `SandboxType::LinuxSeccomp` when the policy requires a platform sandbox, via `get_platform_sandbox`; otherwise it returns `SandboxType::None`.
   - It builds a `SandboxAttempt` and calls **`ToolOrchestrator::run_attempt`**.
   - `run_attempt` calls `tool.run(req, &attempt, ..)`. If the sandboxed attempt is denied and approval allows it, `run` may call `run_attempt` again with `SandboxType::None`.
10. **`UnifiedExecRuntime::run`** (`impl ToolRuntime`, `codex-core`, `core/src/tools/runtimes/unified_exec.rs`)
    - It wraps the command for the shell snapshot with `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`). This is not the sandbox.
    - It calls `build_sandbox_command` (same `mod.rs`), producing a `SandboxCommand`.
    - It then calls **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`).
11. **The command is wrapped for the sandbox here.** `SandboxAttempt::env_for` calls **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`).
    - For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported`.
    - It then calls **`create_linux_sandbox_command_args_for_permission_profile`** (`codex-sandboxing`, `sandboxing/src/landlock.rs`).
    - It prepends the `codex_linux_sandbox_exe` path to argv and sets an `arg0` override (`linux_sandbox_arg0_override`).
    - With `SandboxType::None` the argv passes through unchanged.
    - The result is converted with `ExecRequest::from_sandbox_exec_request` (`codex-core`, `core/src/sandboxing/mod.rs`).
12. Back in `UnifiedExecRuntime::run`, it calls **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`), passing `NoopSpawnLifecycle`, which has no inherited fds.
    - The environment is not remote, so the `get_exec_backend().start(..)` path is skipped. The Windows branch is `cfg`'d out on Linux.
    - It splits argv into program and args.
13. **The process is spawned in `codex-utils-pty`:**
    - **With `tty == false`:** `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds` → `spawn_process_with_stdin_mode` (`utils/pty/src/pipe.rs`). This builds a `tokio::process::Command`, with a `pre_exec` that calls `detach_from_tty` and sets the parent-death signal on Linux, then calls **`command.spawn()`**.
    - **With `tty == true`:** `codex_utils_pty::pty::spawn_process_with_inherited_fds` → `spawn_process_portable` (`utils/pty/src/pty.rs`). This calls `pair.slave.spawn_command(..)` from `portable_pty`. When fds are inherited it takes `spawn_process_preserving_fds` instead, which calls `StdCommand::spawn`.
    - When the Linux sandbox is selected, the program spawned is the `codex-linux-sandbox` executable with the original command in its arguments.
    - `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`) then wraps the spawned process.

I did not trace what the `codex-linux-sandbox` executable does with that command line after it is spawned.
