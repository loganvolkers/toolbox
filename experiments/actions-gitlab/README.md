# actions-gitlab: a governed self-service action on a GitLab-only org, Toolbox vs GitLab

*Pre-registered 2026-11-24, before the first counted run (dx-benchmarks §4.1, protocol 2). About
the box: running a declared action on a forge it doesn't own, remembering its run, and carrying it
from project to org. The designs under test are [forges.md](../../docs/design/forges.md) and
[federated-actions.md](../../docs/design/federated-actions.md). It extends
[actions-ab](../actions-ab/README.md), which ran on local bare origins, to a real GitLab. Results
are in [results/](./results/).*

**Result (round 1): the bar passes.** All six runs fully pass; Toolbox's medians are 247 lines to
maintain against 1,076 and 38 tool calls against 78, each lower by more than *W*. Read the labels
in [results/r1](./results/r1/README.md) before quoting the numbers.

## 1. Question

**On an org whose code lives only on GitLab, can a fresh coding agent deliver a self-service action
that projects own, that is validated, previewed, approved by a GitLab group other than the
requester, done only when observed on `main`, offered from one org-level place and audited, with
less to maintain and less work in Toolbox than in GitLab's own CI/CD, groups and API?**

Layers: the operator SDLC (building the action and the org's place for it) and the user SDLC
(requesting, approving and checking runs as an agent). Stages: build, test, deploy, operate.

If the bar passes, GitLab support stands beside GitHub as a default, and forges.md's claim that
Toolbox is not coupled to one forge has a benchmark behind it. If it fails, the friction is triaged
into fixes and a new round, and forges.md says GitLab is supported but not yet better than GitLab.

## 2. Arms

| Arm | Platform | What the workspace adds |
|---|---|---|
| `toolbox` | Toolbox at the frozen commit, with GitLab as its forge | `bin/toolbox*` (the binaries, also on the runner's `PATH`), `toolbox-docs/` (the operator skills, `examples/federated-actions` with its GitLab portals, `examples/repo-portal-starter`, the manifest schema) |
| `gitlab` | GitLab CE 19.6.1 itself: pipelines with `spec:inputs`, manual jobs, environments, groups, protected branches, merge requests, the REST API | Nothing beyond the shared environment. Premium and Ultimate features CE lacks (approval rules, deployment approvals, code owner approvals) count when configured as code and explained (§7, limit 1) |

The rival is the strongest for the category: an org that is GitLab-only already has GitLab's CI/CD,
groups and API, and that is what a capable team would build on. actions-ab's other rivals don't
fit: GitHub Actions doesn't run on GitLab, and Backstage's and a vanilla app's results there
(they lost on lines to maintain, by more than W) don't depend on the forge.

**Shared by both arms**, from `harness/prepare.sh` and `harness/setup.sh`: one GitLab CE 19.6.1
container and one instance runner (shell executor; bash, git, curl, python3, jq) from
`scripts/gitlab/harness.sh`, the one `scripts/gitlab-acceptance.sh` runs on; per run, a private
group `acme-w<k>` with the three projects, an empty `org` project and the `release` subgroup; the
agent as the group's Owner `ops-w<k>` with git configured; an API token per person; `bin/glab`
(the GitLab CLI 1.132.0, checksum pinned); and `gitlab-docs/`, the instance's own documentation at
its version, Markdown only, uncurated except that the guide to developing GitLab and the docs'
contributor instructions are left out. Only "Your platform" differs between the prompts.

## 3. Repositories

actions-ab's three forks at the same commits (`../actions-ab/harness/repos.tsv`), each imported as
one commit on a project's `main`:

| Project | Fork | Commit | Language | Root `CODEOWNERS` |
|---|---|---|---|---|
| alpha | `loganvolkers/dependabot-proxy` | `5f186c53` | Go | yes (`* @dependabot/maintainers`) |
| beta | `loganvolkers/valtio` | `c97e2aa9` | TypeScript | no |
| gamma | `loganvolkers/graphify` | `48888a7c` | Python | no |

## 4. Procedure

1. `harness/prepare.sh /tmp/actions-gitlab/src` downloads the forks and glab, copies the frozen
   Toolbox binaries, starts GitLab and the runner, copies GitLab's docs and creates the people
   (ann, bob, carol, dave, erin). `harness/round.sh <n> /tmp/actions-gitlab/src` builds three
   workspaces per arm under `/tmp/actions-gitlab/r<n>/<arm>-<k>`, each with its own group, and
   renders `prompts/task.md` with the arm's `prompts/platform-<arm>.md` into
   `prompts/rendered/r<n>/` (P1: committed before the first launch).
2. Each run is a fresh local subagent with the rendered prompt and nothing else, on the same model.
   All six run at once; they share GitLab and its runner (four concurrent jobs).
3. After a run ends, before anyone probes it, `harness/measure.sh` records `score.py check` (the
   final state on GitLab), `score.py lines`, `settings.py diff`, `count.py` on its transcript
   (fetched with `batch-fetch-details`), `isolation.py` and `prompt_check.py`.
4. Round 0 is a dry run, one run per arm, not counted: it checks the harness and the measures on
   both arms, and its friction is triaged, for both arms alike, before round 1 is frozen.
5. An independent adjudicator (P7) probes every run by following its `HANDOFF.md` (§5.2), with
   `prompts/adjudicator.md`, and `harness/tally.py` applies §6.

## 5. Measures and probes

### 5.1 Measures (P6)

| Measure | Source | Kind |
|---|---|---|
| **Lines to maintain** | `score.py lines`: actions-ab's measure over the `main` of every project in the run's group (subgroups included), then the workspace, skipping the platform copies, `home/` and `people/`. Non-blank lines of code and configuration added, except Markdown, copies of existing files, and runtime data the adjudicator strikes (each strike recorded) | Primary |
| **Tool calls** | actions-ab's `count.py`: tool calls in the run's transcript, also by kind | Primary |
| GitLab settings changed | `settings.py diff`: protected branches, variables, hooks, triggers, schedules, tokens, environments, job token scopes, members and CI and merge settings, per project and group, set outside files | Reported |
| Wall seconds, characters of tool results | `count.py` | Reported |

Both primary measures are actions-ab's, which its round 0 dry run showed the runtime reports
(P6); `score.py lines` over GitLab was checked on a scratch workspace before the freeze.

### 5.2 Probes (adjudicator, per run)

The adjudicator follows the run's `HANDOFF.md` without reading its source unless the handoff fails,
and records each probe as holds, fails, or **credited** (gitlab arm only: a Premium feature
configured as code and explained). "Unchanged" means the project's `main` has the same commit
before and after. Each probe rule is a line `<pattern> <owners>`.

| # | Requirement | Probe |
|---|---|---|
| C1 | Task done | `score.py check` passes: each root `CODEOWNERS` on `main` is exactly the import's with the task's rule set (alpha's first line replaced; beta's and gamma's file created with one line) |
| P1 | R1 | Each project's definition is committed on its `main`, inside that project |
| P2 | R2 | As ann on alpha: pattern `docs folder` (owners `@acme/probe`), then owners `@a @b @c @d`, then owners `carol` (pattern `/probe-invalid/` for both). All refused; alpha unchanged |
| P3 | R3 | As ann on beta: a valid preview (`/probe-preview/ @acme/probe`) shows the `CODEOWNERS` change; beta unchanged |
| P4 | R4 | As dave on gamma, from `org/`: request `/probe-approval/ @acme/probe`. Gamma unchanged; dave's approval refused; ann's refused; carol's refused; gamma still unchanged |
| P5 | R4, R5 | Erin approves P4's run. Gamma's `CODEOWNERS` is the task's line then the probe's, and the run is reported succeeded only after the system read `main` |
| P6 | R5 | With the administrator's token, alpha's `main` is protected with no one allowed to push or merge (the old protection recorded). Ann requests `/probe-reject/ @acme/probe`, dave approves: the run is not reported succeeded, and alpha is unchanged. The protection is then restored |
| P7 | R6 | From `org/`, one command lists `set-codeowners` for alpha, beta and gamma, and P4 was requested from there |
| P8 | R7 | P5's record shows dave as requester, erin as approver, the inputs, the outcome, and a time for each |
| P9 | R9 | With the administrator's token, erin is removed from `<group>/release` in GitLab, and nothing in the system is changed. Ann requests `/probe-group/ @acme/probe` on beta; erin's approval refused; dave's approval succeeds, and beta's `CODEOWNERS` gains the probe's line |
| P10 | R8 | Every probe ran non-interactively, with machine-readable output |
| P11 | Handoff | The adjudicator could operate the system from `HANDOFF.md` alone, allowing trivial fixes (a missing `PATH` entry, a port), each recorded |

**A run fully passes** when C1 and P1 to P11 all hold or are credited.

## 6. Hypotheses and bar

*W* is the wider of the two arms' ranges (max − min over three runs). A cost hypothesis **holds**
when Toolbox's median is lower than the rival's by more than *W*; it is **lost** when the rival's
median is lower by more than *W*; otherwise it is **inconclusive** (P2).

| # | Hypothesis |
|---|---|
| H1 | **Delivery.** Toolbox's arm fully passes in at least 2 of 3 runs, and in at least as many runs as the gitlab arm |
| H2 | **Lines to maintain.** Toolbox's median is lower by more than *W* |
| H3 | **Tool calls.** Toolbox's median is lower by more than *W* |

**The bar: the benchmark passes when H1 holds, and H2 or H3 holds, and neither is lost.** Anything
else fails, and the friction behind the failure is triaged into product fixes and a new round,
which reruns both arms at the new frozen commit (L11, L15).

## 7. Limits, stated before the run

1. **Premium is credited, not run.** CE has no approval rules, deployment approvals or code owner
   approvals. The gitlab arm gets them by configuration, which favours it, as actions-ab's credit
   for GitHub-only features favoured GitHub Actions. A Premium org could also run them.
2. **The designer chose the task, its people and its probes**, after building GitLab support. The
   task is held out from the Toolbox docs: no skill, example or doc has an action that edits
   `CODEOWNERS` or a rule file like it. They do mention `CODEOWNERS`, as an ownership source
   Toolbox reads (the `owner` and `add-permission` skills, `repo-portal-starter`). Its shape (a project-owned action, group approvers, an org place) is
   actions-ab's round 8 shape, which the shipped `examples/federated-actions/gitlab-org` portal
   and `scripts/gitlab-acceptance.sh` step 8 already show on GitLab. That example ships with
   Toolbox, so it counts as product documentation, as in actions-ab limit 4. R9 is not held out:
   GitLab group approvers were built for the acceptance script before this benchmark.
3. **Agents know GitLab from training; they know Toolbox only from its docs.** That weighs against
   Toolbox.
4. **GitLab group inheritance is a trap for both arms.** Owners and Maintainers of the parent group
   are inherited members of `release`; R9 says direct members. Toolbox's connector reports direct
   members; the gitlab arm must choose the right API.
5. **Settings are not lines.** What the gitlab arm sets through the API (protected branches,
   variables, tokens) is maintained too but is not in the primary measure; `settings.py` reports
   it, and the write-up gives it beside lines.
6. **Runs share one GitLab and one runner.** The people are global users, and each person's token
   can see every run's group they belong to. `isolation.py` reports any call that names another
   group or another workspace, and any gitlab-arm call that names the runner's Toolbox binaries.
7. **The runner has the Toolbox binaries on its `PATH` in both arms.** The gitlab arm's prompt
   doesn't mention them.
8. **One task, one forge version, one host.** A self-managed CE on loopback, with no TLS and no
   SSO; gitlab.com, GitLab Dedicated and Premium are not run.

## Layout

| Path | What |
|---|---|
| `harness/prepare.sh` | The shared environment: forks, glab, the frozen binaries, GitLab and its runner, GitLab's docs, the people |
| `harness/setup.sh`, `round.sh` | One workspace and its GitLab group; one round of workspaces and rendered prompts |
| `harness/score.py`, `settings.py` | C1 and lines to maintain (over GitLab and the workspace); GitLab settings changed |
| `harness/isolation.py`, `measure.sh`, `tally.py` | Isolation checks; one run's measures in one command; §6 from the adjudicator's records |
| `prompts/` | The task, each arm's platform section, the adjudicator's prompt, and the rendered prompts per round |
| `results/` | Per-run measures and probe records |
