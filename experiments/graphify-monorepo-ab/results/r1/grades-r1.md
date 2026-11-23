# Grades, round r1 (blind)

Graded blind, from `/tmp/graphify-monorepo-ab/blind-r1/` (15 answers under random labels), against
`gold/codex.md` with the scoring rules in README §4 and the matching rules at the top of the gold. Written
before the label key, `prompts/rendered/r1/arms.tsv`, `results/r1.tsv` or any per-run JSON or answers file
was opened.

**Gold hash check.** `sha256sum /tmp/graphify-monorepo-ab/gold/codex.md` gives
`d597a2fb17109f6a92981fa936eaf9981b3c9694e7ba6f5a80d40e1c687823a2`, which matches the hash recorded in
README §4.

## Result in one line

The 15 answers are almost identical. Every answer gets Q2 and Q4 fully right and Q1's direct set fully
right. They differ on two items only:

- **Q1 indirect:** 8 answers list all 22. The other 7 list 18 and leave out `codex-chatgpt`,
  `app_test_support`, `core_test_support` and `mcp_test_support`, the four workspace packages that are not
  in the explicit `[workspace] members` list.
- **Q3:** 7 answers list all 20. The other 8 leave out `ExternalAgentConfigMigrationScreen::display_description`.

No answer lists a false positive on any question.

## Grade table

Q1 is the mean of the direct F1 and the indirect F1. Q2 and Q3 are F1 over the listed functions. Q4 is
coverage of the gold's 8 facts. Score is the mean of Q1 to Q4; code is the mean of Q2 to Q4.

| Label | Q1 direct TP/FP/FN, F1 | Q1 indirect TP/FP/FN, F1 | Q1 | Q2 TP/FP/FN, F1 | Q3 TP/FP/FN, F1 | Q4 (facts) | Score | Code |
|---|---|---|---|---|---|---|---|---|
| 154 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.981 | 0.991 |
| 210 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.994 | 0.991 |
| 283 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.981 | 0.991 |
| 324 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.994 | 0.991 |
| 421 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 1.000 | 1.000 |
| 476 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.994 | 0.991 |
| 478 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 0.988 | 1.000 |
| 537 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 0.988 | 1.000 |
| 565 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 1.000 | 1.000 |
| 683 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 0.988 | 1.000 |
| 721 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 1.000 | 1.000 |
| 833 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 20/0/0, 1.000 | 8/8, 1.000 | 1.000 | 1.000 |
| 855 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.981 | 0.991 |
| 885 | 7/0/0, 1.000 | 22/0/0, 1.000 | 1.000 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.994 | 0.991 |
| 891 | 7/0/0, 1.000 | 18/0/4, 0.900 | 0.950 | 12/0/0, 1.000 | 19/0/1, 0.974 | 8/8, 1.000 | 0.981 | 0.991 |

Exact values: indirect F1 with 18 of 22 = 36/40 = 0.9; Q3 F1 with 19 of 20 = 38/39 = 0.97436. The scores
are therefore 0.98109 (18 and 19), 0.98750 (18 and 20), 0.99359 (22 and 19) or 1 (22 and 20). The code
subscore is 0.99145 (Q3 19) or 1 (Q3 20).

### False positives and false negatives by name

- **Q1 direct:** no false positives or false negatives in any answer.
- **Q1 indirect, false negatives:** 154, 283, 478, 537, 683, 855 and 891 each miss the same four crates:
  `codex-chatgpt`, `app_test_support`, `core_test_support` and `mcp_test_support`. None has a false
  positive. The 8 answers that list all 22 include the three test-support crates with a hedge ("leave them
  out if…", in 210, 324 and 721). Hedged items count as listed.
- **Q2:** every answer lists exactly the 12 gold functions. There are no false positives: all 15 name
  `CodexThread::read_thread`, `ThreadManager::spawn_subagent`, `start_detached_review`,
  `apply_bespoke_event_handling` and `LocalThreadStore`'s trait impl only as excluded. There are no false
  negatives.
- **Q3, false negatives:** 154, 210, 283, 324, 476, 855, 885 and 891 each miss
  `ExternalAgentConfigMigrationScreen::display_description`. 154 and 210 say outright that it is two hops
  away, through `reformat_description`. The others list only `reformat_description`. No answer has a Q3
  false positive. Each answer lists the 15 intermediates only as intermediates, and lists
  `MessageProcessor::handle_initialized_client_request` / `GitRequestProcessor::git_diff_to_remote`,
  `thread-manager-sample`'s `run_main`, and the test callers only as excluded.
- **Q4:** every answer covers F1 to F8 with the names that identify them. That includes the spawned task in
  `handle_tool_call_with_source` (F2), the `&dyn CoreToolRuntime` trait object in `handle_any_tool` (F4),
  the generic `ToolRuntime` dispatch in `ToolOrchestrator::run`/`run_attempt` (F6), `SandboxManager::transform`
  with the `codex-linux-sandbox` helper (F7), and `codex-utils-pty` with one of the two spawn functions
  (F8). I found no wrong claim that contradicts a fact.

### Judgement calls

- **855, Q3.** The answer lists `reformat_description` in its table, then adds in a note: "If
  `reformat_description` should be attributed to its enclosing function,
  `ExternalAgentConfigMigrationScreen::display_description` is the one-hop caller instead." I graded
  `display_description` as **not listed**. The note offers it as a replacement under a convention the
  answer did not adopt; it does not hedge an extra item. Counting it as listed would credit both sides of
  an either/or. Counting it as listed would raise 855's Q3 from 0.974 to 1 and its score from 0.981 to
  0.988.
- **537 and 721, Q3.** Both list `display_description` and `reformat_description`, and then say the
  answer depends on the convention for nested functions. Both are in their tables, so both count as listed.

## Disputed gold items

**No answer names an item outside the gold.** That holds for every set in Q1, Q2 and Q3, so there is no
outside-gold claim to check.

Two gold items are disputed the other way: some answers leave them out, and two answers argue against one
of them. I checked both in `/tmp/graphify-monorepo-ab/src/codex`, and the gold stands for both.

| Gold item | Answers that omit it | What the code shows | Ruling |
|---|---|---|---|
| Q3 `ExternalAgentConfigMigrationScreen::display_description` | 8 (154 and 210 explicitly say it is two hops away) | `codex-rs/tui/src/external_agent_config_migration.rs:233-237`: in `display_description`'s own body, outside the nested `reformat_description` (lines 194–208), the `"Migrate enabled plugins from "` branch calls `display_path_for(std::path::Path::new(source), cwd)` directly. The function starts at line 189 and is not test code. | The gold is right; the omission is a false negative. |
| Q1 `codex-chatgpt`, `app_test_support`, `core_test_support`, `mcp_test_support` (indirect) | 7 | They are not in `codex-rs/Cargo.toml`'s `members`, but they are path entries in `[workspace.dependencies]` (lines 131, 148, 243, 244). `cargo metadata --no-deps --format-version 1 --offline` lists all four among its 120 packages. Each has a normal `[dependencies]` edge into the closure: `chatgpt/Cargo.toml` and `core/tests/common/Cargo.toml` depend on `codex-core`, `app-server/tests/common` on `codex-core` and `core_test_support`, and `mcp-server/tests/common` on `codex-mcp-server` and `core_test_support`. | The gold is right. It was computed from `cargo metadata`, and these are workspace packages with normal edges. |

## Notes for the unblinding

- **Partial blinding leak.** 476, 565, 833 and 855 cite "the [tool] portal's `codex-tools` entity"
  (`dependents`, and in 855 `dependentCount: 7`). The tool name is redacted, but the word "portal" and the
  entity fields point at one arm. I graded those answers by the same rules as the rest; no grade depends
  on the citation.
- **Spread.** The whole round lies between 0.981 and 1.000. The only things that separate answers are
  four implicit workspace members in Q1 and one nested-function boundary in Q3.
