# Answers

## Q1

Only normal dependencies (`[dependencies]` and target-specific dependency tables) between workspace packages in `codex-rs/` are counted. Dev- and build-dependencies are not. The data comes from each crate's `Cargo.toml`, as resolved by `cargo metadata --no-deps`, where the `kind` field is null for normal dependencies.

**Direct dependents (7).** Each of these lists `codex-tools` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `codex-rs/app-server/Cargo.toml` |
| `codex-core` | `codex-rs/core/Cargo.toml` |
| `codex-extension-api` | `codex-rs/ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `codex-rs/ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `codex-rs/ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `codex-rs/ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `codex-rs/ext/web-search/Cargo.toml` |

**Indirect dependents (22).** None of these lists `codex-tools` itself. Each reaches it through the crates shown in the second column, which are its own normal dependencies that lead to `codex-tools`:

| Crate | Reaches `codex-tools` through |
|---|---|
| `codex-app-server-client` | `codex-app-server`, `codex-core` |
| `codex-app-server-daemon` | `codex-app-server-transport` |
| `codex-app-server-test-client` | `codex-core` |
| `codex-app-server-transport` | `codex-core` |
| `codex-chatgpt` | `codex-core` |
| `codex-cli` | `codex-app-server`, `codex-core`, `codex-tui`, `codex-exec`, `codex-mcp-server`, and others |
| `codex-cloud-config` | `codex-core` |
| `codex-cloud-tasks` | `codex-core`, `codex-tui` |
| `codex-core-api` | `codex-core`, `codex-extension-api` |
| `codex-exec` | `codex-core`, `codex-app-server-client`, and others |
| `codex-guardian` | `codex-core`, `codex-extension-api` |
| `codex-lmstudio` | `codex-core` |
| `codex-mcp-server` | `codex-core`, `codex-extension-api` |
| `codex-memories-write` | `codex-core` |
| `codex-ollama` | `codex-core` |
| `codex-thread-manager-sample` | `codex-core-api` |
| `codex-tui` | `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-utils-oss` | `codex-core`, `codex-lmstudio`, `codex-ollama` |
| `codex-utils-sandbox-summary` | `codex-core` |
| `app_test_support` (`app-server/tests/common`) | `codex-core`, `core_test_support` |
| `core_test_support` (`core/tests/common`) | `codex-core`, `codex-extension-api` |
| `mcp_test_support` (`mcp-server/tests/common`) | `codex-mcp-server`, `core_test_support` |

The last three are test-support packages. They are workspace members, and they reach `codex-tools` through normal `[dependencies]`, so they are listed. Leave them out if you only want product crates.

## Q2

`ThreadStore::read_thread` is declared in `codex-rs/thread-store/src/store.rs` (trait `ThreadStore`, crate `codex-thread-store`). It is implemented by `LocalThreadStore` (`local/mod.rs`) and `InMemoryThreadStore` (`in_memory.rs`).

These are the non-test functions that call the trait method:

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `thread_title_from_thread_store` (free fn) | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` (free fn) | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`Arc<dyn ThreadStore>`, `SessionServices` in `core/src/state/service.rs`) |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` (two call sites) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two call sites) | `codex-app-server` | same | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same | same |

Every call goes through `Arc<dyn ThreadStore>`. No non-test call site uses a generic or calls the trait method on a concrete implementing type.

These calls were excluded because they target other functions that happen to be named `read_thread`:
- `CodexThread::read_thread` (`core/src/codex_thread.rs`), which calls `LiveThread::read_thread`. Its own callers include `ThreadManager::spawn_subagent`, `TurnRequestProcessor` in `app-server/.../turn_processor.rs`, and `bespoke_event_handling.rs`.
- The `LiveThread::read_thread` call inside `thread_title_from_thread_store`. That function is still listed above, because its `None` branch calls the trait method.
- The free function `codex_thread_store::local::read_thread::read_thread`. It is called by `LocalThreadStore::read_thread` and `read_thread_by_rollout_path` in `local/mod.rs`, by `local/live_writer.rs`, and by `local/update_thread_metadata.rs`.
- Calls in test code: the `#[cfg(test)] mod tests` blocks in `local/read_thread.rs`, `local/update_thread_metadata.rs` and `local/mod.rs`, and anything under `tests/` directories.

## Q3

`get_git_repo_root` is defined in `codex-rs/git-utils/src/info.rs` (crate `codex-git-utils`) and re-exported from `git-utils/src/lib.rs`.

**Non-test direct callers (the intermediates):**
- `terminal_title_project_root` (`codex-cli`, `cli/src/doctor/title.rs`)
- `git_check` (`codex-cli`, `cli/src/doctor/git.rs`)
- `git_diff_to_remote` (`codex-git-utils`, `git-utils/src/info.rs`)
- `write_session_meta` (free `async fn`; `codex-rollout`, `rollout/src/recorder.rs`)
- `AnalyticsReducer::ingest_skill_invoked` (`codex-analytics`, `analytics/src/reducer.rs`)
- `discover_marketplace_paths_from_roots` (`codex-core-plugins`, `core-plugins/src/marketplace.rs`)
- `ThreadMetadataSync::for_create` (`codex-thread-store`, `thread-store/src/thread_metadata_sync.rs`)
- `run_exec_session` (`codex-exec`, `exec/src/lib.rs`)
- `run_turn` (`codex-core`, `core/src/session/turn.rs`)
- `Session::new` (`codex-core`, `core/src/session/session.rs`)
- `build_turn_metadata_header` (`codex-core`, `core/src/turn_metadata.rs`)
- `TurnMetadataState::new` (`codex-core`, `core/src/turn_metadata.rs`)
- `display_path_for` (`codex-tui`, `tui/src/diff_render.rs`)
- `ChatWidget::status_line_project_root_for_cwd` (`codex-tui`, `tui/src/chatwidget/status_surfaces.rs`)
- `environment_id_from_cwd` (`codex-secrets`, `secrets/src/lib.rs`)

**Functions exactly one intermediate away.** Each calls one of the intermediates above and does not call `get_git_repo_root` itself:

| Caller | Crate | File | Via |
|---|---|---|---|
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` (two call sites) |
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `RegularTask::run` (`impl SessionTask for RegularTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `reformat_description` (nested fn inside `ExternalAgentConfigMigrationScreen::display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `status_line_project_root_for_cwd` |

Notes:
- `environment_id_from_cwd` (`codex-secrets`) has only a test caller, `environment_id_fallback_has_cwd_prefix`, so it contributes nothing.
- The app-server method `GitRequestProcessor::git_diff_to_remote` calls `git_diff_to_origin`. That puts it two intermediates away, so it is not listed.
- `run_turn` is `pub(crate)`. Its only non-test call is in `RegularTask::run`.
- `Session::new` and `TurnMetadataState::new` were found by searching for their qualified names. A `Self::new(...)` call inside their own `impl` blocks would also count, and none was found.
- A call made inside a closure or async block is attributed to the named function that contains it. For example, the `get_git_repo_root` call in `Session::new` sits inside the metrics-emitting block of that function.

## Q4

This is the path for a local (non-remote) environment on Linux, with the default direct unified-exec shell mode (not zsh-fork):

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`, free function). It receives each event from the model stream. On `ResponseEvent::OutputItemDone(item)`, it builds a `HandleOutputCtx` and calls `handle_output_item_done`. Upstream, the `OutputItemDone` events are forwarded from the API stream by a task that `tokio::spawn` starts inside `map_response_events` (`core/src/client.rs`), through an mpsc channel.
2. **`handle_output_item_done`** (`codex-core`, `core/src/stream_events_utils.rs`). It calls `ToolRouter::build_tool_call` (`core/src/tools/router.rs`), which turns `ResponseItem::FunctionCall` into a `ToolCall` with `ToolPayload::Function`. It records the item, then calls `ToolCallRuntime::handle_tool_call` and stores the returned future as `tool_future`. `try_run_sampling_request` pushes that future onto its `in_flight` `FuturesOrdered`, which `drain_in_flight` later awaits.
3. **`ToolCallRuntime::handle_tool_call`**, then **`ToolCallRuntime::handle_tool_call_with_source`** (`codex-core`, `core/src/tools/parallel.rs`). **This is where control passes through a spawned task.** `handle_tool_call_with_source` calls `tokio::spawn` right away (wrapped in an `AbortOnDropHandle`). The spawned task takes the parallel-execution `RwLock` and calls the next function.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`**, then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`codex-core`, `core/src/tools/router.rs`). These build a `ToolInvocation` and call `self.registry.dispatch_any_with_terminal_outcome`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`codex-core`, `core/src/tools/registry.rs`). It looks up the handler with `ToolRegistry::tool`, which returns an `Arc<dyn CoreToolRuntime>`, and runs the pre-tool-use hooks. It then calls **`SessionTelemetry::log_tool_result_with_tags`** (`codex-otel`, `otel/src/events/session_telemetry.rs`), which is generic over a closure `F`. That closure calls the next function.
6. **`handle_any_tool`** (`codex-core`, `core/src/tools/registry.rs`). It calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. **This is a trait-object dispatch.** `CoreToolRuntime` extends `ToolExecutor<ToolInvocation>`, an `async_trait` declared in `codex-tools` (`tools/src/tool_executor.rs`) and re-exported by `core/src/tools/registry.rs`.
7. **`ExecCommandHandler::handle`** (`impl ToolExecutor<ToolInvocation> for ExecCommandHandler`; `codex-core`, `core/src/tools/handlers/unified_exec/exec_command.rs`). Its `tool_name()` is `exec_command`. It parses the arguments, resolves the turn environment and cwd, and builds the shell argv with `get_command`. It checks permissions and tries `intercept_apply_patch`, then calls `session.services.unified_exec_manager.exec_command(ExecCommandRequest { .. }, &context)`.
8. **`UnifiedExecProcessManager::exec_command`**, then **`UnifiedExecProcessManager::open_session_with_sandbox`** (`codex-core`, `core/src/unified_exec/process_manager.rs`). `open_session_with_sandbox` builds the environment, computes the exec-approval requirement, and creates a `UnifiedExecRuntime` and a `ToolOrchestrator`. It then calls `orchestrator.run(&mut runtime, &req, ..)`.
9. **`ToolOrchestrator::run`**, then **`ToolOrchestrator::run_attempt`** (`codex-core`, `core/src/tools/orchestrator.rs`). **These are generic over `T: ToolRuntime<Rq, Out>`, statically dispatched.**
   - `run` handles approval.
   - `run` picks the sandbox with `SandboxManager::select_initial` (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, `get_platform_sandbox` returns `SandboxType::LinuxSeccomp` when the policy needs a sandbox, and `SandboxType::None` otherwise.
   - `run` builds a `SandboxAttempt`, which carries `codex_linux_sandbox_exe` from the `TurnContext`.
   - `run_attempt` calls `tool.run(req, &attempt, &ctx)`. A retry without the sandbox after a denial goes through `run_attempt` again.
10. **`UnifiedExecRuntime::run`** (`impl ToolRuntime<UnifiedExecRequest, UnifiedExecProcess> for UnifiedExecRuntime`; `codex-core`, `core/src/tools/runtimes/unified_exec.rs`). For a local environment it first calls `maybe_wrap_shell_lc_with_snapshot`. That is a shell-snapshot wrapper, not the sandbox. It then calls `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`) to make a `SandboxCommand`, and then `attempt.env_for(command, options, managed_network)`.
11. **`SandboxAttempt::env_for`** (`codex-core`, `core/src/tools/sandboxing.rs`), then **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). **This is where the command is wrapped for the sandbox.**
    - For `SandboxType::LinuxSeccomp`, `transform` calls `ensure_linux_bubblewrap_is_supported` and `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`).
    - It builds the argv `[codex_linux_sandbox_exe, <sandbox args>…, -- original command]` and sets an `arg0` override with `linux_sandbox_arg0_override`.
    - For `SandboxType::None`, it leaves the argv unchanged.

    `env_for` then turns the result into a core `ExecRequest` with `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing`).
12. Back in `UnifiedExecRuntime::run`, it calls **`UnifiedExecProcessManager::open_session_with_exec_env`** (`codex-core`, `core/src/unified_exec/process_manager.rs`) with `Box::new(NoopSpawnLifecycle)`. `SpawnLifecycle` is a trait object; the no-op version has no inherited fds. `environment.is_remote()` is false, so it skips the exec-server backend and the Windows branch, splits `request.command` into the program and its arguments, and spawns locally:
    - With `tty == true`: **`codex_utils_pty::pty::spawn_process_with_inherited_fds`**, then **`spawn_process_portable`** (`codex-utils-pty`, `utils/pty/src/pty.rs`). The process is spawned by `pair.slave.spawn_command(command_builder)` from the `portable-pty` crate. With a non-empty fd list it would use `spawn_process_preserving_fds` instead, which calls `std::process::Command::spawn`.
    - With `tty == false`: **`codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`**, then **`spawn_process_with_stdin_mode`** (`codex-utils-pty`, `utils/pty/src/pipe.rs`). The process is spawned by `tokio::process::Command::spawn()`.

    When the sandbox applies, the program spawned here is the Linux sandbox helper, `codex_linux_sandbox_exe`, and it then runs the model's command. When no sandbox is selected, the command is spawned directly. `UnifiedExecProcess::from_spawned` then wraps the child.

Two variants are not on the main path:
- With `UnifiedExecShellMode::ZshFork` on Unix, `UnifiedExecRuntime::run` first tries `zsh_fork_backend::maybe_prepare_unified_exec`, which uses the same `env_for` wrapping. If that succeeds, it calls `open_session_with_exec_env` with the prepared spawn lifecycle.
- For a remote environment, `open_session_with_exec_env` instead calls `environment.get_exec_backend().start(..)`, the exec-server backend.
