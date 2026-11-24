# HANDOFF: the `mark-deprecated` self-service action

`mark-deprecated` is a GitHub Actions workflow that each repository owns. It is
committed to the `main` of `alpha`, `beta` and `gamma`. One CLI in this org
repository, `org/bin/acme-actions`, lists the action across all three
repositories, records runs, enforces the approval rule that act cannot enforce,
runs the workflow with act in host mode, and reports a run as `succeeded` only
after it has itself read `DEPRECATED.md` with the approved content on the
origin's `main`.

The three requested runs were made with it: ann requested each one and bob
approved each one. All three are `succeeded`:

| Repository | Run id | `main` after the run |
|---|---|---|
| alpha | `alpha-20261124T130937Z-695d53` | `e32d089` |
| beta | `beta-20261124T130945Z-7850e0` | `13e3ee3` |
| gamma | `gamma-20261124T130948Z-bd6c3f` | `52e2e4e` |

Paths below are relative to `/tmp/actions-ab/r5/gha-2`.

## What is where

| Path | Owner | What it is |
|---|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | repository | The workflow. It has three jobs: `validate`, then `preview`, then `apply` (environment `mark-deprecated`, only when `mode=apply`). |
| `<repo>/.github/self-service/mark-deprecated.json` | repository | The action's manifest: input rules, approvers, `preventSelfReview`, target file and branch. The CLI and the workflow both read it. |
| `<repo>/.github/self-service/mark_deprecated.py` | repository | The workflow's logic: `validate`, `preview`, `apply` (with an approval guard) and `verify`. Each prints JSON. |
| `<repo>/.github/self-service/apply-github-settings.sh` | repository | Applies, through the GitHub API, the settings act cannot run: the environment's reviewers and self-review rule, and a ruleset on `main`. |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | repository | Make ann and bob the code owners of the workflow, `.github/self-service/` and the CODEOWNERS file. alpha already had a root `CODEOWNERS`, which GitHub reads before `.github/`, so its rules were appended there. |
| `org/bin/acme-actions` | org | The CLI (Python 3, standard library only). |
| `org/catalog.json` | org | The org's repositories and their origins, the people (ann, bob, carol), and the act path and platform. |
| `org/state/` (git-ignored) | runtime | `runs/<id>/record.json` holds each run's audit record. Next to it are `act-preview.log`, `act-apply.log`, and `results-*/<job>.json` (the workflow's own JSON output). `refusals.jsonl` logs refused requests. `mirrors/` holds fetched copies of each origin's `main`. |

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's workflow, manifest and logic are committed to its own origin's `main`, in the commit "Add self-service action mark-deprecated" (alpha `d08cc82`, beta `cc8e41a`, gamma `605e0bd`). CODEOWNERS makes ann and bob own them. The CLI hard-codes nothing about the action. It finds actions by reading `.github/self-service/*.json` from each origin's `main`, runs the workflow those files name, and pins a run to the `main` commit that was current when the run was requested (`definition.sha` in the record). A team that edits its manifest or workflow on `main` changes the action. On GitHub, the ruleset written by `apply-github-settings.sh` requires a code-owner review for those edits. | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/*`, the CODEOWNERS files above |
| R2 Validated | The `validate` job runs `mark_deprecated.py validate` against the manifest's rules. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself. `reason` must be 20 to 200 characters. Every input is required, and control characters such as newlines are refused so that the file keeps its exact format. `preview` and `apply` both `need` `validate`, and `apply` validates again itself. Before running act, the CLI also refuses inputs that the manifest does not declare. A refused request exits 2, records no run, changes nothing, and is logged in `org/state/refusals.jsonl`. | `<repo>/.github/self-service/mark_deprecated.py` (`validation_errors`), `mark-deprecated.json` (`inputs`), `org/bin/acme-actions` (`preview_run`) |
| R3 Previewed | The `preview` job fetches the origin's `main` and outputs the current content, the proposed content, `change` (`create`, `update` or `none`) and a unified diff. On GitHub it writes these to the run summary. `acme-actions preview` shows the preview without recording anything. `acme-actions request` stores the preview in the run record, so the approver reads the same diff before approving. `succeeded` requires that `main` holds exactly the previewed `proposed` content. | `mark_deprecated.py` (`cmd_preview`), `org/bin/acme-actions` (`cmd_preview`, `cmd_request`) |
| R4 Approved | **Here:** a request runs the workflow with `mode=preview`, which never pushes. The `apply` job runs only when `acme-actions approve` succeeds. That command refuses a person who is not in the manifest's `approval.approvers` (carol is refused with `not_an_approver`) and refuses the requester (`self_approval`), exiting 3. Refused attempts are recorded as `approval_refused` events. A lock on each run stops two approvals of one run from both executing. `mark_deprecated.py apply` also refuses to push under act unless the CLI passes it an approver who is listed and is not the requester. A direct `act ... mode=apply` therefore fails with `no approval recorded`. **On GitHub:** `apply` uses `environment: mark-deprecated`. `apply-github-settings.sh` sets that environment's required reviewers to `approval.approvers`, turns on `prevent_self_review`, and limits deployments to `main`. GitHub then holds the job until a reviewer other than the person who triggered it approves. The push credential, `MARK_DEPRECATED_PUSH_TOKEN`, is meant to be an environment secret, so it does not exist before approval. | `mark-deprecated.json` (`approval`), `.github/workflows/mark-deprecated.yml` (`apply.environment`), `apply-github-settings.sh`, `mark_deprecated.py` (`check_approval`), `org/bin/acme-actions` (`cmd_approve`) |
| R5 Done means observed | `apply` commits and pushes. It retries only when `main` moved underneath it and fails at once on a rejection. `verify` then fetches `main` again and compares the file byte for byte. Afterwards the CLI fetches the origin's `main` itself and compares `DEPRECATED.md` to the approved preview. The run is `succeeded` only if the workflow exited 0, `apply` reported ok, and the CLI's own observation matches. Otherwise it is `failed`, with `outcome.reasons`. Two cases were tested: a `pre-receive` hook that rejects the push, and a `post-receive` hook that accepts the push and then resets `main`. Both ended `failed` and left `main` unchanged. | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `org/bin/acme-actions` (`cmd_approve`, `observe`) |
| R6 One place for the org | `org/bin/acme-actions` reads `org/catalog.json`, then lists, previews, requests, approves and inspects runs in any repository, all from `org/`. It reads every definition from the origin, not from the local checkouts. | `org/bin/acme-actions`, `org/catalog.json` |
| R7 Audited | `org/state/runs/<id>/record.json` records `requested_by` and `requested_at`, `approved_by` and `approved_at`, the `inputs`, the `preview`, the `definition` (origin, commit, workflow) and the `outcome` (status, time, observed `main` commit and content, pushed commit, workflow exit code, reasons). It also holds a timestamped `events` list: `requested`, `validated`, `previewed`, `approval_refused`, `approved`, `applied` or `apply_failed`, `workflow_verified`, `observed`, then `succeeded` or `failed` (or `rejected`). The act logs are kept next to the record. The commit on `main` carries `Run:`, `Requested-by:` and `Approved-by:` trailers, with the requester as author and `acme-actions[bot]` as committer. On GitHub, the run history and the environment's deployment history record the same information. | `org/bin/acme-actions` (`cmd_request`, `cmd_approve`), `org/state/runs/*/record.json` |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document on stdout, including usage errors. Identity comes from `--as <user>` or `ACME_USER`. Exit codes: 0 ok or succeeded; 1 run failed; 2 invalid inputs or usage; 3 not allowed (unknown person, not an approver, self-approval); 4 not found or wrong state (for example, approving a run twice); 5 the runner failed. | `org/bin/acme-actions` |

## Commands

Nothing needs to be running. There is no server, and each command runs act in
the foreground. The commands need `python3`, `git` and `bin/act`. `jq` is
optional, for reading the output.

```bash
cd /tmp/actions-ab/r5/gha-2/org
A=bin/acme-actions

# List mark-deprecated in all three repositories (read from each origin's main)
$A list --action mark-deprecated
$A list --repo beta

# Preview, without recording a run
$A preview --repo alpha --as ann -f successor=acme/dependabot-proxy-v2 \
  -f 'reason=Replaced by the v2 proxy, which supports OIDC registries.'

# Request as ann: validate, preview and record a run awaiting approval; prints the run id
RUN=$($A request --repo alpha --as ann -f successor=acme/dependabot-proxy-v2 \
  -f 'reason=Replaced by the v2 proxy, which supports OIDC registries.' | jq -r .run.id)

# Read the preview before approving
$A record "$RUN" | jq .record.preview

# Approve as bob: runs the apply job, observes main, exits 0 only if succeeded
$A approve "$RUN" --as bob --comment "ok"

# Reject instead (an approver or the requester)
$A reject "$RUN" --as bob --comment "not yet"

# Status, full record, all runs
$A status "$RUN"
$A record "$RUN"
$A runs --repo alpha --status succeeded
```

### Testing the refusals

```bash
# Invalid inputs: exit 2, no run recorded, nothing changed
$A request --repo beta --as ann -f successor=Acme/Store -f reason=short
$A request --repo beta --as ann -f successor=acme/beta -f 'reason=Pointing at itself is refused.'

# Self-approval and carol: exit 3, recorded as approval_refused events
RUN=$($A request --repo beta --as ann -f successor=acme/valtio-store \
  -f 'reason=Testing the approval rules on beta.' | jq -r .run.id)
$A approve "$RUN" --as ann     # self_approval
$A approve "$RUN" --as carol   # not_an_approver

# Rejected push: exit 1, status failed, main unchanged
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
$A approve "$RUN" --as bob
$A status "$RUN" | jq '.run.status, .run.outcome.reasons'
rm ../origins/beta.git/hooks/pre-receive
```

To test a rejected push, use inputs that differ from what is already on `main`,
as above. The action is idempotent: if `main` already has exactly the requested
`DEPRECATED.md`, `apply` reports `change: none`, pushes nothing, and the run
succeeds because the desired state is observed. With the three runs above done,
re-requesting the same inputs therefore never reaches the push hook.

The CLI keeps state in `org/state` by default. To test without touching the
real audit records, point `ACME_ACTIONS_STATE` at another directory. Runs still
change the real origins.

## How GitHub would enforce what act cannot run

| Feature | Configured as code in | How GitHub enforces it |
|---|---|---|
| Approval by a maintainer | `apply` has `environment: mark-deprecated`. `apply-github-settings.sh` makes `PUT /repos/{repo}/environments/mark-deprecated` with `reviewers` taken from `approval.approvers`. | The `apply` job waits until one required reviewer approves it in the run's "Review deployments" dialog. |
| No self-approval | `prevent_self_review: true`, from `approval.preventSelfReview`, in the same call. | The person who triggered the run cannot approve it. |
| Only from `main` | `deployment_branch_policy` with a custom policy for `main` (`POST .../deployment-branch-policies`). | A run dispatched from another branch cannot deploy to the environment. |
| Push credential only after approval | `secrets.MARK_DEPRECATED_PUSH_TOKEN`, created as an environment secret. | Environment secrets are only released to a job after its approval. |
| Branch protection | A ruleset named `main` (`POST /repos/{repo}/rulesets`) requires a pull request with a code-owner review and blocks force pushes and deletion. When `MARK_DEPRECATED_APP_ID` is set, the GitHub App behind the push token is the only bypass actor. | People change `main`, including the action's own definition, only through reviewed pull requests. Only an approved run pushes `DEPRECATED.md` directly. |
| Run history | Built into GitHub. | Every dispatch, approval and job log appears under the repository's Actions tab, and the environment keeps a deployment history. |

On GitHub, the org-level commands map to `gh`:

- **list:** `gh api repos/acme/<repo>/contents/.github/self-service/mark-deprecated.json`
- **request:** `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=... -f reason=... -f mode=apply`
- **approve:** `gh api repos/acme/<repo>/actions/runs/<id>/pending_deployments -f state=approved ...`
- **status and record:** `gh run view <id> -R acme/<repo> --json status,conclusion,jobs`

The workflow already handles both sides. It defaults `ORIGIN_URL` to
`https://github.com/<repo>.git` and authenticates with `GH_TOKEN`. On GitHub it
reads the approver from the run's approvals API (`github_approver`).

## What I could not run here

- The GitHub API calls in `apply-github-settings.sh` (environments, reviewers, `prevent_self_review`, rulesets). There is no GitHub here. `apply-github-settings.sh --dry-run` prints the requests, and their JSON was checked, but GitHub never received them.
- GitHub's own environment gate, the release of environment secrets, ruleset and CODEOWNERS enforcement, the run history, and the run-approvals API used by `github_approver`. Here, `acme-actions` and the guard in `mark_deprecated.py` enforce the approval, and `org/state/runs/` is the run history.
- The `https://github.com` checkout and push path, which authenticates with `GH_TOKEN`. Every run here used the local bare origins through `--var ORIGIN_URL=<path>`.
- Identity is self-asserted (`--as`), as this sandbox requires. On GitHub, the triggering actor and the environment reviewer come from GitHub sign-in. Here, anyone with shell access could also edit `org/state` or run act with a forged `ACME_APPROVED_BY`. GitHub's environment gate is what makes the approval binding.
- act ignores `concurrency:`. Concurrent pushes are handled instead: `apply` retries when `main` has moved.
- The workflow uses no `uses:` steps and checks itself out with `git fetch`. In host mode act starts each job in an empty workspace, and it downloads every referenced action from github.com, even for skipped steps. With no `uses:` steps, local runs are offline and need no Docker.

## Verification done

- `act -l --strict` parses the workflow.
- `apply-github-settings.sh --dry-run` emits valid request bodies.
- **Validation boundaries:** a reason of 19 characters is refused, 20 and 200 are accepted, and 201 is refused. A newline is refused. Uppercase in `successor`, a missing input, an unknown input and the repository itself as successor are all refused.
- **Approval refusals:** ann approving her own run and carol approving are both refused. So are an unknown person, a second approval of a finished run, and two approvals racing on one run (one runs, the other gets `wrong_state`).
- **Rejected push** (`pre-receive` hook): the run is `failed`, with the reason `push to main was rejected`.
- **Push accepted then undone** (`post-receive` hook resets `main`): the run is `failed`. The workflow's `verify` step and the CLI's own observation both flag the mismatch.
- **A direct act run with `mode=apply` and no approval:** refused, with `no approval recorded`.
- **The three real runs:** all `succeeded`. `DEPRECATED.md` on each origin's `main` matches the required content byte for byte, with a trailing newline.

These tests ran against a scratch state directory, which was deleted afterwards.
One test approval on gamma ran without a reject hook in place and pushed a test
`DEPRECATED.md`. gamma's `main` was reset to the definition commit `605e0bd`
before the real run.
