# HANDOFF: `mark-deprecated` self-service action

**Status.** `mark-deprecated` is a GitHub Actions workflow that each repository owns. It is
committed to `main` on all three origins. ann requested a run on each repository and bob approved
each one. All three runs are reported `succeeded`, and each origin's `main` now has
`DEPRECATED.md` with the exact content:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `run-20261124T124415Z-5a72` | `bef85b0c9652` |
| beta | `run-20261124T124416Z-816c` | `1eaaa17ae202` |
| gamma | `run-20261124T124416Z-1c6b` | `1e7d82c068b4` |

**Nothing needs to be running.** There is no server. The org CLI (`org/bin/self-service`) runs
each repository's workflow with act in host mode, then exits.

## How it fits together

| Where | What | Owned by |
|---|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: `workflow_dispatch` with jobs `validate` → `preview` → `apply`. `apply` runs only in `mode=apply` and waits on the `mark-deprecated` environment | the repository (CODEOWNERS: `@ann @bob`) |
| `<repo>/.github/self-service/config.json` | The repository's slug, its maintainers (the approvers), input rules, approval policy and the postcondition (`DEPRECATED.md` template on `main`) | the repository |
| `<repo>/.github/self-service/mark_deprecated.py` | The workflow's steps: `validate`, `preview`, `check-approval`, `apply`, `verify`. Python stdlib only | the repository |
| `<repo>/.github/self-service/apply-github-settings.sh` | The GitHub settings, as code: the environment with required reviewers and `prevent_self_review`, and a ruleset on `main` | the repository |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | Make ann and bob the owners of the files above | the repository |
| `org/bin/self-service` | The org CLI: list, preview, request, approve, reject, status, show, runs | the org |
| `org/self-service.json` | The repositories the org knows (origin paths), the act binary and its platform | the org |
| `org/people.json` | Known people (ann, bob, carol). Who maintains what is read from each repository's `config.json`, not from here | the org |
| `org/.state/runs/<run-id>/` | The run ledger: `record.json`, `request.log` and `apply.log` (act output), and the step outputs in `request/` and `apply/`. Gitignored; set `SELF_SERVICE_STATE` to use another directory | written by the CLI |

A run goes like this:

1. **Request.** `request` reads the definition from the origin's `main` and pins its commit. It
   runs the workflow in `mode=preview`, so only `validate` and `preview` run. If validation fails,
   the CLI exits 2 and writes no record. Otherwise it writes a record with status
   `awaiting_approval` and the preview.
2. **Approve.** `approve` refuses the requester and anyone who is not a maintainer in the
   repository's `config.json`. The approver must be a maintainer both at the pinned commit and on
   current `main`. A refused attempt is added to the record as `approval_denied`, and the CLI
   exits 3. Otherwise the CLI runs the pinned workflow in `mode=apply`. Under act, the `apply` job
   checks the approver again (`check-approval`), then commits and pushes `DEPRECATED.md` to `main`,
   then fetches `main` and compares the file (`verify`).
3. **Observe.** The CLI then reads `DEPRECATED.md` straight from the bare origin's
   `refs/heads/main` and compares it, byte for byte, with the rendered template. The run is
   `succeeded` only if act exited 0, the workflow's `verify` matched, and the CLI's own
   observation matched. Anything else is `failed`, with the reason in the record.

## Commands

Run these from `/tmp/actions-ab/r4/gha-1/org`. They need `python3` with PyYAML (installed here),
`git`, and `../bin/act`. Every command prints JSON on stdout.

**Exit codes:**

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | the run failed (or its preview failed) |
| 2 | invalid input or usage |
| 3 | not allowed |
| 4 | unknown repository, action or run |
| 5 | wrong state (for example, approving a run that already finished) |

```bash
cd /tmp/actions-ab/r4/gha-1/org

# Start: nothing to start.

# List the action on all three repositories (or one: --repo beta)
bin/self-service list
bin/self-service list | jq -c '.actions[] | {repository, action, maintainers, commit: .definition.commit}'

# Preview: validates and shows the diff; changes nothing and records nothing
bin/self-service preview alpha mark-deprecated \
  --input successor=acme/dependabot-proxy-v2 \
  --input 'reason=Replaced by the v2 proxy, which supports OIDC registries.'

# Request as ann: prints the run id, status awaiting_approval, and the preview
RUN=$(bin/self-service request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input 'reason=Replaced by the v2 proxy, which supports OIDC registries.' | jq -r .id)

# Approve as bob: runs the workflow; exits 0 only once DEPRECATED.md is observed on main
bin/self-service approve "$RUN" --as bob

# Status, and the full record
bin/self-service status "$RUN"
bin/self-service show "$RUN"

# Optional
bin/self-service reject "$RUN" --as bob --comment "not yet"   # a maintainer or the requester
bin/self-service runs --repo alpha --status succeeded           # the ledger
```

`-f name=value` is short for `--input name=value`, and `SELF_SERVICE_AS=ann` can stand in for
`--as ann`. To re-test a run on a repository that is already deprecated, use a different
`successor` or `reason`. The same inputs show `"change": "none"` in the preview, and an approved
run then succeeds without pushing, because the file is already right.

### Testing the refusals

These commands all exit non-zero, and none of them changes `main`.

```bash
# Invalid inputs: exit 2, every error listed, no record
bin/self-service preview beta mark-deprecated -f successor=Acme/Store -f reason=short
bin/self-service request beta mark-deprecated --as ann -f successor=acme/beta -f 'reason=Points at itself, which is refused.'
bin/self-service request beta mark-deprecated --as ann -f successor=acme/x -f reason=$(printf 'a%.0s' {1..201})
bin/self-service request beta mark-deprecated --as ann -f successor=acme/x -f 'reason=ok' -f extra=1

# Self-approval and carol: exit 3, logged as approval_denied in the record; the run stays awaiting_approval
RUN=$(bin/self-service request beta mark-deprecated --as ann -f successor=acme/other-store \
  -f 'reason=Testing the approval rules for this action.' | jq -r .id)
bin/self-service approve "$RUN" --as ann
bin/self-service approve "$RUN" --as carol
bin/self-service show "$RUN" | jq '.events'

# Rejected push: make the origin refuse pushes, then approve; exit 1, status failed
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/self-service approve "$RUN" --as bob
bin/self-service status "$RUN" | jq '{status, outcome}'
rm ../origins/beta.git/hooks/pre-receive
```

I ran each of these here, using a scratch ledger (`SELF_SERVICE_STATE`). With a pre-receive hook
on gamma's origin, bob's approval gave `failed` with the reason "push to main was rejected: …
(pre-receive hook declined)", and gamma's `main` did not move.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | The workflow, its step script, its config (maintainers, input rules, postcondition) and its GitHub settings script are committed to each origin's `main`. CODEOWNERS makes ann and bob the owners. The CLI reads the definition from the origin's `main` on every call, so a change the team merges takes effect at once. A run executes the definition commit it was previewed at (recorded in `definition.commit`) | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/{config.json,mark_deprecated.py,apply-github-settings.sh}`, `alpha/CODEOWNERS`, `{beta,gamma}/.github/CODEOWNERS` |
| R2 Validated | The workflow's first job, `validate`, applies the rules in `config.json`. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$`, so a trailing newline fails, and must not be the repository itself. `reason` must be 20–200 characters (Unicode code points), on one line, and not blank. Every job, including `apply`, depends on `validate`. The CLI also refuses unknown inputs and the reserved `mode`, `ledger_id` and `approver`. A refused request exits 2 with every error and writes no record | `mark_deprecated.py` (`validation_errors`), `config.json` (`inputs`), `org/bin/self-service` (`parse_inputs`, `preview_run`) |
| R3 Previewed | The `preview` job fetches `main` and reports `create`, `update` or `none`, with the base commit, the exact content and a unified diff. On GitHub this goes to the run's step summary; locally it goes to JSON. `preview` shows it without a record. `request` stores it in the record before anyone can approve | `mark_deprecated.py` (`step_preview`), `org/bin/self-service` (`cmd_preview`, `cmd_request`) |
| R4 Approved | On GitHub, the `apply` job uses `environment: mark-deprecated`. `apply-github-settings.sh` gives that environment required reviewers (the maintainers in `config.json`), `prevent_self_review: true`, and deployment from `main` only. Locally, `approve` refuses the requester and non-maintainers (exit 3, `approval_denied` in the record). The `apply` job's `check-approval` step then checks again under act. The push token is an environment secret, so only an approved job can push | `mark-deprecated.yml` (`apply`), `apply-github-settings.sh`, `mark_deprecated.py` (`step_check_approval`), `org/bin/self-service` (`cmd_approve`) |
| R5 Done means observed | `apply` pushes without force; a rejected push fails the job. `verify` fetches `main` and compares the file. Then the CLI reads `refs/heads/main:DEPRECATED.md` from the bare origin itself and compares SHA-256 hashes against the rendered template. The status is `succeeded` only if all of these agree, and the observed commit and hashes go into `outcome.observed`. Otherwise the status is `failed`, with the reason: push rejected, approval refused, or not observed | `mark_deprecated.py` (`step_apply`, `step_verify`), `org/bin/self-service` (`observe`, `cmd_approve`) |
| R6 One place for the org | From `org/`, `bin/self-service list` shows the action on every repository in `self-service.json`, with its inputs, rules, maintainers and definition commit. `request <repo> …` works on any of them without entering the repository | `org/bin/self-service`, `org/self-service.json`, `org/AGENTS.md` |
| R7 Audited | `org/.state/runs/<id>/record.json` holds `requested_by` and `requested_at`, `approved_by` and `approved_at`, `inputs`, the preview, `outcome` (status, at, reason, commit, observation) and `events`. `events` is a timestamped list: requested, previewed, approval_denied, approved, started, succeeded, failed, rejected. It sits beside the full act logs. The commit on `main` has `Self-Service-Run`, `Requested-by` and `Approved-by` trailers | `org/bin/self-service` (`event`, `save_record`), `mark_deprecated.py` (`step_apply`) |
| R8 Usable by agents | Every step is a non-interactive subcommand that prints JSON and uses the exit codes above. `org/AGENTS.md` points agents here | `org/bin/self-service`, `org/AGENTS.md` |

## On GitHub, and what could not run here

act cannot run the following. They are configured as code, and this is how GitHub would enforce
them:

- **Required reviewers and preventing self-review.** `apply-github-settings.sh` (with
  `SELF_SERVICE_APP_ID=<id>`, as a repository admin; `--dry-run` prints the API calls) sends
  `PUT /repos/{repo}/environments/mark-deprecated`. That sets `reviewers` to the maintainers in
  `config.json`, sets `prevent_self_review: true`, and allows deployment from `main` only. GitHub
  then holds the `apply` job until a reviewer approves it, refuses an approval from the person who
  dispatched the run, and refuses anyone who is not a reviewer (carol). The workflow reads the
  approver from `GET /actions/runs/{id}/approvals` for the commit trailer.
- **Branch protection.** The same script creates a ruleset on `main`: no deletion, no
  non-fast-forward pushes, and pull requests with code owner review. Its only bypass actor is the
  self-service GitHub App. The `apply` job mints that App's token with
  `actions/create-github-app-token`, from an environment secret (`SELF_SERVICE_APP_PRIVATE_KEY`),
  so only an approved run on `main` can push. A workflow edited on another branch cannot reach the
  secret.
- **Run history.** GitHub keeps each run, its step summary (the preview and the observation) and
  the environment's deployment and approval log. Here, `org/.state/runs/` plays that part.
- **Requests and approvals on GitHub.** Requesting maps to
  `gh workflow run mark-deprecated.yml --repo acme/<repo> -f successor=… -f reason=… -f mode=apply`.
  Approving maps to `gh api repos/acme/<repo>/actions/runs/<id>/pending_deployments -X POST`, and
  the status to `gh run view <id> --json conclusion,jobs`. The org CLI drives act instead; I did
  not build a GitHub backend for it.

**Not run here:**

- the settings script against the GitHub API (only `--dry-run`);
- environment reviews;
- rulesets;
- the App token step (it is skipped under act, which pushes to the local bare origin directly);
- the `gh api …/approvals` step.

Under act, the `actions/checkout` steps copy the CLI's clone of the pinned commit instead of
fetching from GitHub. act also clones `actions/create-github-app-token` into `~/.cache/act`, even
though the step is skipped; it is cached on this machine, and the CLI passes
`--action-offline-mode`.

**Limits of the local stand-in:**

- Identity is whatever `--as` says, as the task allows.
- Locally, the approval gate is the org CLI plus the workflow's `check-approval` step. Someone who
  runs `act` on a repository by hand, with `mode=apply` and an `approver` input, can skip the
  ledger. On GitHub the `approver` input is ignored, and the environment is the gate.
- A failed run is final. Request a new run to try again.
