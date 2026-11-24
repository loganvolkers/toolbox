# Answers

## Q1

`codex-tools` is the package in `codex-rs/tools/Cargo.toml`, declared in the root `codex-rs/Cargo.toml` as `codex-tools = { path = "tools" }`. Every crate that uses it does so through `codex-tools = { workspace = true }` under `[dependencies]`. No crate lists it only as a dev- or build-dependency.

**Direct dependents (7):**

| Crate | Manifest |
|---|---|
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-core` | `core/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents (22):** These crates reach `codex-tools` only through other crates' normal dependencies. The "through" column names the first-hop workspace dependencies that lead to `codex-tools`.

| Crate | Manifest | Through |
|---|---|---|
| `codex-app-server-client` | `app-server-client/Cargo.toml` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `app-server-daemon/Cargo.toml` | `codex-app-server-transport` (which depends on `codex-core`) |
| `codex-app-server-test-client` | `app-server-test-client/Cargo.toml` | `codex-core` |
| `codex-app-server-transport` | `app-server-transport/Cargo.toml` | `codex-core` |
| `codex-chatgpt` | `chatgpt/Cargo.toml` | `codex-core` |
| `codex-cli` | `cli/Cargo.toml` | `codex-core`, `codex-app-server`, `codex-exec`, `codex-tui`, `codex-mcp-server`, `codex-chatgpt`, `codex-cloud-tasks`, `codex-memories-write`, `codex-app-server-daemon`, `codex-app-server-test-client` |
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
| `core_test_support` | `core/tests/common/Cargo.toml` | `codex-core`, `codex-extension-api` |
| `app_test_support` | `app-server/tests/common/Cargo.toml` | `codex-core`, `core_test_support` |
| `mcp_test_support` | `mcp-server/tests/common/Cargo.toml` | `codex-mcp-server`, `core_test_support` |

That is 22 crates: 18 from the explicit `members` list, plus `codex-chatgpt` and the 3 test-support crates, which are implicit members.

Notes on scope:
- `codex-chatgpt` (`chatgpt/`) and the three test-support crates `core_test_support`, `app_test_support` and `mcp_test_support` are not in the root `[workspace] members` list. They are workspace members only implicitly, because members use them as path dependencies. I counted them because Cargo treats them as workspace packages. `codex-chatgpt` is a normal dependency of `codex-cli`. The three `*_test_support` crates are test helpers that other crates normally pull in as dev-dependencies. Drop those four if you only want the explicit `members` list.
- `codex-message-history` and `codex-windows-sandbox` (`windows-sandbox-rs/`) are also implicit members. Neither depends on `codex-tools`, directly or indirectly.

## Q2

The trait method is `ThreadStore::read_thread(&self, params: ReadThreadParams)` in `codex-thread-store`, `thread-store/src/store.rs`. It is implemented by `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`). Every non-test call to it goes through an `Arc<dyn ThreadStore>`. I found no call through a generic bound or a concrete implementing type outside tests.

| Function | Crate | File | Receiver |
|---|---|---|---|
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`SessionServices::thread_store: Arc<dyn ThreadStore>`, `core/src/state/service.rs`). Used for the `SubAgent(ThreadSpawn)` branch to find the parent's rollout path. |
| `thread_title_from_thread_store` | `codex-core` | `core/src/session/mod.rs` | the `thread_store: &Arc<dyn ThreadStore>` parameter, in the `None` (no live thread) branch |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` (two calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same file | same (the branch without a rollout path) |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls) | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same file | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same file | same |

Excluded, because they call a different `read_thread`:
- `CodexThread::read_thread` (`core/src/codex_thread.rs`) calls `LiveThread::read_thread`, not the trait method.
- Several functions call `CodexThread::read_thread` or `LiveThread::read_thread`, not the trait method:
  - `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`);
  - the `Some(live_thread)` branch of `thread_title_from_thread_store`;
  - the rollback branch in `app-server/src/bespoke_event_handling.rs`;
  - `TurnRequestProcessor::start_detached_review` (`app-server/src/request_processors/turn_processor.rs`).
- In `codex-thread-store`, `LocalThreadStore`'s methods and `local/update_thread_metadata.rs` and `local/live_writer.rs` call the free function `local::read_thread::read_thread` (`thread-store/src/local/read_thread.rs`), not the trait method.

All other `.read_thread(...)` calls on stores are test functions and are excluded:
- the `#[cfg(test)] mod tests` blocks of `thread-store/src/local/mod.rs`, `local/read_thread.rs` and `local/update_thread_metadata.rs`;
- `core/src/session/tests.rs`;
- `app-server/tests/...`.

`LiveThread::read_thread` is itself called by `CodexThread::read_thread`, so those callers reach the trait method indirectly. I have not listed them as callers.

## Q3

`get_git_repo_root(base_dir: &Path) -> Option<PathBuf>` is defined in `codex-git-utils`, `git-utils/src/info.rs`, and re-exported from `git-utils/src/lib.rs`.

**Direct non-test callers (the intermediates):**
- `AnalyticsReducer::ingest_skill_invoked` (`codex-analytics`, `analytics/src/reducer.rs`)
- `git_diff_to_remote` (`codex-git-utils`, `git-utils/src/info.rs`)
- `Session::new` (`codex-core`, `core/src/session/session.rs`)
- `run_turn` (`codex-core`, `core/src/session/turn.rs`)
- `build_turn_metadata_header` (`codex-core`, `core/src/turn_metadata.rs`)
- `TurnMetadataState::new` (`codex-core`, `core/src/turn_metadata.rs`)
- `environment_id_from_cwd` (`codex-secrets`, `secrets/src/lib.rs`). Its only caller is a test, so it contributes nothing.
- `ChatWidget::status_line_project_root_for_cwd` (`codex-tui`, `tui/src/chatwidget/status_surfaces.rs`)
- `display_path_for` (`codex-tui`, `tui/src/diff_render.rs`)
- `discover_marketplace_paths_from_roots` (`codex-core-plugins`, `core-plugins/src/marketplace.rs`)
- `ThreadMetadataSync::for_create` (`codex-thread-store`, `thread-store/src/thread_metadata_sync.rs`)
- `write_session_meta` (`codex-rollout`, `rollout/src/recorder.rs`)
- `git_check` (`codex-cli`, `cli/src/doctor/git.rs`)
- `terminal_title_project_root` (`codex-cli`, `cli/src/doctor/title.rs`)
- `run_exec_session` (`codex-exec`, `exec/src/lib.rs`)

**Functions exactly one intermediate away.** None of these calls `get_git_repo_root` directly, and none is a test function.

| Function | Crate | File | Calls (intermediate) |
|---|---|---|---|
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `AnalyticsReducer::ingest_skill_invoked` |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `from` in `impl From<DiffSummary> for Box<dyn Renderable>` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::display_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` (directly, for the "Migrate enabled plugins from" case) |
| `reformat_description` (a nested `fn` inside `ExternalAgentConfigMigrationScreen::display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |

Excluded:
- Calls from test code: `environment_id_fallback_has_cwd_prefix` in `secrets/src/lib.rs` `mod tests`, `display_path_prefers_cwd_without_git_repo` in `tui/src/diff_render.rs` `mod tests`, and the callers in `core/src/git_info_tests.rs`, `core/src/turn_metadata_tests.rs` and `core/src/session/tests.rs`.
- `MessageProcessor`'s call to `self.git_processor.git_diff_to_remote(params)` (`app-server/src/message_processor.rs`). That is the method `GitRequestProcessor::git_diff_to_remote`, a different function from `codex_git_utils::git_diff_to_remote`. It reaches `get_git_repo_root` through two intermediates.

Uncertainty: `reformat_description` is a nested function, so whether it counts as its own function depends on your convention. Its enclosing `display_description` qualifies either way.

## Q4

The trace assumes Linux, a local (non-remote) environment, the unified-exec shell mode not set to `ZshFork`, and a platform sandbox being selected. Every function is in `codex-core` unless a different crate is named.

1. **`try_run_sampling_request`** (`core/src/session/turn.rs`). It is reached from `run_turn` → `run_sampling_request`.
   - It loops over the model stream from `ModelClientSession::stream`.
   - On `ResponseEvent::OutputItemDone(item)` it builds a `HandleOutputCtx`, which carries a clone of the `ToolCallRuntime`, and calls `handle_output_item_done`.
   - It pushes the returned `tool_future` into the `in_flight: FuturesOrdered<BoxFuture<…>>`, which is awaited later (also in `drain_in_flight`).
2. **`handle_output_item_done`** (`core/src/stream_events_utils.rs`).
   - It calls **`ToolRouter::build_tool_call`** (`core/src/tools/router.rs`), which turns `ResponseItem::FunctionCall { name: "exec_command", arguments, call_id, .. }` into a `ToolCall` with `ToolPayload::Function { arguments }`.
   - It records the item with `record_completed_response_item`.
   - It wraps `ToolCallRuntime::handle_tool_call(call, token)` in `Box::pin` as an `InFlightFuture`.
3. **`ToolCallRuntime::handle_tool_call`**, then **`ToolCallRuntime::handle_tool_call_with_source`** (`core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` runs `tokio::spawn(async move { … router.dispatch_tool_call_with_terminal_outcome(…) })` inside an `AbortOnDropHandle`. The spawn happens when the method is called, not when it is polled.
   - The task takes the `parallel_execution` `RwLock`: a read lock, because `exec_command` supports parallel calls.
   - The returned future awaits the join handle, or handles cancellation. This is the future that sits in `in_flight`.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`core/src/tools/router.rs`). These build a `ToolInvocation` and call the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`core/src/tools/registry.rs`).
   - It looks up `self.tool(&tool_name)`, which returns an **`Arc<dyn CoreToolRuntime>`** (trait object). For `exec_command` this is an `ExecCommandHandler`, registered by `planned_tools.add(ExecCommandHandler::new(…))` in `core/src/tools/spec_plan.rs`.
   - It checks `matches_kind`, calls `notify_tool_start`, and runs the pre-tool-use hooks.
   - It calls `SessionTelemetry::log_tool_result_with_tags` (`codex-otel`, `otel/src/events/session_telemetry.rs`). That method is **generic** over a closure `F: FnOnce() -> Fut` and runs the closure.
6. The closure calls **`handle_any_tool`** (`core/src/tools/registry.rs`), which calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. `handle` is the `ToolExecutor<ToolInvocation>::handle` method of the `codex-tools` trait `ToolExecutor` (`tools/src/tool_executor.rs`), a supertrait of `CoreToolRuntime`, and it is dispatched **dynamically**.
7. **`ExecCommandHandler::handle`** (impl `ToolExecutor<ToolInvocation>`, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - It parses the arguments, resolves the turn environment (`resolve_tool_environment`) and the cwd, and allocates a process id.
   - `get_command` turns `cmd` into a shell argv, e.g. `[shell, "-lc", cmd]`.
   - It applies the permission checks and `intercept_apply_patch`.
   - It calls `session.services.unified_exec_manager.exec_command(ExecCommandRequest { … }, &context)`.
8. **`UnifiedExecProcessManager::exec_command`**, then **`UnifiedExecProcessManager::open_session_with_sandbox`** (`core/src/unified_exec/process_manager.rs`).
   - `open_session_with_sandbox` builds the env with `create_env` and `apply_unified_exec_env`.
   - It computes `exec_approval_requirement` through `exec_policy.create_exec_approval_requirement_for_command`.
   - It creates `ToolOrchestrator::new()` (which holds `codex_sandboxing::SandboxManager`) and `UnifiedExecRuntime::new(self, shell_mode)`, then calls `orchestrator.run(&mut runtime, &req, &tool_ctx, …)`.
9. **`ToolOrchestrator::run`** (`core/src/tools/orchestrator.rs`).
   - It is **generic** over `T: ToolRuntime<Rq, Out>`, monomorphised with `T = UnifiedExecRuntime`, so dispatch is static.
   - It handles approval (`ToolOrchestrator::request_approval` if needed).
   - It chooses the sandbox with **`SandboxManager::select_initial`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). `UnifiedExecRuntime::sandbox_preference` is `Auto`, and on Linux `get_platform_sandbox` returns `SandboxType::LinuxSeccomp` when a sandbox is required.
   - It builds a `SandboxAttempt` that carries `codex_linux_sandbox_exe` and `use_legacy_landlock`.
   - **`ToolOrchestrator::run_attempt`** then calls `tool.run(req, &attempt, &ctx)`, which is the `ToolRuntime::run` trait method, resolved statically through the generic. If the sandbox denies the command, `run` may retry through `run_attempt` with `SandboxType::None` after approval.
10. **`UnifiedExecRuntime::run`** (impl `ToolRuntime`, `core/src/tools/runtimes/unified_exec.rs`).
    - It calls `exec_env_for_sandbox_permissions`.
    - It calls `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`). This is a shell-snapshot wrapper, not the sandbox wrapper.
    - It calls **`build_sandbox_command`** (`core/src/tools/runtimes/mod.rs`), which produces a `SandboxCommand { program, args, cwd, env, … }`.
    - It calls **`SandboxAttempt::env_for`** (`core/src/tools/sandboxing.rs`).
    - With `ZshFork` shell mode it would first try `zsh_fork_backend::maybe_prepare_unified_exec`. Not taken in this trace.
11. **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). **This is where the command is wrapped for the sandbox.**
    - For `SandboxType::LinuxSeccomp` it runs `ensure_linux_bubblewrap_is_supported`.
    - It builds the helper arguments with `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`): `--sandbox-policy-cwd … --command-cwd … --permission-profile <json> [--use-legacy-landlock] [--allow-network-for-proxy] -- <original argv>`.
    - It prepends the `codex-linux-sandbox` executable path and sets `arg0` through `linux_sandbox_arg0_override`.
    - At run time, the `codex-linux-sandbox` helper applies the actual bubblewrap and seccomp (or legacy Landlock) restrictions.
    - `env_for` then converts the `SandboxExecRequest` with `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing/mod.rs`).
12. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`core/src/unified_exec/process_manager.rs`). It is called from `UnifiedExecRuntime::run` with `Box::new(NoopSpawnLifecycle)`, so no fds are inherited.
    - The Windows branch is compiled out on Linux.
    - `environment.is_remote()` is false for a local session, so it skips the remote branch that would call `Environment::get_exec_backend().start(…)`.
    - It splits `request.command` into the program (the sandbox helper) and its args.
13. **The process is spawned** in `codex-utils-pty`:
    - With `tty: true`: `codex_utils_pty::pty::spawn_process_with_inherited_fds`, then `spawn_process_portable` (`utils/pty/src/pty.rs`), which spawns with `portable_pty` `pair.slave.spawn_command(CommandBuilder)` and uses `arg0` as the program name.
    - Otherwise: `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`, then `spawn_process_with_stdin_mode` (`utils/pty/src/pipe.rs`), which calls `tokio::process::Command::new(program)` with `.arg0(arg0)` and then `command.spawn()`.
    - Afterwards, `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`) wraps the child. Back in `UnifiedExecProcessManager::exec_command`, output streaming (`start_streaming_output`) and `store_process` begin.

Summary of where control passes through indirection:
- **Spawned task:** `tokio::spawn` in `ToolCallRuntime::handle_tool_call_with_source`. The `Box::pin` future from `handle_output_item_done` waits on it from `in_flight` in `try_run_sampling_request`.
- **Trait objects:** `Arc<dyn CoreToolRuntime>` in `ToolRegistry`, with `ToolExecutor::handle` called in `handle_any_tool` (an `async_trait` method from `codex-tools`).
- **Generics:**
  - the closure passed to `SessionTelemetry::log_tool_result_with_tags`;
  - `ToolOrchestrator::run` and `run_attempt` over `T: ToolRuntime<Rq, Out>`, which call `UnifiedExecRuntime::run` statically.
- **Sandbox wrapping:** `SandboxAttempt::env_for` → `SandboxManager::transform` in `codex-sandboxing`, which prefixes the argv with `codex-linux-sandbox` and its policy flags.

Uncertainty: I did not trace what happens inside the `codex-linux-sandbox` helper after it is spawned, where it re-executes bubblewrap and the command. The question stops at the spawn of the wrapped command.
