# Handoff: `mark-deprecated` on GitHub Actions

`mark-deprecated` is a GitHub Actions workflow that each repository owns and commits to its own
`main`. `org/bin/acme` is the org's single entry point. It lists the action across `alpha`, `beta`
and `gamma`, previews and requests runs, enforces the approval rules, runs the workflow locally
with act (host mode), and reports a run as succeeded only after it has itself read the right
`DEPRECATED.md` on the origin's `main`. Every command prints JSON.

The three requested runs have succeeded (`run-0001` alpha, `run-0002` beta, `run-0003` gamma).
In each, ann requested the run and bob approved it, and `DEPRECATED.md` is on each origin's
`main` in a commit by `github-actions[bot]` with `Requested-by: ann` and `Approved-by: bob`
trailers.

Nothing needs to be running: there is no server or daemon. Each command runs to completion,
including the act run that an approval triggers.

## Files

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: a `workflow_dispatch` workflow. It has a `preview` job (validate, show the diff) and an `apply` job (on the `mark-deprecated` environment: check the approval, commit and push, then verify on `main`) |
| `<repo>/.github/self-service/mark-deprecated.json` | The approval policy as code: environment `mark-deprecated`, reviewers `ann` and `bob`, `prevent_self_review: true`, no admin bypass, deployments only from `main`. It also lets `acme` discover the action |
| `<repo>/.github/self-service/mark_deprecated.py` | The workflow's steps: `preview`, `check-approval`, `apply`, `verify` (Python standard library only) |
| `<repo>/.github/self-service/apply-github-settings.sh` | Applies the policy to GitHub through its API: the environment (required reviewers, prevent self-review, branch policy) and a ruleset on `main` |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | ann and bob own the workflow and `.github/self-service/` |
| `org/bin/acme` | The org CLI (Python 3 with PyYAML, both present here) |
| `org/catalog.json` | The org: people (`ann`, `bob`, `carol`), the repositories and their origins, the path to act |
| `org/tests/selftest.sh` | 32 checks of every rule, run against throwaway copies of the origins |
| `org/state/` (git-ignored) | Run records `state/runs/run-NNNN.json`, act logs `state/runs/run-NNNN/{preview,apply}.log` and step results `state/runs/run-NNNN/{preview,apply}/*.json` |

`<repo>` is `alpha`, `beta` or `gamma`. All three carry identical copies, and each team can change
its own.

## Commands

Run these from any directory. `ACME_USER=<person>` can replace `--as <person>`.

```bash
A=/tmp/actions-ab/r3/gha-1/org/bin/acme

# Start: nothing to start.

# List the actions in all three repositories (read from each origin's main)
$A list
$A list --repo beta

# Preview without requesting (validates and shows the diff; changes nothing, keeps no record)
$A preview --repo alpha -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann: validates, runs the workflow's preview job and returns the diff and the run id.
# Nothing changes until approval.
$A request --as ann --repo alpha -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
#   -> {"ok": true, "id": "run-0004", "status": "awaiting_approval", "preview": {"diff": ...}, ...}

# Approve as bob: checks the rules, runs the workflow with act, observes main, then reports
$A approve run-0004 --as bob --comment "ok"

# Reject (a reviewer or the requester)
$A reject run-0004 --as bob --comment "not yet"

# Check a run's status, read its full record, list runs
$A status run-0001
$A record run-0001
$A runs --repo gamma --status succeeded

# Self-test every rule on throwaway copies of the origins (the real origins are not touched)
/tmp/actions-ab/r3/gha-1/org/tests/selftest.sh
```

Inputs can also be passed as `--inputs-json '{"successor": "...", "reason": "..."}'`. `--action`
defaults to `mark-deprecated`.

Exit codes: `0` ok; `1` the run failed (for example, the push was rejected) or an internal error;
`2` refused (invalid input, self-approval, not a reviewer, wrong state, unknown person, usage);
`3` not found. A refusal prints `{"ok": false, "error": {"code": ..., "message": ...}}`. Run
commands print the run summary, with `ok`, `status`, `outcome.reason` and `errors`.

Run statuses: `refused` (invalid inputs; terminal, recorded), `awaiting_approval`, `rejected`,
`running`, `succeeded`, `failed`.

### Testing the rules by hand

- **Invalid inputs.** For example `-i successor=Acme/x`, `-i successor=acme`, `-i successor=acme/alpha`
  on alpha (its own name), a 19- or 201-character reason, a reason with a newline, a missing input,
  or an undeclared input such as `-i branch=dev`. Each is refused with exit `2`, status `refused` and
  per-input `errors`, and `main` is unchanged. Requesting content identical to what is already on
  `main` is refused with `no_change`. All three repositories now have `DEPRECATED.md`, so a new run
  needs a different successor or reason.
- **Self-approval.** `$A approve <id> --as ann` on a run ann requested gives exit `2`, `self_review`.
- **carol.** `$A approve <id> --as carol` gives exit `2`, `not_a_reviewer`. carol may *request*, and
  then ann or bob approves.
- **Rejected push.** Request a run, then make the origin refuse pushes and approve:

  ```bash
  H=/tmp/actions-ab/r3/gha-1/origins/beta.git/hooks/pre-receive
  printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > $H && chmod +x $H
  $A request --as ann --repo beta -i successor=acme/other -i "reason=Testing a rejected push here."
  $A approve <id> --as bob      # exit 1, "status": "failed", "outcome.reason": "push_rejected"
  rm $H
  ```

  A push that is accepted and then undone (for example, a `post-receive` hook that resets `main`)
  is also reported `failed`, because the observation does not find the commit on `main`.
  `org/tests/selftest.sh` runs all of these cases.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The workflow, its step script and its approval policy are committed to each origin's `main`, and CODEOWNERS gives ann and bob ownership of them. `acme` reads the definition from the origin's `main` (never from a checkout), records the definition commit at request time, and runs that commit, as GitHub runs a dispatch's commit. A team that changes its files changes its action | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/`, CODEOWNERS files above |
| R2 Validated | Before any change, `acme` checks the inputs against the workflow's declared `inputs` (unknown or missing inputs). The workflow's `preview` job then applies the repository's rules: `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` (a trailing newline is refused) and must not be the repository itself; `reason` must be 20 to 200 characters (Unicode code points), on one line and not blank; the result must differ from what is on `main`. A refused request is recorded with status `refused` and never reaches approval. `apply` and `verify` validate again. Inputs reach scripts only through `env:`, never through `${{ }}` in a shell | `mark_deprecated.py` `validate()`, `inputs_or_refuse()`; `org/bin/acme` `check_declared()`, `preview_into()` |
| R3 Previewed | `request` runs the workflow's `preview` job and returns a unified diff of `DEPRECATED.md` against the requested commit, with before and after content. `preview` does the same without a record. On GitHub the same diff goes to the run's job summary while `apply` waits for review. The diff stays in the record for the approver, and `apply` refuses to run if `DEPRECATED.md` on `main` changed since the preview (`preview_stale`) | `mark_deprecated.py` `cmd_preview()`; `acme preview` and `acme request` |
| R4 Approved | The `apply` job uses `environment: mark-deprecated`. Its reviewers and `prevent_self_review` are code in `mark-deprecated.json`, applied by `apply-github-settings.sh`. Locally, act cannot pause for an environment, so `acme approve` enforces the policy read from the origin's `main` at approval time: the approver must be a listed reviewer (carol is not) and must not be the requester. Refused attempts are recorded as `approval_refused` events. Inside the workflow, `check-approval` checks the approver again: on GitHub it reads the run's approvals API, under act the approver `acme` passes. So running the workflow directly with act and no approval fails before `apply` | `<repo>/.github/self-service/mark-deprecated.json`, `apply-github-settings.sh`, `mark_deprecated.py` `cmd_check_approval()`, `org/bin/acme` `cmd_approve()` |
| R5 Done means observed | The workflow's `verify` step fetches `main` and checks both the exact content and that the pushed commit is on `main`. `acme` then observes the origin itself: `main`'s `DEPRECATED.md` must equal the previewed content and the run's commit must be an ancestor of `main`. Only then is the run `succeeded`. A rejected push (`push_rejected`), a push undone after it landed (`not_observed`) or a stale preview (`preview_stale`) gives `failed`, exit `1`. The push is never forced and is never retried on a newer `main` | `mark_deprecated.py` `cmd_apply()`, `cmd_verify()`; `org/bin/acme` `observe()`, `cmd_approve()` |
| R6 One place for the org | `org/bin/acme` lists `mark-deprecated` for every repository in `org/catalog.json` and requests, approves and inspects runs on any of them by `--repo`, without entering a checkout | `org/bin/acme`, `org/catalog.json` |
| R7 Audited | Each run is `org/state/runs/run-NNNN.json`. It holds `requested_by`, `requested_at`, `approved_by`, `approved_at`, `approval_comment`, `inputs`, the definition commit, the preview, and `outcome` (status, time, reason, commit and the observation of `main`). It also holds a timestamped `events` list (requested, previewed, approval_refused, approved, started, observed, succeeded, failed, rejected, refused) and the act logs. The commit on `main` carries `Requested-by`, `Approved-by` and `Acme-run` trailers. On GitHub the run history and the environment's deployment reviews are the audit | `org/bin/acme` `new_record()`, `event()`, `finish()`; `acme record <id>` |
| R8 Usable by agents | Every step is a non-interactive command with flags (or `ACME_USER`), JSON on stdout (usage errors too), and the exit codes above. `request` returns the run id and the next command | `org/bin/acme` |

## How GitHub would enforce what act cannot

| Feature | Configured in | Enforced on GitHub by |
|---|---|---|
| Required reviewers ann and bob | `mark-deprecated.json` → `apply-github-settings.sh` (`PUT /repos/{repo}/environments/mark-deprecated`) | The `apply` job waits until a reviewer approves the deployment |
| Prevent self-review | Same, `prevent_self_review: true` | The user who triggered the run cannot approve it. `check-approval` checks again through `GET /repos/{repo}/actions/runs/{id}/approvals` |
| carol cannot approve | Same, carol not in `reviewers` | Only listed reviewers can approve |
| No admin bypass | Same, `can_admins_bypass: false` | Admins cannot skip the review |
| Deployments only from `main` | Same, deployment branch policy `main` | A run dispatched from another branch cannot use the environment. Both jobs also require `github.ref == 'refs/heads/main'` |
| Changes to `main` and to the action reviewed | `apply-github-settings.sh`: a ruleset on `main` (no deletion, no force push, pull request with code-owner review); CODEOWNERS | Changes to the workflow or policy need ann's or bob's review. GitHub Actions (app id 15368) is the bypass actor, so the approved `apply` job can push |
| Least-privilege token | The workflow's `permissions` (`{}` at the top, `contents: read` for preview, `contents: write` and `actions: read` for apply) | `GITHUB_TOKEN` scopes |
| One run at a time per repository | The workflow's `concurrency` group | Queued runs. Locally `acme` holds a per-repository lock |
| Run history | GitHub Actions | The run list, logs and deployment reviews (`gh run list -R acme/alpha -w mark-deprecated.yml`) |

On GitHub the same steps would be:
`gh workflow run mark-deprecated.yml -R acme/alpha -f successor=... -f reason=...` (request);
`gh run view <id> -R acme/alpha` (preview in the job summary, and status);
`gh api -X POST repos/acme/alpha/actions/runs/<id>/pending_deployments -F 'environment_ids[]=<env id>' -f state=approved -f comment=ok`
(approve, as bob); and `gh api repos/acme/alpha/actions/runs/<id>/approvals` (record). `acme` has
only the local backend; a GitHub backend would call these same endpoints.

## What could not run here

- **Environment protection.** act ignores `environment:` and does not pause, so `acme` holds the
  run until approval and enforces the same policy file. The GitHub API branch of `check-approval`
  (the approvals endpoint) is untested; only its act branch ran.
- **`apply-github-settings.sh`** was not run (there is no GitHub). It passes `bash -n`. The
  ruleset's bypass actor id 15368 for GitHub Actions is from memory and not verified here.
- **Rulesets, branch protection and CODEOWNERS enforcement.** These exist as code only. The local
  bare origins have no protection unless you add a hook.
- **`actions/checkout`.** act skips it for the same repository and copies the clone `acme` prepared
  instead. The pinned SHA (`11bd719…`, v4.2.2) was not checked against GitHub from here.
- **`GITHUB_TOKEN` permissions, `concurrency` and `run-name`.** act does not enforce or show them.
- **Run history.** Locally the history is `org/state/runs/`. It is git-ignored and local to this
  machine, and anyone with shell access could edit it, unlike GitHub's run history.
- **Identity.** `--as` is trusted, as the sandbox allows. On GitHub the actor is the signed-in user.
- Under act every run's `github.run_id` is `1`, so the `Workflow-run:` trailer in the commit points
  at `runs/1`. The `Acme-run:` trailer identifies the run.

What did run: the workflow itself, under act 0.2.98 with `-P ubuntu-latest=-self-hosted`, for
every preview and apply; actionlint 1.7.12 on the workflow (clean); and `org/tests/selftest.sh`
(32 of 32 passed, before and after the real runs).

## Notes

- `DEPRECATED.md` is exactly `# Deprecated: use <successor>`, a blank line, then `<reason>`. The file
  ends with a newline.
- act is called with `--env-file`, `--secret-file`, `--var-file` and `--input-file` set to
  `/dev/null`, so dotfiles in a repository cannot inject values. It reads `github.repository`
  (`acme/<name>`) from a `github` remote that `acme` adds to its temporary clone. `origin` stays the
  bare repository, so the workflow's `git push origin` works unchanged under act and on GitHub.
