# HANDOFF: `mark-deprecated`, a repository-owned self-service action on GitHub Actions

`mark-deprecated` is a GitHub Actions workflow that each repository owns on its own `main`.
`org/bin/acme` is the org-level catalogue that lists it on alpha, beta and gamma, previews it,
requests it, enforces the approval rules, runs it with act and records each run. The three
required runs succeeded: ann requested them, bob approved them, and `acme` read `DEPRECATED.md`
back from each origin's `main` before reporting success.

| Run | Repository | Commit on `main` | Outcome |
|---|---|---|---|
| `alpha-mark-deprecated-20261124t124658z-d235` | acme/alpha | `ff0ab40` | succeeded |
| `beta-mark-deprecated-20261124t124701z-081d` | acme/beta | `0e7c433` | succeeded |
| `gamma-mark-deprecated-20261124t124702z-0a73` | acme/gamma | `0909260` | succeeded |

## What is where

Paths are relative to `/tmp/actions-ab/r4/gha-2`.

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: `workflow_dispatch` with inputs `successor` and `reason`. Job `plan` validates and previews. Job `apply` is gated by the environment `mark-deprecated`, then pushes and verifies |
| `<repo>/.github/scripts/mark_deprecated.py` | The workflow's steps: `validate`, `preview`, `apply`, `verify` (Python standard library only) |
| `<repo>/.github/scripts/test_mark_deprecated.py` | Unit tests for the validation and content rules |
| `<repo>/.github/environments/mark-deprecated.json` | Environment settings as code: reviewers `ann` and `bob`, `prevent_self_review: true`, deployments from `main` only, the deploy-key secret |
| `<repo>/.github/rulesets/protect-main.json` | Ruleset for `main`: no deletion, no force push, pull requests need a code owner's review. Only deploy keys bypass it |
| `<repo>/.github/scripts/apply-github-settings.sh` | Applies the environment, deploy key and ruleset through the GitHub API (`gh api`) |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | `@ann @bob` own every file above |
| `org/bin/acme` | Org CLI: `list`, `preview`, `request`, `approve`, `reject`, `cancel`, `status`, `record`, `runs` |
| `org/catalog.json` | The repositories and their origins, the people (`ann`, `bob`, `carol`) and the act settings |
| `org/runs/<run-id>/record.json` | A run's audit record. Next to it are the act JSON logs (`plan-plan.jsonl`, `apply.jsonl`) |
| `org/AGENTS.md` | A short pointer for agents |

## Commands

Nothing has to be running: there is no server. Each command calls act itself and returns when the
run step is done. Act's local plan and apply runs each take a few seconds. You need `python3` with
PyYAML (6.0 is installed), `git` and `jq` (jq only for the examples below). act is
`bin/act` (0.2.98, host mode `-P ubuntu-latest=-self-hosted`).

```bash
cd /tmp/actions-ab/r4/gha-2/org

# List the actions on every repository (read from each origin's main)
bin/acme list
bin/acme list --repo beta

# Preview (validates, then shows the diff to main; changes nothing and writes no record)
bin/acme preview alpha mark-deprecated \
  --input successor=acme/new-thing --input 'reason=Replaced by new-thing, which does it better.'

# Request as ann (validates and previews again, then waits for approval; prints the run id)
RUN=$(bin/acme request alpha mark-deprecated \
  --input successor=acme/new-thing --input 'reason=Replaced by new-thing, which does it better.' \
  --as ann | jq -r .id)

# Approve as bob (runs the apply job, then observes main; returns succeeded or failed)
bin/acme approve "$RUN" --as bob

# Status and the full record
bin/acme status "$RUN"
bin/acme record "$RUN"

# Other steps
bin/acme reject "$RUN" --as bob --comment "not yet"   # a reviewer declines
bin/acme cancel "$RUN" --as ann                       # the requester withdraws
bin/acme runs --repo alpha --state succeeded
```

Rules for the CLI:

- **Identity:** `--as <person>` or `ACME_USER=<person>`. Unknown names are refused.
- **Inputs:** `-f` is short for `--input`.
- **Output:** every command prints one JSON document on stdout, including errors (`{"error": {"code", "message", ...}}`).
- **Exit codes:**

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | error or usage error |
| 2 | invalid inputs, refused |
| 3 | forbidden |
| 4 | run failed |
| 5 | wrong state, for example approving twice |
| 6 | not found |

A run's `state` is one of these:

| State | Meaning |
|---|---|
| `validating` | The request was recorded and the plan job is running |
| `refused` | The inputs were invalid; nothing ran past validation |
| `awaiting_approval` | Validated and previewed; waiting for a reviewer |
| `rejected` | A reviewer declined the run |
| `cancelled` | The requester withdrew the run |
| `running` | Approved; the apply job is running |
| `succeeded` | The change was observed on `main` |
| `failed` | The push failed or the change was not observed on `main` |

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | The whole definition is committed to each origin's `main`: the workflow, its script and tests, the approval settings and the ruleset. CODEOWNERS makes ann and bob its owners. `acme` keeps no copy. It reads the definition from `main` on every `list`. A run executes the workflow at the commit that was on `main` when the run was requested (`definition_sha` in the record), so the team can change it with an ordinary commit and approvers approve a pinned version | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/scripts/mark_deprecated.py`, `<repo>/.github/environments/mark-deprecated.json`, CODEOWNERS |
| R2 Validated | The `validate` step runs first in the `plan` job. `successor` must match `\A[a-z0-9-]+/[a-z0-9-]+\Z` and must not be this repository. `reason` must be 20 to 200 characters. On failure it writes errors per input and exits 1, before any clone or push. `apply` and `verify` validate again. `acme` also refuses unknown or missing input names. Inputs reach scripts only through `env:`, never through `${{ }}` inside a script | `mark_deprecated.py` (`validate`, `require_valid`), `mark-deprecated.yml` (top-level `env:`), `org/bin/acme` (`parse_inputs`) |
| R3 Previewed | The `preview` step clones `main`, reports `create`, `update` or `none` with a unified diff and the exact expected content, and writes it to the job summary. `acme preview` runs it as a dry run. `acme request` runs it again and stores it in the record (`preview`), so the reviewer sees it before approving | `mark_deprecated.py` (`cmd_preview`), `org/bin/acme` (`cmd_preview`, `cmd_request`) |
| R4 Approved | On GitHub, the `apply` job's `environment: mark-deprecated` has required reviewers ann and bob and `prevent_self_review`. Locally, `acme approve` enforces the same settings, read from the repository's file at the run's `definition_sha`: the approver must be a listed reviewer (carol is refused with `not_a_reviewer`) and must not be the requester (`self_review`). Refused attempts are recorded as `approve_refused` events. A run is approved at most once | `<repo>/.github/environments/mark-deprecated.json`, `apply-github-settings.sh`, `org/bin/acme` (`review_rules`, `_review`) |
| R5 Done means observed | The workflow's `verify` step clones `main` again and compares `DEPRECATED.md` byte for byte. The job fails if it does not match. After act returns, `acme` independently reads `refs/heads/main:DEPRECATED.md` from the origin and compares it with the plan's postcondition. It reports `succeeded` only if the apply job succeeded, `verify` observed the file and its own read matches. A rejected push fails the `apply` step, so the run is `failed`, with the remote's message in `result.errors` | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `org/bin/acme` (`observe`, `cmd_approve`) |
| R6 One place for the org | `org/bin/acme` and `org/catalog.json` list and run actions on all three repositories from `org`. The org repository is committed to `origins/org.git`. Actions are discovered on each origin's `main`: any `workflow_dispatch` workflow with an environment-gated job whose settings file exists. A repository adds a self-service action without changing `org` | `org/bin/acme` (`discover`), `org/catalog.json` |
| R7 Audited | `org/runs/<id>/record.json` holds `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `definition_sha`, `preview`, `result` (the apply job's output, `verify` and the independent observation) and `outcome`/`finished_at`. Its `events` list timestamps every step and actor: `requested`, `previewed`, `approve_refused`, `approved`, `apply_started`, `apply_finished`, `observed`, then `succeeded` or `failed`. The full act logs sit next to it. The commit on `main` carries the requester as author and `Requested-by`, `Approved-by` and `Request-id` trailers. The three records above are committed in `org` | `org/runs/`, `mark_deprecated.py` (commit message) |
| R8 Usable by agents | Every step is a flag-driven command with no prompts. Each prints JSON on stdout, uses distinct exit codes, and `status` includes a `next` hint | `org/bin/acme`, `org/AGENTS.md` |

## How to test the failure paths

```bash
cd /tmp/actions-ab/r4/gha-2/org
OK='reason=This reason is certainly long enough.'

# Invalid inputs: refused (exit 2) and nothing changes on main
bin/acme preview alpha mark-deprecated -f successor=Acme/X -f "$OK"
bin/acme request alpha mark-deprecated -f successor=acme/alpha -f "$OK" --as ann                # not another repository
bin/acme request alpha mark-deprecated -f successor=acme/x -f 'reason=too short' --as ann
bin/acme request alpha mark-deprecated -f successor=acme/x -f "reason=$(printf 'a%.0s' {1..201})" --as ann
bin/acme request alpha mark-deprecated -f successor=acme/x --as ann                             # missing reason

# Self-approval and carol: forbidden (exit 3), recorded, and the run is still awaiting approval
RUN=$(bin/acme request beta mark-deprecated -f successor=acme/other-store -f "$OK" --as ann | jq -r .id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN"

# Rejected push: the run is reported failed (exit 4), and main is unchanged
printf '#!/bin/sh\necho "push rejected by test" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
bin/acme record "$RUN" | jq '{state, observed: .result.observed, errors: .result.errors}'
rm ../origins/beta.git/hooks/pre-receive
```

Use inputs that differ from what is already on `main` when you test a rejected push. If
`DEPRECATED.md` already has the requested content, `apply` has nothing to push. It reports
`change: none`, and the run succeeds because the content is observed on `main`.

## How GitHub would enforce what act cannot

Run `.github/scripts/apply-github-settings.sh acme/<repo>` once per repository, as an admin with
`gh` and `jq`. It applies the settings below. Then GitHub enforces them:

| Setting | What GitHub enforces |
|---|---|
| Environment `mark-deprecated` | Required reviewers ann and bob. `prevent_self_review` stops the person who triggered the run from approving it. The deployment branch policy lets only `main` deploy to it. The `apply` job waits until a reviewer approves, and `GET /repos/{owner}/{repo}/actions/runs/{run_id}/approvals` records who approved. The script reads that to write `Approved-by` |
| Deploy key | A write deploy key whose private half is the environment secret `MARK_DEPRECATED_DEPLOY_KEY`. Only a job that runs in the environment can read it: an approved `apply` job on `main`. `GITHUB_TOKEN` stays read-only (`permissions: contents: read`) |
| Ruleset `protect-main` | Blocks direct pushes, force pushes and deletion on `main`. Changes need a pull request with a code owner's approval, so changes to the action itself need ann's or bob's review. Deploy keys are the only bypass, so only an approved `mark-deprecated` run can push straight to `main` |
| Run history | The Actions run list, with each run's job summary (preview, verification) and its approval history. `concurrency: mark-deprecated` makes runs queue rather than overlap |

The org CLI's steps map onto `gh` like this:

| `acme` command | `gh` equivalent |
|---|---|
| `request` | `gh workflow run mark-deprecated.yml -R acme/alpha -f successor=... -f reason=...` |
| `approve` | `gh api -X POST repos/acme/alpha/actions/runs/<id>/pending_deployments -F 'environment_ids[]=<env id>' -f state=approved -f comment=...` |
| `status` | `gh run view <id> -R acme/alpha --json status,conclusion,jobs` |
| `record` | `gh api repos/acme/alpha/actions/runs/<id>/approvals` plus the run log |

## What I could not run here

- **No GitHub here:** none of the GitHub-side enforcement was exercised: environments, required reviewers, prevent self-review, deployment branch policies, environment secrets, deploy keys, rulesets, CODEOWNERS review and the run history and approvals API. In this sandbox `acme` enforces the same reviewer rules from the same settings file. Nothing stops someone with shell access from pushing to an origin directly or editing `org/runs/`.
- **`apply-github-settings.sh` never ran:** it was checked only with `bash -n`. The `DeployKey` bypass actor in `protect-main.json` follows the rulesets API as I understand it, but I have not verified it against GitHub.
- **GitHub-only code paths in `mark_deprecated.py` never ran:** cloning over HTTPS with `GH_TOKEN`, pushing over SSH with the deploy key, and reading the approver from the approvals API. Every act run used the local origin, through the `ORIGIN_URL` variable that `acme` passes, and the approver through `APPROVED_BY`, which `acme` sets only after it has enforced the rules.
- **act skips `actions/checkout`:** it copies the working tree, which `acme` checks out at `definition_sha`. act ignores `environment:` and `concurrency:`.
- **Identity is self-declared** (`--as`), as the sandbox allows, and records are plain files. `acme` serialises state changes with a file lock, but two applies on the same repository can still overlap. The push retries on non-fast-forward, and `verify` catches the rest. If `acme` is killed mid-apply, the run stays `running`.
