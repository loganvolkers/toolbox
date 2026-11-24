# Answers

## Q1

`codex-tools` is the package in `codex-rs/tools/Cargo.toml`. It is declared as a workspace dependency in `codex-rs/Cargo.toml` (`codex-tools = { path = "tools" }`). The lists below count normal `[dependencies]` only, including target-specific ones. Dev- and build-dependencies are left out.

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

**Indirect dependents (18).** These depend on `codex-tools` only through other workspace crates. Each row shows one shortest chain:

| Crate | Example chain |
|---|---|
| `codex-app-server-client` | → `codex-app-server` → `codex-tools` |
| `codex-app-server-daemon` | → `codex-app-server-transport` → `codex-core` → `codex-tools` |
| `codex-app-server-test-client` | → `codex-core` → `codex-tools` |
| `codex-app-server-transport` | → `codex-core` → `codex-tools` |
| `codex-cli` | → `codex-app-server` → `codex-tools` |
| `codex-cloud-config` | → `codex-core` → `codex-tools` |
| `codex-cloud-tasks` | → `codex-core` → `codex-tools` |
| `codex-core-api` | → `codex-core` → `codex-tools` |
| `codex-exec` | → `codex-core` → `codex-tools` |
| `codex-guardian` | → `codex-core` → `codex-tools` |
| `codex-lmstudio` | → `codex-core` → `codex-tools` |
| `codex-mcp-server` | → `codex-core` → `codex-tools` |
| `codex-memories-write` | → `codex-core` → `codex-tools` |
| `codex-ollama` | → `codex-core` → `codex-tools` |
| `codex-thread-manager-sample` | → `codex-core-api` → `codex-core` → `codex-tools` |
| `codex-tui` | → `codex-app-server-client` → `codex-app-server` → `codex-tools` |
| `codex-utils-oss` | → `codex-core` → `codex-tools` |
| `codex-utils-sandbox-summary` | → `codex-core` → `codex-tools` |

These lists come from the `[dependencies]` and `[target.*.dependencies]` tables of every workspace member listed in `codex-rs/Cargo.toml` `[workspace] members`. No workspace dependency renames `codex-tools` with `package = ...`.

## Q2

The trait method is `ThreadStore::read_thread(&self, params: ReadThreadParams)` in `codex-rs/thread-store/src/store.rs`, crate `codex-thread-store`. It is implemented by `LocalThreadStore` (`local/mod.rs`) and `InMemoryThreadStore` (`in_memory.rs`).

Every non-test caller below calls it on an `Arc<dyn ThreadStore>` (or `&Arc<dyn ThreadStore>`).

**Crate `codex-app-server`, file `app-server/src/request_processors/thread_processor.rs`.** The receiver is the field `ThreadRequestProcessor::thread_store: Arc<dyn ThreadStore>`. The callers are these methods of `ThreadRequestProcessor`:
- `thread_archive_response` (two calls: the thread itself, then each descendant thread)
- `load_persisted_thread_for_read`
- `load_thread_turns_list_history`
- `read_stored_thread_for_resume`
- `read_stored_thread_for_new_fork`
- `load_thread_from_resume_source_or_send_internal` (two calls)
- `attach_thread_name`
- `get_thread_summary_response_inner`

**Crate `codex-thread-store`.**
- `LiveThread::read_thread` in `thread-store/src/live_thread.rs`. This is a different, two-argument method (`include_archived`, `include_history`). It calls `self.thread_store.read_thread(ReadThreadParams { .. })` on its field `thread_store: Arc<dyn ThreadStore>`.

**Crate `codex-core`.**
- `run_turn_stop_hooks` (free function) in `core/src/hook_runtime.rs`. It calls `sess.services.thread_store.read_thread(..)`, where `SessionServices::thread_store` is an `Arc<dyn ThreadStore>` (`core/src/state/service.rs`).
- `thread_title_from_thread_store` (free function) in `core/src/session/mod.rs`. Its `None` branch calls `thread_store.read_thread(..)` on a `&Arc<dyn ThreadStore>` parameter. Its `Some` branch calls `LiveThread::read_thread`, which doesn't count.
- `ThreadManagerState::read_stored_thread` in `core/src/thread_manager.rs`. It calls `self.thread_store.read_thread(params)`, where the field is `Arc<dyn ThreadStore>`.

**Same-named calls that don't count.** Several call sites call a different `read_thread`:
- the two-argument `LiveThread::read_thread` or `CodexThread::read_thread`, as in `TurnRequestProcessor::start_detached_review`, `apply_bespoke_event_handling`, `CodexThread::read_thread` and `ThreadManager::spawn_subagent`;
- the free function `codex_thread_store::local::read_thread::read_thread`, as in `LocalThreadStore::read_thread` and `update_thread_metadata.rs`.

All other `.read_thread(ReadThreadParams{..})` calls are in test code. They are in the `#[cfg(test)] mod tests` blocks of `local/read_thread.rs`, `local/update_thread_metadata.rs` and `local/mod.rs`, and in `tests/` directories.

## Q3

`get_git_repo_root(base_dir: &Path) -> Option<PathBuf>` is defined in `codex-rs/git-utils/src/info.rs`, crate `codex-git-utils`.

**Direct non-test callers.** These are the intermediates:

| Intermediate | Crate | File |
|---|---|---|
| `write_session_meta` | `codex-rollout` | `rollout/src/recorder.rs` |
| `git_diff_to_remote` | `codex-git-utils` | `git-utils/src/info.rs` |
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics` | `analytics/src/reducer.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store` | `thread-store/src/thread_metadata_sync.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` |
| `terminal_title_project_root` | `codex-cli` | `cli/src/doctor/title.rs` |
| `git_check` | `codex-cli` | `cli/src/doctor/git.rs` |
| `environment_id_from_cwd` | `codex-secrets` | `secrets/src/lib.rs` |
| `run_exec_session` | `codex-exec` | `exec/src/lib.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` |
| `display_path_for` | `codex-tui` | `tui/src/diff_render.rs` |
| `run_turn` | `codex-core` | `core/src/session/turn.rs` |
| `Session::new` | `codex-core` | `core/src/session/session.rs` |
| `build_turn_metadata_header` | `codex-core` | `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core` | `core/src/turn_metadata.rs` |

**Functions exactly one intermediate away.** None of these calls `get_git_repo_root` itself, and none is a test:

| Function | Crate, file | Via |
|---|---|---|
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout`, `rollout/src/recorder.rs` | `write_session_meta` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server`, `app-server/src/request_processors/git_processor.rs` | `git_diff_to_remote` |
| `AnalyticsReducer::ingest` | `codex-analytics`, `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `LiveThread::create` | `codex-thread-store`, `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `list_marketplaces_with_home` | `codex-core-plugins`, `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `terminal_title_check` | `codex-cli`, `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `build_report` | `codex-cli`, `cli/src/doctor.rs` | `git_check` (called twice inside `run_async_check(..)`) |
| `run_main` | `codex-exec`, `exec/src/lib.rs` | `run_exec_session` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui`, `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui`, `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::reformat_description` | `codex-tui`, `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `from` in `impl From<DiffSummary> for Box<dyn Renderable>` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui`, `tui/src/history_cell/patches.rs` | `display_path_for` |
| `RegularTask::run` (`impl SessionTask for RegularTask`) | `codex-core`, `core/src/tasks/regular.rs` | `run_turn` |
| `Codex::spawn_internal` | `codex-core`, `core/src/session/mod.rs` | `Session::new` |
| `Session::make_turn_context` | `codex-core`, `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core`, `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write`, `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |

Notes:
- `build_report` also reaches `get_git_repo_root` through two intermediates (`terminal_title_check` → `terminal_title_project_root`). It still qualifies, because of `git_check`.
- `environment_id_from_cwd` (`codex-secrets`) is called only by the test `environment_id_fallback_has_cwd_prefix`, so it adds nothing.
- `display_path_for` is also called by the test `display_path_prefers_cwd_without_git_repo`, which doesn't count.
- `GitRequestProcessor::git_diff_to_remote` (app-server) calls `git_diff_to_origin`, so it is two intermediates away and is not listed.
- The `run_turn` in `codex-thread-manager-sample` (`thread-manager-sample/src/main.rs`) is a different function, so it is not a caller of `codex-core`'s `run_turn`.

## Q4

This is the path for a local session on Linux with the default (direct) unified-exec shell mode. All functions are in `codex-core` (`codex-rs/core/src`) unless another crate is named.

1. **`try_run_sampling_request`** (`session/turn.rs`) reads the model stream. On `ResponseEvent::OutputItemDone(item)` it builds a `HandleOutputCtx` and calls `handle_output_item_done`. If that returns a `tool_future`, it pushes the future onto `in_flight: FuturesOrdered<BoxFuture<..>>`. That set is awaited later, for example by `drain_in_flight`.
2. **`handle_output_item_done`** (`stream_events_utils.rs`) calls **`ToolRouter::build_tool_call`** (`tools/router.rs`), which turns the `ResponseItem::FunctionCall` into a `ToolCall` with a `ToolPayload::Function`. It then records the item and calls `ctx.tool_runtime.clone().handle_tool_call(call, token)`, boxed as an `InFlightFuture`.
3. **`ToolCallRuntime::handle_tool_call`** → **`ToolCallRuntime::handle_tool_call_with_source`** (`tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` calls `tokio::spawn(async move { .. })`, wrapped in an `AbortOnDropHandle`. It does this when the future is built, not when it is first polled.
   - Inside the task, it takes the parallel-execution `RwLock` (a read lock for parallel tools, a write lock otherwise) and calls the next step.
   - The returned future selects between the task handle and cancellation.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`** → **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`tools/router.rs`) builds a `ToolInvocation`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`tools/registry.rs`).
   - It looks up the handler with `ToolRegistry::tool`, which returns an `Arc<dyn CoreToolRuntime>` from `HashMap<ToolName, Arc<dyn CoreToolRuntime>>`.
   - It runs the pre-use hooks.
   - It calls `SessionTelemetry::log_tool_result_with_tags` (crate `codex-otel`, `otel/src/events/session_telemetry.rs`). That function is generic over a closure `F: FnOnce() -> Fut`, and the closure calls the next step.
6. **`handle_any_tool`** (`tools/registry.rs`) calls `tool.handle(invocation)` on a `&dyn CoreToolRuntime`.
   - **Trait object:** this is dynamic dispatch through `ToolExecutor<ToolInvocation>::handle`. The `ToolExecutor` trait is defined in crate `codex-tools` (`tools/src/tool_executor.rs`), and `CoreToolRuntime` is a supertrait-based trait in `codex-core`.
   - If the tool's exposure was overridden, the call first goes through `ExposureOverride::handle`.
7. **`ExecCommandHandler::handle`** (`tools/handlers/unified_exec/exec_command.rs`).
   - It parses the arguments, resolves the turn environment and cwd, and calls `get_command` (`tools/handlers/unified_exec.rs`), which builds the shell argv.
   - It handles permissions and an `apply_patch` intercept.
   - It then calls `session.services.unified_exec_manager.exec_command(ExecCommandRequest { .. }, &context)`.
8. **`UnifiedExecProcessManager::exec_command`** → **`UnifiedExecProcessManager::open_session_with_sandbox`** (`unified_exec/process_manager.rs`).
   - `open_session_with_sandbox` builds the environment and the exec-policy approval requirement.
   - It creates a `ToolOrchestrator` and a `UnifiedExecRuntime`, then calls `orchestrator.run(&mut runtime, &req, ..)`.
9. **`ToolOrchestrator::run`** (`tools/orchestrator.rs`).
   - **Generic:** it is generic over `T: ToolRuntime<Rq, Out>`, monomorphised here for `UnifiedExecRuntime`, so this is static dispatch.
   - It handles approval, then picks the sandbox with `SandboxManager::select_initial` (crate `codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, `get_platform_sandbox` gives `SandboxType::LinuxSeccomp` when a sandbox is required.
   - It builds a `SandboxAttempt` and calls **`ToolOrchestrator::run_attempt`**.
   - `run_attempt` calls `tool.run(req, &attempt, &ctx)` through the generic. If the sandbox denies the command, a retry without the sandbox can go through `run_attempt` again.
10. **`UnifiedExecRuntime::run`** (`impl ToolRuntime<UnifiedExecRequest, UnifiedExecProcess>`, `tools/runtimes/unified_exec.rs`). This is where the command is wrapped for the sandbox:
    - `maybe_wrap_shell_lc_with_snapshot` wraps the command in the shell snapshot. This is not the sandbox.
    - `build_sandbox_command` (`tools/runtimes/mod.rs`) turns it into a `SandboxCommand`.
    - **`SandboxAttempt::env_for`** (`tools/sandboxing.rs`) calls **`SandboxManager::transform`** (crate `codex-sandboxing`, `sandboxing/src/manager.rs`).
    - For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`codex-sandboxing`, `sandboxing/src/landlock.rs`). It prepends the `codex_linux_sandbox_exe` path to the argv and sets an `arg0` override (`linux_sandbox_arg0_override`).
    - For `SandboxType::None`, the argv is passed through unchanged.
    - `ExecRequest::from_sandbox_exec_request` (`sandboxing/mod.rs`) turns the result into an `ExecRequest`.
    - `run` then calls `self.manager.open_session_with_exec_env(process_id, &exec_env, tty, Box::new(NoopSpawnLifecycle), environment)`.
    - In zsh-fork shell mode, `zsh_fork_backend::maybe_prepare_unified_exec` may build the `ExecRequest` instead, before the same call.
11. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`unified_exec/process_manager.rs`).
    - The Windows branch is compiled out on Linux.
    - Because the environment is local (`!environment.is_remote()`), it skips the exec-server backend.
    - It splits `request.command` into the program (the `codex-linux-sandbox` executable when sandboxed) and its args.
    - With `tty` it calls `codex_utils_pty::pty::spawn_process_with_inherited_fds`. Without it, it calls `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`.
12. **The spawn, in crate `codex-utils-pty`:**
    - **TTY path** (`utils/pty/src/pty.rs`): `spawn_process_with_inherited_fds` calls `spawn_process_portable`, because `NoopSpawnLifecycle` has no inherited fds. That function calls `pair.slave.spawn_command(command_builder)`, a portable-pty `CommandBuilder`, and this is where the process is spawned. With inherited fds it would call `spawn_process_preserving_fds` instead, which uses `command.spawn()`.
    - **Pipe path** (`utils/pty/src/pipe.rs`): `spawn_process_no_stdin_with_inherited_fds` calls `spawn_process_with_stdin_mode`, which runs `command.spawn()` on a `tokio::process::Command`.

    After the spawn, `UnifiedExecProcess::from_spawned` wraps the child.

I am not sure about one point. Whether `ExposureOverride` sits in front of `ExecCommandHandler` depends on how the registry was built for the session. Either way, the call goes through the `dyn CoreToolRuntime` trait object.
