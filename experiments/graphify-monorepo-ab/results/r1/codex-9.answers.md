# Answers

## Q1

Source: the `[dependencies]` sections (including target-specific ones) of each workspace member's `Cargo.toml` under `codex-rs/`. Dev- and build-dependencies are excluded.

**Direct dependents of `codex-tools` (7).** Each of these lists `codex-tools` as a normal dependency:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `codex-rs/app-server/Cargo.toml` |
| `codex-core` | `codex-rs/core/Cargo.toml` |
| `codex-extension-api` | `codex-rs/ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `codex-rs/ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `codex-rs/ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `codex-rs/ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `codex-rs/ext/web-search/Cargo.toml` |

**Indirect dependents (22).** These reach `codex-tools` only through other crates. The column on the right names the workspace crates through which each one gets there.

| Crate | Reaches `codex-tools` through |
|---|---|
| `app_test_support` | `codex-core`, `core_test_support` |
| `codex-app-server-client` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `codex-core` |
| `codex-app-server-transport` | `codex-core` |
| `codex-chatgpt` | `codex-core` |
| `codex-cli` | `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, `codex-chatgpt`, `codex-cloud-tasks`, `codex-memories-write`, `codex-app-server-daemon`, `codex-app-server-test-client` |
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
| `core_test_support` | `codex-core`, `codex-extension-api` |
| `mcp_test_support` | `codex-mcp-server`, `core_test_support` |

The table includes test-support library crates (`app_test_support`, `core_test_support`, `mcp_test_support`) because they are workspace packages and depend on these crates through normal `[dependencies]`.

## Q2

The trait method is `ThreadStore::read_thread` (`codex-rs/thread-store/src/store.rs`, crate `codex-thread-store`). Its implementations are `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`). Below are the non-test functions that call the trait method. In every case the receiver is an `Arc<dyn ThreadStore>`.

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `thread_title_from_thread_store` (free fn) | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>` (only on the `None` branch; the `Some` branch calls `LiveThread::read_thread`) |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` (free fn) | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`Arc<dyn ThreadStore>`, field declared in `core/src/session/mod.rs`) |
| `ThreadRequestProcessor::thread_archive_response` (two calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls) | `codex-app-server` | same | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same | same |

Some similarly named functions were excluded because they are not calls to the trait method:

- **Other methods named `read_thread`.** `CodexThread::read_thread` (`core/src/codex_thread.rs`) calls `LiveThread::read_thread`, not the trait method. For that reason these callers of `CodexThread::read_thread` are not listed: `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`), `TurnRequestProcessor` code in `app-server/src/request_processors/turn_processor.rs`, and the `ThreadRolledBack` handling in `app-server/src/bespoke_event_handling.rs`.
- **The free function `codex_thread_store::local::read_thread::read_thread`.** This is what `LocalThreadStore` uses inside its implementation, and it is not the trait method.
- **Test code.** The calls in the `#[cfg(test)] mod tests` of `thread-store/src/local/read_thread.rs`, `local/update_thread_metadata.rs` and `local/mod.rs`, in `*_tests.rs` files, and in `tests/` directories are all tests.

## Q3

`get_git_repo_root` is defined in `codex-rs/git-utils/src/info.rs` (crate `codex-git-utils`).

These are its direct, non-test callers, which serve as the intermediates:

| Intermediate | Crate | File |
|---|---|---|
| `terminal_title_project_root` | `codex-cli` | `cli/src/doctor/title.rs` |
| `git_check` | `codex-cli` | `cli/src/doctor/git.rs` |
| `git_diff_to_remote` | `codex-git-utils` | `git-utils/src/info.rs` |
| `write_session_meta` | `codex-rollout` | `rollout/src/recorder.rs` |
| `run_exec_session` | `codex-exec` | `exec/src/lib.rs` |
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics` | `analytics/src/reducer.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store` | `thread-store/src/thread_metadata_sync.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` |
| `display_path_for` | `codex-tui` | `tui/src/diff_render.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` |
| `run_turn` | `codex-core` | `core/src/session/turn.rs` |
| `Session::new` | `codex-core` | `core/src/session/session.rs` |
| `build_turn_metadata_header` | `codex-core` | `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core` | `core/src/turn_metadata.rs` |
| `environment_id_from_cwd` | `codex-secrets` | `secrets/src/lib.rs` |

**Answer: the functions exactly one hop away.** Each calls one of the intermediates above, and none calls `get_git_repo_root` directly.

| Function | Crate | File | Calls (intermediate) |
|---|---|---|---|
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::display_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` (it also calls the nested fn below) |
| `reformat_description` (fn nested inside `display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |

Notes:

- **`git_diff_to_remote` has no non-test callers of its own.** The `git_diff_to_remote(&cwd)` call in `GitRequestProcessor::git_diff_to_origin` is the git-utils function. `MessageProcessor`'s call `self.git_processor.git_diff_to_remote(...)` (`app-server/src/message_processor.rs`) is a different method name on a different type, so it does not count. The other callers, in `core/src/git_info_tests.rs`, are tests.
- **Callers of `environment_id_from_cwd` are tests only.** Its only caller is in the `#[cfg(test)]` module of `secrets/src/lib.rs`, so it contributes no answer.
- **`run_turn` in `thread-manager-sample/src/main.rs` is not the same function.** It is that crate's own `run_turn`, not `codex-core`'s.
- **`build_report` calls `git_check` directly.** It does so inside `tokio::join!` (`git_check(config.cwd.as_path())`), and that future is passed to `run_async_check`.

## Q4

This is the path for a local (not remote) environment on Linux, with the default `exec_command` arguments (`tty` defaults to `false` in `default_tty`, `core/src/tools/handlers/unified_exec.rs`) and the default (non-ZshFork) shell mode. Each step lists the function, its crate and its file.

1. **`try_run_sampling_request`**, `codex-core`, `core/src/session/turn.rs`. Its stream loop matches `ResponseEvent::OutputItemDone(item)`, builds a `HandleOutputCtx`, and calls `handle_output_item_done`. The returned `tool_future` is pushed onto `in_flight` and awaited later. Its caller is `run_sampling_request`, in the same file.
2. **`handle_output_item_done`**, `codex-core`, `core/src/stream_events_utils.rs`. It calls `ToolRouter::build_tool_call`.
3. **`ToolRouter::build_tool_call`**, `codex-core`, `core/src/tools/router.rs`. It turns `ResponseItem::FunctionCall` into a `ToolCall { tool_name: "exec_command", payload: ToolPayload::Function { arguments } }`. Back in `handle_output_item_done`, the item is recorded and then `ctx.tool_runtime.clone().handle_tool_call(call, cancellation_token)` is called.
4. **`ToolCallRuntime::handle_tool_call`**, `codex-core`, `core/src/tools/parallel.rs`. It calls `handle_tool_call_with_source(call, ToolCallSource::Direct, ...)`.
5. **`ToolCallRuntime::handle_tool_call_with_source`**, `codex-core`, `core/src/tools/parallel.rs`. **This is the spawned task.** This plain `fn` runs `tokio::spawn(async move { ... })`, wrapped in an `AbortOnDropHandle`, as soon as it is called. Inside the task, it takes the parallel-execution `RwLock` and calls `router.dispatch_tool_call_with_terminal_outcome(...)`. The future that is returned only awaits the task handle or cancellation.
6. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`**, `codex-core`, `core/src/tools/router.rs`. These build a `ToolInvocation` and call `self.registry.dispatch_any_with_terminal_outcome(...)`.
7. **`ToolRegistry::dispatch_any_with_terminal_outcome`**, `codex-core`, `core/src/tools/registry.rs`. **This is the first trait object.** `self.tool(&tool_name)` looks up an `Arc<dyn CoreToolRuntime>` in `ToolRegistry.tools` (a `HashMap<ToolName, Arc<dyn CoreToolRuntime>>`). The method then runs pre-tool-use hooks and calls `SessionTelemetry::log_tool_result_with_tags` (crate `codex-otel`, `otel/src/events/session_telemetry.rs`). That function is generic over a closure `F: FnOnce() -> Fut` and runs the closure, which calls `handle_any_tool`.
8. **`handle_any_tool`**, `codex-core`, `core/src/tools/registry.rs`. It calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. `handle` belongs to the supertrait `ToolExecutor<ToolInvocation>`, which is defined in crate `codex-tools` (`tools/src/tool_executor.rs`) and re-exported by `core/src/tools/registry.rs`. The call goes through dynamic dispatch, using `async_trait`.
9. **`<ExecCommandHandler as ToolExecutor<ToolInvocation>>::handle`**, `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`. If the registry wrapped the handler in `ExposureOverride` (`registry.rs`), that wrapper's `handle` forwards to this one through another `Arc<dyn CoreToolRuntime>`. This function parses the arguments, resolves the environment and cwd, and builds the shell command with `get_command`. It allocates a process id and normalizes the permissions. It also tries `intercept_apply_patch` and, when that does not take the command, calls `UnifiedExecProcessManager::exec_command`. That call is on the concrete `session.services.unified_exec_manager`.
10. **`UnifiedExecProcessManager::exec_command`**, `codex-core`, `core/src/unified_exec/process_manager.rs`. It calls `self.open_session_with_sandbox(&request, cwd, context)`.
11. **`UnifiedExecProcessManager::open_session_with_sandbox`**, `codex-core`, same file. It builds the env (`create_env`, `apply_unified_exec_env`), computes `exec_approval_requirement` through `exec_policy.create_exec_approval_requirement_for_command`, and creates a `ToolOrchestrator` and a `UnifiedExecRuntime`. It then calls `orchestrator.run(&mut runtime, &req, ...)`.
12. **`ToolOrchestrator::run`**, `codex-core`, `core/src/tools/orchestrator.rs`. **This step is generic:** `T: ToolRuntime<Rq, Out>`, monomorphized here for `UnifiedExecRuntime`. It handles approval (`request_approval` and related) and then picks the sandbox with `SandboxManager::select_initial` (crate `codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, `get_platform_sandbox` returns `SandboxType::LinuxSeccomp` when the policy requires a platform sandbox. Otherwise it returns `SandboxType::None`, for example for full access or a bypass on the first attempt. `run` then builds a `SandboxAttempt` and calls `Self::run_attempt`. If the sandbox denies the command and the request is approved, it builds a retry attempt and calls `run_attempt` again.
13. **`ToolOrchestrator::run_attempt`**, `codex-core`, same file. It begins network approval and calls `tool.run(req, &attempt, &ctx)`. This call is static dispatch through the generic `ToolRuntime` bound.
14. **`<UnifiedExecRuntime as ToolRuntime<UnifiedExecRequest, UnifiedExecProcess>>::run`**, `codex-core`, `core/src/tools/runtimes/unified_exec.rs`. It prepares the env and wraps the command for the shell snapshot with `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`), which applies to local environments only. It then calls `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`) and **`SandboxAttempt::env_for`** (`core/src/tools/sandboxing.rs`).
15. **`SandboxAttempt::env_for`**, `codex-core`, `core/src/tools/sandboxing.rs`. It calls **`SandboxManager::transform`** (crate `codex-sandboxing`, `sandboxing/src/manager.rs`). **This is where the command is wrapped for the sandbox.** For `SandboxType::LinuxSeccomp`, `transform` checks bubblewrap support with `ensure_linux_bubblewrap_is_supported`. It then builds the arguments with `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`) and prepends the `codex-linux-sandbox` executable as argv[0], with `arg0` set by `linux_sandbox_arg0_override`. For `SandboxType::None`, the argv passes through unchanged. Back in `env_for`, the `SandboxExecRequest` is converted to `crate::sandboxing::ExecRequest` by `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing/mod.rs`).
16. **`UnifiedExecProcessManager::open_session_with_exec_env`**, `codex-core`, `core/src/unified_exec/process_manager.rs`. `UnifiedExecRuntime::run` calls it through `self.manager`, passing `Box::new(NoopSpawnLifecycle)` as a `Box<dyn SpawnLifecycle>`. That trait is in `core/src/unified_exec/process.rs`, and its `inherited_fds()` returns an empty list. The function skips the Windows branch and the `environment.is_remote()` branch, which would go through `get_exec_backend().start(...)`. It splits `request.command` into program and args, then calls the `codex-utils-pty` spawn function:
    - When `tty` is `false` (the default), it calls **`codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`**, which calls **`spawn_process_with_stdin_mode`** (crate `codex-utils-pty`, `utils/pty/src/pipe.rs`). That function builds a `tokio::process::Command` with a `pre_exec` hook that detaches from the TTY and sets the parent-death signal on Linux. **`command.spawn()` there is the point where the process is spawned.**
    - When `tty` is `true`, it calls **`codex_utils_pty::pty::spawn_process_with_inherited_fds`** (`utils/pty/src/pty.rs`). Because the inherited-fd list is empty, that goes to **`spawn_process_portable`**, which runs `openpty` and spawns with `pair.slave.spawn_command(...)` from portable-pty.
17. The spawned process is wrapped by `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`). Control returns up to `exec_command`, which stores the process and collects output until the yield deadline.

**Where the command is actually run under the sandbox.** With `LinuxSeccomp`, the process that gets spawned is the `codex-linux-sandbox` helper, and the original command is in its arguments. The helper then applies bubblewrap, seccomp or Landlock before it runs the command. With `SandboxType::None`, the command is spawned directly.

**A second path not followed above.** If the shell mode is `UnifiedExecShellMode::ZshFork`, `UnifiedExecRuntime::run` first tries `zsh_fork_backend::maybe_prepare_unified_exec`, after the same `build_sandbox_command` and `env_for` steps. When that succeeds, it calls `open_session_with_exec_env` with the prepared request and its spawn lifecycle. I did not trace that path further.
