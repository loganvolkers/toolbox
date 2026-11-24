# Friction triage: the gha arm, rounds 6 and 7

*Triage under rule P4 (symmetric arms) of [dx-benchmarks.md](../../../../docs/design/dx-benchmarks.md)
§4.1, prompted by [FIT-AUDIT.md](../FIT-AUDIT.md) §4 correction 7. The triager built no arm and has
no stake in the result. Sources: the six transcripts (r6 and r7, gha-1 to gha-3; bcIds in
`r6/runs.tsv` and `r7/runs.tsv`), walked call by call with Python; the adjudications
`r{6,7}/adjudication/gha-*.json`; the measures `r{6,7}/gha-*.count.json` and `*.lines.json`; the
handoffs `r{6,7}/gha-*.HANDOFF.md`; and, read-only, the upstream docs named in G1. "#n" is the n-th
tool call in a run's transcript. Each run's count matches its `count.json` (51, 38, 53, 30, 37, 42).
The calls were sorted by hand from the call list, so a bucket may be off by one.*

## Verdict

**One prompt defect drives most of this arm's cost. Everything else the scaffold, prompt or
environment caused is small.**

**The defect: the prompt contradicts itself on GitHub-only features.**

- `platform-gha.md` says such features (environments with required reviewers, preventing
  self-review, branch protection, run history) "count if you configure them as code … and explain
  in `HANDOFF.md` how GitHub would enforce them". README §5.2 calls that *credited*, and §7 limit 1
  pre-registers it as a favour to this arm.
- The shared `task.md` says "your system must … enforce the approval rules", wants the exact
  command to "approve (as bob)", and warns that someone will test "self-approval, carol's approval
  and a rejected push".

All six runs read the task literally. Each built a local stand-in for GitHub's server: an org CLI
that gates approvals, keeps run records and audits them.

- **Lines:** 466–789 of each run's 879–1,210 kept lines are org-side.
- **Calls:** 5–18 per run went to writing and testing that CLI.
- No run considered the credit path, and none of the six adjudications credited a probe.
- One handoff says it outright. On GitHub, "`bin/acme` is then not needed: GitHub provides the
  gate and the run history" (r7 gha-3).

So the round 6 and 7 gha numbers measure an "emulate GitHub's server" reading of the prompt, not
the pre-registered one.

**The rest costs 4–6 calls per run:**

- learning act with no docs in the workspace: 2–3 calls of `--help` and probe workflows;
- act reading the sandbox's file-path origins as the repository's name: 0–4 calls;
- one fork workflow that act's schema rejects: 1 call;
- host faults that every arm shares: 0–3 calls.

Separately, three handoffs rely on GitHub API details from memory that they flag as unverified,
such as a ruleset bypass actor's id. That is the configuration the credit depends on.

**Fair equivalent of Toolbox's help.** Toolbox gets its shipped docs, not a solution. The same
level for this arm is GitHub's and act's own docs at pinned versions, copied as whole subtrees
(G1). No example workflow is added beyond what those pages contain, and no recipe. Recipe-level
parity is FIT-AUDIT §4.7's separate question.

**Effect on the measures (estimate).** G2 is the only fix that moves the result. Take its upper
bound: every org-CLI call and line disappears.

- **H3 (calls) against gha turns inconclusive in both rounds:**
  - round 6 is 21, 35, 42 against Toolbox's 32 (W 21);
  - round 7 is 25, 28, 30 against Toolbox's 24 (W 5).
- **H2 (lines) still holds:**
  - round 6 is a median of 413 against Toolbox's 139 (W 32);
  - round 7 is 463 against Toolbox's 111 (W 193).

The real effect is smaller, because R6's org-level listing and dispatch still need glue. G1 should
leave calls flat: reading would replace probing. So the round 6 and 7 calls wins over gha carry a
second label beside FIT-AUDIT's. They were obtained under the emulate-everything reading of the
gha prompt.

## Proposed fixes

| # | Fix | Files in `experiments/actions-ab/` |
|---|---|---|
| G1 | **Put the platform's own docs in the workspace, as `gha-docs/`, beside `bin/act`.** Pin every source. Copy whole subtrees, uncurated, the same rule as backstage B1. Sources: <br>1. From `github/docs` at `44f009da482ef1470b37ffea456821014d892a1f` (main, 2026-11-24): <br>• `content/actions/`: 251 files, 1.9 MB. Covers `workflow_dispatch`, workflow syntax, contexts, environments, reviewing deployments and manually running a workflow <br>• `content/repositories/configuring-branches-and-merges-in-your-repository/`: rulesets and branch protection <br>• `content/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners.md` <br>• `data/reusables/actions/`: the Liquid includes those pages use. The pages ship unrendered, as Toolbox's skills do <br>• `src/rest/data/fpt-2022-11-28/{actions,deployments,repos}.json`: the REST reference for workflow runs, pending deployments and approvals; environments and their protection rules; and rulesets. These are the APIs the prompt's "script that applies the settings through the GitHub API" calls <br>2. `nektos/act` `README.md` at tag `v0.2.98`, the act binary's version <br>3. The act user guide behind nektosact.com: `nektos/act-docs` `src/` at `35cab6107d2afb3269a61dc5de5c15c151a4574f`. It is unversioned and last changed 2025-05-01, so it is older than act 0.2.98; say so in the prompt <br>`fetch.sh` gets them with a sparse, depth-1 fetch of each pinned commit into `<src>/gha-docs/`, and `setup.sh` copies them. The prompt gets one sentence in the Toolbox prompt's form: "Its documentation is in `{DEST}/gha-docs`: … `act --help` also works." | `harness/fetch.sh`, `harness/setup.sh` (gha branch), `prompts/platform-gha.md`, `README.md` §2 ("What the workspace adds") |
| G2 | **Make the prompt say one thing about GitHub-only features.** Recommended: keep the pre-registered reading (README §2, §5.2, §7 limit 1). Append to `platform-gha.md`, after "…how GitHub would enforce them", a draft: "You need not rebuild such a feature here. Where GitHub would perform a step (a reviewer approving, refusing a self-approval, keeping the run history), `HANDOFF.md` may give the GitHub command for it (`gh` or a REST call); a probe of that step is credited when its configuration is in the repository and explained. For the three runs, say which steps GitHub would have performed." `task.md` stays shared and unchanged. The alternative is equally symmetric: drop the credit clause from `platform-gha.md` and README §2, §5.2 and §7 limit 1, and say every probe must run here. The r6 and r7 numbers already reflect that reading. Either way, decide and record it before the next freeze | `prompts/platform-gha.md` (or, for the alternative, `prompts/platform-gha.md` and `README.md`); rerender with `round.sh` |
| G3 | **Disclose the sandbox's one deviation from GitHub that act exposes, as the prompt already discloses "no Docker".** Add: "The origins here are local paths, so under act `github.repository` is an origin's path, not `acme/<name>` as on GitHub." This states a fact and prescribes no workaround | `prompts/platform-gha.md` |
| G4 | **Turn off the host's commit signing and fsmonitor in the sandbox's checkouts.** This is the same change as vanilla [F1](vanilla.md#proposed-fixes), in shared `setup.sh`, so it applies to every arm. Scratch clones an agent makes still inherit the VM's global config. A complete fix launches the runs under a global git config without `commit.gpgsign` and `core.fsmonitor` (a README §4 procedure step) | `harness/setup.sh` (shared with vanilla F1) |

## Where the calls went

| Run | Calls | Kept lines (org-side) | Learning act (help, probe workflows) | Sandbox origins vs act's repository name | Fork workflow rejected by act's schema | Org CLI: writing and testing | Host environment faults | Agent's own slips and choices | Writing the action and settings, real runs, handoff, orientation |
|---|---|---|---|---|---|---|---|---|---|
| r6 gha-1 | 51 | 1,187 (789) | 2 | 4 | 0 | 9 | 0 | 11 | 25 |
| r6 gha-2 | 38 | 879 (466) | 3 | 1 | 0 | 17 | 0 | 1 | 16 |
| r6 gha-3 | 53 | 918 (488) | 3 | 2 | 1 | 18 | 0 | 2 | 27 |
| r7 gha-1 | 30 | 958 (629) | 3 | 1 | 0 | 5 | 0 | 1 | 20 |
| r7 gha-2 | 37 | 986 (523) | 3 | 0 | 0 | 9 | 3 | 1 | 21 |
| r7 gha-3 | 42 | 1,210 (688) | 3 | 0 | 0 | 12 | 1 | 4 | 22 |

Kept lines are the adjudicators' counts after strikes. "Org-side" counts the files under `org/`:
the CLI, its config and the agent's tests. Repository-side lines run from 329 to 522: the workflow,
its step script, the settings files and their apply script, and CODEOWNERS. For comparison, the
Toolbox arm spends 5–7 calls per run reading its docs ([r6/README.md](../r6/README.md)), against
this arm's 2–3 calls on act's `--help` and probe workflows.

## Friction

| Friction | Runs and calls spent | Evidence | Cause | Fix or reject |
|---|---|---|---|---|
| **Rebuilding GitHub's server locally.** The approval gate (reviewers, no self-review, carol refused), run records, audit and run history, all in an org CLI | All six. Org CLI calls: 9, 17, 18, 5, 9, 12. Org-side lines: 789, 466, 488, 629, 523, 688 (an upper bound: R6's org-level list and request would need some glue anyway) | r6 gha-2, thinking before #3: "Since act can't pause for an approval gate, I'm weighing how to emulate that gate locally in the glue". r6 gha-3, before #4: "Since act can't simulate environment approvals, I'm also weighing an org-level CLI to enforce approval locally and keep run records". r7 gha-3, before #3: "I'm weighing how to enforce approval locally, since act ignores GitHub's environment reviewers". r7 gha-3 `HANDOFF.md`: "`bin/acme` is then not needed: GitHub provides the gate and the run history". No transcript mentions the credit; 0 of the 60 P1–P10 verdicts in the six adjudications are `credited` | **Prompt.** `platform-gha.md` ("count if you configure them as code") against `task.md` ("must … enforce the approval rules"; "approve (as bob)"; testing "self-approval, carol's approval") | **Fix: G2** |
| **No act docs in the workspace.** Flags learned from `act --help`; host-mode behaviour learned from probe workflows: what `-j` does with `needs`, that `environment:` is ignored, how inputs and `--var` arrive, `ACT` for skipping, the empty workspace, `GITHUB_TOKEN` | All six: 2, 3, 3, 3, 3, 3 | r6 gha-2 #4 (`act --help \| sed -n 80,140p` printed nothing: the help is shorter). r6 gha-3 #6–#7 (probes of `-j` with `needs`, `environment:`, `--var`, checkout). r7 gha-1 #4–#5, r7 gha-2 #4–#5, r7 gha-3 #5–#6 (the same questions). The act user guide answers most of them: `usage/index.md` (Workflows, Jobs, Vars, `GITHUB_TOKEN`, "Skipping steps", "Pass Inputs to Manually Triggered Workflows"), `support_matrix.md` (`environment` unchecked) and `not_supported.md` ("`job.environment` ignored") | **Scaffold.** Toolbox gets `toolbox-docs/`; this arm gets only the binary | **Fix: G1.** Calls are expected to stay about flat (reads replace probes); the point is symmetry |
| **GitHub API and workflow details from memory**: the ruleset bypass actor for a deploy key or app, the context available in a job-level `if`, and whether a deployment branch policy covers rulesets | Calls: r6 gha-3 1 (#18), r7 gha-3 3 (#14, #28, #29). Three more runs wrote shapes they could not check (0 calls) | r6 gha-3 #18: "Per GitHub's ruleset API, a `DeployKey` bypass takes `actor_id: null`". r7 gha-3 thinking before #14: "the job-level `if: ${{ !env.ACT }}` won't work on GitHub, since the `env` context isn't available at that level". r6 gha-2 `HANDOFF.md`: "That id is from memory and not verified". r7 gha-2 `HANDOFF.md`: "**Unverified:** whether the ruleset's `Integration` bypass for GitHub Actions (app id 15368) lets …". r6 gha-1 `HANDOFF.md`: "It is not verified that GitHub accepts an App as a ruleset bypass actor in exactly this JSON form" | **Scaffold.** This is exactly the configuration a credited probe depends on, and its reference isn't in the workspace | **Fix: G1** (`github/docs` pages and the REST reference data) |
| **act takes the repository's name from the sandbox's file-path origin.** `github.repository` is `/tmp/…/origins/alpha.git`, not `acme/alpha`. In r6 gha-1 the workaround then cascaded: rewriting the remote to `https://github.com/acme/…` with `insteadOf` made `actions/checkout` run for real, which wanted a token, and a dummy token broke act's own clone of remote actions | 4, 1, 2, 1, 0, 0. Lines: a few override lines per run (`ACME_REPOSITORY`, `ORIGIN_URL`), plus r6 gha-1's generated 6-line `insteadOf` gitconfig (struck) | r6 gha-1 #4: "Invalid repository '/tmp/actions-ab/r6/gha-1/origins/alpha.git'. Expected format {owner}/{repo}". #5: "Input required and not supplied: token". #6 and #8: "authentication required: Invalid username or token". r6 gha-3 #38: `"repository": "/tmp/actions-ab/r6/gha-3/.scratch/origins/alpha.git"`, then #39 "`GITHUB_REPOSITORY` is the origin's file path, so the settings script will also read `ACME_REPOSITORY`". r7 gha-1 #10: "ACME_REPOSITORY overrides it where act cannot derive it" | **Environment** (the local code host, README §7 limit 2). Every arm has the same origins, but only this platform derives identity from the remote URL | **Fix: G3.** The token cascade is the agent's chosen workaround and gets no fix of its own; the act guide's `GITHUB_TOKEN` section (G1) covers it |
| **A fork's own workflow fails act's schema.** alpha's `ghcr.yml` uses a permission act 0.2.98 doesn't know, so a bare `act workflow_dispatch` fails until `-W` names one file | r6 gha-3: 1 (#5). The other runs passed `-W` from the start (0) | r6 gha-3 #5: "workflow is not valid. 'ghcr.yml' … Unknown Property artifact-metadata"; then "Act parses every workflow in the folder … so I'll always pass `-W`" | **Platform** (act's schema lags GitHub's) | **Reject.** It is act's own limit on a real repository, the kind a real user meets. The user guide (G1) documents `-W` |
| **Host git config:** `commit.gpgsign=true` with an SSH signer, and `core.fsmonitor=true`, from the VM's global config | Calls: r7 gha-3 1 (#39, checking that no fsmonitor daemon outlived the run). No commit failed. Four runs (r6 gha-2, r7 gha-1, r7 gha-2, r7 gha-3) pass `-c commit.gpgsign=false` defensively, and all four put it into committed code. r7 gha-3 also wrote `core.fsmonitor=false` into its committed scripts and workflow. This adds flags to existing lines, not new lines | r7 gha-1 thinking before #3: "Global git config turns on commit signing with an SSH key … I'll just pass `-c commit.gpgsign=false`". r7 gha-3 #8: `GIT_FLAGS = ["-c", "core.fsmonitor=false", "-c", "commit.gpgsign=false", …]` | **Environment**, shared by every arm | **Fix: G4** (= vanilla F1) |
| **Python on the host**: no `python` on `PATH`, so `shell: python` fails in host mode, and `python3 -m venv` has no `pip` | r7 gha-3: 0 extra (#5 was a probe). r7 gha-2: 3 (#26–#28), to install `yamllint` for an optional lint | r7 gha-3 #5: "Cannot find: python in PATH". r7 gha-2 #27: `pip=1` inside the venv; #28 then installed it with `pip install --target` | **Platform and agent.** Host mode runs on the host's tools, which the prompt states ("no Docker … host mode") and act's `usage/runners.md` explains. The lint was the agent's choice | **Reject.** Not a scaffold fault for this arm, and the cost came from an optional step. If another arm hits the missing venv `pip`, fix it in the VM image for all arms |
| **act's default cache `~/.cache/act` is shared by every workspace on the machine** | 0 extra calls. Every run moved it with `--action-cache-path`, `XDG_CACHE_HOME` or `HOME`. r6 gha-3 #36–#37 lost 2 calls to a relative `XDG_CACHE_HOME` (counted as the agent's slip) | r7 gha-1 thinking before #4: "the shared ~/.cache/act holds other workspaces' data". r6 gha-3 #7 listed other runs' cache directories | **Environment** | **Reject.** act names each host workspace by a hash (r7 gha-3, thinking before #7: "the act cache key is hashed per working directory"), no run saw a collision, and it cost no calls |
| **Pinning actions by SHA**, looked up with `git ls-remote` on github.com | r6 gha-1: 1 (#7). r7 gha-3: 1 (#7) | r7 gha-3 after #7: "Pins found: `actions/checkout` v4.2.2 is `11bd7190…`" | **Agent** (a hardening choice) | **Reject.** The agent's choice, and the network allowed it |
| **Linting the workflows** with `act --validate --strict` and zizmor | r6 gha-2: 0 extra (inside #30). r7 gha-2: 0 extra (inside #25). r7 gha-3: 3 (#32–#34, zizmor from the host's `~/.local/bin`, outside the workspace) | r7 gha-3 after #31: "zizmor 1.32.0 is installed as a Python package, so I'll lint the workflows with it" | **Agent** | **Reject.** `act --validate` already plays the role of `toolbox validate`. zizmor is a third-party tool, not the platform's. The host's `~/.local/bin` (zizmor, yamllint, ruff) is visible to every arm alike; record it as a shared-environment note |
| **The agents' own slips and rework** | r6 gha-1: 10 (#34–#42, a push-rejection message lost to the agent's own shell pipeline under the run step's `pipefail`; #45, its exported test variables leaking into the real runs). r6 gha-2: 1 (#31, the CLI committed without its executable bit). r6 gha-3: 2 (#36–#37, a relative cache path). r7 gha-1: 1 (#6, a `sed` delimiter clash). r7 gha-2: 1 (#30, CODEOWNERS not staged). r7 gha-3: 0 | r6 gha-1 #40: "the `said=` pipeline is the likely spot where pipefail is tripping". r6 gha-1 #46: "My shell still had the sandbox variables exported from testing". r6 gha-2 before #32: "The real copy of the CLI wasn't marked executable" | **Agent** | **Reject.** These are the agent's own errors. The run step's default shell (`bash -e` with `pipefail`) is documented in `workflow-syntax.md` `defaults.run.shell`, which G1 ships |
| **Writing the action, its settings as code, CODEOWNERS, the real runs and the handoff** | All six: 16–27 calls (last column above), and 329–522 repository-side lines | r6 gha-1 #10 (a 252-line workflow), r7 gha-2 #8–#14 (script, workflow, catalogue, settings script, settings workflow, CODEOWNERS) | **Platform** | **Reject.** This is the arm's inherent cost: the prompt requires the settings as code for the credit, and every arm writes and runs its action and its handoff |

## Notes for the next freeze

- G2 is a decision about what the arm is asked to do, not about help. Settle it, and record which
  reading the round runs under, before rendering the next round's prompts. If the pre-registered
  reading stands, the adjudicator prompt needs no change: it already defers to README §5.2 for
  "credited".
- G1 adds about 7 MB of upstream files. The REST data files are large JSON; agents search them,
  they don't read them whole. If the owner prefers no category pick, ship all of
  `src/rest/data/fpt-2022-11-28/` instead.
- All four fixes either change shared files (G4) or give this arm what Toolbox already has (G1),
  and none adds a recipe. G2 and G3 change only this arm's platform paragraph, which already
  carries this arm's other environment facts.
