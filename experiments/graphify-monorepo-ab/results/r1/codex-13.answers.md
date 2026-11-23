# Answers

## Q1

Only normal dependencies count here: `[dependencies]` and `[target.*.dependencies]`. Dev- and build-dependencies are left out. Path dependencies that are not in `members` (`codex-chatgpt`, `codex-message-history`, `codex-windows-sandbox` and the `tests/common` support crates) still count as workspace members, because Cargo treats path dependencies inside the workspace as members (`codex-rs/Cargo.toml`, `[workspace.dependencies]`).

**Direct dependents.** Each of these declares `codex-tools = { workspace = true }` under `[dependencies]`:

| Crate | Manifest |
|---|---|
| `codex-app-server` | `codex-rs/app-server/Cargo.toml` |
| `codex-core` | `codex-rs/core/Cargo.toml` |
| `codex-extension-api` | `codex-rs/ext/extension-api/Cargo.toml` |
| `codex-goal-extension` | `codex-rs/ext/goal/Cargo.toml` |
| `codex-image-generation-extension` | `codex-rs/ext/image-generation/Cargo.toml` |
| `codex-memories-extension` | `codex-rs/ext/memories/Cargo.toml` |
| `codex-web-search-extension` | `codex-rs/ext/web-search/Cargo.toml` |

**Indirect dependents.** These depend on `codex-tools` only through other crates. The middle column lists the dependencies that carry `codex-tools` in.

| Crate | Through | Manifest |
|---|---|---|
| `codex-app-server-client` | `codex-app-server`, `codex-core` | `app-server-client/Cargo.toml` |
| `codex-app-server-daemon` | `codex-app-server-transport` | `app-server-daemon/Cargo.toml` |
| `codex-app-server-test-client` | `codex-core` | `app-server-test-client/Cargo.toml` |
| `codex-app-server-transport` | `codex-core` | `app-server-transport/Cargo.toml` |
| `codex-chatgpt` | `codex-core` | `chatgpt/Cargo.toml` |
| `codex-cli` | `codex-core`, `codex-app-server`, `codex-tui`, `codex-exec`, `codex-mcp-server`, … | `cli/Cargo.toml` |
| `codex-cloud-config` | `codex-core` | `cloud-config/Cargo.toml` |
| `codex-cloud-tasks` | `codex-core`, `codex-tui` | `cloud-tasks/Cargo.toml` |
| `codex-core-api` | `codex-core`, `codex-extension-api` | `core-api/Cargo.toml` |
| `codex-exec` | `codex-core`, `codex-app-server-client`, … | `exec/Cargo.toml` |
| `codex-guardian` | `codex-core`, `codex-extension-api` | `ext/guardian/Cargo.toml` |
| `codex-lmstudio` | `codex-core` | `lmstudio/Cargo.toml` |
| `codex-mcp-server` | `codex-core`, `codex-extension-api` | `mcp-server/Cargo.toml` |
| `codex-memories-write` | `codex-core` | `memories/write/Cargo.toml` |
| `codex-ollama` | `codex-core` | `ollama/Cargo.toml` |
| `codex-thread-manager-sample` | `codex-core-api` | `thread-manager-sample/Cargo.toml` |
| `codex-tui` | `codex-app-server-client`, `codex-cloud-config`, `codex-utils-oss`, `codex-utils-sandbox-summary` | `tui/Cargo.toml` |
| `codex-utils-oss` | `codex-core`, `codex-lmstudio`, `codex-ollama` | `utils/oss/Cargo.toml` |
| `codex-utils-sandbox-summary` | `codex-core` | `utils/sandbox-summary/Cargo.toml` |

The test-support crates are also workspace members, and they depend on `codex-tools` indirectly through normal dependencies:

| Crate | Through | Manifest |
|---|---|---|
| `core_test_support` | `codex-core`, `codex-extension-api` | `core/tests/common/Cargo.toml` |
| `app_test_support` | `codex-core`, `core_test_support` | `app-server/tests/common/Cargo.toml` |
| `mcp_test_support` | `codex-mcp-server`, `core_test_support` | `mcp-server/tests/common/Cargo.toml` |

Other crates reach these three only as dev-dependencies, but they are packages in their own right, so they are listed. Leave them out if you count only crates outside test support.

All paths in the tables are relative to `codex-rs/`.

## Q2

The trait method is declared as `ThreadStore::read_thread` in `codex-rs/thread-store/src/store.rs`. Every non-test call to it goes through an `Arc<dyn ThreadStore>`. There are no calls through a generic, through an implementing type, or by fully qualified path outside tests.

| Function | Crate | File | Receiver |
|---|---|---|---|
| `LiveThread::read_thread` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `run_turn_stop_hooks` | `codex-core` | `core/src/hook_runtime.rs` | `sess.services.thread_store: Arc<dyn ThreadStore>` (`core/src/state/service.rs`) |
| `thread_title_from_thread_store` | `codex-core` | `core/src/session/mod.rs` | its `thread_store: &Arc<dyn ThreadStore>` argument, in the `None` branch |
| `ThreadManagerState::read_stored_thread` | `codex-core` | `core/src/thread_manager.rs` | `self.thread_store: Arc<dyn ThreadStore>` |
| `ThreadRequestProcessor::thread_archive_response` | `codex-app-server` | `app-server/src/request_processors/thread_processor.rs` | `self.thread_store: Arc<dyn ThreadStore>` (two calls) |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | `codex-app-server` | same file | `self.thread_store` |
| `ThreadRequestProcessor::load_thread_turns_list_history` | `codex-app-server` | same file | `self.thread_store` |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | `codex-app-server` | same file | `self.thread_store` |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | `codex-app-server` | same file | `self.thread_store` |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` | `codex-app-server` | same file | `self.thread_store` (two calls) |
| `ThreadRequestProcessor::attach_thread_name` | `codex-app-server` | same file | `self.thread_store` |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | `codex-app-server` | same file | `self.thread_store` |

Paths in the table are relative to `codex-rs/`.

Some calls look similar but are excluded, because they reach other functions named `read_thread`:

- **`CodexThread::read_thread`** (`core/src/codex_thread.rs`). It calls `LiveThread::read_thread`, and is itself called by:
  - `spawn_subagent` in `core/src/thread_manager.rs`;
  - `start_detached_review` in `app-server/.../turn_processor.rs`;
  - `apply_bespoke_event_handling` in `app-server/src/bespoke_event_handling.rs`.
- **The `Some(live_thread)` branch of `thread_title_from_thread_store`.** It calls `LiveThread::read_thread`.
- **`LocalThreadStore::read_thread`** (`thread-store/src/local/mod.rs`). This is the trait impl, and it calls the free function `local::read_thread::read_thread`.
- **Other callers of that free function.** These are in `local/mod.rs`, `local/live_writer.rs` and `local/update_thread_metadata.rs`.

All the other `.read_thread(ReadThreadParams{..})` calls in `thread-store/src/local/*.rs` are inside `#[cfg(test)] mod tests` blocks. The ones in `app-server/tests/suite/v2/thread_resume.rs` are under a `tests/` directory. Both are left out.

## Q3

`get_git_repo_root` is defined in `codex-rs/git-utils/src/info.rs` (crate `codex-git-utils`). These non-test functions call it directly, and so act as the intermediates:

- `git_diff_to_remote` (`codex-git-utils`)
- `terminal_title_project_root` and `git_check` (`codex-cli`)
- `AnalyticsReducer::ingest_skill_invoked` (`codex-analytics`)
- `write_session_meta` (`codex-rollout`)
- `environment_id_from_cwd` (`codex-secrets`)
- `ThreadMetadataSync::for_create` (`codex-thread-store`)
- `display_path_for` and `ChatWidget::status_line_project_root_for_cwd` (`codex-tui`)
- `discover_marketplace_paths_from_roots` (`codex-core-plugins`)
- `run_exec_session` (`codex-exec`)
- In `codex-core`:
  - `run_turn` (`session/turn.rs`)
  - `Session::new` (`session/session.rs`)
  - `build_turn_metadata_header` and `TurnMetadataState::new` (`turn_metadata.rs`)

These are the functions that call one of those intermediates, don't call `get_git_repo_root` directly, and aren't tests:

| Function | Crate | File | Calls (intermediate) |
|---|---|---|---|
| `GitRequestProcessor::git_diff_to_origin` | `codex-app-server` | `app-server/src/request_processors/git_processor.rs` | `codex_git_utils::git_diff_to_remote` |
| `terminal_title_check` | `codex-cli` | `cli/src/doctor/title.rs` | `terminal_title_project_root` |
| `build_report` | `codex-cli` | `cli/src/doctor.rs` | `git_check` (both `tokio::join!` branches) |
| `AnalyticsReducer::ingest` | `codex-analytics` | `analytics/src/reducer.rs` | `ingest_skill_invoked` |
| `RolloutWriterState::write_session_meta_if_needed` | `codex-rollout` | `rollout/src/recorder.rs` | `write_session_meta` |
| `LiveThread::create` | `codex-thread-store` | `thread-store/src/live_thread.rs` | `ThreadMetadataSync::for_create` |
| `<Box<dyn Renderable> as From<DiffSummary>>::from` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `render_changes_block` | `codex-tui` | `tui/src/diff_render.rs` | `display_path_for` |
| `new_view_image_tool_call` | `codex-tui` | `tui/src/history_cell/patches.rs` | `display_path_for` |
| `Notification::display` | `codex-tui` | `tui/src/chatwidget/notifications.rs` | `display_path_for` |
| `reformat_description` | `codex-tui` | `tui/src/external_agent_config_migration.rs` | `display_path_for` |
| `ChatWidget::status_line_project_root_name_for_cwd` | `codex-tui` | `tui/src/chatwidget/status_surfaces.rs` | `ChatWidget::status_line_project_root_for_cwd` |
| `list_marketplaces_with_home` | `codex-core-plugins` | `core-plugins/src/marketplace.rs` | `discover_marketplace_paths_from_roots` |
| `run_main` | `codex-exec` | `exec/src/lib.rs` | `run_exec_session` |
| `RegularTask::run` (impl `SessionTask`) | `codex-core` | `core/src/tasks/regular.rs` | `session::turn::run_turn` |
| `Codex::spawn_internal` | `codex-core` | `core/src/session/mod.rs` | `Session::new` |
| `Session::make_turn_context` | `codex-core` | `core/src/session/turn_context.rs` | `TurnMetadataState::new` |
| `spawn_review_thread` | `codex-core` | `core/src/session/review.rs` | `TurnMetadataState::new` |
| `MemoryStartupContext::stage_one_request_context` | `codex-memories-write` | `memories/write/src/runtime.rs` | `codex_core::build_turn_metadata_header` |

Paths in the table are relative to `codex-rs/`.

Notes on the table:

- **`reformat_description`** is a nested `fn` declared inside `ExternalAgentConfigMigrationScreen::display_description`. So `display_description` calls `get_git_repo_root` through two intermediates, not one.
- **`environment_id_from_cwd`** (`codex-secrets`) has only a test caller, so it contributes nothing.
- **`GitRequestProcessor::git_diff_to_remote`** (app-server) is a different function that happens to share the name `git_diff_to_remote`. It calls `git_diff_to_origin`, so it is three hops away.
- **`thread-manager-sample`** has its own `run_turn`, which is unrelated to the `codex-core` one.

## Q4

Linux, local session, `exec_command` tool. All functions are in `codex-core` unless another crate is named.

1. **`try_run_sampling_request`** (`core/src/session/turn.rs`). It receives `ResponseEvent::OutputItemDone(item)` from the model stream and calls `handle_output_item_done`.
2. **`handle_output_item_done`** (`core/src/stream_events_utils.rs`). It calls `ToolRouter::build_tool_call` (`core/src/tools/router.rs`), which turns `ResponseItem::FunctionCall` into a `ToolCall` with `ToolPayload::Function`. It records the item, then calls `ToolCallRuntime::handle_tool_call`. The future that call returns is boxed, handed back as `tool_future`, and pushed onto `in_flight` in `try_run_sampling_request`.
3. **`ToolCallRuntime::handle_tool_call`** then **`ToolCallRuntime::handle_tool_call_with_source`** (`core/src/tools/parallel.rs`). Both are plain (non-async) functions that return futures.
   - **Spawned task:** `handle_tool_call_with_source` calls `tokio::spawn` straight away, wrapped in an `AbortOnDropHandle`.
   - Inside the task, it takes the parallel-execution `RwLock` (read or write) and calls `ToolRouter::dispatch_tool_call_with_terminal_outcome`.
4. **`ToolRouter::dispatch_tool_call_with_terminal_outcome`** then **`ToolRouter::dispatch_tool_call_with_code_mode_result_inner`** (`core/src/tools/router.rs`). These build the `ToolInvocation` and call `ToolRegistry::dispatch_any_with_terminal_outcome`.
5. **`ToolRegistry::dispatch_any_with_terminal_outcome`** (`core/src/tools/registry.rs`). It looks the tool up as an `Arc<dyn CoreToolRuntime>` and runs the pre-tool-use hooks.
   - It calls `SessionTelemetry::log_tool_result_with_tags` in crate `codex-otel` (`otel/src/events/session_telemetry.rs`). That is a generic over the closure `F: FnOnce() -> Fut`, and it awaits `f()`.
   - The closure calls `handle_any_tool`.
6. **`handle_any_tool`** (`core/src/tools/registry.rs`).
   - **Trait object:** it calls `tool.handle(invocation)` on a `&dyn CoreToolRuntime`. `handle` is the `ToolExecutor<ToolInvocation>` trait method, a trait defined in crate `codex-tools` (`tools/src/tool_executor.rs`) and implemented through `async_trait`.
   - If the tool's exposure was overridden, the call first goes through `ExposureOverride::handle` (`registry.rs`), which forwards to the inner `Arc<dyn CoreToolRuntime>`.
7. **`ExecCommandHandler::handle`** (impl `ToolExecutor<ToolInvocation>`, `core/src/tools/handlers/unified_exec/exec_command.rs`).
   - It parses the arguments and builds the shell argv with `get_command` (`core/src/tools/handlers/unified_exec.rs`).
   - It applies permissions and checks `intercept_apply_patch`, then calls `UnifiedExecProcessManager::exec_command`.
8. **`UnifiedExecProcessManager::exec_command`** then **`UnifiedExecProcessManager::open_session_with_sandbox`** (`core/src/unified_exec/process_manager.rs`).
   - `open_session_with_sandbox` builds the environment, the exec approval requirement, the `UnifiedExecRuntime` and the `UnifiedExecRequest`.
   - It then calls `ToolOrchestrator::run`.
9. **`ToolOrchestrator::run`** then **`ToolOrchestrator::run_attempt`** (`core/src/tools/orchestrator.rs`).
   - **Generic:** both are generic over `T: ToolRuntime<Rq, Out>`, here `UnifiedExecRuntime`.
   - `run` handles approval, then picks the sandbox with `SandboxManager::select_initial` in crate `codex-sandboxing` (`sandboxing/src/manager.rs`). On Linux, when a platform sandbox is required, that is `SandboxType::LinuxSeccomp`.
   - `run_attempt` calls `tool.run(req, attempt, ctx)`. A sandbox-denied result can lead to a second `run_attempt` without the sandbox, after approval.
10. **`UnifiedExecRuntime::run`** (impl `ToolRuntime<UnifiedExecRequest, UnifiedExecProcess>`, `core/src/tools/runtimes/unified_exec.rs`).
    - It wraps the command with `maybe_wrap_shell_lc_with_snapshot`, which adds the shell snapshot, not the sandbox.
    - It builds a `SandboxCommand` with `build_sandbox_command` (`core/src/tools/runtimes/mod.rs`), then calls `SandboxAttempt::env_for`.
    - In `ZshFork` shell mode it may instead go through `zsh_fork_backend::maybe_prepare_unified_exec`. The default path is `Direct`.
11. **`SandboxAttempt::env_for`** (`core/src/tools/sandboxing.rs`) then **`SandboxManager::transform`** (crate `codex-sandboxing`, `sandboxing/src/manager.rs`).
    - **Sandbox wrapping happens here.** For `SandboxType::LinuxSeccomp`, `transform` prefixes the argv with the `codex-linux-sandbox` executable and the arguments from `create_linux_sandbox_command_args_for_permission_profile` (`sandboxing/src/landlock.rs`).
    - It also sets an `arg0` override with `linux_sandbox_arg0_override`.
    - `env_for` then converts the result with `ExecRequest::from_sandbox_exec_request` (`core/src/sandboxing/mod.rs`).
12. **`UnifiedExecProcessManager::open_session_with_exec_env`** (`core/src/unified_exec/process_manager.rs`). Back in `UnifiedExecRuntime::run`, this is called with the wrapped `ExecRequest`.
    - The environment is not remote, and the Windows branch is compiled only on Windows.
    - With `tty == false`, the default (`default_tty` in `handlers/unified_exec.rs`), it calls `codex_utils_pty::pipe::spawn_process_no_stdin_with_inherited_fds`.
    - With `tty == true`, it calls `codex_utils_pty::pty::spawn_process_with_inherited_fds`.
13. **Spawn**, in crate `codex-utils-pty`.
    - **Pipe path:** `spawn_process_no_stdin_with_inherited_fds` calls `spawn_process_with_stdin_mode` (`utils/pty/src/pipe.rs`). That function builds a `tokio::process::Command` and spawns the process with `command.spawn()`.
    - **PTY path:** `spawn_process_with_inherited_fds` (`utils/pty/src/pty.rs`) calls either:
      - `spawn_process_portable`, which uses `pair.slave.spawn_command(...)`; or
      - `spawn_process_preserving_fds` (when inherited fds are non-empty), which uses `command.spawn()`.
    - The spawned program is the Linux sandbox helper, which then runs the user's command. If no sandbox was selected, the command runs directly.
    - Afterwards, `UnifiedExecProcess::from_spawned` (`core/src/unified_exec/process.rs`) wraps the child and starts its output task.

I have not traced what the `codex-linux-sandbox` helper does after it is spawned (bubblewrap or landlock setup and its own exec).
