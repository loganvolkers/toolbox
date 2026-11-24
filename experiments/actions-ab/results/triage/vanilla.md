# Friction triage: the vanilla arm, rounds 6 and 7

*Triage under rule P4 (symmetric arms) of [dx-benchmarks.md](../../../../docs/design/dx-benchmarks.md)
§4.1, prompted by [FIT-AUDIT.md](../FIT-AUDIT.md) §4 correction 7. The triager built no arm and has
no stake in the result. Sources: the six transcripts (r6 and r7, vanilla-1 to vanilla-3; bcIds in
`r6/runs.tsv` and `r7/runs.tsv`), walked call by call with Python; the adjudications
`r{6,7}/adjudication/vanilla-*.json`; and the measures `r{6,7}/vanilla-*.count.json` and
`*.lines.json`.*

## Verdict

**The vanilla arm met almost no friction from the scaffold or the prompt. Three small items are
worth fixing, and none of them is a recipe.** Of 106 tool calls across the six runs, 7 trace to
the scaffold, the environment or the prompt: 0 to 4 per run, median 1. A further 3 calls, all in
one run, went to a broken todo tool in the agent runtime, which every arm shares. No run lacked a
tool or language version, failed an install, hit an environment error, or redid work after an
approach the scaffold had broken. Every run fully passed (C1 and P1–P10 held in all six), and no
adjudicator needed a fix. The rest of the calls are the inherent cost of writing an app:
orienting on the repos, writing a CLI of 479–630 non-blank lines, testing it, running it, and writing the
handoff. A smaller part is the agent's own choices.

All three fixes change files that every arm shares (`harness/setup.sh`, `harness/score.py`,
`prompts/task.md`), so they apply to every arm alike. A vanilla app's equivalent of Toolbox's
docs is its language's standard library, which the agent already knows. So this arm gets no
recipe, and these fixes add none (FIT-AUDIT §4.7: "vanilla: none, recorded as such").

**Effect on the measures.** Suppose each fix removed exactly the calls attributed to it below.
The vanilla median for calls would then stay at 18 in round 6 (12, 18, 21 becomes 11, 18, 20).
In round 7 it would fall from 20 to 16 (20, 15, 20 becomes 16, 15, 19), mostly because of one
run's three edits. Lines would not move beyond a few lines per run. These fixes can only lower
vanilla's numbers, so they cannot help Toolbox's H2 or H3.

## Proposed fixes

| # | Fix | Files in `experiments/actions-ab/` |
|---|---|---|
| F1 | Give the sandbox's repositories a local git config that turns off the host's commit signing and fsmonitor. `setup.sh` already sets ann's identity locally to override the host's `Cursor Agent` identity; set these two keys the same way. In `import()` and for `org/`, add `git -C <checkout> config commit.gpgsign false` and `git -C <checkout> config core.fsmonitor false` | `harness/setup.sh` |
| F2 | Keep harness files out of the workspace. Create `bin/` only in the arms that fill it (`toolbox`, `gha`), and write the baseline beside the workspace (`<dest>.baseline.json`) rather than inside it. `score.py lines` then reads it from there. This changes where the measure's input lives, not what it measures | `harness/setup.sh`; `harness/score.py` (docstring line 4, `lines` line 85) |
| F3 | Say which rejected push the tester will try. In the handoff paragraph, change "and a rejected push" to "and a rejected push of a change that is not yet on `main`". That is what probe P6 already does (successor `acme/probe-reject`). Then rerender with `round.sh` | `prompts/task.md` |

## Friction table

Calls are counted from the transcripts, numbered as in each run's `count.py` order. "Folded" means
the item shared a shell call with other work and cost no call of its own.

| Friction | Runs and calls spent | Evidence | Cause | Fix or reject |
|---|---|---|---|---|
| **Host git config leaks into the sandbox.** The global config has `commit.gpgsign=true` with Cursor's SSH signing program and key, `core.fsmonitor=true`, Git LFS filters and GitHub token rewrites. A vanilla agent writes code that runs git, so it has to check this config and guard against it. The prompt's "stop any long-running process you started" also makes it check for fsmonitor daemons | **2 calls**: r6 vanilla-1 #3, r7 vanilla-3 #3. Folded in r6-2 #2, r6-3 #2, r7-1 #2, r7-2 #3 and the final checks of r6-1 (#10, #12) and r7-3 (#13). 1–3 lines of `-c` flags per tool. 5 of 6 runs disabled signing explicitly | r6-1 #3: `git config --global -l \| sed -n '10,40p'; … pgrep -af fsmonitor`. Its reasoning: "pass `-c commit.gpgsign=false` so they don't depend on the environment … turn off fsmonitor … so temp clones don't leave daemons running". r7-3 #3: `git config --global -l \| grep -i -E "sign\|hook"`; its tool defines `GIT_SAFE = ["-c", "core.fsmonitor=false", "-c", "commit.gpgsign=false", …]`. No commit failed: r7-1 never disabled signing and its commits succeeded | environment | **Fix (F1).** The config belongs to the agent host, not to Acme, and `setup.sh` already overrides the host identity in each checkout. This fix covers the checkouts and `org/`. Clones that the agent's own tool makes elsewhere still inherit the global config, so expect at most about 1 call saved per run. The Toolbox arm meets the same config, but its agent writes no git code, so the extra work falls on this arm |
| **Harness files in the workspace.** The 21 MB `.baseline.json` and an empty `bin/` sit at the workspace root. Toolbox and gha get a `bin/` with their tools in it; vanilla and Backstage get an empty one | **2 calls**: r6 vanilla-3 #3, r7 vanilla-1 #3. Folded in r6-1 #2, r6-2 #2, r7-2 #2, r7-3 #2 (`ls bin`) | r6-3 #3 and r7-1 #3, the same command in both: `head -c 600 .baseline.json; echo; file bin/*` gives `bin/*: cannot open`. r6-1's reasoning after #1: "I'm noticing a bin directory and a .baseline.json file, so next I'll look into the bin folder". After #2 it concludes "the baseline snapshot belongs to the evaluator" | scaffold | **Fix (F2).** Every arm sees the baseline. An empty `bin/` suggests that something was meant to be there |
| **Rejected-push test against an already-deprecated repository.** The task asks for three real runs first, so all three repositories carry `DEPRECATED.md` before the tester arrives. The prompt says the tester will try "a rejected push" but not with which inputs. A re-request with the same inputs is a no-op: its content is already on `main`, so R5's first sentence allows success, while its "push is rejected" example suggests failure. Agents built for both readings | **3 calls**: r7 vanilla-1 #15–#17 (edits that make the outcome say a run "already" had its content). Reasoned about in all 6 runs; it shaped design and handoff text in r6-2 (refuses no-op requests), r6-3 (handoff caveat) and r7-2 | r7-1 before #15: "One edge case for the handoff: a repeat run whose content is already on `main` succeeds without pushing. I'll make the outcome say so explicitly, so a tester who triggers it isn't misled." r6-2: "a run on a repo that already has the identical DEPRECATED.md would succeed without pushing, which could hide a rejected-push failure". The r7-3 adjudication notes the same reading ("a rejected push whose content is already on main is reported succeeded") | prompt | **Fix (F3).** This clarifies the test, not the design: P6 already uses a fresh successor. It applies to every arm. Whether to make re-runs idempotent stays the builder's decision |
| Todo tool discovered, then failed twice | 3 calls: r7 vanilla-3 #4 (`get_mcp_tools`), #5 and #20 (`todo_write`, both `success: False`) | `{'resultType': 'todoWriteResult', 'value': {'success': False, …}}` | environment (agent runtime) and agent | **Reject here.** It is the subagent runtime, shared by every arm and not in `experiments/actions-ab/`. It happened once in six runs. Excluding such calls from `count.py` would change a pre-registered measure for every arm. That is a pre-registration question, not a triage fix. Report the broken tool to whoever runs the runtime |
| Looked outside the workspace at the shared machine | 1 call: r6 vanilla-1 #4 (`ls /tmp/actions-ab/r6/; tmux ls`). Folded: r7-1 #20 `pgrep -af acme` matched an unrelated Toolbox test process (`toolbox run … add-code-of-conduct`) | r6-1 #4 lists every arm's workspace (`backstage-1 … toolbox-3 … vanilla-3`) and other tmux sessions | agent | **Reject.** It went against "Work only inside {DEST}". The prompt's shared-machine line is needed for ports and tmux names in every arm. Note for whoever runs the harness: sibling arms' workspaces are readable from every run. No run read one, and it is an all-arm isolation matter, not friction |
| Orienting on the repos: layout, git logs, remotes, `CODEOWNERS`, hooks, Python and git versions | 2–4 calls per run (r6-1 #1–2, r6-2 #1–2, r6-3 #1–2 and #4–5, r7-1 #1–2, r7-2 #1–3, r7-3 #1–2) | e.g. r6-3 #4: `cat org/.git/config; git -C org status; python3 --version; …` | inherent | **Reject.** Every arm orients on the same origins and checkouts. The Toolbox arm spends its equivalent calls reading skills |
| Writing the app, its config and the per-repo definition, then committing them | 2–6 calls per run. Several runs used one edit call per small file (r6-2 #3–#7: CLI, `repos.json`, `people.json`, wrapper, `.gitignore`) | Stdlib Python CLIs of 479–630 non-blank lines in every run (`*.lines.json`) | inherent | **Reject.** This is the arm. Choosing an edit call per file rather than heredocs is the agent's choice |
| Testing the app on scratch copies of the origins, and optional test suites | 2–5 calls per run (r6-1 #7–8; r6-2 #9, #11, #16; r6-3 #13–14, #20; r7-1 #10–11, #19; r7-2 #9–11; r7-3 #10–12, #18). Lines: committed suites of 102, 176 and 188 lines in r6-2, r6-3 and r7-1 | r6-1 before #7: "exercise the failure paths in a throwaway copy of `org/` and `origins/` so the real audit log stays clean" | inherent (agent's choice of depth) | **Reject.** An app without a platform must be tested by its author. The Toolbox arm's equivalent is `toolbox test`, and its test cases are counted too. Telling agents to test less would be a directive aimed at the measure, which FIT-AUDIT §1 faults in Toolbox's skill |
| Rework of the agent's own code or tests | r6-2 #10, #12 (bugs in its own test script); r6-3 #9–10 (exception handling, handler returns); r7-1 #5 (a leftover line); r7-3 #15–16 (a stricter definition check) | r6-2 after #11: "The two failures are harness bugs, not tool bugs: the `git cat-file` exit code is 128, and `jq` can't judge commands with empty output" (its own `tests/scenarios.sh`) | agent | **Reject.** Self-review of its own code, not caused by the scaffold |
| A defensive `core.hooksPath=/dev/null` in the tool's git wrapper, then removed once the agent saw it would reach the origin's `receive-pack` and turn off its `pre-receive` hook | 2 calls: r6 vanilla-3 #8, r7 vanilla-3 #7. r7-1 and r7-2 kept a `hooksPath` override, and P6 still held in both | r7-3 reasoning before #7: "`-c` overrides reach the local `git-receive-pack` through `GIT_CONFIG_PARAMETERS`, so `core.hooksPath=/dev/null` would skip the origin's hooks; I'm removing it." | agent | **Reject.** The host config sets no `hooksPath`. This is a habit inside the same "make git safe" wrapper as the first row. F1 may remove the reason to write the wrapper, but the call is not the scaffold's |

## Not friction

These were checked and showed no cost to the runs:

- **Languages and tools:** all six runs used Python 3.12 and its standard library. `jq` was there
  when r6-2 needed it. No run installed a package or hit a version problem.
- **Ports and servers:** no run started a server, and none needed `{PORT}`.
- **The lines measure:** the adjudicators struck runtime state alike in all six runs (run records;
  r6-2's `state/mirrors`). Test suites count as the agent's choice, as the Toolbox arm's test
  cases do.
- **`preview` without `--as`:** three runs made `preview` identity-free, which tripped two
  adjudicators. R3 says "the requester can see", which leaves room for that, but no run spent a
  call on it and no probe failed. No change.

## Calls per run, by what they were spent on

| Run | Total | Orient | Build | Self-test | Real runs | Handoff | Agent rework | Scaffold, env, prompt | Runtime / agent other |
|---|---|---|---|---|---|---|---|---|---|
| r6 vanilla-1 | 12 | 2 | 2 | 2 | 2 | 2 | 0 | 1 (git config) | 1 (outside peek) |
| r6 vanilla-2 | 18 | 2 | 7 | 3 | 2 | 2 | 2 | 0 | 0 |
| r6 vanilla-3 | 21 | 4 | 5 | 3 | 3 | 2 | 3 | 1 (baseline/bin) | 0 |
| r7 vanilla-1 | 20 | 2 | 6 | 3 | 2 | 2 | 1 | 4 (baseline/bin 1, R5 no-op 3) | 0 |
| r7 vanilla-2 | 15 | 3 | 5 | 3 | 2 | 2 | 0 | 0 | 0 |
| r7 vanilla-3 | 20 | 2 | 3 | 4 | 2 | 2 | 3 | 1 (git config) | 3 (todo tool) |

"Build" covers writing the CLI, config and definitions, and committing them to the origins and
`org/`. "Real runs" covers the three required runs and syncing the checkouts. "Handoff" covers
writing `HANDOFF.md` and committing it. Rows sum to `toolCalls` in `count.json`.
