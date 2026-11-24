# Handoff: the `add-notice` self-service action

`add-notice` is a GitHub Actions workflow that each repository defines and owns. It runs
locally with act in host mode and is driven from this org repository by `bin/acme-actions`.
On GitHub the same workflow is gated by a `release` environment. Here, `acme-actions`
enforces that environment's rules, read from the org's config files, before it lets act
run the gated job.

The three requested runs succeeded:

| Run | Repository | Requested by | Approved by | Outcome | Commit on `main` |
|---|---|---|---|---|---|
| `alpha-1` | alpha (`README.md`) | ann | dave | succeeded | `d235237` |
| `beta-1` | beta (`readme.md`) | ann | erin | succeeded | `6c47586` |
| `gamma-1` | gamma (`README.md`) | ann | erin | succeeded | `ef27149` |

Each commit changes only the README (two added lines), is authored by `add-notice[bot]`,
and carries the trailers `Add-Notice-Run`, `Requested-By` and `Approved-By`.

## What is where

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/add-notice.yml` | The action: `workflow_dispatch` inputs, a `plan` job (validate, preview) and an `apply` job in environment `release` (push, then read `main` back) |
| `<repo>/.github/add-notice/add_notice.py` | The logic: validate, plan (diff), apply, verify. Python standard library only |
| `<repo>/.github/add-notice/README.md` | The action's documentation, for the repository's team |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | The action is owned by `@acme/maintainers`, the README by `@acme/release` |
| `org/teams.yaml` | Org members and teams. **The only definition of the release group** |
| `org/environments.yaml` | The `release` environment's rules: reviewers (team `release`), `prevent_self_review`, branch `main`, the deploy-key secret |
| `org/repos.yaml` | The catalog (alpha, beta, gamma; their origins and GitHub names), the offered actions, repository access and the `main` ruleset |
| `org/bin/acme-actions` | The org-level command: list, preview, request, approve, reject, cancel, status, record, runs, group |
| `org/github/apply-settings.py` | Applies teams, environments, rulesets and deploy keys to GitHub through the REST API (`--dry-run` is the default) |
| `org/.github/workflows/apply-settings.yml` | On GitHub, runs `apply-settings.py --apply` when those files change on `main` (dry-run on pull requests) |
| `org/tests/e2e.sh` | End-to-end test of every rule below, on throwaway copies of the origins (45 checks) |
| `org/.state/` (git-ignored) | Run records (`runs/<id>.json`), act logs per run (`logs/<id>/{preview,apply}.jsonl`), act's cache |

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is committed to its origin's `main`: the workflow, its script and its doc, with CODEOWNERS naming `@acme/maintainers`. `acme-actions` runs the definition exactly as it is on `main` when the run is requested (the commit is pinned in the record as `definition_commit`), and the workflow runs the script from that commit (`git show $GITHUB_SHA:...`). The org repository only lists where to look. Its rules about *who approves* are not part of the action | `<repo>/.github/workflows/add-notice.yml`, `<repo>/.github/add-notice/`, CODEOWNERS, `org/repos.yaml` |
| R2 Validated | Validation happens in two places. (1) At dispatch, `acme-actions` checks input names, required inputs and `choice` options against the workflow's `inputs:`, as GitHub's dispatch API does. (2) The `plan` job's first step after fetching, `Validate inputs`, enforces `level` ∈ {info, warning} and that `message` is one line (no control or line-separator characters), 10 to 160 characters long and not blank. The `apply` job validates again. An invalid request is recorded with status `invalid`, exits 2, and never reaches a job that writes | `add_notice.py` (`validate`), `add-notice.yml` (`plan` job), `acme-actions` (`check_inputs`) |
| R3 Previewed | The `plan` job writes the unified diff to its log and its step summary, plus a JSON report as the job output `report`. `acme-actions preview` shows it without starting a run. `acme-actions request` shows it and stores it in the record (`preview.diff`) while the run waits for approval. When the run is approved, the job first checks that the README blob on `main` is still the one previewed; otherwise the run fails ("changed on main since the preview") and nothing is pushed | `add-notice.yml` (`Preview` and `Check the README is still the one previewed` steps), `acme-actions` (`preview`, `request`, `apply`) |
| R4 Approved by the release group | The `apply` job declares `environment: release`. `environments.yaml` gives that environment the reviewers `team: release` and `prevent_self_review: true`. `acme-actions approve` reads both files from the org origin's `main` at approval time. It refuses (exit 3, and records an `approve_refused` event) anyone outside the team and the requester, even when the requester is in the team. Only after that does it run the `apply` job. `reject` uses the same rules | `org/environments.yaml`, `org/teams.yaml`, `acme-actions` (`check_reviewer`), `add-notice.yml` (`environment: release`) |
| R5 Done means observed | The success path is: push, then the `Verify main` step fetches `main` from the origin and fails the job unless the README's SHA-256 equals the previewed `expected_sha256`. After the job, `acme-actions` reads `main` on the origin itself (event `observed`). The run is `succeeded` only if the job succeeded **and** that observation matches. A rejected push fails the `Apply and push` step, so the run is `failed` (exit 4). An accepted push that does not land fails both checks | `add-notice.yml` (`Apply and push`, `Verify main`), `add_notice.py` (`verify`), `acme-actions` (`apply`, `readme_on_main`) |
| R6 One place for the org | `org/repos.yaml` catalogs the repositories and the actions. From `org/`, `bin/acme-actions list` shows `add-notice` for all three: its inputs, its definition commit and who may approve. `preview`, `request` and the rest take `--repo alpha\|beta\|gamma`. Nobody needs to enter a repository | `org/repos.yaml`, `org/bin/acme-actions` |
| R7 Audited | Each run has a record, `org/.state/runs/<id>.json`, with `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `outcome`/`outcome_at`/`outcome_detail`, the preview, the result (commit, `main` SHA, observed hash) and an append-only `events` list. Every event carries a UTC timestamp and an actor: requested, previewed, waiting, every refused approval, approved (with the team membership and the config commits used), job_started, job_finished, observed, succeeded/failed. The act logs are kept beside it. On `main`, the commit trailers repeat the run id, requester and approver | `acme-actions` (`event`, `summary`), `org/.state/` |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document. Identity comes from `--as USER` or `$ACME_USER`, inputs from `-f name=value` (as in `gh workflow run`). Exit codes: 0 done, 2 invalid input, 3 not allowed, 4 run failed, 5 no such run/repository or wrong state, 1 other error | `org/bin/acme-actions` |
| R9 The group is defined once | The release group exists only in `org/teams.yaml` (`teams.release.members`). The repositories name the environment, `environments.yaml` names the team, and nothing else lists people. `acme-actions` reads `teams.yaml` from the org origin's `main` on every approval and records the commit it used, so a change pushed there applies to all three repositories at once, including runs already waiting. On GitHub, `apply-settings.yml` syncs the file to the `release` team, and every repository's `release` environment names that team | `org/teams.yaml`, `org/environments.yaml`, `org/github/apply-settings.py`, `org/.github/workflows/apply-settings.yml` |

## Commands

Nothing needs to be running: there is no server. Each command runs act itself (in host
mode, `-P ubuntu-latest=-self-hosted`, binary `/tmp/actions-ab/r8/gha-1/bin/act`) and returns
when the run step is done. Prerequisites are Python 3 with PyYAML, git and jq, all present
here.

```bash
cd /tmp/actions-ab/r8/gha-1/org
A=bin/acme-actions          # or: export ACME_USER=ann and drop --as
```

**List** the actions in all three repositories:

```bash
$A list                     # one repository: $A list --repo beta
```

**Preview** without starting a run. Nothing is recorded or changed:

```bash
$A preview --repo alpha -f level=warning -f "message=This proxy is frozen while the v2 rewrite lands; please file new issues against v2."
```

**Request** as ann. The result has the run id, `status: waiting`, `waiting_for` and the diff:

```bash
RUN=$($A request --repo alpha --as ann -f level=warning -f "message=Some notice of 10 to 160 characters." | tee /dev/stderr | jq -r .run.id)
```

**Approve** as dave or erin. This runs the `apply` job and returns the outcome. Exit 0 means
the run succeeded, exit 4 means it failed:

```bash
$A approve "$RUN" --as dave --comment "ok"
$A reject  "$RUN" --as erin --comment "not now"   # turn it down instead
$A cancel  "$RUN" --as ann                         # the requester withdraws it
```

**Status** and **record**:

```bash
$A status "$RUN"            # brief: status, who, when, outcome
$A record "$RUN"            # the full audit record with all events
$A runs                     # every run, newest first; filter with --repo R --status S
```

**Change who is in the release group.** Edit `teams.yaml` (`teams` → `release` → `members`),
commit, and push to the org origin's `main`. It takes effect on the next approval, for every
repository. Only what is on the origin's `main` counts, so an unpushed edit changes nothing.

```bash
cd /tmp/actions-ab/r8/gha-1/org
git pull --rebase origin main
python3 -c 'import yaml; t=yaml.safe_load(open("teams.yaml")); t["teams"]["release"]["members"]=["dave","carol"]; yaml.safe_dump(t, open("teams.yaml","w"), sort_keys=False)'
git commit -am "release group: dave, carol" && git push origin main
bin/acme-actions group release        # shows the members and the teams.yaml commit now in force
```

To restore it, set `["dave","erin"]` and push again (or `git revert --no-edit HEAD && git push origin main`).
`yaml.safe_dump` drops the comments; editing the file by hand keeps them.

### Test scenarios and what to expect

| Scenario | Command | Expected |
|---|---|---|
| Invalid level | `$A request --repo alpha --as ann -f level=error -f "message=Ten or more characters"` | exit 2, `status: invalid`, `main` unchanged |
| Message too short or too long | `-f message=short`, or 161 characters | exit 2, `invalid` (the workflow's `Validate inputs` step fails) |
| Two lines | `-f "message=$(printf 'line one is long\nline two')"` | exit 2, `invalid` |
| Unknown or missing input | `-f colour=red`, or no `-f message` | exit 2, `invalid` |
| Self-approval | See below | exit 3 (prevent_self_review); erin can then approve |
| Approval from outside the group | `$A approve $RUN --as ann`, `--as bob`, `--as carol`, `--as mallory` | exit 3 each time, recorded as `approve_refused`; the run keeps waiting |
| Group change | Push carol in and erin out as above, then `$A approve $RUN --as erin` / `--as carol` | erin refused (exit 3), carol succeeds |
| Rejected push | See below | exit 4, `status: failed`, `main` unchanged |

Self-approval, by a requester who is in the release group:

```bash
RUN=$($A request --repo beta --as dave -f level=info -f "message=Requested by a release member." | jq -r .run.id)
$A approve "$RUN" --as dave; echo "exit $?"    # exit 3: dave requested it
$A approve "$RUN" --as erin                     # succeeds
```

Rejected push. The run must not succeed, because the change never reaches `main`:

```bash
cd /tmp/actions-ab/r8/gha-1/org
RUN=$(bin/acme-actions request --repo gamma --as ann -f level=warning -f "message=This change is not on main yet." | jq -r .run.id)
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
bin/acme-actions approve "$RUN" --as dave; echo "exit $?"   # exit 4, status failed, outcome_detail quotes git's rejection
bin/acme-actions record "$RUN" | jq '.record.result'          # observed_on_main: false
rm ../origins/gamma.git/hooks/pre-receive
```

The hook can also be installed before the request: the preview only reads. A run whose
README already has exactly the requested notice pushes nothing and succeeds, because the
README on `main` is already right. A failed run is final; request a new one.

**The whole suite.** It runs on copies of the four origins under `org/.state/e2e`, so the
real origins and run records are not touched:

```bash
/tmp/actions-ab/r8/gha-1/org/tests/e2e.sh     # "passed 45, failed 0"
```

To try things by hand without touching the real origins, point the tool at that sandbox:
`export ACME_ORG_DIR=/tmp/actions-ab/r8/gha-1/org/.state/e2e/org ACME_STATE_DIR=/tmp/actions-ab/r8/gha-1/org/.state/e2e/state`.

## How GitHub enforces what act cannot

`org/github/apply-settings.py` (run by `org/.github/workflows/apply-settings.yml` with an
org-owner token in `ACME_ORG_ADMIN_TOKEN`) configures the following through the REST API.
Its dry run prints every call: `python3 github/apply-settings.py --dry-run`.

| Feature | Configuration as code | How GitHub enforces it |
|---|---|---|
| Release team | `teams.yaml` → `PUT /orgs/acme/teams/release/memberships/{user}`, and `DELETE` for anyone no longer listed | Team membership is the one list GitHub checks |
| Required reviewers, no self-approval | `environments.yaml` → `PUT /repos/acme/{repo}/environments/release` with `reviewers: [{type: Team, id: <release>}]`, `prevent_self_review: true` | The `apply` job waits until a release member approves. The run's triggering actor cannot approve it, and nobody outside the team can |
| Only `main` may deploy | `deployment_branch_policy.custom_branch_policies: true` + `POST .../deployment-branch-policies {name: main}` | A dispatch from another branch, carrying a modified workflow, cannot enter the environment |
| Only an approved job can push | `repos.yaml` `main_ruleset` → `POST /repos/acme/{repo}/rulesets`: pull request + code-owner review, no force push or deletion; bypass only `DeployKey`. A write deploy key per repository, whose private half is stored only as the `release` environment secret `ADD_NOTICE_DEPLOY_KEY` | People, and a workflow that dropped `environment: release`, cannot push to `main`. The key reaches the job only after approval. A rejected push fails `Apply and push` |
| Who may request | Org members in `teams.yaml`, base permission `write` (dispatch needs write) | `workflow_dispatch` is refused for non-members. Write does not allow pushing to `main` because of the ruleset |
| Code owners | CODEOWNERS in each repository | Changes to the action need a maintainer's review, and README changes need release's |
| Run history | Built in | Runs, logs, job summaries (the preview diff) and the review history are kept by GitHub |

The same steps with `gh` on GitHub (run as the person named):

```bash
for r in alpha beta gamma; do gh workflow view add-notice.yml -R acme/$r --yaml | head -25; done     # list
gh workflow run add-notice.yml -R acme/alpha -f level=warning -f message='...'                     # request (as ann)
RUN_ID=$(gh run list -R acme/alpha -w add-notice.yml -L 1 --json databaseId -q '.[0].databaseId')
gh run view $RUN_ID -R acme/alpha --log | sed -n '/add-notice preview/,$p'                          # preview (also the run's summary page)
gh api repos/acme/alpha/actions/runs/$RUN_ID/pending_deployments                                    # waiting; current_user_can_approve
ENV_ID=$(gh api repos/acme/alpha/environments/release -q .id)
gh api -X POST repos/acme/alpha/actions/runs/$RUN_ID/pending_deployments \
  -F "environment_ids[]=$ENV_ID" -f state=approved -f comment='ok'                                   # approve (as dave); refused for ann, carol, or the requester
gh run view $RUN_ID -R acme/alpha --json status,conclusion,jobs                                     # status
gh api repos/acme/alpha/actions/runs/$RUN_ID -q '{triggering_actor: .triggering_actor.login, created_at, updated_at, conclusion}'
gh api repos/acme/alpha/actions/runs/$RUN_ID/approvals                                              # record: who approved, when, comment
gh api -X PUT orgs/acme/teams/release/memberships/carol                                             # group change; normally a PR to teams.yaml
```

### Which steps GitHub would have performed in the three runs

| Step | On GitHub | Here |
|---|---|---|
| Dispatch, run creation, choice validation | `POST .../dispatches` (`gh workflow run`) | `acme-actions request` checks the inputs, then `act -j plan` |
| Holding `apply` for the `release` environment; checking reviewer and self-review | Environment protection rules | `acme-actions approve`, from `environments.yaml` + `teams.yaml` on the org's `main` |
| Releasing the push credential to the approved job; letting only it push | Environment secret + ruleset `DeployKey` bypass | Not emulated: the origins are local paths and accept pushes |
| Who approved, inside the job | `GET .../runs/{id}/approvals` (in `Apply and push`) | `ADD_NOTICE_APPROVED_BY`, passed by `acme-actions` |
| Run id | `github.run_id` | `ADD_NOTICE_RUN_ID` (`alpha-1`, …) |
| Run history and review history | Kept by GitHub | `org/.state/runs/*.json` and `org/.state/logs/` |

The `plan` job, the validation, the diff, the push to `main` and both reads of `main` ran
for real, under act, in all three runs.

## What could not run here

- **No GitHub enforcement was exercised**: environments, required reviewers,
  `prevent_self_review`, branch policies, rulesets, deploy keys, teams and code owners.
  `acme-actions` enforces the reviewer, self-review and main-only rules locally, from the same
  files that `apply-settings.py` sends to GitHub. `apply-settings.py --apply` and the GitHub
  branches of the workflow have not been run against GitHub: the `https` clone with
  `GITHUB_TOKEN`, the SSH clone and push with the deploy key, and the approvals lookup with
  `gh`. `apply-settings.py` was run in dry-run, and `apply-settings.yml` was run under act
  (dry-run).
- **Identity is a claim.** `--as` is trusted, as this sandbox has no passwords. On GitHub,
  the token's user is the identity.
- **The gate can be bypassed locally.** Anyone with filesystem access can push to an
  origin, or run act on the workflow directly. On GitHub, the ruleset and the environment
  secret close that path.
- **act runs `plan` again** when it runs `apply`, because `-j apply` includes the jobs it
  needs. On GitHub both jobs belong to one run. Locally, `acme-actions` checks before `apply`
  that the README is still the one previewed, and judges the result against the original
  preview's expected content.
- **The records are local and not tamper-evident** (`org/.state/`, git-ignored). GitHub's
  run history plays that role there.
