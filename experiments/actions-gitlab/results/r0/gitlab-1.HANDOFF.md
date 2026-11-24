# HANDOFF: `set-codeowners`, a self-service action on GitLab CE

**State:** built and in use. The three requested runs succeeded and each was observed on `main`:

| Run | Requested by | Approved by | Rule now in `CODEOWNERS` on `main` | Commit | Executor (pipeline / job on `main`) |
|---|---|---|---|---|---|
| `alpha/65` | ann | dave | `* @acme/proxy-maintainers` (replaced `* @dependabot/maintainers`) | `eeaa76ff` | 68 / 112 |
| `beta/66` | ann | erin | `/docs/ @acme/docs @carol` (file created) | `4baaac00` | 69 / 113 |
| `gamma/67` | ann | erin | `/graphify/security.py @acme/security` (file created) | `e202296b` | 70 / 114 |

**Nothing has to be running.** The system is GitLab itself: each project's pipelines, its runner and the
REST API, plus a stateless CLI in this project. No server, daemon or tmux session was started, and none is
left running.

## How it works

```
 org/bin/acme-actions (as you, with your token)                 project acme-w2/<p> (owned by its maintainers)
 ────────────────────────────────────────────                   ─────────────────────────────────────────────
 list    -> reads .gitlab/actions/*/action.json on each project's main
 preview -> branch actions/set-codeowners/preview-…  ─► pipeline (inputs mode=preview): validate ─► preview
 request -> branch actions/set-codeowners/request-…  ─► pipeline (inputs mode=request): validate ─► preview ─► approve (manual)
 approve -> plays the manual job "set-codeowners:approve" as the approver ───────────────────────────┘
                approve job ── bot-owned trigger, inputs only ─► pipeline on main (mode=execute): execute
                                                               (group bot; re-checks request, approver, inputs;
                                                                commits CODEOWNERS; reads main back)
                approve job waits for that deployment, reads its record, reads main itself ─► success / failure
 status / record / runs -> read the pipelines, jobs and their artifacts back from GitLab
```

* **A run is one request pipeline.** Its id is the run id (`alpha/65` = project alpha, pipeline 65). GitLab
  records who created it (the requester) and who played its `set-codeowners:approve` job (the approver).
* Request pipelines run on a fresh branch cut from `main` (`actions/set-codeowners/request-<time>-<user>`),
  because Developers (dave, erin, carol) may not run pipelines or play jobs on the protected `main`.
* The **executor** is the only thing that writes. It is a pipeline on protected `main`, created through a
  pipeline trigger owned by the group bot `r0-gitlab-1-actions-bot` (a group access token on `acme-w2`, role
  Maintainer). The bot token is a **protected** group variable, so only pipelines on protected refs can
  read it, never a request branch. The executor trusts nothing from the request branch. Through the API it
  checks:
  1. the approval job is a running `set-codeowners:approve`, and who played it;
  2. who created the request pipeline;
  3. that the request ran a commit already on `main`, so nobody edited the definition on the branch;
  4. the previewed inputs, validated again;
  5. the live direct members of `acme-w2/release`.

  Only then does it commit and read `main` back.

## Files

| Where | File | What |
|---|---|---|
| each project (`alpha`, `beta`, `gamma`) | `.gitlab-ci.yml` | `spec:inputs` (action, mode, pattern, owners, approval_job_id, with GitLab-enforced regexes), `workflow:rules`, include |
| | `.gitlab/actions/set-codeowners/action.json` | the catalog entry: description, inputs and their regexes, approval policy (`acme-w2/release`, direct members, requester may not approve), target |
| | `.gitlab/actions/set-codeowners/pipeline.yml` | the four jobs: `validate`, `preview`, `approve` (manual, environment `codeowners-approval`), `execute` (`main` only, environment `codeowners`, `resource_group`) |
| | `.gitlab/actions/set-codeowners/set_codeowners.py` | validation, the CODEOWNERS rewrite, preview, approval job, executor; `selftest` and `render` for local checks |
| `org` | `bin/acme-actions` | the org's one place: `list`, `preview`, `request`, `approve`, `status`, `record`, `runs`, `whoami`; JSON out |
| | `admin/setup.py` | applies the GitLab settings as code: bot token and protected variable, bot-owned triggers, `ci_pipeline_variables_minimum_override_role=no_one_allowed`; `--premium` applies the Premium ones |
| | `HANDOFF.md` | this file |

Checkouts are `/tmp/actions-gitlab/r0/gitlab-1/{alpha,beta,gamma,org}`.

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the project | Each project's definition is on its own `main`. The CLI discovers it there, and the pipeline runs it from there, so a project's maintainers change their action by changing these files. `main` is protected for Maintainers. The executor only accepts request pipelines whose commit is on `main`, so an edited copy on a branch cannot act. This was tested: carol's edited branch was refused even with dave's approval. | `<project>/.gitlab-ci.yml`, `<project>/.gitlab/actions/set-codeowners/*` |
| R2 Validated | Validation has four layers, each before anything changes. (1) The CLI checks the project's `action.json` regexes (exit 2, no branch, no pipeline). (2) GitLab refuses the pipeline: the `spec:inputs` `regex` gives HTTP 400 *"provided value does not match required RegEx pattern"*. (3) The `set-codeowners:validate` job refuses, then the run is `rejected`. (4) The executor validates again. Pipeline variables are disabled (`no_one_allowed`), so inputs are the only way in. | `.gitlab-ci.yml` `spec:inputs`, `set_codeowners.py` `validate_inputs`, `org/bin/acme-actions` `check_inputs` |
| R3 Previewed | `preview` is a dry-run pipeline: the project's own code renders the new `CODEOWNERS` against current `main` and returns a unified diff, plus the current and proposed text. Every request runs the same `preview` job before its approval job is unlocked. `request` prints that preview, and `status` and `record` keep it. The approver approves exactly the previewed inputs, because the executor takes them from the preview's artifact. | `set_codeowners.py` `cmd_preview`, artifact `acme-run/run.json` |
| R4 Approved by the release group | The approval is a play of the manual job `set-codeowners:approve`, and GitLab records the player. The executor refuses an approver who created the run (self-approval), or who is not a direct member of `acme-w2/release`, read live through `GET /groups/acme-w2%2Frelease/members`. The approval job also refuses self-approval before it starts anything. A refused attempt fails that job, the run stays `awaiting_approval` with `last_refusal`, and another person can approve: the CLI retries the job. Tested: ann (the requester) and carol (not in the group) were refused; dave and erin were accepted. | `set_codeowners.py` `cmd_approve`, `cmd_execute` steps 1, 2 and 5; `action.json` `approval` |
| R5 Done means observed | The executor commits through the API as the bot, then reads `CODEOWNERS` on `main` back and compares bytes. Only on a match does it record `succeeded`. Then the approval job reads `main` itself with its job token, and compares that against the executor's expected content. `status` reports `succeeded` only if the approval job succeeded, its executor job is a successful `set-codeowners:execute` on `main` for this run and this approval, and that record says `observed.matches: true`. On a protected `main`, GitLab refuses: with "No one" push and merge, it will not create the executor pipeline (HTTP 400); with "No one" push and merge for Maintainers, the commit gets HTTP 403. The run is then `failed`, with GitLab's message. Both cases were tested: run `alpha/57` and run `gamma/63`. | `set_codeowners.py` `cmd_execute` steps 6 and 7, `cmd_approve`; `acme-actions` `verify_execution` |
| R6 One place for the org | `org/bin/acme-actions` lists every action in every `acme-w2` project, and previews, requests, approves and audits runs on any of them, all from `org`. A project that adds `.gitlab/actions/<name>/action.json` appears without any change here. | `org/bin/acme-actions` |
| R7 Audited | `record` assembles the audit record from GitLab's own data, all timestamped: <br>• who requested and when (the pipeline's user and `created_at`) <br>• the inputs, and the preview with its time <br>• every approval attempt: who played the job, when, and whether it was approved, refused (with the reason) or failed <br>• the executor's record: bot, commit, and the group members it checked against <br>• the time of the observation on `main` <br>• the outcome and when it was reported <br>The records are pipelines, jobs, and artifacts kept with `expire_in: never`. There are also the deployments in the `codeowners` and `codeowners-approval` environments, and the commit trailers `Requested-by:` and `Approved-by:` on `main`. | `acme-run/{validation,run,result,record}.json` artifacts; `acme-actions record` |
| R8 Usable by agents | Every step is one non-interactive command with JSON on stdout. Exit codes: `0` ok; `1` failed or API error; `2` invalid inputs or usage; `3` approval refused; `4` still in progress with `--no-wait`. `status` carries a `next` hint naming the next command. The token comes from `--token-file` or `GITLAB_TOKEN`. Every step also has a REST or `glab` form (below). | `org/bin/acme-actions` |
| R9 The group is GitLab's | The approver set is not stored anywhere in this system. On each approval, the executor reads the direct members of the GitLab group named in `action.json` (`acme-w2/release`). Tested: removing dave from the group in GitLab made his approval refused for `beta/59`. Re-adding him made him an approver again, with no change to any file. | `set_codeowners.py` `cmd_execute` step 5 |

## Commands

The setup for every shell:

```bash
export HOME=/tmp/actions-gitlab/r0/gitlab-1/home
export PATH=/tmp/actions-gitlab/r0/gitlab-1/bin:$PATH
cd /tmp/actions-gitlab/r0/gitlab-1/org
git pull -q origin main
P=/tmp/actions-gitlab/r0/gitlab-1/people   # tokens: $P/<name>.token
```

**Start:** nothing; GitLab and its runner are all that run. Only after a fresh install, or to rotate the
trigger and bot tokens, re-apply the settings. It is idempotent:
`GITLAB_TOKEN=$(cat $P/ops.token) python3 admin/setup.py` (add `--premium` for the Premium settings).

| Step | Command |
|---|---|
| list | `bin/acme-actions list --token-file $P/ann.token` |
| preview (dry run) | `bin/acme-actions preview alpha set-codeowners --pattern '/src/' --owners '@acme/src' --token-file $P/ann.token` |
| request (as ann) | `bin/acme-actions request alpha set-codeowners --pattern '/src/' --owners '@acme/src @bob' --token-file $P/ann.token` (prints `"run": "alpha/<id>"`, the preview, `"state": "awaiting_approval"`) |
| approve (as dave or erin) | `bin/acme-actions approve alpha/<id> --token-file $P/dave.token` (waits; `"state": "succeeded"` and exit 0, or `this_approval.result: "refused"` and exit 3) |
| status | `bin/acme-actions status alpha/<id> --token-file $P/ann.token` (`--wait` blocks until settled) |
| record | `bin/acme-actions record alpha/<id> --token-file $P/ann.token` |
| all runs | `bin/acme-actions runs --token-file $P/ann.token` or `bin/acme-actions runs beta …` |

`--input pattern=… --input owners=…` is the generic form of `--pattern` and `--owners`. Timings here: preview
about 8 s, request about 8 s, approve about 10–20 s.

**The same steps in GitLab terms** (`glab` needs `glab config set --host 127.0.0.1:8929 api_protocol http`
once; then `export GITLAB_HOST=http://127.0.0.1:8929 GITLAB_TOKEN=$(cat $P/<name>.token)`):

```bash
glab api "projects/acme-w2%2Falpha/repository/files/.gitlab%2Factions%2Fset-codeowners%2Faction.json/raw?ref=main"   # list (one project)
glab api -X POST "projects/acme-w2%2Falpha/repository/branches?branch=actions/set-codeowners/request-x-ann&ref=main"   # request: branch
glab ci run -R acme-w2/alpha -b actions/set-codeowners/request-x-ann \
  --input action:set-codeowners --input mode:request --input 'pattern:/src/' --input 'owners:@acme/src'                # request: pipeline
glab api "projects/acme-w2%2Falpha/pipelines/<id>/jobs?include_retried=true"                                          # status (job user = approver)
glab api -X POST "projects/acme-w2%2Falpha/jobs/<approve-job-id>/play"                                                 # approve (or: glab ci trigger <job-id> -R acme-w2/alpha)
glab api -X POST "projects/acme-w2%2Falpha/jobs/<failed-approve-job-id>/retry"                                         # approve after a refused attempt
glab api "projects/acme-w2%2Falpha/jobs/<job-id>/artifacts/acme-run/record.json"                                       # record (executor job)
```

These go through the same jobs, so the same checks apply. Use `mode:preview` for a dry run.

## Test recipes

| Case | Do | Expect |
|---|---|---|
| invalid inputs | `bin/acme-actions request beta set-codeowners --pattern 'docs/' --owners '@a  @b' --token-file $P/ann.token`; also `--pattern '/a#b'`, `--owners '@a @b @c @d'`, a 101-character pattern, an `owners` value without `@` | exit 2, `"state": "rejected"`, errors listed, no pipeline. Bypassing the CLI with `POST /projects/:id/pipeline` and bad `inputs` gets HTTP 400 from GitLab. Passing `variables` gets *"Insufficient permissions to set pipeline variables"*. |
| self-approval | request as ann, then `approve … --token-file $P/ann.token`; or request as dave and approve as dave | exit 3, `this_approval.result: "refused"`, *"ann requested this run and cannot approve it"*; state stays `awaiting_approval` |
| approver outside the group | `approve … --token-file $P/carol.token` (or `bob`, `ops`) | exit 3, *"carol is not a direct member of acme-w2/release and cannot approve"* |
| membership change | `curl -s -H "PRIVATE-TOKEN: $(cat $P/ops.token)" -X POST http://127.0.0.1:8929/api/v4/groups/acme-w2%2Frelease/members -d user_id=4 -d access_level=30` (adds carol, id 4); or `-X DELETE …/groups/acme-w2%2Frelease/members/5` (removes dave, id 5) | carol's approval is now accepted; dave's is refused. Nothing in this system changes. |
| protected `main` | `curl -s -H "PRIVATE-TOKEN: $(cat $P/ops.token)" -X DELETE http://127.0.0.1:8929/api/v4/projects/acme-w2%2Fgamma/protected_branches/main` then `… -X POST "…/projects/acme-w2%2Fgamma/protected_branches?name=main&push_access_level=0&merge_access_level=0"`; request, then approve as erin | exit 1, `"state": "failed"`, reason *"could not start the executor on main: … do not have sufficient permission to run a pipeline on 'main'"*. `CODEOWNERS` is unchanged. Restore with `push_access_level=40&merge_access_level=40`. |
| edited definition on a run branch | push a commit to a run branch, then run a request pipeline on it and approve | refused: *"request pipeline ran commit …, which is not on main"* |
| unit checks | `python3 ../alpha/.gitlab/actions/set-codeowners/set_codeowners.py selftest`; `printf '* @x\n' \| python3 …/set_codeowners.py render '*' '@y'` | `selftest: ok` |

An approver needs the Developer role on the project to play the job. Members of `acme-w2` have it, and
`acme-w2/release` is also shared with each project as Developer, by `admin/setup.py --premium`. A person
given only Guest or Reporter in the release group cannot play the job at all.

My own test runs are kept as records. `alpha/56` is a preview. `alpha/57` holds the refusals of ann and
carol, then a failure on a "No one" `main`. `beta/59` holds dave's refusal while he was outside the
group. `alpha/61` is carol's edited branch, refused. `gamma/63` failed with HTTP 403 when push was "No
one". The runs with deleted branches show as `closed`.

## Premium mapping: what GitLab would enforce, and what ran here instead

`admin/setup.py --premium` applies the Premium settings through the API. On this CE instance it reports
them as unavailable: protected environments and approval rules answer 404, and `code_owner_approval_required`
is ignored. Sharing the release group with each project was applied.

| Premium feature (configured in `admin/setup.py`) | GitLab would | Here (CE) |
|---|---|---|
| Protected environment `codeowners-approval` (the `approve` job's environment) with `deploy_access_levels: [{group_id: acme-w2/release, group_inheritance_type: 0}]` | only direct release-group members could play the approval job | anyone with Developer can play it; the executor refuses non-members |
| Deployment approval rule on `codeowners-approval`: `approval_rules: [{group_id: release, required_approvals: 1}]`, `allow_pipeline_trigger_approve_deployment: false` | the job waits for an approval: `POST /projects/:id/deployments/:deployment_id/approval` with `{"status":"approved"}` as dave or erin. GitLab rejects the pipeline's triggerer (the requester) and anyone outside the group. | the play of the job is the approval; the job and the executor refuse self-approval and non-members |
| Protected environment `codeowners` with `deploy_access_levels: [{user_id: bot}]` | only the bot could run the executor job | the bot token is a protected variable, readable only on protected `main` |
| MR approval rule (release group, 1 approval), `merge_requests_author_approval: false`, `code_owner_approval_required` on `main` | hand edits to `CODEOWNERS` or to the action definition need a release-group approval that is not the author's | `main` push and merge stay with Maintainers only |

**For the three runs**, a Premium GitLab would have performed these steps:
* dave's approval (alpha) and erin's (beta, gamma), as deployment approvals on `codeowners-approval`;
* the refusal of ann, the requester, and of anyone outside `acme-w2/release`;
* restricting the approval job to release members.

The executor's own checks would remain as defence in depth. The steps GitLab CE performed itself:
* recording the requester and the approver, as the pipeline user and the job player;
* refusing invalid inputs (`spec:inputs` regex) and pipeline variables;
* withholding the bot token from unprotected refs;
* enforcing `main`'s protection on the commit;
* serializing executors per project (`resource_group`);
* the deployment records.

## What could not run here, and known limits

* None of the Premium enforcement above could run on this CE instance. It is configured as code and
  explained, but not exercised.
* The runner is a **shared shell executor**. Jobs from other projects or branches run as the same OS user
  on the same host, so job isolation (and with it the secrecy of the bot token while an executor job runs)
  is only as strong as that runner. In production, protected-ref jobs should go to a dedicated runner
  (`tags` plus a protected runner) or to the Docker or Kubernetes executor. I did not register runners,
  because runners are out of bounds.
* `ACME_ACTIONS_TRIGGER_TOKEN` is readable by Developers, since request branches are not protected. That
  is by design: it can only start the executor with typed inputs, and the executor re-verifies everything
  through the API. An abuser can at most re-apply an already approved run.
* The preview is computed against `main` at request time. The executor re-applies the same rule to `main`
  as it is at approval time, so a change in between applies the rule on top of it. The record keeps both
  the previewed diff and the applied diff.
* A project Maintainer can change their project's action, including `action.json`'s approval group. That is
  R1, the project owning its action. On Premium, the code owner and MR approval rule above would put such
  changes under the release group.
* The bot is a group access token with 360-day expiry. Rerun `admin/setup.py` to rotate it. Group access
  tokens create a GitLab bot user inside `acme-w2`; that is how GitLab implements them.
* Run branches (`actions/set-codeowners/request-*`) are left in place as the runs' refs. Preview branches
  are deleted after the preview. Deleting a run branch closes the run.
