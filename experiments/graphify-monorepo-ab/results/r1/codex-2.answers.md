# Answers

## Q1

Dependencies counted here are normal ones only, between packages of the `codex-rs/` workspace. Each fact comes from the crate's `Cargo.toml` (`codex-tools = { workspace = true }`; the workspace entry is `codex-tools = { path = "tools" }` in `codex-rs/Cargo.toml`). The Seikika portal entity `crate:codex-tools` (field `dependents`) and `cargo metadata` agree.

**Direct dependents (7):**

| Crate | Declared in |
|---|---|
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-core` | `core/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents (22).** These depend on `codex-tools` only through other crates. One example path is shown for each:

| Crate | Example path |
|---|---|
| `app_test_support` | → `codex-core` → `codex-tools` |
| `codex-app-server-client` | → `codex-app-server` → `codex-tools` |
| `codex-app-server-daemon` | → `codex-app-server-transport` → `codex-core` → `codex-tools` |
| `codex-app-server-test-client` | → `codex-core` → `codex-tools` |
| `codex-app-server-transport` | → `codex-core` → `codex-tools` |
| `codex-chatgpt` | → `codex-core` → `codex-tools` |
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
| `core_test_support` | → `codex-core` → `codex-tools` |
| `mcp_test_support` | → `codex-mcp-server` → `codex-core` → `codex-tools` |

`app_test_support`, `core_test_support` and `mcp_test_support` are test-support packages in the workspace (`app-server/tests/common`, `core/tests/common`, `mcp-server/tests/common`). They declare these crates under `[dependencies]`, so they count.

## Q2

The trait method is declared as `async fn read_thread(&self, params: ReadThreadParams)` on `trait ThreadStore` in `thread-store/src/store.rs`. Twelve non-test functions call it. Every one calls it through an `Arc<dyn ThreadStore>`; none goes through a generic or a concrete implementing type.

| # | Function | Crate | File | Receiver |
|---|---|---|---|---|
| 1 | `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| 2 | `thread_title_from_thread_store` | `codex-core` | `core/src/session/mod.rs` | parameter `thread_store: &Arc<dyn ThreadStore>`, in the `None` branch |
| 3 | `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`SessionServices.thread_store: Arc<dyn ThreadStore>`, `core/src/state/service.rs`) |
| 4 | `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| 5 | `ThreadRequestProcessor::thread_archive_response` (two calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| 6 | `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same file | same |
| 7 | `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same file | same |
| 8 | `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same file | same |
| 9 | `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same file | same |
| 10 | `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls) | `codex-app-server` | same file | same |
| 11 | `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same file | same |
| 12 | `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same file | same |

These calls look similar but do not count, because they call a different function:

- **Calls to `LiveThread::read_thread` (an inherent method), not the trait method:**
  - `CodexThread::read_thread` in `core/src/codex_thread.rs`. It calls `live_thread.read_thread(..)`.
  - The `Some(live_thread)` branch of `thread_title_from_thread_store`.
- **Calls to `CodexThread::read_thread`:**
  - `ThreadManager::spawn_subagent` in `core/src/thread_manager.rs`.
  - The `review_thread.read_thread(..)` call in `app-server/src/request_processors/turn_processor.rs`.
  - The `conversation.read_thread(..)` call in `app-server/src/bespoke_event_handling.rs`.
- **Calls to the free function `codex_thread_store::local::read_thread::read_thread`:**
  - The implementation `impl ThreadStore for LocalThreadStore` → `read_thread` in `thread-store/src/local/mod.rs`, plus other functions in that file.
  - `resume_thread` in `thread-store/src/local/live_writer.rs`.
  - The functions in `thread-store/src/local/update_thread_metadata.rs`.
- **Test code:** the `.read_thread(...)` calls inside the `#[cfg(test)] mod tests` blocks of `thread-store/src/local/{mod,read_thread,update_thread_metadata}.rs` and inside `tests/` directories are excluded.

## Q3

`get_git_repo_root` is defined in `git-utils/src/info.rs` (crate `codex-git-utils`). Its direct non-test callers, which are the intermediates, are:

| Intermediate (calls `get_git_repo_root` directly) | Crate, file |
|---|---|
| `terminal_title_project_root` | `codex-cli`, `cli/src/doctor/title.rs` |
| `git_check` | `codex-cli`, `cli/src/doctor/git.rs` |
| `git_diff_to_remote` | `codex-git-utils`, `git-utils/src/info.rs` |
| `write_session_meta` | `codex-rollout`, `rollout/src/recorder.rs` |
| `run_exec_session` | `codex-exec`, `exec/src/lib.rs` |
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics`, `analytics/src/reducer.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store`, `thread-store/src/thread_metadata_sync.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins`, `core-plugins/src/marketplace.rs` |
| `run_turn` | `codex-core`, `core/src/session/turn.rs` |
| `Session::new` | `codex-core`, `core/src/session/session.rs` |
| `build_turn_metadata_header` | `codex-core`, `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core`, `core/src/turn_metadata.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui`, `tui/src/chatwidget/status_surfaces.rs` |
| `environment_id_from_cwd` | `codex-secrets`, `secrets/src/lib.rs` |
| `display_path_for` | `codex-tui`, `tui/src/diff_render.rs` |

**Answer: the functions that call `get_git_repo_root` through exactly one intermediate.** None of them calls `get_git_repo_root` directly.

| Function | Crate, file | Via |
|---|---|---|
| `build_report` | `codex-cli`, `cli/src/doctor.rs` | `git_check` |
| `terminal_title_check` | `codex-cli`, `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server`, `app-server/src/request_processors/git_processor.rs` | `git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout`, `rollout/src/recorder.rs` | `write_session_meta` |
| `run_main` | `codex-exec`, `exec/src/lib.rs` | `run_exec_session` |
| `AnalyticsReducer::ingest` | `codex-analytics`, `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `LiveThread::create` | `codex-thread-store`, `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `list_marketplaces_with_home` | `codex-core-plugins`, `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `RegularTask::run` (`impl SessionTask for RegularTask`) | `codex-core`, `core/src/tasks/regular.rs` | `run_turn` |
| `Codex::spawn_internal` | `codex-core`, `core/src/session/mod.rs` | `Session::new` |
| `Session::make_turn_context` | `codex-core`, `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core`, `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write`, `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui`, `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui`, `tui/src/diff_render.rs` | `display_path_for` |
| `reformat_description` (a nested fn inside `ExternalAgentConfigMigrationScreen::display_description`) | `codex-tui`, `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui`, `tui/src/history_cell/patches.rs` | `display_path_for` |
| `Notification::display` | `codex-tui`, `tui/src/chatwidget/notifications.rs` | `display_path_for` |

Notes:

- `GitRequestProcessor::git_diff_to_remote` (the app-server method) calls `git_diff_to_origin`, so it is two intermediates away and is not listed.
- `environment_id_from_cwd` is only called from its `#[cfg(test)]` module, so it adds no caller.
- These callers are excluded because they are test code:
  - `core/src/git_info_tests.rs`, which calls `git_diff_to_remote`.
  - `core/src/turn_metadata_tests.rs`.
  - `core/src/session/tests.rs`, which calls `Session::new`.
  - The `display_path_for` test in `tui/src/diff_render.rs`.

## Q4

The path below is the default one: a local environment, the unified-exec tool set, and no zsh-fork backend. Each step names the function, its crate and its file.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`). It reads `ResponseEvent::OutputItemDone(item)` from the model stream (`client_session.stream(..)`), builds a `HandleOutputCtx`, and calls `handle_output_item_done`.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`).
   - **`ToolRouter::build_tool_call`** (`codex-core`, `core/src/tools/router.rs`) turns the `ResponseItem::FunctionCall` into a `ToolCall { tool_name: "exec_command", payload: ToolPayload::Function { arguments } }`.
   - It records the item, then calls `ctx.tool_runtime.clone().handle_tool_call(call, token)` and pushes the returned future into the `in_flight` `FuturesOrdered`. `try_run_sampling_request` and `drain_in_flight` await it later.
3. **`ToolCallRuntime::handle_tool_call`** → **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` is a plain `fn`, and it immediately calls `tokio::spawn(async move { .. })`, wrapped in `AbortOnDropHandle`.
   - The task takes the parallel-execution `RwLock` (a read lock if the tool supports parallel calls, otherwise a write lock), then calls the router. The caller only selects on the join handle and on cancellation.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`** → **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`). These build a `ToolInvocation`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`).
   - It looks up the tool as an **`Arc<dyn CoreToolRuntime>` (trait object)** with `ToolRegistry::tool`, then runs pre-tool-use hooks.
   - It wraps execution in `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`), which takes a generic closure.
   - Inside that closure it calls **`handle_any_tool`** (same file).
6. **`handle_any_tool`** calls `tool.handle(invocation)`. This is dynamic dispatch on the `ToolExecutor<ToolInvocation>` trait (declared in `codex-tools`, `tools/src/tool_executor.rs`; `CoreToolRuntime: ToolExecutor<ToolInvocation>`), using `async_trait`.
   - It resolves to **`<ExecCommandHandler as ToolExecutor<ToolInvocation>>::handle`** (`codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - `ExecCommandHandler` is registered with `PlannedTools::add` in `core/src/tools/spec_plan.rs` and is not wrapped in `ExposureOverride`.
   - `handle` parses the arguments, resolves the environment and cwd, and builds the shell argv with `get_command` (`core/src/tools/handlers/unified_exec.rs`).
   - It handles permissions and the `intercept_apply_patch` short-circuit, then calls `UnifiedExecProcessManager::exec_command`.
7. **`UnifiedExecProcessManager::exec_command`** (`codex-core`, `core/src/unified_exec/process_manager.rs`) calls **`UnifiedExecProcessManager::open_session_with_sandbox`** (same file).
   - That function builds the env, a `UnifiedExecRuntime`, the exec-policy approval requirement and a `UnifiedExecToolRequest`.
   - It then calls `ToolOrchestrator::run`.
8. **`ToolOrchestrator::run`** (`codex-core`, `core/src/tools/orchestrator.rs`).
   - It is **generic** over `T: ToolRuntime<Rq, Out>`; here `T = UnifiedExecRuntime`, so this is static dispatch.
   - It handles approval (`request_approval` when needed), then picks the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, with a restricted policy, that is `SandboxType::LinuxSeccomp`.
   - It builds a `SandboxAttempt` and calls **`ToolOrchestrator::run_attempt`**, which calls `tool.run(req, attempt, ctx)`. This is the generic `ToolRuntime::run`, declared in `core/src/tools/sandboxing.rs`.
   - If the sandbox denies the command and the runtime allows escalation, `run_attempt` runs a second time after approval.
9. **`<UnifiedExecRuntime as ToolRuntime<..>>::run`** (`codex-core`, `core/src/tools/runtimes/unified_exec.rs`).
   - It prepares the command: `maybe_wrap_shell_lc_with_snapshot` (for local environments) and `build_sandbox_command` (both in `core/src/tools/runtimes/mod.rs`).
   - **Sandbox wrapping:**
     - `run` calls **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`).
     - `env_for` calls **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`).
     - For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`).
     - It then prepends the `codex-linux-sandbox` executable (`codex_linux_sandbox_exe`) to the argv, with an `arg0` override. For `SandboxType::None` the argv is unchanged.
     - The result is converted into an `ExecRequest` by `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing/mod.rs`).
   - `run` then calls `UnifiedExecProcessManager::open_session_with_exec_env` with `NoopSpawnLifecycle`.
   - When `UnifiedExecShellMode::ZshFork` applies, `zsh_fork_backend::maybe_prepare_unified_exec` prepares a different `ExecRequest` first, and that request is passed to the same function.
10. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`).
    - The environment is not remote, so the `get_exec_backend().start(..)` exec-server branch is skipped. The Windows branch is compiled out on Linux.
    - It splits `request.command` into program and args. With `NoopSpawnLifecycle` there are no inherited fds.
11. **Process spawn** (`codex-utils-pty`):
    - **`tty = false` (the default, `default_tty` in `core/src/tools/handlers/unified_exec.rs`):** `pipe::spawn_process_no_stdin_with_inherited_fds` → `spawn_process_with_stdin_mode` (`utils/pty/src/pipe.rs`). The latter builds a `tokio::process::Command::new(program)` and calls `command.spawn()`.
    - **`tty = true`:** `pty::spawn_process_with_inherited_fds` → `spawn_process_portable` (`utils/pty/src/pty.rs`). The latter calls `openpty` and `pair.slave.spawn_command(..)`. `spawn_process_preserving_fds` is used only when inherited fds are present.

    The spawned child is wrapped by `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`).
    - If sandboxed, the program actually spawned is `codex-linux-sandbox`. It applies bubblewrap or landlock/seccomp and then execs the user's command.
    - Back in `UnifiedExecProcessManager::exec_command`, output streaming (`start_streaming_output`) and `collect_output_until_deadline` follow.

**Where control passes through something indirect:**

| Kind | Where |
|---|---|
| Spawned task | the `tokio::spawn` in `ToolCallRuntime::handle_tool_call_with_source` |
| Trait objects | `Arc<dyn CoreToolRuntime>` and `ToolExecutor::handle` in `ToolRegistry::dispatch_any_with_terminal_outcome` and `handle_any_tool` |
| Generics | `ToolOrchestrator::run` and `run_attempt` over `T: ToolRuntime`, calling `UnifiedExecRuntime::run`; the closure passed to `SessionTelemetry::log_tool_result_with_tags` |
| Sandbox wrapping | `SandboxAttempt::env_for` → `SandboxManager::transform`, called from `UnifiedExecRuntime::run` |

I did not trace inside `codex-linux-sandbox` itself, past the point where it is spawned.
