# actions-ab: a governed self-service action across an org, four ways

*Pre-registered 2026-11-24, before the first counted run (dx-benchmarks §4.1, protocol 2). About
the box: running a declared action, remembering its run, and carrying it from repo to org. The
design under test is [federated-actions.md](../../docs/design/federated-actions.md). Results are in
[results/](./results/) and the verdict in [ADJUDICATION.md](./ADJUDICATION.md).*

## 1. Question

**Can a fresh coding agent deliver a self-service action that repositories own, that is validated,
previewed, approved by someone other than the requester, done only when observed, offered from one
org-level place and audited, with less to maintain and less work in Toolbox than in GitHub
Actions, Backstage's scaffolder, or an app written from scratch?**

Layers: the operator SDLC (building the action and the org's place for it) and the user SDLC
(requesting, approving and checking runs as an agent). Stages: build, test, deploy, operate.

## 2. Arms

| Arm | Platform | What the workspace adds |
|---|---|---|
| `toolbox` | Toolbox at the frozen commit | `bin/` (the binaries), `toolbox-docs/` (the operator skills, `examples/federated-actions`, `examples/repo-portal-starter`, the manifest schema) |
| `gha` | GitHub Actions, run by act 0.2.98 in host mode | `bin/act`. GitHub-hosted features act cannot run count when configured as code and explained (§6, limit 1) |
| `backstage` | Backstage 1.57 (`@backstage/create-app`, scaffolder backend 4.3, GitHub module) | `backstage/` (the app, its `node_modules`, and an `app-config.local.yaml` giving the run's port and one static token per person) |
| `vanilla` | None: any language | Nothing. Python 3.12, Go 1.22 and Node 24 are installed |

The rivals are the strongest per category: GitHub Actions is where repo-level actions already live
(the research brief's conclusion 2), Backstage is the most-used open-source IDP, and a vanilla app
is what an agent writes when told to use nothing.

## 3. Repositories

Three open-source forks, imported at pinned commits as each run's origins (`harness/repos.tsv`):

| Name in the run | Fork | Commit | Language |
|---|---|---|---|
| alpha | `loganvolkers/dependabot-proxy` | `5f186c53` | Go (has a root `CODEOWNERS`) |
| beta | `loganvolkers/valtio` | `c97e2aa9` | TypeScript |
| gamma | `loganvolkers/graphify` | `48888a7c` | Python |

## 4. Procedure

1. `harness/fetch.sh /tmp/actions-ab/src` downloads the tarballs; `harness/round.sh <n>
   /tmp/actions-ab/src` builds three workspaces per arm under `/tmp/actions-ab/r<n>/<arm>-<k>` and
   renders `prompts/task.md` with the arm's `prompts/platform-<arm>.md` into
   `prompts/rendered/r<n>/` (P1: committed before the first launch). Every workspace has the same
   origins, checkouts and empty `org/`; only "Your platform" differs (P4).
2. Each run is a fresh local subagent with the rendered prompt and nothing else, on the same model,
   in batches of one run per arm (four at once), each with its own ports.
3. After a run ends, before anyone probes it: `score.py check` (the final state), `score.py lines`
   (lines to maintain), and `count.py` on its transcript (fetched with `batch-fetch-details`).
4. An independent adjudicator (P7) probes every run by following its `HANDOFF.md` (§5.2) and labels
   each hypothesis.

## 5. Measures and probes

### 5.1 Measures (P6)

| Measure | Source | Kind |
|---|---|---|
| **Lines to maintain** | `score.py lines`: non-blank lines of code and configuration added anywhere in the workspace, except the origins, the platform copies, Markdown, copies of existing files, and runtime data the adjudicator strikes (each strike recorded) | Primary |
| **Tool calls** | `count.py`: tool calls in the run's transcript, also by kind (edit, read, search, shell, other; L18) | Primary |
| Wall seconds, characters of tool results, dropped results | `count.py` | Reported (L20) |

A dry run (round 0, one Toolbox run, not counted) shows both primary measures before the bar uses
them (L14).

### 5.2 Probes (adjudicator, per run)

The adjudicator follows the run's `HANDOFF.md` without reading its source unless the handoff fails,
and records each probe as holds, fails, or **credited** (gha only: configured as code for a GitHub
feature act can't run, and explained). "Unchanged" means the origin's `main` has the same commit
before and after.

| # | Requirement | Probe |
|---|---|---|
| C1 | Task done | `score.py check` passes: the three `DEPRECATED.md` files on the origins, exact first line, reason present |
| P1 | R1 | Each repository's definition is committed on its origin's `main`, inside that repository |
| P2 | R2 | As ann on alpha: successor `Acme/Bad Name`, then a 5-character reason. Both refused; alpha unchanged |
| P3 | R3 | As ann on beta: a valid preview (successor `acme/probe-preview`) shows the file and its content; beta unchanged |
| P4 | R4 | As ann on gamma, from `org/`: request (successor `acme/probe-approval`). Gamma unchanged; ann's approval refused; carol's approval refused; gamma still unchanged |
| P5 | R4, R5 | Bob approves P4's run: gamma's `main` gets the probe's `DEPRECATED.md`, and the run is reported succeeded only after the system read it from the origin |
| P6 | R5 | A `pre-receive` hook in `origins/alpha.git` rejects every push; ann requests (successor `acme/probe-reject`), bob approves: the run is not reported succeeded. The hook is then removed |
| P7 | R6 | From `org/`, one command lists `mark-deprecated` for alpha, beta and gamma, and P4 was requested from there |
| P8 | R7 | P5's record shows ann as requester, bob as approver, the inputs, the outcome, and a time for each |
| P9 | R8 | Every probe ran non-interactively, with machine-readable output |
| P10 | Handoff | The adjudicator could operate the system from `HANDOFF.md` alone, allowing trivial fixes (a missing `PATH` entry, a port), each recorded |

**A run fully passes** when C1 and P1 to P10 all hold or are credited.

## 6. Hypotheses and bar

*W* is the wider of the two arms' ranges (max − min over three runs). A cost hypothesis **holds**
when Toolbox's median is lower than the rival's by more than *W*; it is **lost** when the rival's
median is lower by more than *W*; otherwise it is **inconclusive** (P2).

| # | Hypothesis |
|---|---|
| H1 | **Delivery.** Toolbox's arm fully passes in at least 2 of 3 runs, and in at least as many runs as each rival |
| H2 | **Lines to maintain.** Against each rival, Toolbox's median is lower by more than *W* |
| H3 | **Tool calls.** Against each rival, Toolbox's median is lower by more than *W* |

**The bar: the benchmark passes when H1 holds and, against every rival, H2 or H3 holds and
neither is lost.** Anything else fails, and the friction behind the failure is triaged into
product fixes and a new round, which reruns every arm at the new frozen commit (L11, L15).

## 7. Limits, stated before the run

1. **GitHub's server features are credited, not run.** Environments with required reviewers,
   self-review prevention, branch protection and run history can't run under act. The gha arm gets
   them by configuration, which favours it.
2. **The code host is local.** Origins are bare repositories; there are no pull requests, and
   identity is a name. Every arm has the same simulation.
3. **One task.** A day-2 action that edits a repository. Creating a repository, provisioning a
   resource, or calling an external API would weigh the arms differently.
4. **The Toolbox arm gets a shipped example close to the task.** `examples/federated-actions`
   has a repo action with approvers and an org portal over repos. It ships with Toolbox, as
   `examples/template` ships with Backstage, so it counts as product documentation, not a leak.
5. **The designer built Toolbox's actions.** The adjudicator is independent (P7); the designer
   wrote this file and the harness.
6. **Agents know GitHub Actions and Backstage from training; they know Toolbox only from its docs.**
   That is the real situation of a new tool, and it weighs against Toolbox.

## Layout

| Path | What |
|---|---|
| `harness/fetch.sh`, `repos.tsv` | The pinned forks |
| `harness/setup.sh`, `round.sh` | One workspace; one round of workspaces and rendered prompts |
| `harness/backstage/app-config.local.yaml` | The Backstage arm's overlay |
| `harness/score.py`, `count.py` | The measures |
| `prompts/` | The task, each arm's platform section, and the rendered prompts per round |
| `results/` | Per-run measures and probe records |
