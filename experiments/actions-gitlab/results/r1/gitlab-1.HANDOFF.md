# HANDOFF: `set-codeowners`, a self-service action on GitLab CE

**Status.** `set-codeowners` is defined in `acme-w12/alpha`, `acme-w12/beta` and `acme-w12/gamma`, each on
its own `main`. It is listed, previewed, requested, approved and audited from this project (`acme-w12/org`)
with one CLI, `bin/acme-actions`. All three requested runs succeeded: the system read `CODEOWNERS` back from
each project's `main` before it reported success.

| Run | Inputs | Requested | Approved | Apply pipeline | Observed on `main` |
|---|---|---|---|---|---|
| [alpha!2](http://127.0.0.1:8929/acme-w12/alpha/-/merge_requests/2) | `*` `@acme/proxy-maintainers` | ann, 19:42:51Z | dave, 19:43:18Z | #111 | `20fa791cecc4`, 19:43:27Z |
| [beta!1](http://127.0.0.1:8929/acme-w12/beta/-/merge_requests/1) | `/docs/` `@acme/docs @carol` | ann, 19:42:52Z | erin, 19:43:19Z | #110 | `b3d6e92f6c93`, 19:43:27Z |
| [gamma!3](http://127.0.0.1:8929/acme-w12/gamma/-/merge_requests/3) | `/graphify/security.py` `@acme/security` | ann, 19:42:53Z | erin, 19:43:20Z | #112 | `a48d4afa44b6`, 19:43:29Z |

All times are on 2026-11-24. alpha's `CODEOWNERS` line `* @dependabot/maintainers` was replaced. beta and gamma
had no root `CODEOWNERS`, so the action created one. Each file now holds exactly the one rule plus a newline.

Earlier test runs are kept as an audit trail. Each one is closed and reported as failed or cancelled:

| Run | What it tested |
|---|---|
| gamma!1 | A pattern containing `$HOME`. It exposed a real bug: the runner expands `$VARS` in `CI_MERGE_REQUEST_DESCRIPTION`. The check failed, and the controller reported the run as failed. Fixed, so the check now reads the inputs from the run commit |
| gamma!2 | carol approving her own run (GitLab CE accepts this). bob, outside the release group, approving through `glab mr approve`. A Maintainer (ops) starting the apply pipeline by hand without a valid approval. The controller ignored both approvals, and the apply job refused and reported the run as failed |
| alpha!1 | `main` protected with "No one" allowed to push or merge. dave removed from the release group, and his approval refused. carol added to the release group, and her approval accepted. GitLab then refused the apply pipeline, so the run was reported as failed, not succeeded. Afterwards protection was restored to Maintainers and the release group to dave and erin |
| beta!2 | `acme-actions cancel`. Approving a cancelled run is refused |

## How it works

This is about the **box** (carrying, running, remembering, reporting). The **tool** is each project's
`set_codeowners.py`, which also runs on its own (`validate`, `compute`, `selftest`) with no GitLab at all.

```
request (anyone in acme-w12, own token)        approve (release group member, own token)
  acme-actions request ...                       acme-actions approve RUN   (or: glab mr approve)
        |                                                |
        v                                                v
 merge request in the project  --- MR pipeline: set-codeowners:check (validate + preview.diff)
 (author = requester, inputs, preview diff, label action::set-codeowners)
        |
        v   controller (bin/acme-actions controller, port 7320, acts as the actions bot)
 valid approval?  direct member of acme-w12/release (read live), not the author, newer than the last push
        |   checks green?
        v
 API pipeline on main with spec:inputs {action, run, pattern, owners}  (GitLab validates them by regex)
        |
        v   set-codeowners:apply  (environment codeowners, resource_group set-codeowners)
 re-verify everything -> merge the MR -> read CODEOWNERS back from main -> record succeeded or failed
```

A run is a GitLab merge request. GitLab authenticates who requested it (the MR author), who approved it (the
MR approval) and when. The controller and the apply job write every decision as a note on the MR. Each note
carries a JSON event and is authored by the bot user `group_139_bot_4f1416e88eb4c6a87eba9f84e6b90ab7`. The
CLI trusts events only from that user, as configured in `actions.json`, and this project's `main` is
protected (Maintainers).

## Requirements

Paths starting `<project>/` are in each of `acme-w12/alpha`, `beta` and `gamma` (identical copies). Other
paths are in this project.

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the project | The definition, logic and pipelines are committed to each project's `main` under `.gitlab/actions/set-codeowners/`. The root `.gitlab-ci.yml` declares the `spec:inputs` and includes them. The org CLI discovers actions by reading each project's `main`, so there is no central copy. It downloads the project's own script at a pinned `main` commit to validate and preview. Project maintainers change the action with a normal MR | `<project>/.gitlab-ci.yml`, `<project>/.gitlab/actions/set-codeowners/{action.json,set_codeowners.py,check.yml,apply.yml,settings.json,README.md}` |
| R2 Validated | Four layers, each refusing before anything changes. (1) `request` and `preview` run the project's `validate`: exit 2, a JSON list of errors, nothing created. (2) The MR pipeline job `set-codeowners:check` re-validates the inputs and checks that the run branch changes only `CODEOWNERS`, exactly as computed. If it fails, the controller fails the run. (3) GitLab refuses to create the apply pipeline when an input misses its `spec:inputs` regex (tested: HTTP 400 "provided value does not match required RegEx pattern"). (4) The apply job re-validates before it merges | `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` (`validate`), `<project>/.gitlab-ci.yml` (`regex:`), `<project>/.gitlab/actions/set-codeowners/check.yml` |
| R3 Previewed | `acme-actions preview` prints the current file, the proposed file and a unified diff, and creates nothing. `request` prints the same diff and puts it in the MR description. The MR's Changes tab is the diff the approver approves, bound to its HEAD SHA (`approve` sends `sha`). The check job publishes `preview.diff` and `preview.json` as artifacts | `bin/acme-actions` (`cmd_preview`, `cmd_request`), `<project>/.gitlab/actions/set-codeowners/check.yml` |
| R4 Approved by the release group | An approval counts only if its author is a direct member of `acme-w12/release` at evaluation time, is not the run's author, and approved after the run's latest push. `acme-actions approve` refuses self-approval and outsiders (exit 3) before calling GitLab. Approvals given another way, such as `glab mr approve` (GitLab CE accepts them), are evaluated by the controller. Any that don't count are recorded as `approval-ignored` with the reason, and the run keeps waiting. The apply job evaluates again before merging, so a Maintainer who starts the apply pipeline by hand without a valid approval gets a failed run (tested on gamma!2). On Premium, GitLab enforces the same rules (see below) | `bin/acme-actions` (`evaluate_approvals`, `cmd_approve`, `Controller.handle_open`), `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` (`evaluate_approvals`, `Run.main`), `<project>/.gitlab/actions/set-codeowners/settings.json` |
| R5 Done means observed | Only the apply job writes `succeeded`, and only after it re-reads `CODEOWNERS` on `main` (polling for up to 60 s) and finds exactly the computed content. The event records the observed commit SHA and time. Any refusal is recorded as `failed` with GitLab's message: the apply pipeline refused on a locked `main` (tested on alpha!1), the merge refused, `CODEOWNERS` changed on `main` since the request, or wrong content. If the apply pipeline ends without recording an outcome, the controller records `failed` | `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` (`Run.main`), `bin/acme-actions` (`Controller.handle_applying`) |
| R6 One place for the org | From this checkout, `bin/acme-actions list` finds every project in `acme-w12` with `.gitlab/actions/*/action.json` on its default branch. `preview`, `request`, `approve`, `status`, `record`, `runs` and `cancel` take a project (`alpha`, or `acme-w12/alpha`) or a run (`alpha:2` or `alpha!2`) | `bin/acme-actions`, `actions.json` |
| R7 Audited | `acme-actions record RUN` returns: the requester and request time (MR author, `created_at`); the inputs and the definition commit; the preview diff; the approval acted on (approver, approval time, when the controller accepted it); approvals that did not count, with reasons; the apply pipeline and its user; the outcome with time, observed SHA or reason; and the MR merge data. Every event is also a permanent note on the MR. GitLab separately records the deployment to environment `codeowners` and the merge commit message (requester, approver, pipeline) | `bin/acme-actions` (`cmd_record`), the MR notes, `<project>/.gitlab/actions/set-codeowners/apply.yml` (artifact `run-record.json`) |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document. Exit codes: 0 ok, 1 error or failed run, 2 invalid input, 3 refused, 4 not found, 124 when `status --wait` times out. `status --wait N` blocks until the run is terminal. The controller exposes `GET /healthz` and `POST /kick` on 127.0.0.1:7320, and `controller --once` does a single pass when nothing may stay running | `bin/acme-actions` |
| R9 The group is GitLab's | The controller, the CLI and the apply job read `GET /groups/acme-w12%2Frelease/members`, which lists direct members only, at the moment they decide. Nothing caches or copies the membership. The group path is set in `actions.json` (org policy) and in each `action.json`, and a run must satisfy both. Tested: removing dave stopped his approval at once, and adding carol let her approve | `bin/acme-actions` (`group_members`), `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` (`evaluate_approvals`) |

## Commands

Run everything from this checkout, as below. `RUN` is the value printed by `request`, for example
`acme-w12/alpha!2`. Use `alpha:2` (or single-quote `'alpha!2'`) in an interactive bash, where `!` triggers
history expansion.

```bash
export HOME=/tmp/actions-gitlab/r1/gitlab-1/home
export PATH=/tmp/actions-gitlab/r1/gitlab-1/bin:$PATH
cd /tmp/actions-gitlab/r1/gitlab-1/org
git pull -q origin main
PEOPLE=/tmp/actions-gitlab/r1/gitlab-1/people
```

**Start the controller.** This is the only long-running process. It reads the bot token from this project's
CI/CD variable using the operator's token:

```bash
tmux new-session -d -s r1-gitlab-1-controller -c "$PWD" \
  "GITLAB_TOKEN=$(cat $PEOPLE/ops.token) ./bin/acme-actions controller >> $HOME/controller.log 2>&1"
curl -s http://127.0.0.1:7320/healthz          # {"ok": true, ...}
# stop it:  tmux kill-session -t r1-gitlab-1-controller
```

Without tmux: `GITLAB_TOKEN=$(cat $PEOPLE/ops.token) ./bin/acme-actions controller >> $HOME/controller.log 2>&1 &`.
If nothing may stay running, run `GITLAB_TOKEN=$(cat $PEOPLE/ops.token) ./bin/acme-actions controller --once`
after each approval. One pass starts the apply pipeline, and the apply job records the outcome itself.

**List, preview, request, approve, check status, read the record:**

```bash
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions list
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions preview beta set-codeowners --pattern '/docs/' --owners '@acme/docs @carol'
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions request beta set-codeowners --pattern '/docs/' --owners '@acme/docs @carol'
#   -> {"ok": true, "run": "acme-w12/beta!3", "status": "checking", "diff": "...", ...}
GITLAB_TOKEN=$(cat $PEOPLE/erin.token) ./bin/acme-actions approve beta:3        # or dave.token
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions status beta:3 --wait 300
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions record beta:3
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions runs                   # every run in the group
GITLAB_TOKEN=$(cat $PEOPLE/ann.token)  ./bin/acme-actions cancel beta:3          # withdraw a run not yet applied
```

`--input pattern=... --input owners=...` is equivalent to `--pattern` and `--owners`. The action name
defaults to `set-codeowners`. Statuses: `checking`, `awaiting-approval`, `approved`, `applying`, `succeeded`,
`failed`, `cancelled`. With the controller running, a run usually goes from approval to `succeeded` in about
10 s, though the shared runner can add queueing time.

GitLab's own commands for the same steps:

- Approve: `GITLAB_TOKEN=$(cat $PEOPLE/dave.token) glab mr approve 3 -R acme-w12/beta`.
- Inspect a run: `glab mr view 3 -R acme-w12/beta --comments`.
- Approval state: `glab api projects/69/merge_requests/3/approvals`.

## Testing guide (what to expect)

Project ids: alpha 68, beta 69, gamma 70. Release group id: 140. User ids: carol 4, dave 5, erin 6.

| Test | Command | Expected |
|---|---|---|
| Invalid pattern | `GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./bin/acme-actions request gamma --pattern 'docs/' --owners '@acme/x'` (also `'/a b'`, `'/a#b'`, a 101-character pattern) | Exit 2, `"error": "invalid inputs"` and the reasons. Nothing is created (`runs` is unchanged) |
| Invalid owners | `... request gamma --pattern '/x/' --owners '@a  @b'` (also `'@a @b @c @d'`, `'acme'`, `'@a/'`) | Exit 2, nothing created |
| Invalid input that bypasses the CLI | `curl -s -X POST -H "PRIVATE-TOKEN: $(cat $PEOPLE/ops.token)" -H 'Content-Type: application/json' http://127.0.0.1:8929/api/v4/projects/68/pipeline -d '{"ref":"main","inputs":{"action":"set-codeowners","run":"1","pattern":"docs x","owners":"@a"}}'` | HTTP 400 from GitLab: "`pattern` input: provided value does not match required RegEx pattern" |
| Self-approval | ann requests, then `GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./bin/acme-actions approve RUN` | Exit 3, "cannot approve their own run". If ann uses `glab mr approve` instead, CE accepts it, but the controller records `approval-ignored` and `status` stays `awaiting-approval` |
| Outsider approves | `GITLAB_TOKEN=$(cat $PEOPLE/bob.token) ./bin/acme-actions approve RUN` (also carol, ops) | Exit 3, "not a direct member of acme-w12/release". Through `glab mr approve` the approval is ignored, and `record` lists it under `approvals_now.ignored` and as an `approval-ignored` event |
| Membership change | Add: `curl -s -X POST -H "PRIVATE-TOKEN: $(cat $PEOPLE/ops.token)" http://127.0.0.1:8929/api/v4/groups/140/members -F user_id=4 -F access_level=30`. Remove: `curl -s -X DELETE -H "PRIVATE-TOKEN: $(cat $PEOPLE/ops.token)" http://127.0.0.1:8929/api/v4/groups/140/members/5` | After adding, carol can approve a run she didn't request. After removing, dave is refused, and an approval he gave to a run not yet applied stops counting. Restore with `.../groups/140/members -F user_id=5 -F access_level=30` and `DELETE .../groups/140/members/4`. Use curl here: this glab build returned 403 on member changes where curl with the same token works |
| Locked `main` | `glab api -X DELETE projects/69/protected_branches/main` then `glab api -X POST projects/69/protected_branches -f name=main -f push_access_level=0 -f merge_access_level=0`. Request and approve a run on beta | `status --wait` exits 1 with `"status": "failed"` and the reason `GitLab refused to start the apply pipeline on main: ... You do not have sufficient permission to run a pipeline on 'main'`. If `main` is locked after the apply pipeline has started, the merge is refused and the apply job records `failed`. That path is designed in but was not exercised here. Restore with `glab api -X DELETE projects/69/protected_branches/main` and `glab api -X POST projects/69/protected_branches -f name=main -f push_access_level=40 -f merge_access_level=40` |
| Apply without approval | A Maintainer starts the apply pipeline on an open run by hand (use the run's own inputs): `curl ... /projects/70/pipeline -d '{"ref":"main","inputs":{"action":"set-codeowners","run":"<iid>","pattern":"<p>","owners":"<o>"}}'` | The apply job fails with "no valid approval ...", records `failed` and closes the MR. Nothing reaches `main` |

## GitLab Premium: configured as code, enforced by GitLab there

Each project's `.gitlab/actions/set-codeowners/settings.json` lists the settings as REST calls.

- `acme-actions apply-settings` applies the project-level ones (Maintainer token).
- The controller applies `run_settings` to each run's merge request as soon as it sees the run, and records
  the result as a `run-settings` event.
- On this CE instance the Premium calls return 404 or 400, reported as "not available on this edition".

The Free items were applied: the project is shared with `acme-w12/release` as Reporter, and only Maintainers
may pass pipeline variables.

| Rule | Here (CE) | On Premium | Setting |
|---|---|---|---|
| One approval from `acme-w12/release` | Controller and apply job, `evaluate_approvals` | MR approval rule `release group` on each run's MR. GitLab blocks the merge until it is satisfied and counts only direct group members (docs: `user/project/merge_requests/approvals/rules.md`, "only direct members of the group can approve") | `run_settings[run-approval-rule-release-group]` |
| Requester cannot approve | CLI refuses. Controller and apply job ignore the approval | **Prevent approval by merge request creator**: GitLab answers ann's `POST .../approve` with an error | `settings[mr-approval-settings]` (`merge_requests_author_approval: false`) |
| Pushing after approval voids it | Approval must be newer than the latest MR diff version | **Remove all approvals when commits are added** | `reset_approvals_on_push: true` |
| Committers cannot approve | Not enforced (runs have one commit, by the requester) | **Prevent approvals by users who add commits** | `merge_requests_disable_committers_approval: true` |
| Only approved runs reach `main` | The apply job re-verifies. The bot token is a protected, masked variable scoped to environment `codeowners` | Protected environment `codeowners` that only the actions bot may deploy to | `settings[protected-environment-codeowners]` |

For the three runs, a Premium GitLab would have performed these steps itself:

- It would have created the `release group` approval rule on alpha!2, beta!1 and gamma!3.
- It would have counted dave's and erin's approvals against that rule and set `approvals_left` to 0.
- It would have refused an approval from ann, and kept the merge blocked against any approval from bob or carol.
- It would have let only the bot deploy to `codeowners`.

The controller would then read GitLab's verdict (`approved`, `approvals_left`) instead of computing its own.
Starting the apply pipeline, merging, and observing `CODEOWNERS` on `main` (R5) stay with this system on
every tier. Deployment approvals on the protected environment are deliberately not used for R4: GitLab's
"prevent self-approval" there blocks the pipeline's triggerer (the bot), not the requester.

## What I could not run here

- The Premium settings above: protected environments, MR approval rules and the approval settings are not
  in CE, and their API calls return 404 or 400. They are committed and applied by code, but GitLab could not
  enforce them here.
- GitLab refusing a self-approval. CE accepted carol's self-approval through the raw API (gamma!2). This
  system's own enforcement was tested instead.
- Webhooks into the controller. It polls the API every 3 s instead, so no instance setting such as "allow
  local requests from webhooks" was needed or changed.
- The web UI. There is no browser here, and everything was done through the REST API, `git`, `glab` and the CLI.

## State created in GitLab (all inside `acme-w12`)

- Group access token `r1-gitlab-1-actions-bot` on `acme-w12`, with the Maintainer role and `api` scope,
  expiring 2026-12-24. Its bot user is `group_139_bot_4f1416e88eb4c6a87eba9f84e6b90ab7`. Rotate it with
  `GITLAB_TOKEN=$(cat $PEOPLE/ops.token) ./bin/acme-actions bootstrap --rotate`, which updates the variables
  below.
- CI/CD variable `ACME_ACTIONS_TOKEN`, protected and masked:
  - on `acme-w12/org`, read by the controller;
  - on alpha, beta and gamma, scoped to environment `codeowners`, read only by the apply job.
- Group labels `run::awaiting-approval`, `run::applying`, `run::succeeded` and `run::failed`, plus the project
  label `action::set-codeowners`. Labels are only a convenience: status comes from the bot's events.
- Free settings from `settings.json` on alpha, beta and gamma (see above). alpha's `main` protection was
  changed for the locked-`main` test and restored to Maintainers / Maintainers.

## Files

| Path | What |
|---|---|
| `bin/acme-actions` | The org CLI and the controller (Python 3 standard library only) |
| `actions.json` | Org policy and config: group, release group, bot user, controller port 7320 |
| `<project>/.gitlab-ci.yml` | `spec:inputs` (with regexes), `workflow:rules`, includes |
| `<project>/.gitlab/actions/set-codeowners/action.json` | The action's definition |
| `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` | Validation, the edit, preview, MR check, apply |
| `<project>/.gitlab/actions/set-codeowners/check.yml` | MR pipeline job (validate and preview) |
| `<project>/.gitlab/actions/set-codeowners/apply.yml` | Apply job (environment `codeowners`, `resource_group`) |
| `<project>/.gitlab/actions/set-codeowners/settings.json` | GitLab settings as code (Free and Premium) |

Limits:

- The controller is a single process. If it is down, approvals wait (nothing is lost), and they are acted on
  when it starts.
- Runs are serialized per project by `resource_group`.
- A run whose `CODEOWNERS` changed on `main` after the request fails as stale and must be requested again.
- Project Maintainers own the action and can change it, as R1 intends. They can push to `main` directly in
  any case.
