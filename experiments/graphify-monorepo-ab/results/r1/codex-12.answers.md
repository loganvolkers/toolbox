# Answers

## Q1

`codex-tools` is the package at `codex-rs/tools` (`tools/Cargo.toml`). It is mapped in the workspace as `codex-tools = { path = "tools" }` in `codex-rs/Cargo.toml`. The dependency edges below come from the `[dependencies]` and `[target.*.dependencies]` tables of each member's `Cargo.toml`. Dev- and build-dependencies are left out.

**Direct dependents (7).** Each of these has `codex-tools = { workspace = true }` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-core` | `core/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents (18).** These depend on `codex-tools` only through other crates. The second column lists their dependencies that lead to `codex-tools`.

| Crate | Reaches `codex-tools` through |
|---|---|
| `codex-app-server-client` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `codex-core` |
| `codex-app-server-transport` | `codex-core` |
| `codex-cli` | `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, and others |
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

The trait method is `ThreadStore::read_thread(&self, params: ReadThreadParams)`, declared in `thread-store/src/store.rs`. It is implemented by `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`).

Every non-test caller below reaches it through an `Arc<dyn ThreadStore>` (or `&Arc<dyn ThreadStore>`). No non-test caller uses a generic or a concrete implementing type.

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `thread_title_from_thread_store` (free function; the `None` branch) | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`SessionServices`, `Arc<dyn ThreadStore>`, `core/src/state/service.rs`) |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` (two calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls) | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same file | same |

Some calls that look similar call a different `read_thread`, so they are left out:

- **`LiveThread::read_thread(include_archived, include_history)`** is called by:
  - `thread_title_from_thread_store` in its `Some(live_thread)` branch;
  - `CodexThread::read_thread` (`core/src/codex_thread.rs`).
- **`CodexThread::read_thread`** is called by:
  - `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`);
  - `TurnRequestProcessor::start_detached_review` (`app-server/src/request_processors/turn_processor.rs`);
  - `apply_bespoke_event_handling` (`app-server/src/bespoke_event_handling.rs`).
- **The free function `read_thread::read_thread`** (`thread-store/src/local/read_thread.rs`) is called by `LocalThreadStore`'s trait implementation and by `update_thread_metadata.rs` and `live_writer.rs`.

The `.read_thread(...)` calls in `thread-store/src/local/read_thread.rs`, `local/update_thread_metadata.rs` and `local/mod.rs` sit inside their `#[cfg(test)] mod tests`. Calls in `tests/` directories are also test code, so neither is counted.

## Q3

`get_git_repo_root(base_dir: &Path) -> Option<PathBuf>` is defined in `git-utils/src/info.rs` and re-exported from `git-utils/src/lib.rs`.

**Direct non-test callers.** These are the intermediates, so they are excluded from the answer:

| Function | Crate | File |
|---|---|---|
| `terminal_title_project_root` | `codex-cli` | `cli/src/doctor/title.rs` |
| `git_check` | `codex-cli` | `cli/src/doctor/git.rs` |
| `git_diff_to_remote` | `codex-git-utils` | `git-utils/src/info.rs` |
| `write_session_meta` (free function) | `codex-rollout` | `rollout/src/recorder.rs` |
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics` | `analytics/src/reducer.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store` | `thread-store/src/thread_metadata_sync.rs` |
| `run_exec_session` | `codex-exec` | `exec/src/lib.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` |
| `environment_id_from_cwd` | `codex-secrets` | `secrets/src/lib.rs` |
| `display_path_for` | `codex-tui` | `tui/src/diff_render.rs` |
| `run_turn` | `codex-core` | `core/src/session/turn.rs` |
| `Session::new` | `codex-core` | `core/src/session/session.rs` |
| `build_turn_metadata_header` | `codex-core` | `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core` | `core/src/turn_metadata.rs` |

**Answer: functions that call `get_git_repo_root` through exactly one intermediate.** None of these calls `get_git_repo_root` itself, and none is test code.

| Function | Crate | File | Through |
|---|---|---|---|
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` (called in both `tokio::join!` branches) |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `AnalyticsReducer::ingest_skill_invoked` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `from` in `impl From<DiffSummary> for Box<dyn Renderable>` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `reformat_description` (a nested fn inside `ExternalAgentConfigMigrationScreen::display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `RegularTask::run` (`impl SessionTask for RegularTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |

Notes:

- **Excluded intermediates.**
  - `environment_id_from_cwd` (`codex-secrets`) has no non-test caller. Its only caller is in `secrets/src/lib.rs`'s `mod tests`.
  - `display_path_for`'s caller `display_path_prefers_cwd_without_git_repo` is a test.
  - `build_turn_metadata_header` and `TurnMetadataState::new` are otherwise called only from `core/src/turn_metadata_tests.rs`.
- **Not one hop.** `GitRequestProcessor::git_diff_to_remote`, called from `MessageProcessor::handle_initialized_client_request`, reaches the free function `git_diff_to_remote` through `git_diff_to_origin`. That makes it two hops away from `get_git_repo_root`. `ExternalAgentConfigMigrationScreen::display_description` is also two hops away, through `reformat_description`.
- **A different `run_turn`.** `thread-manager-sample/src/main.rs` calls its own local `run_turn`, not `codex-core`'s.
- **Possible gaps.** This list is built from calls by name. Calls made through function pointers or macros may not be complete.

## Q4

This is the path for a local (not remote exec-server) session on Linux. It assumes the default `UnifiedExecShellMode` (not ZshFork) and the default `tty: false`.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`). It reads the model stream. On `ResponseEvent::OutputItemDone(item)` it builds a `HandleOutputCtx` and calls `handle_output_item_done`. The returned `tool_future` is pushed onto a `FuturesOrdered` called `in_flight`, which is later awaited by `drain_in_flight`.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`). It calls `ToolRouter::build_tool_call(item)` (`core/src/tools/router.rs`), which turns the `ResponseItem::FunctionCall` into a `ToolCall` with `ToolPayload::Function`. It records the item, then boxes `ctx.tool_runtime.clone().handle_tool_call(call, token)` as an `InFlightFuture`, a boxed future.
3. **`ToolCallRuntime::handle_tool_call`**, which then calls **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` runs the dispatch inside `tokio::spawn(async move { ... })`, wrapped in an `AbortOnDropHandle`. The task takes the parallel-execution `RwLock` (read or write) and calls the router. The outer future waits on the task or on cancellation in a `tokio::select!`.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, which then calls **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`). The inner function builds a `ToolInvocation` and calls the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`).
   - It looks up the tool as an `Arc<dyn CoreToolRuntime>` and runs the pre-tool-use hooks.
   - It then calls `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`). That function is generic over a closure `F: FnOnce() -> Fut` and invokes the closure, which calls `handle_any_tool`.
6. **`handle_any_tool`** (`codex-core`, `core/src/tools/registry.rs`).
   - **Trait object:** it calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. `handle` comes from the supertrait `ToolExecutor<ToolInvocation>`, which `codex-tools` re-exports.
   - If the handler was wrapped by `override_tool_exposure`, `ExposureOverride::handle` forwards to the inner `Arc<dyn CoreToolRuntime>` first.
7. **`ExecCommandHandler::handle`** (`impl ToolExecutor<ToolInvocation> for ExecCommandHandler`; `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - It parses `ExecCommandArgs`, resolves the environment and cwd, allocates a process id, and resolves the shell command with `get_command`.
   - It handles the permissions and the `intercept_apply_patch` check.
   - It then calls `session.services.unified_exec_manager.exec_command(ExecCommandRequest { .. }, &context)`.
8. **`UnifiedExecProcessManager::exec_command`**, which then calls **`UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`). The second function:
   - builds the env;
   - gets the exec-approval requirement from `exec_policy.create_exec_approval_requirement_for_command`;
   - builds a `UnifiedExecToolRequest` and a `UnifiedExecRuntime::new(self, shell_mode)`;
   - calls `ToolOrchestrator::run(&mut runtime, &req, ..)`.
9. **`ToolOrchestrator::run`**, which then calls **`ToolOrchestrator::run_attempt`** (`codex-core`, `core/src/tools/orchestrator.rs`).
   - **Generic:** both are generic over `T: ToolRuntime<Rq, Out>`; here `T` is `UnifiedExecRuntime`.
   - `run` handles approval, then chooses the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`). On Linux with a restricted policy this is `SandboxType::LinuxSeccomp`.
   - It builds a `SandboxAttempt`. `run_attempt` starts any network approval and calls `tool.run(req, &attempt, &ctx)`.
   - If the sandbox denies the command, `run` may retry without the sandbox after approval.
10. **`UnifiedExecRuntime::run`** (`impl ToolRuntime<UnifiedExecRequest, UnifiedExecProcess> for UnifiedExecRuntime`; `codex-core`, `core/src/tools/runtimes/unified_exec.rs`).
    - It prepares the env and calls `maybe_wrap_shell_lc_with_snapshot`, then `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`).
    - **Sandbox wrapping:** it then calls `SandboxAttempt::env_for` (`core/src/tools/sandboxing.rs`), which calls **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported`. It then rewrites the argv as `[codex_linux_sandbox_exe, ...create_linux_sandbox_command_args_for_permission_profile(argv, ...)]` and sets an `arg0` override with `linux_sandbox_arg0_override`.
    - `env_for` converts the result with `ExecRequest::from_sandbox_exec_request`.
    - In ZshFork mode, `zsh_fork_backend::maybe_prepare_unified_exec` may prepare the request instead. Both paths then call the same manager method.
11. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`), called with `Box::new(NoopSpawnLifecycle)`. It skips the Windows branch (`#[cfg(target_os = "windows")]`) and the `environment.is_remote()` branch. It splits the program from its args and:
    - if `tty` is false (the default), calls `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`;
    - if `tty` is true, calls `codex_utils_pty::pty::spawn_process_with_inherited_fds`.
12. **Process spawn** (`codex-utils-pty`).
    - Pipe path, `utils/pty/src/pipe.rs`: `spawn_process_no_stdin_with_inherited_fds` calls `spawn_process_with_stdin_mode`, which calls `command.spawn()` on a Tokio `Command`.
    - PTY path, `utils/pty/src/pty.rs`: with no inherited fds, `spawn_process_with_inherited_fds` calls `spawn_process_portable`, which calls `pair.slave.spawn_command(command_builder)` from `portable_pty`. With inherited fds it calls `spawn_process_preserving_fds`, which calls `command.spawn()`.
    - The spawned program is the `codex-linux-sandbox` executable with the wrapped command, unless no sandbox was selected.
13. **After the spawn.** `UnifiedExecProcess::from_spawned` wraps the child. Back in `UnifiedExecProcessManager::exec_command`, `start_streaming_output` (`core/src/unified_exec/async_watcher.rs`) and `store_process` take over; `store_process` calls `spawn_exit_watcher`.

I am not certain which sandbox is chosen in every configuration. `select_initial` returns `SandboxType::None` for unrestricted policies, and `BypassSandboxFirstAttempt` also skips it. In those cases `transform` does not wrap the command, and the original argv is spawned.
