# HANDOFF: the `set-codeowners` self-service action

`set-codeowners` sets one rule in a project's root `CODEOWNERS` on `main`. It runs entirely on
GitLab CE 19.6.1. Each run is a GitLab pipeline. GitLab validates the inputs, records who requested
and who approved, and makes the commit. The jobs enforce the release group's approval and confirm
the result on `main`. Nothing has to be kept running: there is no server, only the instance runner
GitLab already has.

All three runs requested by ann succeeded:

| Project | Run (pipeline) | Rule | Requested | Approved | Outcome, observed on `main` |
|---|---|---|---|---|---|
| alpha | [79](http://127.0.0.1:8929/acme-w16/alpha/-/pipelines/79) | `* @acme/proxy-maintainers` | ann, 19:33:31Z | dave, 19:34:07Z | succeeded 19:34:11Z, commit `e4b5228e` |
| beta | [80](http://127.0.0.1:8929/acme-w16/beta/-/pipelines/80) | `/docs/ @acme/docs @carol` | ann, 19:33:31Z | erin, 19:34:07Z | succeeded 19:34:12Z, commit `83fc9b21` (file created) |
| gamma | [81](http://127.0.0.1:8929/acme-w16/gamma/-/pipelines/81) | `/graphify/security.py @acme/security` | ann, 19:33:32Z | erin, 19:34:07Z | succeeded 19:34:13Z, commit `04ea89c0` (file created) |

(All on 2026-11-24.)

## How it works

```
org/actions request alpha ...          ann's token: POST /projects/:id/pipeline {ref: self-service, inputs}
  GitLab checks inputs against spec:inputs regex -> refuses (HTTP 400) or creates the pipeline (user = ann)
  validate  -> run/request.json   requester, time, inputs (re-validated in Python)
  preview   -> run/preview.json   diff against CODEOWNERS on main now (also printed in the job log)
  apply     (manual, environment `codeowners`): playing it IS the approval
             refuses unless the player is a direct member of acme-w16/release and not the requester,
             then commits to main through the API as the bot -> run/approval.json
  verify    -> run/outcome.json   reads CODEOWNERS on main again; succeeded only if the rule is in place
org/actions approve alpha <run>         dave's token: POST /jobs/:id/play (or /retry after a refusal)
```

The pieces:

| Piece | Where | What it does |
|---|---|---|
| Action definition | each project, `main`: `.gitlab/actions/set-codeowners/action.yml` | `spec:inputs` with the validation regexes, and the four jobs |
| Action logic | each project, `main`: `.gitlab/actions/set-codeowners/codeowners.py` | validation, the CODEOWNERS edit, the approval check, the write, the verification. The pipeline and `actions preview` both run this file from `main` |
| GitLab settings as code | each project, `main`: `.gitlab/actions/set-codeowners/settings.sh` | protected `self-service` ref, `ci_config_path` pinned to `main`, pipeline variables off, Premium settings (attempted) |
| CI entry point | each project, `main`: `.gitlab-ci.yml` | includes the action when a pipeline asks for `action: set-codeowners` on `self-service`, and creates no pipelines otherwise |
| Org CLI | org, `main`: `actions` | list, preview, request, approve, status, record, runs, cancel. Python stdlib only, JSON output |
| Bot | org, `main`: `setup/bot-token.sh` | group access token `r1-gitlab-3-set-codeowners-bot` (Maintainer on `acme-w16`), held in the protected, masked and hidden group variable `SET_CODEOWNERS_BOT_TOKEN` |
| Settings for all projects | org, `main`: `setup/apply-settings.sh` | runs each project's own `settings.sh` from its `main` |

**Why runs use a `self-service` ref, not `main`.** On CE, only a user who may merge to a pipeline's
branch can run its manual job. `main` allows Maintainers only, so dave and erin (Developers) could
not approve a pipeline on `main`, and once `main` is protected against everyone, nobody could. So
`settings.sh` protects a `self-service` branch: nobody may push and Developers+ may merge. Any
Developer can request a run there and play its job, and only protected refs receive the bot token.
`ci_config_path` is `.gitlab-ci.yml@acme-w16/<project>:main`, so every pipeline uses `main`'s
definition. The jobs use `GIT_STRATEGY: empty` and download `codeowners.py` from `main`. Nothing
merged into `self-service` can change what a run does.

## R1 to R9

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the project | Each project's definition, logic and settings are committed to its own `main`, and its Maintainers change them by changing `main`. `ci_config_path` pins runs to that copy. The org CLI discovers actions from each project's `main` and holds no definition. | `<project>/.gitlab-ci.yml`, `<project>/.gitlab/actions/set-codeowners/{action.yml,codeowners.py,settings.sh}` |
| R2 Validated | GitLab refuses to create the pipeline when `pattern` or `owners` fails the `spec:inputs` regex (RE2), so no job runs and no ref or file changes: `POST /pipeline` returns 400 *"`pattern` input: provided value does not match required RegEx pattern"*. The `validate` job re-checks in Python (it also rejects Unicode spaces and control characters), and `apply` checks again before writing. `actions preview` validates locally first. | `action.yml` (`spec:inputs`), `codeowners.py` (`validate`) |
| R3 Previewed | `actions preview` computes the change with the project's own `codeowners.py` against `CODEOWNERS` on `main`, with no side effects. Every run's `preview` job stores the diff (`run/preview.json`, `run/preview.diff`, job log) before `apply` can run. `request --wait` and `status` show it. `apply` refuses if `CODEOWNERS` on `main` changed since the preview. | `codeowners.py` (`job_preview`, `job_apply`), `org/actions` |
| R4 Approved by the release group | `apply` is a blocking manual job (`allow_failure: false`), so nothing after it runs until it succeeds. It reads its own job record with `CI_JOB_TOKEN` (`GET /job`) to learn who played it, so this cannot be spoofed with variables, and variables are disabled anyway. It refuses, before any write, if the player is the requester (the pipeline's user) or is not a direct member of `acme-w16/release`. Each refusal is a failed job and a failed deployment recording that user. A valid member then approves by retrying the job. Developers cannot override job variables (`ci_pipeline_variables_minimum_override_role: no_one_allowed`). `apply` also refuses runs that were canceled or already applied. | `action.yml` (`apply`), `codeowners.py` (`job_apply`), `settings.sh` |
| R5 Done means observed | `apply` commits with the Commits API (with `last_commit_id`). If GitLab refuses (e.g. `main` protected against everyone: *403 You are not allowed to push into this branch*), the job fails with that reason and `verify` never runs. `verify` re-reads `CODEOWNERS` on `main` and checks: the rule is the first line for the pattern, re-applying is a no-op, and the commit is an ancestor of `main`. The CLI reports `succeeded` only when `verify` succeeded and its `outcome.json` says so. | `codeowners.py` (`job_apply`, `job_verify`), `org/actions` (`compute`) |
| R6 One place for the org | From `org/`: `./actions list` discovers every project in `acme-w16` whose `main` defines the action. `preview`, `request`, `approve`, `status`, `record`, `runs` and `cancel` take a project name. | `org/actions` |
| R7 Audited | The record is GitLab's own data. Pipeline user and `created_at` give the requester and when. Each `apply` job's user and `started_at` give every approval attempt and when (also mirrored as deployments of environment `codeowners`). `request.json` holds the inputs. `approval.json` holds the decision and reason, `outcome.json` the observation, and the commit trailers repeat `Run:`, `Requested-by:` and `Approved-by:`. Artifacts are kept with `expire_in: never`. `./actions record` assembles all of it. | `action.yml` (artifacts, environment), `codeowners.py`, `org/actions` (`record`) |
| R8 Usable by agents | Every command is non-interactive, prints JSON and has distinct exit codes: 0 ok, 1 usage or API error, 2 inputs refused, 3 approval refused, 4 run failed. `--wait` blocks until the next decision point. Every step is also a plain GitLab REST call or a `glab` command (below). | `org/actions` |
| R9 The group is GitLab's | `apply` reads `GET /groups/acme-w16%2Frelease/members` (direct members, active) at the moment of approval. Nothing is cached or copied. Tested: bob's approval of run 85 was refused, bob was added to `acme-w16/release` in GitLab, his next approval was accepted, and he was removed again. | `codeowners.py` (`release_members`), `action.yml` (`RELEASE_GROUP`) |

## Commands

### Setup (once per shell; nothing has to be started)

```bash
export HOME=/tmp/actions-gitlab/r1/gitlab-3/home
export PATH=/tmp/actions-gitlab/r1/gitlab-3/bin:$PATH
cd /tmp/actions-gitlab/r1/gitlab-3/org && git pull -q origin main
PEOPLE=/tmp/actions-gitlab/r1/gitlab-3/people
# only for the glab equivalents: this GitLab speaks plain http
glab config set --host 127.0.0.1:8929 api_protocol http; export GITLAB_HOST=127.0.0.1:8929
```

The runs need only GitLab and its shared runner, which are already up. To re-apply the settings,
or to set the action up after adding it to another project, run as an Owner:
`GITLAB_TOKEN=$(cat $PEOPLE/ops.token) setup/bot-token.sh && GITLAB_TOKEN=$(cat $PEOPLE/ops.token) setup/apply-settings.sh`.

### The steps

```bash
# list set-codeowners for all projects
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions list

# preview (no side effects)
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions preview beta --pattern '/docs/' --owners '@acme/docs @carol'

# request as ann (prints {"run": <id>, ...}; --wait returns once the preview is ready)
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions request beta --pattern '/docs/' --owners '@acme/docs @carol' --wait

# approve as dave or erin (--wait returns the decision and, if approved, the outcome)
GITLAB_TOKEN=$(cat $PEOPLE/erin.token) ./actions approve beta <run> --wait

# status of one run, and of every run
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions status beta <run>
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions runs

# the run's record: requester, approval attempts, inputs, preview, outcome, timestamps, jobs, deployments
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions record beta <run>

# withdraw a run that has not been approved yet (not after a refused approval; see Limits)
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions cancel beta <run>
```

`status` values: `previewing`, `awaiting-approval` (also after a refused approval), `approving`,
`verifying`, `succeeded`, `failed`, `invalid`, `canceled`.

The same steps with `glab` (GitLab's own records, without the CLI):

```bash
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) glab ci run -R acme-w16/beta -b self-service \
  -i action:set-codeowners -i 'pattern:/docs/' -i 'owners:@acme/docs @carol'            # request
GITLAB_TOKEN=$(cat $PEOPLE/erin.token) glab ci trigger apply -R acme-w16/beta -p <run>  # approve (manual job)
GITLAB_TOKEN=$(cat $PEOPLE/erin.token) glab api -X POST projects/acme-w16%2Fbeta/jobs/<apply job id>/retry  # approve after a refusal
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) glab api projects/acme-w16%2Fbeta/pipelines/<run>/jobs?include_retried=true  # who did what, when
GITLAB_TOKEN=$(cat $PEOPLE/ann.token) glab api projects/acme-w16%2Fbeta/jobs/<job id>/artifacts/run/approval.json
```

### Test recipes

- **Invalid inputs** are refused by GitLab with exit 2, and no pipeline is created. Examples:
  `--pattern docs/`, `--pattern '/a#b'`, a 101-character pattern, `--owners '@a @b @c @d'`,
  `--owners '@a  @b'`, `--owners 'a'`. `./actions preview` refuses them too, locally.
- **Self-approval:** ann requests, then `GITLAB_TOKEN=$(cat $PEOPLE/ann.token) ./actions approve <p> <run> --wait`
  exits 3 with `"reason": "ann requested this run and cannot approve it"`. The run stays
  `awaiting-approval`. A release member who requests a run cannot approve it either.
- **Approval from outside the release group** (bob, carol, ops): exit 3,
  `"<user> is not a direct member of acme-w16/release"`.
- **Release group membership change:** `POST /groups/acme-w16%2Frelease/members` (bob needs
  `access_level=40` because he inherits Maintainer) or `DELETE .../members/<user_id>` in GitLab.
  The next approval follows the new membership in all three projects.
- **`main` protected so nobody may push or merge:**
  ```bash
  OPS=$(cat $PEOPLE/ops.token); A=http://127.0.0.1:8929/api/v4
  curl -s -H "PRIVATE-TOKEN: $OPS" -X DELETE $A/projects/acme-w16%2Fgamma/protected_branches/main
  curl -s -H "PRIVATE-TOKEN: $OPS" -X POST $A/projects/acme-w16%2Fgamma/protected_branches -d name=main -d push_access_level=0 -d merge_access_level=0
  # request as ann and approve as erin: approve exits 4, status "failed", reason "GitLab refused the commit to main: HTTP 403 ..."
  # restore: same DELETE, then POST with push_access_level=40 merge_access_level=40
  ```
  The request and approval still work because runs live on `self-service`. This was run on
  alpha before release, and the refused commit was reported as failed.

## What GitLab Premium would do here, and what could not run

`settings.sh` applies these settings through the API. On this CE instance they return 404 and are
reported as `skipped`:

| Premium feature | Setting (in `settings.sh`) | What GitLab would enforce |
|---|---|---|
| Protected environment `codeowners` | `POST /projects/:id/protected_environments` with `deploy_access_levels: [{group_id: <acme-w16/release>}]` | Only direct members of the release group could run the `apply` job. GitLab would refuse ann's, bob's and carol's attempts outright (403), instead of the job refusing them. |
| Deployment approvals | the same call with `approval_rules: [{group_id: <acme-w16/release>, required_approvals: 1}]` | The deployment would wait for an approval: `glab api -X POST projects/:id/deployments/:deployment_id/approval -f status=approved` as dave or erin. |
| Prevent self-approval | `PUT /projects/:id` with `allow_pipeline_trigger_approve_deployment=false` | GitLab would refuse an approval from the user who triggered the pipeline (the requester). |

For the three runs, a Premium GitLab would have performed dave's approval of alpha and erin's
approvals of beta and gamma as deployment approvals on `codeowners`, and would itself have refused
approvals from the requester or outside the release group. Here, the `apply` job made those checks
and GitLab recorded the user who played it. The job's check would stay as a second line of
defence.

What could not run here:

- The Premium settings above, and the deployment-approval command (HTTP 404 on CE).
- No browser: everything was done and tested through the REST API, `git` and the CLI. The UI
  shows the same pipelines, jobs and deployments.

## Limits and history

- **Runs predating the fix:** runs 79, 80 and 81 ran before the `GIT_STRATEGY: empty` fix. Their
  results were confirmed by `verify` and by reading `CODEOWNERS` on each `main` afterwards. Run 83
  (alpha, a no-op approved by bob during the R9 test) failed because of the bug that fix addresses:
  the shell runner's reused directory leaked a stale `preview.json`. Run 85 repeated that test
  after the fix and succeeded.
- **Other test runs:** run 88 (beta) is a test request I canceled before approval, and a later
  attempt to apply it was refused as canceled. Run 96 (gamma) tested `cancel`. Run 98 (gamma, a
  no-op) tested the `glab` commands: ann's self-approval was refused, and it is still
  `awaiting-approval`. Pipeline 77 on
  alpha was a pre-release test (`main` protected against everyone, R5), which I deleted because
  its definition predates the current one.
- **Commit author:** commits are authored by the bot (`r1-gitlab-3-set-codeowners-bot`). The
  requester and approver are in the commit trailers and the pipeline record.
- **Org-level trust:** Maintainers of a project can change its action and its settings (R1).
  Owners of `acme-w16` hold the bot token, which can be rotated with `setup/bot-token.sh --rotate`
  and expires 2027-11-20.
- **Cancelling after a refusal:** a refused approval fails the `apply` job and, with it, the
  pipeline, and GitLab cannot cancel a failed pipeline. Such a run cannot be withdrawn. It stays
  `awaiting-approval` and changes nothing unless a release member approves it.
- **Retries outside the CLI:** a run is final once succeeded, failed or canceled; the CLI refuses
  to approve it. GitLab still lets someone with merge access retry a job directly. `apply` then
  refuses runs that were canceled or already applied. A retry of an approved run that failed
  (for example once `main` is unprotected again) would run the full approval check again.
