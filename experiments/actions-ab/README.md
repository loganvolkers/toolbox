# actions-ab: a governed self-service action across an org, four ways

*Pre-registered 2026-11-24, before the first counted run (dx-benchmarks §4.1, protocol 2). About
the box: running a declared action, remembering its run, and carrying it from repo to org. The
design under test is [federated-actions.md](../../docs/design/federated-actions.md). Results are in
[results/](./results/); the adjudicator's verdict goes in `ADJUDICATION.md`.*

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
| `gha` | GitHub Actions, run by act 0.2.98 in host mode | `bin/act`; from round 8, `gha-docs/` (GitHub's docs pages for Actions, branch rules and code owners, the REST reference for actions, deployments and repos, act's README and user guide, all at pinned commits). GitHub-hosted features act cannot run count when configured as code and explained (§7, limit 1) |
| `backstage` | Backstage 1.57 (`@backstage/create-app`, scaffolder backend 4.3, GitHub module) | `backstage/` (the app, its `node_modules`, and an `app-config.local.yaml` giving the run's port and one static token per person), built by `harness/backstage/build-app.sh`; from round 8, its install committed, and `backstage-docs/` (the `docs/` tree of backstage/backstage at v1.57.0 and the scaffolder and catalog backends' OpenAPI specs) |
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

1. `harness/fetch.sh /tmp/actions-ab/src` downloads the tarballs and the rivals' docs;
   `harness/backstage/build-app.sh /tmp/bs` builds the Backstage app; `[TASK=<name>]
   harness/round.sh <n> /tmp/actions-ab/src` builds three workspaces per arm under
   `/tmp/actions-ab/r<n>/<arm>-<k>` and renders `prompts/<name>.md` (default `task`) with the arm's
   `prompts/platform-<arm>.md` into `prompts/rendered/r<n>/` (P1: committed before the first
   launch). Every workspace has the same
   origins, checkouts and empty `org/`; only "Your platform" differs (P4).
2. Each run is a fresh local subagent with the rendered prompt and nothing else, on the same model,
   in batches of one run per arm (four at once), each with its own ports. From round 8, the VM's
   global `commit.gpgsign` and `core.fsmonitor` are off while runs are live (the workspace's
   checkouts turn them off locally too), so no arm meets the host's commit signing.
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
7. **Backstage pays for the local code host.** Its GitHub integration (discovery provider, URL
   reader) would replace the catalog provider and file reads every Backstage run wrote for bare
   local origins; Toolbox reads git paths natively, and vanilla calls `git`. Decided before round
   8: no credit, the primary measure stays as registered, and write-ups list those lines
   separately (triage B4).

## 8. Round 8: a held-out task

*Pre-registered before round 8's freeze.* Round 7 met the bar on its tally, but the
[fit audit](results/FIT-AUDIT.md) found the Toolbox docs fitted to the task (P5) and the rivals
short of fair help (P4), so it is not a pass ([results/r7/README.md](results/r7/README.md)).
Round 8 keeps §1–§7 and changes three things.

**1. The task is new.** No Toolbox doc, skill or example was written for it.
[`prompts/task-notice.md`](prompts/task-notice.md) asks for `add-notice`. It edits an existing
file rather than adding one: a `> **Info:**` or `> **Warning:**` line at the top of each root
README, which replaces any earlier notice. Its approvers are a group defined once at the org
level (R9), which the round 0–7 task did not have:

| Person | Role |
|---|---|
| ann, bob | Maintainers of alpha, beta and gamma |
| carol | Developer |
| dave, erin | The release group: the only approvers, never of their own run |

**2. Every fix from the triage is applied, to every arm it concerns:**

- shared, in `setup.sh`:
  - the host's commit signing and fsmonitor are off (vanilla F1, gha G4, §4 step 2);
  - harness files are kept out of the workspace (F2);
- the task says which rejected push the tester will try (F3);
- the rivals' own docs are in the workspace, pinned and uncurated (gha G1, backstage B1);
- the gha prompt says one thing about GitHub-only features: they need not be rebuilt, and a test
  of a step GitHub would perform is credited when its configuration is committed and explained
  (G2). It also discloses that `github.repository` is an origin's path under act (G3);
- the Backstage app arrives with its install committed (B2), and its tokens are described as what
  they are (B3);
- §7 limit 7 records B4.

The Toolbox arm runs with the P5 corrections (`7193247`: generic skills, fixtures and `--help`;
`fb5dc3f`: group approvers in a parent's checkout, and a decision on who serverless approvals are
for). The adjudicator prompt now takes the probe table as a parameter (`{PROBES}`, `{LAST}`).

**3. Probes.** The probes in the table below replace §5.2's for this round. A run fully passes
when C1 and P1 to P11 all hold or are credited.

| # | Requirement | Probe |
|---|---|---|
| C1 | Task done | `score.py check` passes: each root README on its origin's `main` is exactly the task's notice line, a blank line, and the README as imported |
| P1 | R1 | Each repository's definition is committed on its origin's `main`, inside that repository |
| P2 | R2 | As ann on alpha: level `urgent`, then a 200-character message, then a message with a newline. All refused; alpha unchanged |
| P3 | R3 | As ann on beta: a valid preview (`info`, "Probe: preview only, nothing should change.") shows the README change; beta unchanged |
| P4 | R4 | As dave on gamma, from `org/`: request (`warning`, "Probe: the release group approves this one."). Gamma unchanged; dave's approval refused; ann's refused; carol's refused; gamma still unchanged |
| P5 | R4, R5 | Erin approves P4's run. Gamma's README starts with the probe's notice, the task's notice is gone (one notice, not two), the rest is as imported, and the run is reported succeeded only after the system read the origin |
| P6 | R5 | A `pre-receive` hook in `origins/alpha.git` rejects every push. Ann requests (`info`, "Probe: the origin rejects this push."), dave approves: the run is not reported succeeded. The hook is then removed |
| P7 | R6 | From `org/`, one command lists `add-notice` for alpha, beta and gamma, and P4 was requested from there |
| P8 | R7 | P5's record shows dave as requester, erin as approver, the inputs, the outcome, and a time for each |
| P9 | R9 | Following the handoff, remove erin from the release group with one change at the org level. Ann requests on beta (`info`, "Probe: erin has left the release group."); erin's approval refused; dave's approval succeeds, and beta's README has the probe's notice in place of the task's |
| P10 | R8 | Every probe ran non-interactively, with machine-readable output |
| P11 | Handoff | The adjudicator could operate the system from `HANDOFF.md` alone, allowing trivial fixes (a missing `PATH` entry, a port), each recorded |

**Limits added for round 8:**

8. **The designer chose the held-out task and its people.** The task was chosen after round 7,
   and group approvers that resolve in a checkout (`fb5dc3f`) were built before the freeze,
   because the fit audit's held-out-task correction called for a requirement the old task lacked.
   An independent fit audit of rounds 8 and on judges whether that change is general (P5).
9. **The gha credit grew.** G2 says a credited step need not be rebuilt, and P9 can be credited
   through a GitHub team configured as code. This favours the gha arm, as limit 1 does.
10. **Workspaces are not isolated from each other.** Every run can read `/tmp/actions-ab/r<n>/`,
    where the other arms' workspaces are (the vanilla triage saw one round 7 run list it; none
    read one). Round 8 checks each transcript with `harness/isolation.py` and reports any hit.

## Layout

| Path | What |
|---|---|
| `harness/fetch.sh`, `repos.tsv` | The pinned forks |
| `harness/setup.sh`, `round.sh` | One workspace; one round of workspaces and rendered prompts |
| `harness/backstage/app-config.local.yaml`, `build-app.sh` | The Backstage arm's overlay; how its app is built |
| `harness/score.py`, `count.py` | The measures |
| `harness/isolation.py` | From round 8: tool calls that name another run's workspace or list the round's directory |
| `harness/tally.py` | §6 from the adjudicator's records: medians, W, H1 to H3 and the bar |
| `prompts/` | The tasks (`task.md` for rounds 0–7, `task-notice.md` for round 8), each arm's platform section, the adjudicator's prompt, and the rendered prompts per round |
| `results/triage/` | Rival-arm friction from rounds 6 and 7, with the fixes applied before round 8 |
| `results/` | Per-run measures and probe records |
