# Answers

## Q1

Dependencies come from each workspace member's `Cargo.toml`: `[dependencies]` plus any
`[target.*.dependencies]`, with dev- and build-dependencies excluded. `codex-tools` is the package
at `codex-rs/tools` (`codex-rs/Cargo.toml` has `codex-tools = { path = "tools" }`).

**Direct dependents (7).** Each lists `codex-tools = { workspace = true }` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `app-server/Cargo.toml` |
| `codex-core` | `core/Cargo.toml` |
| `codex-extension-api` | `ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `ext/web-search/Cargo.toml` |

**Indirect dependents (18).** These crates do not list `codex-tools` but reach it through the
crates shown:

| Crate | Reaches `codex-tools` through (direct deps on the path) |
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
| `codex-guardian` (`ext/guardian`) | `codex-core`, `codex-extension-api` |
| `codex-lmstudio` | `codex-core` |
| `codex-mcp-server` | `codex-core`, `codex-extension-api` |
| `codex-memories-write` (`memories/write`) | `codex-core` |
| `codex-ollama` | `codex-core` |
| `codex-thread-manager-sample` | `codex-core-api` |
| `codex-tui` | `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` |
| `codex-utils-oss` (`utils/oss`) | `codex-core`, `codex-lmstudio`, `codex-ollama` |
| `codex-utils-sandbox-summary` (`utils/sandbox-summary`) | `codex-core` |

No other workspace member reaches `codex-tools` through normal dependencies.

## Q2

The trait method is `ThreadStore::read_thread(&self, params: ReadThreadParams)`, declared in
`thread-store/src/store.rs` (`codex-thread-store`). It has two implementations,
`LocalThreadStore` (`thread-store/src/local/mod.rs`) and `InMemoryThreadStore`
(`thread-store/src/in_memory.rs`). In this checkout, every non-test call goes through
`Arc<dyn ThreadStore>`. No non-test call goes through a generic or a concrete implementing type.

| Calling function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store` (`SessionServices.thread_store: Arc<dyn ThreadStore>`, `core/src/state/service.rs`) |
| `thread_title_from_thread_store` (`None` branch) | `codex-core` | `core/src/session/mod.rs` | `thread_store: &Arc<dyn ThreadStore>` parameter |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` (two calls) | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` (two calls) | `codex-app-server` | same | same |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same | same |

The following calls are excluded:

- **Different methods with the same name.** These call `LiveThread::read_thread(include_archived, include_history)` or `CodexThread::read_thread(include_archived, include_history)`, not the trait method:
  - `CodexThread::read_thread` (`core/src/codex_thread.rs`), which calls `LiveThread::read_thread`.
  - `thread_title_from_thread_store`'s `Some(live_thread)` branch.
  - `ThreadManager::spawn_subagent` (`core/src/thread_manager.rs`).
  - `apply_bespoke_event_handling` (`app-server/src/bespoke_event_handling.rs`).
  - `start_detached_review` (`app-server/src/request_processors/turn_processor.rs`).
- **A free function with the same name.** `<LocalThreadStore as ThreadStore>::read_thread`, `update_thread_metadata.rs` and `live_writer.rs` call `read_thread::read_thread`, the `pub(super)` free function in `thread-store/src/local/read_thread.rs`.
- **Test code.** The remaining `.read_thread(ReadThreadParams{..})` calls sit in `#[cfg(test)] mod tests` in `thread-store/src/local/{mod.rs, read_thread.rs, update_thread_metadata.rs}` and in `core/src/session/tests.rs`.

## Q3

**Direct callers of `codex_git_utils::get_git_repo_root`** (defined in `git-utils/src/info.rs`).
These are the intermediates. All are non-test, and calls to `get_git_repo_root_with_fs` are left out.

| Direct caller | Crate | File |
|---|---|---|
| `git_diff_to_remote` | `codex-git-utils` | `git-utils/src/info.rs` |
| `AnalyticsReducer::ingest_skill_invoked` | `codex-analytics` | `analytics/src/reducer.rs` |
| `Session::new` | `codex-core` | `core/src/session/session.rs` |
| `run_turn` | `codex-core` | `core/src/session/turn.rs` (in the `None =>` branch; the other branch uses `get_git_repo_root_with_fs`) |
| `build_turn_metadata_header` | `codex-core` | `core/src/turn_metadata.rs` |
| `TurnMetadataState::new` | `codex-core` | `core/src/turn_metadata.rs` |
| `environment_id_from_cwd` | `codex-secrets` | `secrets/src/lib.rs` |
| `ChatWidget::status_line_project_root_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` |
| `display_path_for` | `codex-tui` | `tui/src/diff_render.rs` |
| `discover_marketplace_paths_from_roots` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` |
| `ThreadMetadataSync::for_create` | `codex-thread-store` | `thread-store/src/thread_metadata_sync.rs` |
| `write_session_meta` | `codex-rollout` | `rollout/src/recorder.rs` |
| `git_check` | `codex-cli` | `cli/src/doctor/git.rs` |
| `terminal_title_project_root` | `codex-cli` | `cli/src/doctor/title.rs` |
| `run_exec_session` | `codex-exec` | `exec/src/lib.rs` |

**Functions that call `get_git_repo_root` through exactly one intermediate.** None of them calls
`get_git_repo_root` directly, and none is test code.

| Function | Crate | File | Intermediate it calls |
|---|---|---|---|
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `git_diff_to_remote` |
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `AnalyticsReducer::ingest_skill_invoked` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `run_turn` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `ExternalAgentConfigMigrationScreen::display_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `reformat_description` (a nested `fn` inside `display_description`) | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |

Notes:

- `environment_id_from_cwd` (`codex-secrets`) is only called from the test `environment_id_fallback_has_cwd_prefix`, so it contributes no caller.
- `git_diff_to_remote` and `build_turn_metadata_header` are also called from `core/src/git_info_tests.rs` and `core/src/turn_metadata_tests.rs`. Those are tests and are excluded.
- The only non-test call to `Session::new` is in `Codex::spawn_internal`; the rest are in `core/src/session/tests.rs`.
- `ExternalAgentConfigMigrationScreen::display_description` qualifies because it calls `display_path_for` itself, in the "Migrate enabled plugins from" branch.
- I counted the nested `reformat_description` as a separate function. If nested functions are folded into their parent, only `display_description` remains.

## Q4

This traces a local session on Linux, with the default `unified_exec` shell tool type, so that
`exec_command` is registered as `ExecCommandHandler` in `core/src/tools/spec_plan.rs`. It follows
the default (non-zsh-fork) shell mode with the Linux sandbox selected. Everything is in
`codex-core` unless another crate is named.

1. **`try_run_sampling_request`** (`codex-core`, `core/src/session/turn.rs`). It polls the model stream and, on `ResponseEvent::OutputItemDone(item)`, builds a `HandleOutputCtx` holding the `ToolCallRuntime`. It then calls `handle_output_item_done`. The returned `tool_future` is pushed into the `in_flight: FuturesOrdered`, which is drained later, after the stream ends, by `drain_in_flight`.
2. **`handle_output_item_done`** (`core/src/stream_events_utils.rs`). It calls `ToolRouter::build_tool_call(item)` (`core/src/tools/router.rs`), which turns `ResponseItem::FunctionCall` into a `ToolCall` with `ToolPayload::Function { arguments }`. It records the item with `record_completed_response_item`, then builds the future with `ToolCallRuntime::handle_tool_call`.
3. **`ToolCallRuntime::handle_tool_call`** → **`ToolCallRuntime::handle_tool_call_with_source`** (`core/src/tools/parallel.rs`). **Spawned task:** `handle_tool_call_with_source` is a plain `fn`, and it calls `tokio::spawn(async move { … })` (wrapped in `AbortOnDropHandle`) as soon as it is called. It takes the parallel-execution `RwLock` (read for parallel tools; `ExecCommandHandler` reports `supports_parallel_tool_calls() == true`). Inside the task it calls the router. The future pushed into `in_flight` only awaits that task's handle, or the cancellation token.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`** → **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`core/src/tools/router.rs`). This builds a `ToolInvocation` and calls the registry.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`core/src/tools/registry.rs`). It looks up the tool as an `Arc<dyn CoreToolRuntime>`, checks `matches_kind`, runs pre-tool-use hooks, and calls **`SessionTelemetry::log_tool_result_with_tags`** (`codex-otel`, `otel/src/events/session_telemetry.rs`). That function is generic over a closure `F: FnOnce() -> Fut`, and the closure calls the next function.
6. **`handle_any_tool`** (free function, `core/src/tools/registry.rs`). It calls `tool.handle(invocation)` on `&dyn CoreToolRuntime`. **Trait object:** this is the `ToolExecutor<ToolInvocation>::handle` method of the `codex-tools` trait (`tools/src/tool_executor.rs`, an `#[async_trait]` trait), dispatched dynamically to `ExecCommandHandler`.
7. **`<ExecCommandHandler as ToolExecutor<ToolInvocation>>::handle`** (`core/src/tools/handlers/unified_exec/exec_command.rs`). It:
   - parses `ExecCommandArgs` and resolves the turn environment and cwd (`resolve_tool_environment`);
   - allocates a process id (`UnifiedExecProcessManager::allocate_process_id`);
   - builds the shell argv with `get_command` (`core/src/tools/handlers/unified_exec.rs`), for example `[shell, "-lc", cmd]`;
   - applies permission checks and the `intercept_apply_patch` short-circuit;
   - calls `UnifiedExecProcessManager::exec_command` on `session.services.unified_exec_manager`.
8. **`UnifiedExecProcessManager::exec_command`** → **`UnifiedExecProcessManager::open_session_with_sandbox`** (`core/src/unified_exec/process_manager.rs`). It builds the env (`create_env`, `apply_unified_exec_env`) and the exec-policy approval requirement (`create_exec_approval_requirement_for_command`). It creates a `ToolOrchestrator` and a `UnifiedExecRuntime` (`core/src/tools/runtimes/unified_exec.rs`), then calls `orchestrator.run(&mut runtime, &req, …)`.
9. **`ToolOrchestrator::run`** (`core/src/tools/orchestrator.rs`). **Generic:** it is `run<Rq, Out, T: ToolRuntime<Rq, Out>>`, monomorphised with `T = UnifiedExecRuntime`. It does approval (`exec_approval_requirement`, possibly `request_approval`), then picks the sandbox with **`SandboxManager::select_initial`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). On Linux, `get_platform_sandbox` yields `SandboxType::LinuxSeccomp` when a sandbox is required. It builds a `SandboxAttempt` that carries `codex_linux_sandbox_exe` and calls **`ToolOrchestrator::run_attempt`**. On a sandbox denial, `run` may retry `run_attempt` with `SandboxType::None` after approval.
10. **`ToolOrchestrator::run_attempt`**. It sets up network approval and calls `tool.run(req, &attempt, &ctx)`. This is the generic `ToolRuntime::run` (trait in `core/src/tools/sandboxing.rs`), dispatched statically.
11. **`<UnifiedExecRuntime as ToolRuntime<UnifiedExecRequest, UnifiedExecProcess>>::run`** (`core/src/tools/runtimes/unified_exec.rs`). It:
    - computes the env (`exec_env_for_sandbox_permissions`, managed-network env);
    - for a local environment, wraps the command with the shell snapshot via `maybe_wrap_shell_lc_with_snapshot` (`core/src/tools/runtimes/mod.rs`). This is a shell wrapper, not the sandbox wrapper;
    - builds a `SandboxCommand` with `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`);
    - calls `attempt.env_for(command, options, managed_network)`.

    (With `UnifiedExecShellMode::ZshFork`, the same `env_for` is followed by `zsh_fork_backend::maybe_prepare_unified_exec` before spawning.)
12. **`SandboxAttempt::env_for`** (`core/src/tools/sandboxing.rs`) → **`SandboxManager::transform`** (`codex-sandboxing`, `sandboxing/src/manager.rs`). **This is where the command is wrapped for the sandbox.** For `SandboxType::LinuxSeccomp`, `transform`:
    - calls `ensure_linux_bubblewrap_is_supported`;
    - builds args with `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`): `--sandbox-policy-cwd`, `--command-cwd`, `--permission-profile <json>`, optionally `--use-legacy-landlock` or `--allow-network-for-proxy`, then the original argv;
    - prepends the `codex-linux-sandbox` executable path;
    - sets `arg0` to `codex-linux-sandbox` (`linux_sandbox_arg0_override`).

    With `SandboxType::None`, the argv is passed through unchanged. The result is converted with `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing/mod.rs`).
13. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`core/src/unified_exec/process_manager.rs`), called with `Box::new(NoopSpawnLifecycle)`. The Windows branch is compiled out on Linux. For a local environment, `environment.is_remote()` is false, so it skips the exec-server backend's `start` and spawns locally through `codex-utils-pty`:
    - `tty == false` (the default, `default_tty()` in `core/src/tools/handlers/unified_exec.rs`): `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds` → `spawn_process_with_stdin_mode` (`codex-utils-pty`, `utils/pty/src/pipe.rs`). This is **where the process is spawned**: `tokio::process::Command::new(program)` with `arg0`, `pre_exec` (detach from the TTY, parent-death signal, close inherited fds), then `command.spawn()`.
    - `tty == true`: `codex_utils_pty::pty::spawn_process_with_inherited_fds` → `spawn_process_portable` (`utils/pty/src/pty.rs`), because `NoopSpawnLifecycle::inherited_fds()` is empty. The process is spawned by `pair.slave.spawn_command(command_builder)` on a portable-pty `Box<dyn PtySystem>` / slave (another trait object). If inherited fds are present, `spawn_process_preserving_fds` is used instead, with `std::process::Command::spawn`.

    The spawned child is then wrapped by `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`).
14. When the Linux sandbox was selected, the process spawned is the `codex-linux-sandbox` helper. Its entry point is `run_main` (`codex-linux-sandbox`, `linux-sandbox/src/linux_run_main.rs`). Per its doc comment, it wraps the command in bubblewrap when needed, applies no_new_privs and seccomp, and `execvp`s the real command (`exec_or_panic`). I did not trace the helper's internals or how its binary gets dispatched (through `codex-arg0`) in detail.

In summary, control passes:

- through a **spawned task** at step 3, the `tokio::spawn` in `ToolCallRuntime::handle_tool_call_with_source`;
- through **trait objects** at steps 5–6, `Arc<dyn CoreToolRuntime>` and `ToolExecutor::handle`, and in the portable-pty spawn on the TTY path;
- through **generics** at step 5, the closure passed to `SessionTelemetry::log_tool_result_with_tags`, and at steps 9–10, `ToolOrchestrator::run` / `run_attempt` over `T: ToolRuntime`;
- through **sandbox wrapping** at step 12, in `SandboxManager::transform` (`codex-sandboxing`), reached from `SandboxAttempt::env_for`.
