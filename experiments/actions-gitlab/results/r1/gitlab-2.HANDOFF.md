# HANDOFF: `set-codeowners`, a self-service action on GitLab CE

`set-codeowners` is live on `acme-w14/alpha`, `acme-w14/beta` and `acme-w14/gamma`. Each project
owns its definition on its own `main`. Runs are GitLab pipelines in this project, `acme-w14/org`.
The three requested runs succeeded, and each change was observed on the project's `main`:

| Run | Project | pattern | owners | Requested | Approved | Succeeded (observed on main) | Change |
|---|---|---|---|---|---|---|---|
| [90](http://127.0.0.1:8929/acme-w14/org/-/pipelines/90) | alpha | `*` | `@acme/proxy-maintainers` | ann 19:37:15Z | dave 19:37:43Z | 19:37:53Z | [alpha!1](http://127.0.0.1:8929/acme-w14/alpha/-/merge_requests/1) replaced line 1 |
| [91](http://127.0.0.1:8929/acme-w14/org/-/pipelines/91) | beta | `/docs/` | `@acme/docs @carol` | ann 19:37:26Z | erin 19:37:58Z | 19:38:07Z | [beta!1](http://127.0.0.1:8929/acme-w14/beta/-/merge_requests/1) created the file |
| [92](http://127.0.0.1:8929/acme-w14/org/-/pipelines/92) | gamma | `/graphify/security.py` | `@acme/security` | ann 19:37:33Z | erin 19:38:14Z | 19:38:22Z | [gamma!2](http://127.0.0.1:8929/acme-w14/gamma/-/merge_requests/2) created the file |

All times are 2026-11-24 UTC. Three test runs are also on record:

- [87](http://127.0.0.1:8929/acme-w14/org/-/pipelines/87) is `failed`. gamma's `main` was protected so that nobody could push or merge, and the merge got a 401. [gamma!1](http://127.0.0.1:8929/acme-w14/gamma/-/merge_requests/1) was closed with a note.
- [93](http://127.0.0.1:8929/acme-w14/org/-/pipelines/93) is `invalid`. It targeted `org`, which has no definition.
- [94](http://127.0.0.1:8929/acme-w14/org/-/pipelines/94) is `succeeded`, with no change. carol was refused, then added to `acme-w14/release`, approved, and was removed again.
- [99](http://127.0.0.1:8929/acme-w14/org/-/pipelines/99) is still `awaiting_approval`. ann tried to approve her own run and was refused. It would change nothing if approved.
- [100](http://127.0.0.1:8929/acme-w14/org/-/pipelines/100) is `canceled`. dave's later approval was refused because the run was canceled.
- [105](http://127.0.0.1:8929/acme-w14/org/-/pipelines/105) is `canceled`. Its pattern `/$SCO_PROJECT` arrived literally, without variable expansion.
- [106](http://127.0.0.1:8929/acme-w14/org/-/pipelines/106) is `succeeded`, with no change. bob requested and dave approved, end to end on the final `.gitlab-ci.yml`. Runs 90–92 used an earlier revision that differed only in the pipeline name and environment naming.

Nothing needs to be running. There is no server or daemon: GitLab and its runner do the work.

## How it works

```
requester ──POST /projects/acme-w14%2Forg/pipeline {inputs: project, pattern, owners}──► org pipeline on main
   GitLab checks spec:inputs regexes                    (refused: no pipeline is created)
   validate  clone <project>@main with the requester's job token; run its set_codeowners.py validate
   preview   run its set_codeowners.py render; publish the diff and the new file (artifacts)
   approve   manual job; whoever runs it is the approver. It passes only for a direct member of
             acme-w14/release who is not the requester (checked live through the API)
   apply     bot: re-check the approval and that main still matches the preview; then commit to
             set-codeowners/run-<id>, open an MR into main, and merge it
   observe   bot: read CODEOWNERS on main until it is byte-identical to the preview; only then succeed
```

| Path | Owner | What it is |
|---|---|---|
| `<project>/.gitlab/actions/set-codeowners/action.json` | project Maintainers | name, description, inputs (listed by `acme-actions list`) |
| `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` | project Maintainers | `validate` and `render`: the CODEOWNERS rule update |
| `<project>/.gitlab/actions/set-codeowners/test_set_codeowners.py` | project Maintainers | unit tests (`python3 .gitlab/actions/set-codeowners/test_set_codeowners.py`) |
| `org/.gitlab-ci.yml` | org Maintainers | the run pipeline: `spec:inputs`, five jobs, environments, `resource_group` |
| `org/ci/runner.py` | org Maintainers | the job steps (validate, preview, approve gate, apply, observe) |
| `org/bin/acme-actions` | org Maintainers | the CLI: list, preview, request, approve, status, record, runs, cancel (JSON output) |
| `org/lib/acme_gitlab.py` | org Maintainers | stdlib GitLab REST client shared by the CLI and the jobs |
| `org/setup/configure-gitlab.sh` | org Owners | the GitLab settings below, as code (idempotent; already applied) |
| `org/setup/premium.sh` | org Owners | the Premium settings, as code (CE answers 404/400; see below) |

GitLab settings (applied by `setup/configure-gitlab.sh`):

- `org` `main` is protected. Push is limited to Maintainers. "Allowed to merge" includes Developers, because GitLab requires merge permission on a branch both to create a pipeline on it and to run its manual jobs. Merge requests are **disabled** in `org`, so merge access cannot change `main`.
- `org` pipeline variables are set to `no_one_allowed`, so a requester can pass inputs only. GitLab also refuses manual-job variables: playing `approve` with `job_variables_attributes` returns 403, as verified.
- The CI variable `ACTIONS_BOT_TOKEN` is protected, masked and hidden, and scoped to the environments `set-codeowners/*`. Only the `approve`, `apply` and `observe` jobs of `org/.gitlab-ci.yml` on `main` receive it. It holds the group access token `r1-gitlab-2-set-codeowners-bot` (Maintainer of `acme-w14`, scope `api`).
- alpha, beta and gamma allow `org`'s CI job tokens in their job-token allowlist, so `validate` can clone them as the requester.
- Pushes to `org` create no pipeline: the inputs default to empty, and `workflow:rules` drops any pipeline with an empty input.
- To rotate the bot token, create a new group access token on `acme-w14` (Maintainer, `api`, name starting `r1-gitlab-2-`), write it to a file and run `GITLAB_TOKEN=$(cat $P/ops.token) BOT_TOKEN_FILE=<file> setup/configure-gitlab.sh`. No other copy of the token is kept.

## R1 to R9

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the project | Each project's definition is committed to its own `main`, in `.gitlab/actions/set-codeowners/`. A run clones it from `main` at request time and records the commit (`definition.commit` in the record). Changing validation or the rule update is an ordinary change by that project's Maintainers. The definition runs with no credentials (`env -i`), and the runner writes back only the `CODEOWNERS` it rendered, so a project's definition cannot touch other files or projects. | `alpha/.gitlab/actions/set-codeowners/*`, `beta/…`, `gamma/…`; `org/ci/runner.py` (`step_validate`, `run_definition`) |
| R2 Validated | Validation happens twice before anything changes. First, `spec:inputs` regexes in `org/.gitlab-ci.yml`: GitLab returns HTTP 400 and creates no pipeline (CLI exit 2). A missing input is refused the same way by `workflow:rules` ("The pipeline did not run"). Second, the project's own `set_codeowners.py validate` in the `validate` job: a failure leaves the run `invalid`, and `approve` and `apply` never run. Both encode the spec: `pattern` starts with `/` or `*` and has no whitespace, no `#`, at most 100 characters; `owners` is 1 to 3 `@path` owners separated by single spaces. Inputs reach scripts as `expand: false` variables, so `$`, quotes and `*` are taken literally (tested). | `org/.gitlab-ci.yml` (`spec`), `<project>/.gitlab/actions/set-codeowners/set_codeowners.py` (`validate`) |
| R3 Previewed | The `preview` job publishes the unified diff, the new file and its sha256 (`run/preview.json`, `run/CODEOWNERS.after`). `request` waits for the preview and prints it. `preview RUN` shows it again, and `preview --project … --pattern … --owners …` is a dry run that changes nothing. `apply` writes exactly the previewed bytes. If `CODEOWNERS` on `main` changed since the preview, the run fails as stale. | `org/ci/runner.py` (`step_preview`, `step_apply`), `org/bin/acme-actions` (`preview`, `request`) |
| R4 Approved by the release group | `approve` is a blocking manual job (`allow_failure: false`), so `apply` cannot start until it succeeds. GitLab records who ran it. The job passes only if that user is a direct member of `acme-w14/release` and is not the pipeline's creator. Anyone else's attempt fails and stays on record as `refused`; a valid approver can still approve later by retrying the job. `apply` repeats the check before writing. | `org/.gitlab-ci.yml` (`approve`, `apply`), `org/ci/runner.py` (`check_approval`, `latest_valid_approval`) |
| R5 Done means observed | `apply` merges through an MR into `main` and fails if GitLab refuses the merge; the MR is then closed with a note and the branch deleted. `observe` reads `CODEOWNERS` on `main` and succeeds only when it is byte-identical to the preview. A run is `succeeded` only when `observe` succeeded. Run 87 shows a locked `main` ending as `failed`. | `org/ci/runner.py` (`step_apply`, `step_observe`), `org/bin/acme-actions` (`state_of`) |
| R6 One place for the org | From `org`, `bin/acme-actions list` finds every project in `acme-w14` that has `.gitlab/actions/*/action.json` on `main`. `request --project <p>` starts a run on any of them. You never enter a project's checkout. | `org/bin/acme-actions` |
| R7 Audited | A run is a GitLab pipeline, and GitLab records it. The requester is the pipeline's user, with its creation time. Each approval attempt is a job, with its user, start and finish times, and a decision artifact. `apply` and `observe` keep their JSON artifacts permanently (`expire_in: never`). `record RUN` assembles all of this: requester and time, inputs, definition commit, validation, preview, every approval attempt with decision, reason and time, the MR, the merge commit, the observation time, and the final state. The project's MR and its merge commit (with `Requested-by:` and `Approved-by:` trailers) carry the same facts. | `org/bin/acme-actions` (`record`, `run_summary`), `org/ci/runner.py` (`out`) |
| R8 Usable by agents | Every step is one non-interactive command that prints JSON and exits with a meaningful code: 0 ok, 1 error, 2 inputs refused, 3 approval refused, 4 run failed. `--wait` and `--timeout` replace polling. The same steps work with `glab` or `curl` (below). | `org/bin/acme-actions` |
| R9 The group is GitLab's | The approve gate calls `GET /groups/acme-w14%2Frelease/members/:user_id`, which lists direct members only, at the moment of approval, and `apply` calls it again. Nothing caches the membership or copies it into config. Run 94 shows carol refused, added in GitLab, then accepted. | `org/ci/runner.py` (`check_approval`), `org/lib/acme_gitlab.py` (`direct_member`) |

## Commands

Run everything from the `org` checkout:

```bash
export HOME=/tmp/actions-gitlab/r1/gitlab-2/home
export PATH=/tmp/actions-gitlab/r1/gitlab-2/bin:$PATH
cd /tmp/actions-gitlab/r1/gitlab-2/org && git pull -q
P=/tmp/actions-gitlab/r1/gitlab-2/people
```

Start nothing: no process needs to be running.

| Step | Command |
|---|---|
| List the actions | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions list` |
| Preview (dry run, changes nothing) | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions preview --project beta --pattern '/docs/' --owners '@acme/docs @carol'` |
| Request (as ann; waits for the preview and prints the run id) | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions request --project alpha --pattern '*' --owners '@acme/proxy-maintainers'` |
| Preview of a requested run | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions preview RUN` |
| Approve (as dave or erin) | `GITLAB_TOKEN=$(cat $P/dave.token) bin/acme-actions approve RUN --wait` (`--wait` also waits for the outcome) |
| Status | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions status RUN` (add `--wait` to block until done) |
| Record | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions record RUN` |
| Recent runs | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions runs [--project alpha]` |
| Cancel a waiting run | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions cancel RUN` |

`status` reports one of these states: `previewing`, `awaiting_approval`, `approving`, `applying`, `observing`, `succeeded`, `failed`, `invalid` or `canceled`.

The same steps with `glab` alone. `glab` is already set up for this host in `$HOME/.config/glab-cli`:

```bash
export GITLAB_HOST=127.0.0.1:8929
# request (as ann)
GITLAB_TOKEN=$(cat $P/ann.token) glab ci run -R acme-w14/org -b main \
  --input project:alpha --input 'pattern:*' --input 'owners:@acme/proxy-maintainers'
# find the approve job, then approve (as dave); after a refusal use .../retry instead of .../play
GITLAB_TOKEN=$(cat $P/dave.token) glab api "projects/acme-w14%2Forg/pipelines/RUN/jobs?include_retried=true" | jq '.[]|select(.name=="approve")|{id,status,user:.user.username}'
GITLAB_TOKEN=$(cat $P/dave.token) glab api -X POST projects/acme-w14%2Forg/jobs/JOB/play
# status and record
GITLAB_TOKEN=$(cat $P/ann.token) glab ci get -R acme-w14/org -p RUN --output json
GITLAB_TOKEN=$(cat $P/ann.token) glab api projects/acme-w14%2Forg/jobs/JOB/artifacts/run/approval-JOB.json
```

## Testing the edge cases

| Scenario | Command | Expected |
|---|---|---|
| Invalid inputs | `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions request --project alpha --pattern 'docs/' --owners 'carol'` (also try `'/a b'`, `'/a#b'`, a 101-character pattern, four owners, `'@a  @b'`) | exit 2, `"refused": true`, GitLab's regex message; no pipeline is created |
| Self-approval | ann requests, then `GITLAB_TOKEN=$(cat $P/ann.token) bin/acme-actions approve RUN` | exit 3, `"decision": "refused"`, `self-approval…`; run stays `awaiting_approval` |
| Approval from outside the release group | `GITLAB_TOKEN=$(cat $P/carol.token) bin/acme-actions approve RUN` (also bob, ann, ops) | exit 3, `not a direct member of acme-w14/release` |
| Release group membership change | `GITLAB_TOKEN=$(cat $P/ops.token) glab api -X POST groups/acme-w14%2Frelease/members -f user_id=4 -f access_level=30` (add carol), then carol approves (accepted). `glab api -X DELETE groups/acme-w14%2Frelease/members/5` (remove dave), then dave approves (refused). Restore with `-X POST … -f user_id=5 -f access_level=30` | Approvals follow GitLab immediately, for all projects |
| Protected `main` | `GITLAB_TOKEN=$(cat $P/ops.token) glab api -X DELETE projects/acme-w14%2Fgamma/protected_branches/main; GITLAB_TOKEN=$(cat $P/ops.token) glab api -X POST projects/acme-w14%2Fgamma/protected_branches -f name=main -f push_access_level=0 -f merge_access_level=0`, then request and approve a run on gamma that changes something | `approve --wait` exits 4. `status` shows `failed` with `error.http_status: 401`. The MR is closed and `main` is unchanged. Restore with the same calls using `push_access_level=40 -f merge_access_level=40` |

To approve after a refusal, a release member runs `approve RUN` again; the CLI retries the job. An approval binds to that run's preview. If `CODEOWNERS` on `main` changed in the meantime, the run fails as stale, and you request again.

## What a Premium GitLab would do, and what could not run here

This is GitLab CE 19.6.1 (`"plan": "free"`). These Premium/Ultimate features are configured as code in `setup/premium.sh`:

- **Protected environments with deployment approvals.** Each environment `set-codeowners/<project>` in `org` gets an approval rule: one approval from `acme-w14/release`, with `group_inheritance_type: 0` (direct members only). GitLab then holds the `apply` job, whose environment `action: start` creates a blocked deployment, until a member approves it:

```bash
glab api -X POST projects/acme-w14%2Forg/deployments/DEPLOYMENT/approval -f status=approved
```

  GitLab answers 403 to anyone outside the group.
- **`allow_pipeline_trigger_approve_deployment: false`.** GitLab refuses the requester's own approval.

For the three runs, a Premium GitLab would have performed dave's and erin's approvals, the self-approval refusal and the release-membership check itself. The `approve` job and `check_approval` performed them here, and could stay on Premium as a second check.

Running `setup/premium.sh` here returned 404 for `POST /projects/:id/protected_environments` and 400 for `allow_pipeline_trigger_approve_deployment`, so neither is in effect. Merge request approval rules and code-owner approvals are not used. The MR is a vehicle for the change and its history; the release-group approval happens on the run.

Not verified here, or deliberately limited:

- **Shared runner isolation.** The single shared runner uses the shell executor, so all jobs, from every workspace, run as one OS user on one host. Nothing isolates `apply`'s token from a concurrent job. In production, give `approve`, `apply` and `observe` a dedicated protected runner (docker executor, `tags:`) that only runs protected-branch jobs. I could not register runners here.
- **Who can change the gate.** `org` Maintainers (ann, bob, ops) can change it, as they can change `main` anywhere in `acme-w14`. Project Maintainers can change their project's definition, including its rendering logic, but the approver sees the rendered preview, and only `CODEOWNERS` is written.
- **Cancelling.** `cancel` works while a run is waiting for its first approval. After a refused attempt the pipeline is already `failed`, so GitLab's cancel no longer applies. Such a run stays approvable until a release member approves it or an Owner deletes the pipeline. Approvals do not expire.
- **Test runs.** Test run 82 (refusal paths, with a throwaway pattern) was deleted so that it could never be approved.
