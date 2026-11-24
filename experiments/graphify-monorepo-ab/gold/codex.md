# Gold: Codex at 859dbe27

Each question scores from 0 to 1, and a run's **score** is the mean of its four question scores.
Q1 to Q3 are sets, scored mechanically; Q4 is key facts. An item an answer lists with a hedge ("probably",
"not sure") still counts as listed. A set item counts when its name identifies the gold item: a crate by
its Cargo package name or its directory under `codex-rs/`; a function by its name plus its crate or file
(a method's type is not required, except where two gold or listed functions share a name in one file).

**F1** over a set: precision P = listed gold items / items listed, recall R = listed gold items / gold
items, F1 = 2PR / (P + R), and 0 when nothing correct is listed.

Gold was computed from `cargo metadata --no-deps --format-version 1` in `codex-rs/` (dependencies
of kind null, between workspace packages), `rg` over the export, and reading the code at every call
site. A function is a test function if it has a test attribute or sits in a `#[cfg(test)]` module, a
`tests/` directory or a `*_tests.rs` file (the question's definition).

## Q1. Dependents of `codex-tools` (score: mean of the two F1s)

A crate listed in the wrong set is a false positive in the set it is listed in and missed in the other.
A crate an answer lists only as "a dependent", in neither set, counts in neither.

- **Direct (7):** codex-app-server, codex-core, codex-extension-api, codex-goal-extension,
  codex-image-generation-extension, codex-memories-extension, codex-web-search-extension.
- **Indirect (22):** app_test_support, codex-app-server-client, codex-app-server-daemon,
  codex-app-server-test-client, codex-app-server-transport, codex-chatgpt, codex-cli,
  codex-cloud-config, codex-cloud-tasks, codex-core-api, codex-exec, codex-guardian, codex-lmstudio,
  codex-mcp-server, codex-memories-write, codex-ollama, codex-thread-manager-sample, codex-tui,
  codex-utils-oss, codex-utils-sandbox-summary, core_test_support, mcp_test_support.

## Q2. Callers of the trait method `ThreadStore::read_thread` (score: F1 over 12 functions)

The trait is in thread-store/src/store.rs. Each gold function calls `read_thread` on a value of type
`Arc<dyn ThreadStore>` or `&dyn ThreadStore` (fields at core/src/state/service.rs,
app-server/src/request_processors/thread_processor.rs, thread-store/src/live_thread.rs,
core/src/thread_manager.rs, and a parameter in core/src/session/mod.rs).

| Function | Crate | File (under codex-rs/) |
|---|---|---|
| `ThreadRequestProcessor::thread_archive_response` | codex-app-server | app-server/src/request_processors/thread_processor.rs |
| `ThreadRequestProcessor::load_persisted_thread_for_read` | codex-app-server | same |
| `ThreadRequestProcessor::load_thread_turns_list_history` | codex-app-server | same |
| `ThreadRequestProcessor::read_stored_thread_for_resume` | codex-app-server | same |
| `ThreadRequestProcessor::read_stored_thread_for_new_fork` | codex-app-server | same |
| `ThreadRequestProcessor::load_thread_from_resume_source_or_send_internal` | codex-app-server | same |
| `ThreadRequestProcessor::attach_thread_name` | codex-app-server | same |
| `ThreadRequestProcessor::get_thread_summary_response_inner` | codex-app-server | same |
| `LiveThread::read_thread` | codex-thread-store | thread-store/src/live_thread.rs |
| `run_turn_stop_hooks` | codex-core | core/src/hook_runtime.rs |
| `thread_title_from_thread_store` | codex-core | core/src/session/mod.rs |
| `ThreadManagerState::read_stored_thread` | codex-core | core/src/thread_manager.rs |

Not gold, and false positives if listed: functions that call the inherent `LiveThread::read_thread` or
`CodexThread::read_thread` (for example `TurnRequestProcessor::start_detached_review`,
`apply_bespoke_event_handling`, `ThreadManager::spawn_subagent`, `CodexThread::read_thread`);
`LocalThreadStore`'s trait impl, which calls the free function `local::read_thread::read_thread`; and
test functions.

## Q3. Callers of `get_git_repo_root` through one intermediate (score: F1 over 20 functions)

The direct, non-test callers (the intermediates) are `git_diff_to_remote` (codex-git-utils),
`write_session_meta`, `AnalyticsReducer::ingest_skill_invoked`, `ThreadMetadataSync::for_create`,
`environment_id_from_cwd`, `discover_marketplace_paths_from_roots`, `run_exec_session`,
`terminal_title_project_root`, `git_check`, `build_turn_metadata_header`, `TurnMetadataState::new`,
`run_turn`, `Session::new`, `display_path_for` and `ChatWidget::status_line_project_root_for_cwd`.
None of them is gold, and each is a false positive if listed.

| Function | Crate | File (under codex-rs/) | Intermediate |
|---|---|---|---|
| `GitRequestProcessor::git_diff_to_origin` | codex-app-server | app-server/src/request_processors/git_processor.rs | `git_diff_to_remote` |
| `RolloutWriterState::write_session_meta_if_needed` | codex-rollout | rollout/src/recorder.rs | `write_session_meta` |
| `AnalyticsReducer::ingest` | codex-analytics | analytics/src/reducer.rs | `ingest_skill_invoked` |
| `LiveThread::create` | codex-thread-store | thread-store/src/live_thread.rs | `ThreadMetadataSync::for_create` |
| `list_marketplaces_with_home` | codex-core-plugins | core-plugins/src/marketplace.rs | `discover_marketplace_paths_from_roots` |
| `run_main` | codex-exec | exec/src/lib.rs | `run_exec_session` |
| `terminal_title_check` | codex-cli | cli/src/doctor/title.rs | `terminal_title_project_root` |
| `build_report` | codex-cli | cli/src/doctor.rs | `git_check` |
| `MemoryStartupContext::stage_one_request_context` | codex-memories-write | memories/write/src/runtime.rs | `build_turn_metadata_header` |
| `Session::make_turn_context` | codex-core | core/src/session/turn_context.rs | `TurnMetadataState::new` |
| `spawn_review_thread` | codex-core | core/src/session/review.rs | `TurnMetadataState::new` |
| `RegularTask::run` | codex-core | core/src/tasks/regular.rs | `run_turn` |
| `Codex::spawn_internal` | codex-core | core/src/session/mod.rs | `Session::new` |
| `ExternalAgentConfigMigrationScreen::display_description` | codex-tui | tui/src/external_agent_config_migration.rs | `display_path_for` (its own call, in the "Migrate enabled plugins from" branch) |
| `reformat_description`, a `fn` nested in `display_description` | codex-tui | tui/src/external_agent_config_migration.rs | `display_path_for` |
| `Notification::display` | codex-tui | tui/src/chatwidget/notifications.rs | `display_path_for` |
| `from` in `impl From<DiffSummary> for Box<dyn Renderable>` | codex-tui | tui/src/diff_render.rs | `display_path_for` |
| `render_changes_block` | codex-tui | tui/src/diff_render.rs | `display_path_for` |
| `new_view_image_tool_call` | codex-tui | tui/src/history_cell/patches.rs | `display_path_for` |
| `ChatWidget::status_line_project_root_name_for_cwd` | codex-tui | tui/src/chatwidget/status_surfaces.rs | `status_line_project_root_for_cwd` |

Not gold, and false positives if listed: `MessageProcessor::handle_initialized_client_request`, which
calls the app-server method `GitRequestProcessor::git_diff_to_remote`, not the git-utils function
(two intermediates); the `run_main` of codex-thread-manager-sample, which calls its own local
`run_turn`; `SecretsManager::environment_id_fallback_has_cwd_prefix` and every other test caller; and
callers of `get_git_repo_root_with_fs` only (for example `resolve_root_git_project_for_trust`).

## Q4. An `exec_command` call, from the response item to the spawn (score: key-fact coverage over 8 facts)

Graded per fact: covered (1), partial (0.5) or missed (0). Covered: the answer states the fact, in any
words, with the names that identify it (function, crate or file). Partial: the gist without the names,
or right with one material error. A wrong claim that contradicts a fact makes it missed. All functions
are in codex-core (core/src/) unless another crate is named.

- **F1.** `try_run_sampling_request` (session/turn.rs) passes each completed output item to
  `handle_output_item_done` (stream_events_utils.rs), which turns the `ResponseItem` into a `ToolCall`
  with `ToolRouter::build_tool_call` (tools/router.rs).
- **F2.** It queues `ToolCallRuntime::handle_tool_call` (tools/parallel.rs) as the item's tool future;
  `handle_tool_call_with_source` runs the call in a `tokio::spawn`ed task, under the parallel-execution
  `RwLock` (a read lock, as `exec_command` supports parallel calls), and calls
  `ToolRouter::dispatch_tool_call_with_terminal_outcome`. Covered needs the spawned task.
- **F3.** The router (`dispatch_tool_call_with_code_mode_result_inner`) builds a `ToolInvocation` and
  calls `ToolRegistry::dispatch_any_with_terminal_outcome` (tools/registry.rs), which looks the
  handler up by tool name, runs the pre-tool-use hooks and calls `handle_any_tool`.
- **F4.** `handle_any_tool` calls `handle` on a `&dyn CoreToolRuntime` trait object, which dispatches to
  `ExecCommandHandler::handle` (tools/handlers/unified_exec/exec_command.rs). That parses the
  arguments, builds the command (`get_command`) and calls `UnifiedExecProcessManager::exec_command`.
  Covered needs the trait object.
- **F5.** `exec_command` (unified_exec/process_manager.rs) calls `open_session_with_sandbox`, which
  computes the approval requirement and calls `ToolOrchestrator::run` (tools/orchestrator.rs) with a
  `UnifiedExecRuntime`.
- **F6.** The orchestrator handles approval, picks the sandbox with `SandboxManager::select_initial`
  (codex-sandboxing, sandboxing/src/manager.rs; `LinuxSeccomp` on Linux) and, in `run_attempt`, calls
  `UnifiedExecRuntime::run` (tools/runtimes/unified_exec.rs) through the generic `ToolRuntime` trait.
  Covered needs the generic dispatch.
- **F7.** `UnifiedExecRuntime::run` builds the command (`build_sandbox_command`) and calls
  `SandboxAttempt::env_for` (tools/sandboxing.rs), which calls `SandboxManager::transform`
  (codex-sandboxing). For `LinuxSeccomp`, `transform` puts the `codex-linux-sandbox` helper in front
  of the command, with arguments from `create_linux_sandbox_command_args_for_permission_profile`
  (sandboxing/src/landlock.rs) and an arg0 override (`linux_sandbox_arg0_override`). Covered needs
  `transform` and the helper.
- **F8.** `UnifiedExecProcessManager::open_session_with_exec_env` spawns the process with
  `spawn_process_with_inherited_fds` (pty, for a tty) or `spawn_process_no_stdin_with_inherited_fds`
  (pipe), both in codex-utils-pty (utils/pty/src/). Covered needs codex-utils-pty and one of the two.
