# Answers

## Q1

Dependencies are taken from each workspace member's `Cargo.toml` under `codex-rs/`: `[dependencies]` plus `[target.*.dependencies]`, and no dev- or build-dependencies.

**Direct dependents of `codex-tools`** (7). Each lists `codex-tools` in its own `[dependencies]`:

| Crate | Path |
|---|---|
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-core` | `core/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents** (18). These reach `codex-tools` only through other crates. The second column shows the dependency they reach it through.

| Crate | Through (normal deps in the closure) |
|---|---|
| `codex-app-server-client` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `codex-core` |
| `codex-app-server-transport` | `codex-core` |
| `codex-cli` | `codex-core`, `codex-app-server`, `codex-exec`, `codex-tui`, `codex-mcp-server`, … |
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

The Toolbox portal's `crate:codex-tools` entity gives the same 7 direct dependents (`dependents`, `dependentCount: 7`).

## Q2

The trait method is `ThreadStore::read_thread` (`thread-store/src/store.rs`, trait `ThreadStore`). Its implementations are `LocalThreadStore::read_thread` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore::read_thread` (`thread-store/src/in_memory.rs`).

These non-test functions call it. In every case the receiver is an `Arc<dyn ThreadStore>`.

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store` (`Arc<dyn ThreadStore>`) |
| `thread_title_from_thread_store` | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>` (in its `None` branch) |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`SessionServices.thread_store: Arc<dyn ThreadStore>`, `core/src/state/service.rs`) |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store` (`Arc<dyn ThreadStore>`) |
| `ThreadRequestProcessor::thread_archive_response` (two calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store` (`Arc<dyn ThreadStore>`) |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls) | `codex-app-server` | same | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same | same |

These look similar but are excluded:
- `CodexThread::read_thread` (`core/src/codex_thread.rs`) calls `LiveThread::read_thread`, which is a different method with the same name.
- The `.read_thread(true, …)` calls in these functions take two `bool`s, so they are `CodexThread::read_thread` or `LiveThread::read_thread`, not the trait method:
  - `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`)
  - `thread_title_from_thread_store`'s `Some(live_thread)` branch
  - the `ThreadRolledBack` handling in `app-server/src/bespoke_event_handling.rs`
  - the review-thread code in `app-server/src/request_processors/turn_processor.rs`
- `LocalThreadStore::read_thread` and `LocalThreadStore::load_history` call the free function `read_thread::read_thread` (`thread-store/src/local/read_thread.rs`), not the trait method. So do `live_writer.rs` and `update_thread_metadata.rs`.
- The remaining `.read_thread(ReadThreadParams{…})` calls are test functions. They are in the `#[cfg(test)] mod tests` of these files:
  - `thread-store/src/local/mod.rs`
  - `local/read_thread.rs`
  - `local/update_thread_metadata.rs`

## Q3

`get_git_repo_root` is defined in `git-utils/src/info.rs` (`codex-git-utils`). These non-test functions call it directly, so they are the intermediates:

| Intermediate | Crate / file |
|---|---|
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics`, `analytics/src/reducer.rs` |
| `git_check` | `codex-cli`, `cli/src/doctor/git.rs` |
| `terminal_title_project_root` | `codex-cli`, `cli/src/doctor/title.rs` |
| `Session::new` | `codex-core`, `core/src/session/session.rs` |
| `run_turn` | `codex-core`, `core/src/session/turn.rs` |
| `build_turn_metadata_header` | `codex-core`, `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core`, `core/src/turn_metadata.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins`, `core-plugins/src/marketplace.rs` |
| `run_exec_session` | `codex-exec`, `exec/src/lib.rs` |
| `git_diff_to_remote` | `codex-git-utils`, `git-utils/src/info.rs` |
| `write_session_meta` | `codex-rollout`, `rollout/src/recorder.rs` |
| `environment_id_from_cwd` | `codex-secrets`, `secrets/src/lib.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store`, `thread-store/src/thread_metadata_sync.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui`, `tui/src/chatwidget/status_surfaces.rs` |
| `display_path_for` | `codex-tui`, `tui/src/diff_render.rs` |

**Functions one intermediate away.** None of these calls `get_git_repo_root` directly.

| Function | Crate | File | Via |
|---|---|---|---|
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` (called twice inside `tokio::join!`) |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `reformat_description` (nested fn inside `ExternalAgentConfigMigrationScreen::display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |

Notes:
- `environment_id_from_cwd` (`codex-secrets`) has only a test caller (`environment_id_fallback_has_cwd_prefix`, a `#[test]`). It contributes no caller.
- The tests in `core/src/turn_metadata_tests.rs`, `core/src/git_info_tests.rs` and `tui/src/diff_render.rs`'s test module are excluded.
- In `run_turn`, the other branch calls `get_git_repo_root_with_fs`. That call does not count, but the `None` branch calls `get_git_repo_root` directly, so `run_turn` is an intermediate.
- `GitRequestProcessor::git_diff_to_remote` (`app-server/src/request_processors/git_processor.rs`) is two hops away, through `git_diff_to_origin`. It is a different function from `codex_git_utils::git_diff_to_remote`.
- If `reformat_description` should be attributed to its enclosing function, `ExternalAgentConfigMigrationScreen::display_description` is the one-hop caller instead.

## Q4

This trace follows the default local path:
- unified-exec shell mode `Direct`, not `ZshFork`;
- a local environment, not a remote exec-server;
- the platform sandbox is selected, which on Linux is `SandboxType::LinuxSeccomp`.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`)
   - Its `stream.next()` loop matches `ResponseEvent::OutputItemDone(item)`.
   - It builds a `HandleOutputCtx`, which includes a clone of the `ToolCallRuntime`, and calls `handle_output_item_done`.
   - It pushes the returned `tool_future` onto the `in_flight` `FuturesOrdered`, and awaits it later in the turn.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`)
   - It calls `ToolRouter::build_tool_call` (`core/src/tools/router.rs`). That turns `ResponseItem::FunctionCall { name: "exec_command", … }` into a `ToolCall` with `ToolPayload::Function`.
   - It records the item, then calls `ctx.tool_runtime.clone().handle_tool_call(call, token)`.
3. **`ToolCallRuntime::handle_tool_call`** → **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`)
   - **Spawned task:** `handle_tool_call_with_source` runs `tokio::spawn(async move { … })`, wrapped in `AbortOnDropHandle`.
   - The task takes the parallel-execution `RwLock`, read or write depending on whether the tool supports parallel calls; `exec_command` does.
   - It then calls `router.dispatch_tool_call_with_terminal_outcome(…)`.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`** → **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`)
   - This builds a `ToolInvocation` and calls `self.registry.dispatch_any_with_terminal_outcome`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`)
   - It looks up `Arc<dyn CoreToolRuntime>` by `ToolName` and runs the pre-tool-use hooks.
   - It calls `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`), which is generic over a closure `F: FnOnce() -> Fut`. The closure calls `handle_any_tool`.
6. **`handle_any_tool`** (`codex-core`, `core/src/tools/registry.rs`)
   - **Trait object:** it calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. `CoreToolRuntime` is a subtrait of `codex_tools::ToolExecutor<ToolInvocation>`, and `handle` is the `ToolExecutor` method.
   - If the tool was registered with a different exposure, the call first goes through `ExposureOverride::handle`, which forwards to the inner `Arc<dyn CoreToolRuntime>`.
7. **`ExecCommandHandler::handle`** (impl `ToolExecutor<ToolInvocation>`; `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`)
   - It parses `ExecCommandArgs` and resolves the turn environment and cwd.
   - It resolves the shell argv with `get_command` and applies the permission and approval checks.
   - It checks `intercept_apply_patch`.
   - It then calls `session.services.unified_exec_manager.exec_command(ExecCommandRequest{…}, &context)`.
8. **`UnifiedExecProcessManager::exec_command`** → **`UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`)
   - `open_session_with_sandbox` builds the env with `create_env` and `apply_unified_exec_env`.
   - It gets the `exec_approval_requirement` from `ExecPolicyManager::create_exec_approval_requirement_for_command`, and builds a `UnifiedExecRequest` and a `ToolCtx`.
   - It then calls `ToolOrchestrator::new().run(&mut UnifiedExecRuntime::new(self, shell_mode), …)`.
9. **`ToolOrchestrator::run`** → **`ToolOrchestrator::run_attempt`** (`codex-core`, `core/src/tools/orchestrator.rs`)
   - **Generic:** both functions are generic over `T: ToolRuntime<Rq, Out>`, here `UnifiedExecRuntime`. `run` handles approval, which may request user or guardian approval.
   - `run` picks the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux that gives `SandboxType::LinuxSeccomp`.
   - `run` builds a `SandboxAttempt` carrying `codex_linux_sandbox_exe`, and `run_attempt` calls `tool.run(req, &attempt, &ctx)`.
   - If the sandbox denies the command, `run` may retry without the sandbox after approval.
10. **`UnifiedExecRuntime::run`** (impl `ToolRuntime<UnifiedExecRequest, UnifiedExecProcess>`; `codex-core`, `core/src/tools/runtimes/unified_exec.rs`)
    - **Shell snapshot:** `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`) may wrap the argv to source the shell snapshot.
    - `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`) makes a `SandboxCommand`.
    - It calls `attempt.env_for(command, options, managed_network)`.
11. **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`) → **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`)
    - **Sandbox wrapping:** for `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`).
    - It prepends the `codex-linux-sandbox` executable to the argv and sets `arg0` with `linux_sandbox_arg0_override`.
    - The result passes through `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing/mod.rs`).
12. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`)
    - `UnifiedExecRuntime::run` calls it with `NoopSpawnLifecycle`, which has no inherited fds.
    - The Windows branch does not apply. It is not remote, so it does not take `environment.get_exec_backend().start(…)`.
    - It splits `request.command` into program and args. Here the program is the `codex-linux-sandbox` executable.
13. **Process spawn** (`codex-utils-pty`). Which function runs depends on whether a TTY was requested:
    - With `tty = true`: `pty::spawn_process_with_inherited_fds` (`utils/pty/src/pty.rs`) calls `spawn_process_portable`, because the fd list is empty. That calls `openpty` and then `pair.slave.spawn_command(command_builder)` from portable-pty.
    - With `tty = false`: `pipe::spawn_process_no_stdin_with_inherited_fds` (`utils/pty/src/pipe.rs`) calls `spawn_process_with_stdin_mode`, which calls `tokio::process::Command::spawn()`.
    - The result is wrapped by `UnifiedExecProcess::from_spawned`.

Notes:
- With `UnifiedExecShellMode::ZshFork`, step 10 first tries `zsh_fork_backend::maybe_prepare_unified_exec`. It still ends in `open_session_with_exec_env`, but with an escalation `SpawnLifecycle` that has inherited fds. On the TTY path, that leads to `spawn_process_preserving_fds`.
- On Linux the spawned process is the `codex-linux-sandbox` helper, which applies bubblewrap/Landlock/seccomp and then runs the command. That happens in the separate `codex-linux-sandbox` crate, and I did not trace it here.
- If the selected sandbox is `SandboxType::None`, `transform` leaves the argv unchanged.
