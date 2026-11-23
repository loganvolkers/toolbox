# Answers

## Q1

Dependencies counted are normal `[dependencies]` (target-specific ones included) between workspace packages, as declared in each crate's `Cargo.toml` under `codex-rs/`.

**Direct dependents of `codex-tools`** (7). Each lists `codex-tools` in its own `[dependencies]`:

| Crate | Path |
|---|---|
| `codex-app-server` | `app-server/` |
| `codex-core` | `core/` |
| `codex-extension-api` | `ext/extension-api/` |
| `codex-goal-extension` | `ext/goal/` |
| `codex-image-generation-extension` | `ext/image-generation/` |
| `codex-memories-extension` | `ext/memories/` |
| `codex-web-search-extension` | `ext/web-search/` |

**Indirect dependents** (22). These depend on `codex-tools` only through other crates. The parenthesis gives the dependency or dependencies that carry them there:

- `app_test_support` (via `codex-core`, `core_test_support`)
- `codex-app-server-client` (via `codex-app-server`, `codex-core`)
- `codex-app-server-daemon` (via `codex-app-server-transport`)
- `codex-app-server-test-client` (via `codex-core`)
- `codex-app-server-transport` (via `codex-core`)
- `codex-chatgpt` (via `codex-core`)
- `codex-cli` (via `codex-app-server`, `codex-core`, `codex-exec`, `codex-tui`, `codex-mcp-server`, and others)
- `codex-cloud-config` (via `codex-core`)
- `codex-cloud-tasks` (via `codex-core`, `codex-tui`)
- `codex-core-api` (via `codex-core`, `codex-extension-api`)
- `codex-exec` (via `codex-core`, `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary`)
- `codex-guardian` (via `codex-core`, `codex-extension-api`)
- `codex-lmstudio` (via `codex-core`)
- `codex-mcp-server` (via `codex-core`, `codex-extension-api`)
- `codex-memories-write` (via `codex-core`)
- `codex-ollama` (via `codex-core`)
- `codex-thread-manager-sample` (via `codex-core-api`)
- `codex-tui` (via `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary`)
- `codex-utils-oss` (via `codex-core`, `codex-lmstudio`, `codex-ollama`)
- `codex-utils-sandbox-summary` (via `codex-core`)
- `core_test_support` (via `codex-core`, `codex-extension-api`)
- `mcp_test_support` (via `codex-mcp-server`, `core_test_support`)

`app_test_support`, `core_test_support` and `mcp_test_support` are test-support packages. They are still workspace crates, and their dependency on `codex-core` and the others is a normal dependency, so they are listed. Source: the `Cargo.toml` of each crate. The Toolbox portal's `crate:codex-tools` entity (`dependents`) gives the same seven direct dependents.

## Q2

The trait method is declared at `codex-rs/thread-store/src/store.rs:68`: `ThreadStore::read_thread(&self, params: ReadThreadParams)`. It is implemented by `LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore` (`thread-store/src/in_memory.rs`).

The non-test callers below all go through an `Arc<dyn ThreadStore>`. No non-test caller uses a generic or calls a concrete implementing type directly.

**`codex-app-server`**, file `app-server/src/request_processors/thread_processor.rs`. The receiver is `self.thread_store: Arc<dyn ThreadStore>`.
- `ThreadRequestProcessor::thread_archive_response`: two calls, one for the thread and one per descendant.
- `ThreadRequestProcessor::load_persisted_thread_for_read`
- `ThreadRequestProcessor::load_thread_turns_list_history`
- `ThreadRequestProcessor::read_stored_thread_for_resume`
- `ThreadRequestProcessor::read_stored_thread_for_new_fork`
- `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal`: two calls.
- `ThreadRequestProcessor::attach_thread_name`
- `ThreadRequestProcessor::get_thread_summary_response_inner`

**`codex-core`**
- `run_turn_stop_hooks` in `core/src/hook_runtime.rs`, through `sess.services.thread_store`. That field is `Arc<dyn ThreadStore>` (`core/src/state/service.rs`).
- `thread_title_from_thread_store` in `core/src/session/mod.rs`. It takes the `thread_store: &Arc<dyn ThreadStore>` parameter, in its `None` branch.
- `ThreadManagerState::read_stored_thread` in `core/src/thread_manager.rs`, through `self.thread_store: Arc<dyn ThreadStore>`.

**`codex-thread-store`**
- `LiveThread::read_thread` in `thread-store/src/live_thread.rs`, through `self.thread_store: Arc<dyn ThreadStore>`. It is a different method that happens to share the name, but it calls the trait method.

**Excluded** because they call a different `read_thread` and not the trait method:
- `TurnRequestProcessor::start_detached_review` (`app-server/src/request_processors/turn_processor.rs`), `apply_bespoke_event_handling` (`app-server/src/bespoke_event_handling.rs`) and `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`). These call `CodexThread::read_thread` (`core/src/codex_thread.rs`).
- `CodexThread::read_thread` itself, which calls `LiveThread::read_thread`.
- The `live_thread.read_thread(..)` branch of `thread_title_from_thread_store`, which calls `LiveThread::read_thread`.
- `LocalThreadStore::read_thread` (the trait impl), `live_writer.rs` and `update_thread_metadata.rs`. These call the free function `local::read_thread::read_thread`.

**Test functions are excluded.** These are the calls in `#[cfg(test)] mod tests` of `thread-store/src/local/read_thread.rs`, `local/update_thread_metadata.rs` and `local/mod.rs`, and in the crates' `tests/` directories.

## Q3

First, the non-test functions that call `get_git_repo_root` (`codex-rs/git-utils/src/info.rs:32`) **directly**:
- `git_diff_to_remote` (codex-git-utils, `info.rs`)
- `terminal_title_project_root` (codex-cli, `cli/src/doctor/title.rs`)
- `git_check` (codex-cli, `cli/src/doctor/git.rs`)
- `AnalyticsReducer::ingest_skill_invoked` (codex-analytics, `analytics/src/reducer.rs`)
- `write_session_meta` (codex-rollout, `rollout/src/recorder.rs`)
- `ThreadMetadataSync::for_create` (codex-thread-store, `thread-store/src/thread_metadata_sync.rs`)
- `run_exec_session` (codex-exec, `exec/src/lib.rs`)
- `discover_marketplace_paths_from_roots` (codex-core-plugins, `core-plugins/src/marketplace.rs`)
- `environment_id_from_cwd` (codex-secrets, `secrets/src/lib.rs`)
- `ChatWidget::status_line_project_root_for_cwd` (codex-tui, `tui/src/chatwidget/status_surfaces.rs`)
- `display_path_for` (codex-tui, `tui/src/diff_render.rs`)
- `run_turn` (codex-core, `core/src/session/turn.rs`)
- `Session::new` (codex-core, `core/src/session/session.rs`)
- `build_turn_metadata_header` (codex-core, `core/src/turn_metadata.rs`)
- `TurnMetadataState::new` (codex-core, `core/src/turn_metadata.rs`)

The functions that call one of those, and do not call `get_git_repo_root` directly themselves, are below. Each line gives the caller, its crate and file, and the intermediate function it goes through.

**`codex-app-server`**
- `GitRequestProcessor::git_diff_to_origin` (`app-server/src/request_processors/git_processor.rs`), via `git_diff_to_remote`.

**`codex-cli`**
- `terminal_title_check` (`cli/src/doctor/title.rs`), via `terminal_title_project_root`.
- `build_report` (`cli/src/doctor.rs`), via `git_check`. It calls it twice, in the config-loaded branch and in the fallback branch.

**`codex-analytics`**
- `AnalyticsReducer::ingest` (`analytics/src/reducer.rs`), via `ingest_skill_invoked`.

**`codex-rollout`**
- `RolloutWriterState::write_session_meta_if_needed` (`rollout/src/recorder.rs`), via `write_session_meta`.

**`codex-thread-store`**
- `LiveThread::create` (`thread-store/src/live_thread.rs`), via `ThreadMetadataSync::for_create`.

**`codex-exec`**
- `run_main` (`exec/src/lib.rs`), via `run_exec_session`.

**`codex-core-plugins`**
- `list_marketplaces_with_home` (`core-plugins/src/marketplace.rs`), via `discover_marketplace_paths_from_roots`.

**`codex-memories-write`**
- `MemoryStartupContext::stage_one_request_context` (`memories/write/src/runtime.rs`), via `codex_core::build_turn_metadata_header`.

**`codex-core`**
- `RegularTask::run`, the `SessionTask` impl (`core/src/tasks/regular.rs`), via `run_turn`.
- `Codex::spawn_internal` (`core/src/session/mod.rs`), via `Session::new`.
- `Session::make_turn_context` (`core/src/session/turn_context.rs`), via `TurnMetadataState::new`.
- `spawn_review_thread` (`core/src/session/review.rs`), via `TurnMetadataState::new`.

**`codex-tui`**
- `ChatWidget::status_line_project_root_name_for_cwd` (`tui/src/chatwidget/status_surfaces.rs`), via `status_line_project_root_for_cwd`.
- `<Box<dyn Renderable> as From<DiffSummary>>::from` (`tui/src/diff_render.rs`), via `display_path_for`.
- `render_changes_block` (`tui/src/diff_render.rs`), via `display_path_for`.
- `new_view_image_tool_call` (`tui/src/history_cell/patches.rs`), via `display_path_for`.
- `Notification::display` (`tui/src/chatwidget/notifications.rs`), via `display_path_for`.
- `ExternalAgentConfigMigrationScreen::display_description` (`tui/src/external_agent_config_migration.rs`), via `display_path_for`.
- `reformat_description` (same file), via `display_path_for`. This is a nested `fn` declared inside `display_description`.

**Notes**
- `environment_id_from_cwd` (codex-secrets) has no non-test caller. Its only caller is the test `environment_id_fallback_has_cwd_prefix`, so it contributes nothing.
- `MessageProcessor::handle_initialized_client_request` calls `GitRequestProcessor::git_diff_to_remote`. That is a method of the same name, not the free function `codex_git_utils::git_diff_to_remote`, so it is not counted.
- The `run_turn` in `thread-manager-sample/src/main.rs` and the one in `core/tests/suite/tool_parallelism.rs` are unrelated functions of the same name, so their callers are not counted.
- Test callers are excluded, for example in `core/src/git_info_tests.rs`, `core/src/turn_metadata_tests.rs`, `core/src/session/tests.rs` and the `diff_render.rs` tests.

## Q4

The path below is the local default: the environment is not remote (`Environment::is_remote()` is false) and the shell mode is not `ZshFork`. Everything is in `codex-core` unless another crate is named.

1. **`try_run_sampling_request`** (codex-core, `core/src/session/turn.rs`). It reads the model stream and matches `ResponseEvent::OutputItemDone(item)`, then calls `handle_output_item_done(&mut ctx, item, …)`. If that returns a `tool_future`, the future is pushed onto `in_flight` and polled later by this loop.
2. **`handle_output_item_done`** (codex-core, `core/src/stream_events_utils.rs`).
   - It calls `ToolRouter::build_tool_call(item)` (`core/src/tools/router.rs`), which turns the `ResponseItem::FunctionCall` for `exec_command` into a `ToolCall` with a `ToolPayload::Function`.
   - It records the item with `record_completed_response_item`.
   - It then builds the boxed future `ctx.tool_runtime.clone().handle_tool_call(call, cancellation_token)`.
3. **`ToolCallRuntime::handle_tool_call`**, then **`ToolCallRuntime::handle_tool_call_with_source`** (codex-core, `core/src/tools/parallel.rs`).
   - **Spawned task:** `handle_tool_call_with_source` runs the dispatch inside `tokio::spawn(async move { … })`, wrapped in an `AbortOnDropHandle`. Inside the task it takes the parallel-execution read or write lock and calls the router.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (codex-core, `core/src/tools/router.rs`). These build a `ToolInvocation` and call the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (codex-core, `core/src/tools/registry.rs`).
   - It looks up the tool as an `Arc<dyn CoreToolRuntime>`.
   - It runs pre-tool-use hooks.
   - It calls **`handle_any_tool(tool: &dyn CoreToolRuntime, …)`** (same file), which calls `tool.handle(invocation)`.
   - **Trait object:** this is dynamic dispatch on `dyn CoreToolRuntime`. Its `handle` comes from the supertrait `ToolExecutor<ToolInvocation>`, an `async_trait` defined in `codex-tools` (`tools/src/tool_executor.rs`).
6. **`ExecCommandHandler::handle`**, the `impl ToolExecutor<ToolInvocation> for ExecCommandHandler` (codex-core, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - It parses the arguments, resolves the turn environment and cwd, allocates a process id, and builds the shell argv with `get_command`.
   - It checks permissions and `intercept_apply_patch`.
   - It then calls `session.services.unified_exec_manager.exec_command(ExecCommandRequest { … }, &context)`.
7. **`UnifiedExecProcessManager::exec_command`** (codex-core, `core/src/unified_exec/process_manager.rs`). It calls `open_session_with_sandbox`.
8. **`UnifiedExecProcessManager::open_session_with_sandbox`** (same file).
   - It builds the environment with `create_env`, the exec-policy approval requirement, and a `UnifiedExecToolRequest`.
   - It creates a `UnifiedExecRuntime` and calls `ToolOrchestrator::run(&mut runtime, &req, …)`.
9. **`ToolOrchestrator::run`**, then **`ToolOrchestrator::run_attempt`** (codex-core, `core/src/tools/orchestrator.rs`).
   - **Generic:** both are generic over `T: ToolRuntime<Rq, Out>` (trait in `core/src/tools/sandboxing.rs`), so this is static dispatch.
   - `run` handles approval. It then picks the sandbox with `SandboxManager::select_initial` (codex-sandboxing, `sandboxing/src/manager.rs`). On Linux, `get_platform_sandbox` returns `SandboxType::LinuxSeccomp` when the policy requires a sandbox.
   - It builds a `SandboxAttempt` with `codex_linux_sandbox_exe`, and `run_attempt` calls `tool.run(req, &attempt, ctx)`.
   - On a sandbox denial, `run` may retry without the sandbox after approval.
10. **`UnifiedExecRuntime::run`**, the `impl ToolRuntime<UnifiedExecRequest, UnifiedExecProcess> for UnifiedExecRuntime` (codex-core, `core/src/tools/runtimes/unified_exec.rs`).
    - It wraps the command with the shell snapshot (`maybe_wrap_shell_lc_with_snapshot`).
    - It builds a `SandboxCommand` with `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`).
    - It calls `attempt.env_for(command, options, managed_network)`.
11. **`SandboxAttempt::env_for`** (codex-core, `core/src/tools/sandboxing.rs`). It calls `SandboxManager::transform`, then converts the result with `ExecRequest::from_sandbox_exec_request`.
12. **`SandboxManager::transform`** (codex-sandboxing, `sandboxing/src/manager.rs`). **This is where the command is wrapped for the sandbox.**
    - For `SandboxType::LinuxSeccomp` it checks bubblewrap support with `ensure_linux_bubblewrap_is_supported`.
    - It calls `create_linux_sandbox_command_args_for_permission_profile` (codex-sandboxing, `sandboxing/src/landlock.rs`). That builds `--sandbox-policy-cwd … --command-cwd … --permission-profile <json> [--use-legacy-landlock] [--allow-network-for-proxy] -- <original argv>`.
    - It prefixes that with the `codex-linux-sandbox` executable path and sets the `arg0` override with `linux_sandbox_arg0_override`.
    - With `SandboxType::None`, the argv is passed through unchanged.
13. **`UnifiedExecProcessManager::open_session_with_exec_env`** (codex-core, `core/src/unified_exec/process_manager.rs`). `UnifiedExecRuntime::run` calls it with the transformed `ExecRequest` and a `NoopSpawnLifecycle`.
    - Because the environment is not remote, it skips the exec-server branch (`environment.get_exec_backend().start(…)`).
    - It splits the program from its args and calls `codex-utils-pty`. By default `tty` is false (`default_tty` in `core/src/tools/handlers/unified_exec.rs`), so it calls `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`. With `tty: true` it calls `codex_utils_pty::pty::spawn_process_with_inherited_fds`.
14. **The process is spawned** in `codex-utils-pty`:
    - **Pipe path:** `spawn_process_no_stdin_with_inherited_fds`, then `spawn_process_with_stdin_mode` (`utils/pty/src/pipe.rs`), which calls `tokio::process::Command::spawn()`. Its `pre_exec` detaches from the TTY and sets the parent-death signal on Linux.
    - **PTY path:** `spawn_process_with_inherited_fds`, then `spawn_process_portable` (`utils/pty/src/pty.rs`), which calls `pair.slave.spawn_command(...)` from portable-pty. With inherited fds it uses `spawn_process_preserving_fds` instead, which calls `std::process::Command::spawn()`.
    - The process started is the `codex-linux-sandbox` helper wrapping the command, or the bare command if no sandbox was selected.
    - `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`) then wraps the spawned process.

The full chain, for reference: `try_run_sampling_request` → `handle_output_item_done` → `ToolCallRuntime::handle_tool_call` → `handle_tool_call_with_source` (spawned task) → `ToolRouter::dispatch_tool_call_with_terminal_outcome` → `dispatch_tool_call_with_code_mode_result_inner` → `ToolRegistry::dispatch_any_with_terminal_outcome` → `handle_any_tool` (`dyn CoreToolRuntime`) → `ExecCommandHandler::handle` → `UnifiedExecProcessManager::exec_command` → `open_session_with_sandbox` → `ToolOrchestrator::run` → `run_attempt` (generic `ToolRuntime`) → `UnifiedExecRuntime::run` → `SandboxAttempt::env_for` → `SandboxManager::transform` (sandbox wrapping, codex-sandboxing) → `UnifiedExecProcessManager::open_session_with_exec_env` → `codex_utils_pty` spawn (codex-utils-pty).

**Uncertain:** I did not trace what happens inside the `codex-linux-sandbox` helper after it is spawned (bubblewrap and seccomp setup). I also did not trace the `ZshFork` shell-mode path, which goes through `zsh_fork_backend::maybe_prepare_unified_exec` before the same `open_session_with_exec_env` call.
